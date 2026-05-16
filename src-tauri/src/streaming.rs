use crate::errors::{AppError, AppResult};
use crate::provider::PreparedOpenAiMultimodalRequest;
use serde::Serialize;
use std::pin::Pin;
use tauri::Emitter;

/// Events emitted to the frontend during a streaming AI request.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AiStreamEvent {
    /// A content chunk arrived from the model.
    Chunk { content: String },
    /// The stream finished normally.
    Done,
    /// The stream failed at the network or parsing layer.
    Error { code: String, message: String },
}

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 1000;

/// Sends a prepared OpenAI-compatible request and emits streaming events to the frontend.
///
/// When `stream` is true the response body is treated as SSE and parsed line-by-line.
/// When `stream` is false the full body is parsed as a normal JSON completion and emitted
/// as a single Chunk followed by Done.
///
/// Temporary failures (timeouts, network errors, 5xx, 429) are retried up to `MAX_RETRIES`
/// with exponential backoff. Non-retryable errors (401, 403, 400 without image keywords)
/// fail immediately.
pub async fn send_openai_request(
    app: tauri::AppHandle,
    request: PreparedOpenAiMultimodalRequest,
) -> AppResult<()> {
    let mut last_error: Option<AppError> = None;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay = BASE_RETRY_DELAY_MS * (1 << (attempt - 1));
            tracing::info!(attempt, delay_ms = delay, "Retrying AI request after delay");
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }

        match execute_openai_request(&app, &request).await {
            Ok(()) => return Ok(()),
            Err(error) if is_retryable_error(&error) => {
                tracing::warn!(attempt, %error, "AI request failed with a retryable error");
                last_error = Some(error);
            }
            Err(error) => {
                tracing::error!(attempt, %error, "AI request failed with a non-retryable error");
                return Err(error);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| AppError::AiRequestFailed {
        code: "maxRetriesExceeded".to_string(),
        message: "The AI request failed after the maximum number of retries.".to_string(),
    }))
}

async fn execute_openai_request(
    app: &tauri::AppHandle,
    request: &PreparedOpenAiMultimodalRequest,
) -> AppResult<()> {
    let client = reqwest::Client::new();
    let mut http_request = client
        .post(&request.endpoint)
        .header("Authorization", &request.authorization_header)
        .header("Content-Type", request.content_type)
        .timeout(request.request_timeout);

    http_request = http_request.json(&request.body);

    let response = http_request.send().await.map_err(map_reqwest_error)?;

    let status = response.status();
    if !status.is_success() {
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error body.".to_string());
        return Err(classify_http_error(status, &body_text));
    }

    if request.body.stream {
        handle_streaming_response(app.clone(), response).await
    } else {
        handle_non_streaming_response(app.clone(), response).await
    }
}

/// Determines whether an error is worth retrying.
fn is_retryable_error(error: &AppError) -> bool {
    let AppError::AiRequestFailed { code, .. } = error else {
        return false;
    };
    matches!(
        code.as_str(),
        "requestTimeout" | "networkError" | "providerServerError" | "rateLimited" | "requestFailed"
    )
}

async fn handle_streaming_response(
    app: tauri::AppHandle,
    response: reqwest::Response,
) -> AppResult<()> {
    let mut stream = ByteStreamAdapter::new(response);
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(map_reqwest_error)?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer.drain(..=line_end).collect::<String>();
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if let Some(event_payload) = trimmed.strip_prefix("data: ") {
                let event_payload = event_payload.trim();

                if event_payload == "[DONE]" {
                    emit_event(&app, AiStreamEvent::Done);
                    return Ok(());
                }

                match parse_sse_chunk(event_payload) {
                    Ok(Some(content)) => {
                        emit_event(&app, AiStreamEvent::Chunk { content });
                    }
                    Ok(None) => {}
                    Err(error) => {
                        emit_event(
                            &app,
                            AiStreamEvent::Error {
                                code: "streamParseError".to_string(),
                                message: error.to_string(),
                            },
                        );
                        return Err(error);
                    }
                }
            }
        }
    }

    // Stream ended without [DONE]; emit Done so the frontend does not hang.
    emit_event(&app, AiStreamEvent::Done);
    Ok(())
}

async fn handle_non_streaming_response(
    app: tauri::AppHandle,
    response: reqwest::Response,
) -> AppResult<()> {
    let body_text = response.text().await.map_err(map_reqwest_error)?;

    // Try to parse as standard OpenAI JSON first.
    let content = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_text) {
        extract_content_from_completion(&json)
    } else {
        None
    };

    let content = content.unwrap_or_else(|| {
        // Fallback: if the response is not valid JSON or lacks the expected
        // structure, emit the raw body so the user can still see something.
        tracing::warn!(
            body_len = body_text.len(),
            "Non-streaming response did not match OpenAI format; falling back to raw text"
        );
        body_text
    });

    emit_event(&app, AiStreamEvent::Chunk { content });
    emit_event(&app, AiStreamEvent::Done);
    Ok(())
}

/// Parses a single SSE `data:` line and returns the delta content, if any.
fn parse_sse_chunk(payload: &str) -> AppResult<Option<String>> {
    let json: serde_json::Value =
        serde_json::from_str(payload).map_err(|error| AppError::AiRequestFailed {
            code: "streamParseError".to_string(),
            message: format!("Could not parse SSE chunk: {error}"),
        })?;

    let choices = json.get("choices").and_then(|c| c.as_array());
    let Some(choices) = choices else {
        return Ok(None);
    };

    let mut combined = String::new();
    for choice in choices {
        if let Some(delta) = choice.get("delta") {
            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                combined.push_str(content);
            }
        } else if let Some(message) = choice.get("message") {
            // Non-streaming shape can also appear inside SSE in some providers.
            if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                combined.push_str(content);
            }
        }
    }

    if combined.is_empty() {
        Ok(None)
    } else {
        Ok(Some(combined))
    }
}

/// Extracts content from a non-streaming OpenAI chat completion response.
fn extract_content_from_completion(body: &serde_json::Value) -> Option<String> {
    let choices = body.get("choices")?.as_array()?;
    let mut combined = String::new();
    for choice in choices {
        if let Some(content) = choice
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
        {
            combined.push_str(content);
        }
    }
    if combined.is_empty() {
        None
    } else {
        Some(combined)
    }
}

fn emit_event(app: &tauri::AppHandle, event: AiStreamEvent) {
    let event_name = "question-scan:ai-stream-event";
    if let Err(error) = app.emit(event_name, &event) {
        tracing::warn!(%error, "Failed to emit AI stream event to frontend");
    }
}

/// Classifies HTTP error status codes and response bodies into user-friendly error types.
fn classify_http_error(status: reqwest::StatusCode, body_text: &str) -> AppError {
    let lower = body_text.to_lowercase();

    match status.as_u16() {
        401 => AppError::AiRequestFailed {
            code: "invalidApiKey".to_string(),
            message: "The API key was rejected. Check that your key is correct and has not expired.".to_string(),
        },
        403 => AppError::AiRequestFailed {
            code: "invalidApiKey".to_string(),
            message: "Access was denied. Your API key may be invalid or the account may be suspended.".to_string(),
        },
        400 if lower.contains("image") || lower.contains("vision") || lower.contains("multimodal") => {
            AppError::AiRequestFailed {
                code: "modelDoesNotSupportImages".to_string(),
                message: "The model does not appear to support image input. Try a vision-capable model such as gpt-4o or gpt-4o-mini.".to_string(),
            }
        }
        429 => AppError::AiRequestFailed {
            code: "rateLimited".to_string(),
            message: "The provider rate limit was hit. Wait a moment and try again.".to_string(),
        },
        500..=599 => AppError::AiRequestFailed {
            code: "providerServerError".to_string(),
            message: format!("The AI provider server returned an error ({}). Try again later.", status),
        },
        _ => AppError::AiRequestFailed {
            code: format!("http{}", status.as_u16()),
            message: format!("The AI provider returned an error ({}): {}", status, body_text),
        },
    }
}

fn map_reqwest_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::AiRequestFailed {
            code: "requestTimeout".to_string(),
            message:
                "The AI request timed out. Try increasing the timeout or checking your network."
                    .to_string(),
        }
    } else if error.is_connect() {
        AppError::AiRequestFailed {
            code: "networkError".to_string(),
            message: format!("Could not connect to the AI provider: {error}"),
        }
    } else {
        AppError::AiRequestFailed {
            code: "requestFailed".to_string(),
            message: format!("Request failed: {error}"),
        }
    }
}

// ------------------------------------------------------------------------------
// Polyfill: reqwest::Response::bytes_stream() returns an opaque type that does
// not implement Unpin.  We need a small adapter so we can call `.next()` in a
// loop while keeping the stream alive across await points.
// ------------------------------------------------------------------------------

struct ByteStreamAdapter {
    inner: Pin<Box<dyn futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
}

impl ByteStreamAdapter {
    fn new(response: reqwest::Response) -> Self {
        Self {
            inner: Box::pin(response.bytes_stream()),
        }
    }

    async fn next(&mut self) -> Option<Result<bytes::Bytes, reqwest::Error>> {
        use futures_util::StreamExt;
        self.inner.next().await
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_http_error, extract_content_from_completion, is_retryable_error, parse_sse_chunk,
    };
    use crate::errors::AppError;
    use serde_json::json;

    #[test]
    fn parses_sse_chunk_with_delta_content() {
        let payload = json!({
            "choices": [{"delta": {"content": "hello"}}]
        })
        .to_string();

        let result = parse_sse_chunk(&payload).expect("should parse");
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn parses_sse_chunk_with_multiple_deltas() {
        let payload = json!({
            "choices": [
                {"delta": {"content": "hello "}},
                {"delta": {"content": "world"}}
            ]
        })
        .to_string();

        let result = parse_sse_chunk(&payload).expect("should parse");
        assert_eq!(result, Some("hello world".to_string()));
    }

    #[test]
    fn returns_none_for_empty_delta_content() {
        let payload = json!({
            "choices": [{"delta": {"content": ""}}]
        })
        .to_string();

        let result = parse_sse_chunk(&payload).expect("should parse");
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_for_missing_choices() {
        let payload = json!({"id": "123"}).to_string();
        let result = parse_sse_chunk(&payload).expect("should parse");
        assert_eq!(result, None);
    }

    #[test]
    fn rejects_malformed_json_in_sse_payload() {
        let result = parse_sse_chunk("not json");
        assert!(result.is_err());
    }

    #[test]
    fn extracts_content_from_non_streaming_completion() {
        let body = json!({
            "choices": [{"message": {"content": "The answer is 42."}}]
        });

        let result = extract_content_from_completion(&body);
        assert_eq!(result, Some("The answer is 42.".to_string()));
    }

    #[test]
    fn returns_none_when_no_content_in_completion() {
        let body = json!({
            "choices": [{"message": {"role": "assistant"}}]
        });

        let result = extract_content_from_completion(&body);
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_when_choices_are_missing() {
        let body = json!({"id": "123"});
        let result = extract_content_from_completion(&body);
        assert_eq!(result, None);
    }

    #[test]
    fn falls_back_to_raw_text_when_json_has_no_expected_structure() {
        let body = json!({"error": "model not found"});
        let result = extract_content_from_completion(&body);
        assert_eq!(result, None);
    }

    #[test]
    fn classifies_401_as_invalid_api_key() {
        let error = classify_http_error(reqwest::StatusCode::UNAUTHORIZED, "bad key");
        assert!(
            error.to_string().contains("API key"),
            "expected API key hint: {error}"
        );
    }

    #[test]
    fn classifies_400_with_image_keyword_as_unsupported_model() {
        let error = classify_http_error(
            reqwest::StatusCode::BAD_REQUEST,
            "This model does not support vision inputs",
        );
        assert!(
            error.to_string().contains("image input"),
            "expected image support hint: {error}"
        );
    }

    #[test]
    fn classifies_429_as_rate_limited() {
        let error = classify_http_error(reqwest::StatusCode::TOO_MANY_REQUESTS, "slow down");
        assert!(
            error.to_string().contains("rate limit"),
            "expected rate limit hint: {error}"
        );
    }

    #[test]
    fn classifies_500_as_provider_server_error() {
        let error = classify_http_error(reqwest::StatusCode::INTERNAL_SERVER_ERROR, "oops");
        assert!(
            error.to_string().contains("server"),
            "expected server error hint: {error}"
        );
    }

    #[test]
    fn classifies_unknown_status_as_generic_http_error() {
        let error = classify_http_error(reqwest::StatusCode::NOT_FOUND, "not found");
        assert!(
            error.to_string().contains("404"),
            "expected status code in message: {error}"
        );
    }

    #[test]
    fn retryable_errors_include_timeout_network_5xx_and_rate_limited() {
        assert!(is_retryable_error(&AppError::AiRequestFailed {
            code: "requestTimeout".to_string(),
            message: "timeout".to_string(),
        }));
        assert!(is_retryable_error(&AppError::AiRequestFailed {
            code: "networkError".to_string(),
            message: "network".to_string(),
        }));
        assert!(is_retryable_error(&AppError::AiRequestFailed {
            code: "providerServerError".to_string(),
            message: "5xx".to_string(),
        }));
        assert!(is_retryable_error(&AppError::AiRequestFailed {
            code: "rateLimited".to_string(),
            message: "429".to_string(),
        }));
    }

    #[test]
    fn non_retryable_errors_fail_immediately() {
        assert!(!is_retryable_error(&AppError::AiRequestFailed {
            code: "invalidApiKey".to_string(),
            message: "401".to_string(),
        }));
        assert!(!is_retryable_error(&AppError::AiRequestFailed {
            code: "modelDoesNotSupportImages".to_string(),
            message: "400".to_string(),
        }));
        assert!(!is_retryable_error(&AppError::AiRequestFailed {
            code: "streamParseError".to_string(),
            message: "parse".to_string(),
        }));
    }

    #[test]
    fn non_ai_errors_are_not_retryable() {
        assert!(!is_retryable_error(&AppError::ConfigDirUnavailable));
        assert!(!is_retryable_error(&AppError::MainWindowUnavailable));
    }
}
