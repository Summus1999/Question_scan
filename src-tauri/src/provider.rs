use crate::errors::{AppError, AppResult};
use crate::settings::AppSettings;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde::Serialize;
use std::time::Duration;
use url::Url;

const HTTP_SCHEMES: [&str; 2] = ["http", "https"];
const OPENAI_JSON_CONTENT_TYPE: &str = "application/json";
const OPENAI_BEARER_PREFIX: &str = "Bearer ";

/// A request-ready AI provider configuration derived from saved app settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRequestConfig {
    pub provider_name: String,
    pub provider_base_url: Url,
    pub provider_api_key: String,
    pub provider_model: String,
    pub request_timeout: Duration,
    pub streaming_enabled: bool,
}

/// The image payload that will be inlined into an OpenAI-compatible multimodal request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiImageInput {
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

impl OpenAiImageInput {
    /// Creates a request image payload from any owned MIME type and byte buffer.
    pub fn new(mime_type: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            mime_type: mime_type.into(),
            bytes: bytes.into(),
        }
    }
}

/// The fully prepared OpenAI-compatible multimodal request ready for an HTTP client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedOpenAiMultimodalRequest {
    pub endpoint: String,
    pub authorization_header: String,
    pub content_type: &'static str,
    pub request_timeout: Duration,
    pub body: OpenAiChatCompletionRequest,
}

/// The OpenAI chat-completions payload that carries both the text instruction and the image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OpenAiChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAiChatMessage>,
    pub stream: bool,
}

/// A single chat message inside the OpenAI-compatible multimodal request body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OpenAiChatMessage {
    pub role: String,
    pub content: Vec<OpenAiChatContentPart>,
}

/// A single text or image part inside the OpenAI-compatible multimodal request body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAiChatContentPart {
    Text { text: String },
    ImageUrl { image_url: OpenAiImageUrl },
}

/// The image URL wrapper used by OpenAI chat-completions multimodal payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OpenAiImageUrl {
    pub url: String,
}

/// Validates the configured provider settings before the request stage starts.
pub fn validate_provider_request_config(
    settings: &AppSettings,
) -> AppResult<ProviderRequestConfig> {
    let provider_base_url = validate_provider_base_url(&settings.provider_base_url)?;
    let provider_api_key = required_trimmed_field(&settings.provider_api_key, "API key")?;
    let provider_model = required_trimmed_field(&settings.provider_model, "model")?;
    let request_timeout = validate_request_timeout(settings.request_timeout_seconds)?;

    Ok(ProviderRequestConfig {
        provider_name: settings.provider_name.trim().to_string(),
        provider_base_url,
        provider_api_key,
        provider_model,
        request_timeout,
        streaming_enabled: settings.streaming_enabled,
    })
}

/// Builds an OpenAI-compatible multimodal request from the validated provider config.
pub fn build_openai_multimodal_request(
    config: &ProviderRequestConfig,
    instruction: impl Into<String>,
    image: OpenAiImageInput,
) -> AppResult<PreparedOpenAiMultimodalRequest> {
    let instruction = required_trimmed_field(&instruction.into(), "multimodal instruction")?;
    let endpoint = build_openai_chat_completions_endpoint(&config.provider_base_url)?;
    let image_url = build_openai_image_data_url(&image)?;

    Ok(PreparedOpenAiMultimodalRequest {
        endpoint,
        authorization_header: format!("{OPENAI_BEARER_PREFIX}{}", config.provider_api_key),
        content_type: OPENAI_JSON_CONTENT_TYPE,
        request_timeout: config.request_timeout,
        body: OpenAiChatCompletionRequest {
            model: config.provider_model.clone(),
            messages: vec![OpenAiChatMessage {
                role: "user".to_string(),
                content: vec![
                    OpenAiChatContentPart::Text { text: instruction },
                    OpenAiChatContentPart::ImageUrl {
                        image_url: OpenAiImageUrl { url: image_url },
                    },
                ],
            }],
            stream: config.streaming_enabled,
        },
    })
}

fn validate_provider_base_url(raw_base_url: &str) -> AppResult<Url> {
    let trimmed = raw_base_url.trim();
    if trimmed.is_empty() {
        return Err(provider_configuration_error(
            "The provider base URL must be set before sending a request.",
        ));
    }

    let url = Url::parse(trimmed).map_err(|error| {
        provider_configuration_error(format!(
            "The provider base URL `{trimmed}` is invalid: {error}"
        ))
    })?;

    if !HTTP_SCHEMES.contains(&url.scheme()) {
        return Err(provider_configuration_error(format!(
            "The provider base URL must use http or https, but `{}` was provided.",
            url.scheme()
        )));
    }

    Ok(url)
}

fn required_trimmed_field(value: &str, field_name: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(provider_configuration_error(format!(
            "The {field_name} must be set before sending a request."
        )));
    }

    Ok(trimmed.to_string())
}

fn build_openai_chat_completions_endpoint(base_url: &Url) -> AppResult<String> {
    let mut endpoint = base_url.clone();
    endpoint
        .path_segments_mut()
        .map_err(|_| {
            provider_configuration_error(
                "The provider base URL cannot be turned into a chat-completions endpoint.",
            )
        })?
        .push("chat")
        .push("completions");

    Ok(endpoint.to_string())
}

fn build_openai_image_data_url(image: &OpenAiImageInput) -> AppResult<String> {
    let mime_type = required_trimmed_field(&image.mime_type, "image MIME type")?;
    if !mime_type.starts_with("image/") {
        return Err(provider_configuration_error(format!(
            "The image MIME type must start with `image/`, but `{mime_type}` was provided."
        )));
    }

    if image.bytes.is_empty() {
        return Err(provider_configuration_error(
            "The image payload must not be empty before sending a request.",
        ));
    }

    Ok(format!(
        "data:{mime_type};base64,{}",
        BASE64_STANDARD.encode(&image.bytes)
    ))
}

fn validate_request_timeout(seconds: u32) -> AppResult<Duration> {
    if seconds == 0 {
        return Err(provider_configuration_error(
            "The request timeout must be greater than zero seconds.",
        ));
    }

    Ok(Duration::from_secs(u64::from(seconds)))
}

fn provider_configuration_error(reason: impl Into<String>) -> AppError {
    AppError::ProviderConfigurationInvalid {
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_openai_multimodal_request, validate_provider_request_config, OpenAiImageInput,
        ProviderRequestConfig,
    };
    use crate::settings::AppSettings;
    use serde_json::json;
    use std::time::Duration;

    fn valid_settings() -> AppSettings {
        AppSettings {
            provider_name: "Example AI".to_string(),
            provider_base_url: "https://api.example.com/v1".to_string(),
            provider_api_key: "sk-test-123".to_string(),
            provider_model: "gpt-4o-mini".to_string(),
            request_timeout_seconds: 90,
            streaming_enabled: true,
            ..AppSettings::default()
        }
    }

    #[test]
    fn validates_a_complete_provider_configuration() {
        let settings = valid_settings();

        let config =
            validate_provider_request_config(&settings).expect("provider config should validate");

        assert_eq!(
            config,
            ProviderRequestConfig {
                provider_name: "Example AI".to_string(),
                provider_base_url: url::Url::parse("https://api.example.com/v1")
                    .expect("valid url"),
                provider_api_key: "sk-test-123".to_string(),
                provider_model: "gpt-4o-mini".to_string(),
                request_timeout: Duration::from_secs(90),
                streaming_enabled: true,
            }
        );
    }

    #[test]
    fn rejects_empty_api_keys_before_request_dispatch() {
        let mut settings = valid_settings();
        settings.provider_api_key = "   ".to_string();

        let error = validate_provider_request_config(&settings)
            .expect_err("empty api key should fail validation");

        let payload = serde_json::to_value(&error).expect("serialize provider error");
        assert_eq!(payload["code"], json!("providerConfigurationInvalid"));
        assert!(payload["message"]
            .as_str()
            .expect("error message")
            .contains("API key"));
    }

    #[test]
    fn rejects_invalid_provider_base_urls() {
        let mut settings = valid_settings();
        settings.provider_base_url = "ftp://api.example.com/v1".to_string();

        let error = validate_provider_request_config(&settings)
            .expect_err("invalid provider base url should fail validation");

        assert!(
            error.to_string().contains("must use http or https"),
            "expected a protocol validation error"
        );
    }

    #[test]
    fn rejects_missing_provider_base_urls() {
        let mut settings = valid_settings();
        settings.provider_base_url = "   ".to_string();

        let error = validate_provider_request_config(&settings)
            .expect_err("missing provider base url should fail validation");

        assert!(
            error.to_string().contains("provider base URL must be set"),
            "expected a missing base url error"
        );
    }

    #[test]
    fn builds_an_openai_multimodal_request_with_text_and_image_content() {
        let config = validate_provider_request_config(&valid_settings())
            .expect("provider config should validate");

        let request = build_openai_multimodal_request(
            &config,
            "Solve the problem and give the answer in C++.",
            OpenAiImageInput::new("image/jpeg", vec![1, 2, 3]),
        )
        .expect("build multimodal request");

        assert_eq!(
            request.endpoint,
            "https://api.example.com/v1/chat/completions"
        );
        assert_eq!(request.authorization_header, "Bearer sk-test-123");
        assert_eq!(request.content_type, "application/json");
        assert_eq!(request.request_timeout, Duration::from_secs(90));
        assert_eq!(request.body.model, "gpt-4o-mini");
        assert!(request.body.stream);

        let payload = serde_json::to_value(&request.body).expect("serialize request body");
        assert_eq!(payload["model"], json!("gpt-4o-mini"));
        assert_eq!(payload["stream"], json!(true));
        assert_eq!(payload["messages"][0]["role"], json!("user"));
        assert_eq!(
            payload["messages"][0]["content"][0],
            json!({
                "type": "text",
                "text": "Solve the problem and give the answer in C++."
            })
        );
        assert_eq!(
            payload["messages"][0]["content"][1]["type"],
            json!("image_url")
        );
        assert_eq!(
            payload["messages"][0]["content"][1]["image_url"]["url"],
            json!("data:image/jpeg;base64,AQID")
        );
    }

    #[test]
    fn rejects_empty_multimodal_instructions() {
        let config = validate_provider_request_config(&valid_settings())
            .expect("provider config should validate");

        let error = build_openai_multimodal_request(
            &config,
            "   ",
            OpenAiImageInput::new("image/jpeg", vec![1, 2, 3]),
        )
        .expect_err("empty instruction should fail");

        assert!(error.to_string().contains("multimodal instruction"));
    }

    #[test]
    fn rejects_invalid_or_empty_multimodal_images() {
        let config = validate_provider_request_config(&valid_settings())
            .expect("provider config should validate");

        let invalid_mime_type = build_openai_multimodal_request(
            &config,
            "Solve the problem.",
            OpenAiImageInput::new("text/plain", vec![1, 2, 3]),
        )
        .expect_err("invalid image mime type should fail");
        assert!(invalid_mime_type.to_string().contains("image/"));

        let empty_bytes = build_openai_multimodal_request(
            &config,
            "Solve the problem.",
            OpenAiImageInput::new("image/jpeg", Vec::new()),
        )
        .expect_err("empty image bytes should fail");
        assert!(empty_bytes.to_string().contains("must not be empty"));

        let empty_content_type = build_openai_multimodal_request(
            &config,
            "Solve the problem.",
            OpenAiImageInput::new("   ", vec![1, 2, 3]),
        )
        .expect_err("empty image mime type should fail");
        assert!(empty_content_type.to_string().contains("image MIME type"));
    }
}
