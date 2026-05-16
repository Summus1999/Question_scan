/**
 * Question Scan 题目区域识别模块（阶段 4）。
 *
 * 职责：通过视觉模型自动识别截图中的算法题目区域，返回边界框和置信度。
 * 包含：低分辨率图片准备、识别请求构造、模型响应解析、置信度路由分类、
 *       低分辨率框到原始高分辨率截图的坐标映射与裁剪。
 * 置信度路由：高置信度自动接受 → 中置信度需用户确认 → 低置信度手动框选兜底。
 */
use crate::screenshot::{compress_image_for_ai, image, ScreenshotCompressionConfig};
use serde::{Deserialize, Serialize};

pub const DEFAULT_RECOGNITION_IMAGE_MAX_LONG_EDGE: u32 = 960;
pub const DEFAULT_RECOGNITION_IMAGE_JPEG_QUALITY: u8 = 72;
pub const RECOGNITION_IMAGE_MIME_TYPE: &str = "image/jpeg";
pub const DEFAULT_RECOGNITION_AUTO_ACCEPT_CONFIDENCE: f64 = 0.85;
pub const DEFAULT_RECOGNITION_CONFIRMATION_MIN_CONFIDENCE: f64 = 0.55;

/** 题目区域边界框，由视觉模型返回。 */
/// The bounding box returned by question-region recognition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionBoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/** 题目区域识别结果，包含边界框、置信度、标题和题目文本。 */
/// The shared recognition result returned by question-region detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionRecognitionResult {
    pub bounding_box: QuestionBoundingBox,
    pub confidence: f64,
    pub title: Option<String>,
    pub question_text: Option<String>,
    pub reason: String,
}

/** 置信度路由决策：根据识别置信度决定后续处理路径。 */
/// The routing decision derived from the recognition confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecognitionConfidenceRoute {
    AutoAccept,
    NeedsConfirmation,
    ManualFallback,
}

/** 置信度阈值配置：划分自动接受、需确认、手动兜底三条路径的边界。 */
/// The confidence thresholds that split recognition into accept, confirm, or fallback paths.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecognitionConfidenceThresholds {
    pub auto_accept_min_confidence: f64,
    pub confirmation_min_confidence: f64,
}

impl Default for RecognitionConfidenceThresholds {
    fn default() -> Self {
        Self {
            auto_accept_min_confidence: DEFAULT_RECOGNITION_AUTO_ACCEPT_CONFIDENCE,
            confirmation_min_confidence: DEFAULT_RECOGNITION_CONFIRMATION_MIN_CONFIDENCE,
        }
    }
}

impl RecognitionConfidenceThresholds {
    /** 修复非法阈值，确保确认区间上限不超过自动接受阈值。 */
    /// Repairs invalid threshold values and keeps the confirmation band below auto accept.
    pub fn sanitized(self) -> Self {
        let auto_accept_min_confidence =
            sanitize_confidence_threshold(self.auto_accept_min_confidence);
        let confirmation_min_confidence =
            sanitize_confidence_threshold(self.confirmation_min_confidence)
                .min(auto_accept_min_confidence);

        Self {
            auto_accept_min_confidence,
            confirmation_min_confidence,
        }
    }
}

/** 题目区域识别请求：包含提示词和低分辨率图片载荷。 */
/// The low-resolution recognition payload and prompt sent to the vision model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionRegionRecognitionRequest {
    pub prompt: String,
    pub image: RecognitionImageInput,
}

/** 从验证后的识别结果生成的高分辨率裁剪区域。 */
/// A high-resolution crop produced from a validated recognition result.
#[derive(Debug, Clone, PartialEq)]
pub struct CroppedQuestionRegion {
    pub recognition: QuestionRecognitionResult,
    pub crop_box: QuestionBoundingBox,
    pub image: image::RgbaImage,
}

/** 识别图片配置：控制发送给视觉模型的低分辨率图片尺寸和质量。 */
/// Controls the size and quality of the low-resolution image sent to recognition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecognitionImageConfig {
    pub max_long_edge: u32,
    pub jpeg_quality: u8,
}

impl Default for RecognitionImageConfig {
    fn default() -> Self {
        Self {
            max_long_edge: DEFAULT_RECOGNITION_IMAGE_MAX_LONG_EDGE,
            jpeg_quality: DEFAULT_RECOGNITION_IMAGE_JPEG_QUALITY,
        }
    }
}

impl RecognitionImageConfig {
    /** 修复非法的识别图片参数。 */
    /// Repairs invalid recognition-image settings so the request shape stays predictable.
    pub fn sanitized(self) -> Self {
        Self {
            max_long_edge: self.max_long_edge.max(1),
            jpeg_quality: self.jpeg_quality.clamp(1, 100),
        }
    }
}

/** 识别图片输入：低分辨率图片载荷，包含原始尺寸和缩放后的尺寸信息。 */
/// The low-resolution image payload sent to the visual recognition step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognitionImageInput {
    pub mime_type: &'static str,
    pub original_width: u32,
    pub original_height: u32,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

impl RecognitionImageInput {
    /** 计算从低分辨率输入到原始图片的水平缩放因子。 */
    /// Returns the horizontal scale factor from the low-resolution input back to the source image.
    pub fn scale_x(&self) -> f64 {
        self.original_width as f64 / self.width as f64
    }

    /** 计算从低分辨率输入到原始图片的垂直缩放因子。 */
    /// Returns the vertical scale factor from the low-resolution input back to the source image.
    pub fn scale_y(&self) -> f64 {
        self.original_height as f64 / self.height as f64
    }
}

impl QuestionRecognitionResult {
    /** 根据置信度阈值将识别结果分类为自动接受、需确认或手动兜底。 */
    /// Classifies a recognition result into auto-accept, confirmation, or fallback.
    pub fn confidence_route(
        &self,
        thresholds: RecognitionConfidenceThresholds,
    ) -> RecognitionConfidenceRoute {
        classify_question_recognition_confidence(self.confidence, thresholds)
    }
}

impl QuestionBoundingBox {
    /** 验证边界框是否在图片范围内且尺寸为正。
     * 用于防止模型返回超出图片边界或零尺寸的非法框。
     */
    /// Verifies that the box stays inside the given image bounds and has a positive size.
    pub fn validate_within(
        &self,
        image_width: u32,
        image_height: u32,
    ) -> RecognitionImageResult<()> {
        if self.width == 0 || self.height == 0 {
            return Err("Recognition bounding boxes must have a positive size".to_string());
        }

        if self.x < 0 || self.y < 0 {
            return Err("Recognition bounding boxes must stay within the image bounds".to_string());
        }

        let right = i64::from(self.x) + i64::from(self.width);
        let bottom = i64::from(self.y) + i64::from(self.height);
        let image_right = i64::from(image_width);
        let image_bottom = i64::from(image_height);

        if right > image_right || bottom > image_bottom {
            return Err("Recognition bounding boxes must stay within the image bounds".to_string());
        }

        Ok(())
    }
}

pub(crate) type RecognitionImageResult<T> = Result<T, String>;

const QUESTION_REGION_RESPONSE_SHAPE: &str = r#"{
  "boundingBox": {
    "x": 0,
    "y": 0,
    "width": 0,
    "height": 0
  },
  "confidence": 0,
  "title": null,
  "questionText": null,
  "reason": ""
}"#;

/** 构建题目区域识别请求：将原始截图压缩为低分辨率图片并生成提示词。 */
/// Builds the request payload for low-resolution question-region recognition.
pub(crate) fn build_question_region_recognition_request(
    image: &image::RgbaImage,
    config: RecognitionImageConfig,
) -> RecognitionImageResult<QuestionRegionRecognitionRequest> {
    let image = prepare_recognition_image(image, config)?;
    let prompt = build_question_region_prompt(&image);

    Ok(QuestionRegionRecognitionRequest { prompt, image })
}

/** 构建提示词，要求模型返回最可能的题目区域边界框（JSON 格式）。 */
/// Creates the prompt that asks the model to return the most likely question region.
pub(crate) fn build_question_region_prompt(image: &RecognitionImageInput) -> String {
    format!(
        concat!(
            "You are identifying the single most likely algorithm problem region from a ",
            "low-resolution screenshot.\n",
            "The attached image is {width}x{height} pixels.\n",
            "Return only a JSON object with these camelCase fields:\n",
            "{shape}\n",
            "Rules:\n",
            "- Return exactly one bounding box for the most likely algorithm question region.\n",
            "- Use integer pixels in the attached image coordinate space.\n",
            "- The origin is the top-left corner of the attached image.\n",
            "- `title` and `questionText` may be null when unreadable.\n",
            "- `reason` should explain why this box was chosen.\n",
            "- Do not include markdown, code fences, or any extra text.\n"
        ),
        width = image.width,
        height = image.height,
        shape = QUESTION_REGION_RESPONSE_SHAPE,
    )
}

/** 解析模型响应为识别结果结构体，支持带代码围栏和不带围栏的 JSON。 */
/// Parses the model response into the shared recognition contract.
pub(crate) fn parse_question_recognition_result(
    response: &str,
) -> RecognitionImageResult<QuestionRecognitionResult> {
    let payload = extract_json_payload(response);
    serde_json::from_str::<QuestionRecognitionResult>(payload).map_err(|error| error.to_string())
}

/** 根据置信度分数和阈值，分类为自动接受、需确认或手动兜底路径。 */
/// Classifies a recognition confidence score into the auto-accept, confirmation, or fallback path.
pub(crate) fn classify_question_recognition_confidence(
    confidence: f64,
    thresholds: RecognitionConfidenceThresholds,
) -> RecognitionConfidenceRoute {
    let thresholds = thresholds.sanitized();
    let confidence = sanitize_confidence_score(confidence);

    if confidence >= thresholds.auto_accept_min_confidence {
        RecognitionConfidenceRoute::AutoAccept
    } else if confidence >= thresholds.confirmation_min_confidence {
        RecognitionConfidenceRoute::NeedsConfirmation
    } else {
        RecognitionConfidenceRoute::ManualFallback
    }
}

/** 验证识别结果是否位于低分辨率识别图片的边界内。 */
/// Validates the model response against the low-resolution recognition image bounds.
pub(crate) fn validate_question_recognition_result(
    recognition: QuestionRecognitionResult,
    image: &RecognitionImageInput,
) -> RecognitionImageResult<QuestionRecognitionResult> {
    recognition
        .bounding_box
        .validate_within(image.width, image.height)?;
    Ok(recognition)
}

/** 使用低分辨率输入上产生的识别框，裁剪原始高分辨率截图。
 * 流程：验证识别结果 → 将低分辨率坐标映射回原始尺寸 → 执行裁剪。
 */
/// Crops the original screenshot using a recognition box that was produced on the low-resolution input.
pub(crate) fn crop_original_question_region(
    source: &image::RgbaImage,
    recognition: QuestionRecognitionResult,
    recognition_image: &RecognitionImageInput,
) -> RecognitionImageResult<CroppedQuestionRegion> {
    if source.width() != recognition_image.original_width
        || source.height() != recognition_image.original_height
    {
        return Err(
            "Original screenshot dimensions must match the recognition image metadata".to_string(),
        );
    }

    let recognition = validate_question_recognition_result(recognition, recognition_image)?;
    let crop_box = map_recognition_box_to_original(recognition.bounding_box, recognition_image)?;
    crop_box.validate_within(source.width(), source.height())?;
    let cropped = image::imageops::crop_imm(
        source,
        crop_box.x as u32,
        crop_box.y as u32,
        crop_box.width,
        crop_box.height,
    )
    .to_image();

    Ok(CroppedQuestionRegion {
        recognition,
        crop_box,
        image: cropped,
    })
}

/** 从模型响应中提取 JSON 载荷，去除可能的 markdown 代码围栏。 */
fn extract_json_payload(response: &str) -> &str {
    let trimmed = response.trim();
    let payload = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim();

    payload.strip_suffix("```").unwrap_or(payload).trim()
}

/** 清理置信度分数：限制在 [0, 1] 范围内，处理 NaN/Inf 等非法值。 */
fn sanitize_confidence_score(confidence: f64) -> f64 {
    if confidence.is_finite() {
        confidence.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/** 清理置信度阈值：复用分数清理逻辑。 */
fn sanitize_confidence_threshold(threshold: f64) -> f64 {
    sanitize_confidence_score(threshold)
}

/** 将低分辨率识别框映射回原始高分辨率图片坐标。
 * 使用 floor/ceil 确保不丢失像素，并进行边界裁剪。
 */
fn map_recognition_box_to_original(
    bounding_box: QuestionBoundingBox,
    image: &RecognitionImageInput,
) -> RecognitionImageResult<QuestionBoundingBox> {
    bounding_box.validate_within(image.width, image.height)?;

    let scale_x = image.scale_x();
    let scale_y = image.scale_y();
    let left = ((bounding_box.x as f64) * scale_x).floor() as u32;
    let top = ((bounding_box.y as f64) * scale_y).floor() as u32;
    let right = (((bounding_box.x as f64) + (bounding_box.width as f64)) * scale_x).ceil() as u32;
    let bottom = (((bounding_box.y as f64) + (bounding_box.height as f64)) * scale_y).ceil() as u32;
    let right = right.min(image.original_width);
    let bottom = bottom.min(image.original_height);
    let crop_box = QuestionBoundingBox {
        x: left as i32,
        y: top as i32,
        width: right.saturating_sub(left),
        height: bottom.saturating_sub(top),
    };

    crop_box.validate_within(image.original_width, image.original_height)?;
    Ok(crop_box)
}

/** 准备用于题目区域识别的紧凑 JPEG 图片。
 * 将原始截图压缩为低分辨率版本，降低视觉模型的 Token 消耗。
 */
/// Prepares a compact JPEG image for question-region recognition.
pub(crate) fn prepare_recognition_image(
    image: &image::RgbaImage,
    config: RecognitionImageConfig,
) -> RecognitionImageResult<RecognitionImageInput> {
    if image.width() == 0 || image.height() == 0 {
        return Err("Recognition images must have positive dimensions".to_string());
    }

    let config = config.sanitized();
    let compressed = compress_image_for_ai(
        image,
        ScreenshotCompressionConfig {
            max_long_edge: config.max_long_edge,
            jpeg_quality: config.jpeg_quality,
        },
    )?;

    Ok(RecognitionImageInput {
        mime_type: RECOGNITION_IMAGE_MIME_TYPE,
        original_width: image.width(),
        original_height: image.height(),
        width: compressed.width,
        height: compressed.height,
        bytes: compressed.bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_question_region_prompt, build_question_region_recognition_request,
        crop_original_question_region, image, parse_question_recognition_result,
        prepare_recognition_image, validate_question_recognition_result, QuestionBoundingBox,
        QuestionRecognitionResult, RecognitionConfidenceRoute, RecognitionConfidenceThresholds,
        RecognitionImageConfig, RecognitionImageInput, DEFAULT_RECOGNITION_AUTO_ACCEPT_CONFIDENCE,
        DEFAULT_RECOGNITION_CONFIRMATION_MIN_CONFIDENCE, DEFAULT_RECOGNITION_IMAGE_JPEG_QUALITY,
        DEFAULT_RECOGNITION_IMAGE_MAX_LONG_EDGE, RECOGNITION_IMAGE_MIME_TYPE,
    };
    use serde_json::json;

    #[test]
    fn recognition_result_serializes_to_camel_case_fields() {
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 48,
                y: 96,
                width: 720,
                height: 420,
            },
            confidence: 0.91,
            title: Some("Longest Substring Without Repeating Characters".to_string()),
            question_text: Some("Given a string s, find the length of the longest substring without repeating characters.".to_string()),
            reason: "The text block groups the title, statement, and examples into one algorithm problem panel."
                .to_string(),
        };

        assert_eq!(
            serde_json::to_value(&recognition).expect("serialize recognition"),
            json!({
                "boundingBox": {
                    "x": 48,
                    "y": 96,
                    "width": 720,
                    "height": 420,
                },
                "confidence": 0.91,
                "title": "Longest Substring Without Repeating Characters",
                "questionText": "Given a string s, find the length of the longest substring without repeating characters.",
                "reason": "The text block groups the title, statement, and examples into one algorithm problem panel.",
            })
        );
    }

    #[test]
    fn recognition_result_serializes_nullable_text_fields() {
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            confidence: 0.0,
            title: None,
            question_text: None,
            reason:
                "Recognition could not isolate the problem statement from the rest of the page."
                    .to_string(),
        };

        assert_eq!(
            serde_json::to_value(&recognition).expect("serialize nullable recognition"),
            json!({
                "boundingBox": {
                    "x": 0,
                    "y": 0,
                    "width": 1920,
                    "height": 1080,
                },
                "confidence": 0.0,
                "title": null,
                "questionText": null,
                "reason": "Recognition could not isolate the problem statement from the rest of the page.",
            })
        );
    }

    #[test]
    fn recognition_image_config_uses_low_resolution_defaults() {
        let config = RecognitionImageConfig::default();

        assert_eq!(
            config.max_long_edge,
            DEFAULT_RECOGNITION_IMAGE_MAX_LONG_EDGE
        );
        assert_eq!(config.jpeg_quality, DEFAULT_RECOGNITION_IMAGE_JPEG_QUALITY);
    }

    #[test]
    fn recognition_image_config_repairs_invalid_values() {
        let config = RecognitionImageConfig {
            max_long_edge: 0,
            jpeg_quality: 255,
        };

        let sanitized = config.sanitized();

        assert_eq!(sanitized.max_long_edge, 1);
        assert_eq!(sanitized.jpeg_quality, 100);
    }

    #[test]
    fn prepare_recognition_image_scales_down_and_preserves_coordinate_ratio() {
        let image = image::RgbaImage::from_fn(8, 4, |x, y| {
            image::Rgba([(x * 20) as u8, (y * 60) as u8, 180, 255])
        });

        let prepared = prepare_recognition_image(
            &image,
            RecognitionImageConfig {
                max_long_edge: 4,
                jpeg_quality: 80,
            },
        )
        .expect("prepare recognition image");

        assert_eq!(
            prepared,
            RecognitionImageInput {
                mime_type: RECOGNITION_IMAGE_MIME_TYPE,
                original_width: 8,
                original_height: 4,
                width: 4,
                height: 2,
                bytes: prepared.bytes.clone(),
            }
        );
        assert_eq!(prepared.scale_x(), 2.0);
        assert_eq!(prepared.scale_y(), 2.0);

        let decoded = image::load_from_memory(&prepared.bytes).expect("decode prepared jpeg");
        assert_eq!(decoded.width(), 4);
        assert_eq!(decoded.height(), 2);
    }

    #[test]
    fn prepare_recognition_image_keeps_small_images_at_original_size() {
        let image = image::RgbaImage::from_pixel(3, 2, image::Rgba([10, 20, 30, 255]));

        let prepared = prepare_recognition_image(
            &image,
            RecognitionImageConfig {
                max_long_edge: 10,
                jpeg_quality: 75,
            },
        )
        .expect("prepare recognition image");

        assert_eq!(prepared.original_width, 3);
        assert_eq!(prepared.original_height, 2);
        assert_eq!(prepared.width, 3);
        assert_eq!(prepared.height, 2);
        assert_eq!(prepared.scale_x(), 1.0);
        assert_eq!(prepared.scale_y(), 1.0);

        let decoded = image::load_from_memory(&prepared.bytes).expect("decode prepared jpeg");
        assert_eq!(decoded.width(), 3);
        assert_eq!(decoded.height(), 2);
    }

    #[test]
    fn prepare_recognition_image_rejects_empty_images() {
        let image = image::RgbaImage::new(0, 0);

        let error = prepare_recognition_image(&image, RecognitionImageConfig::default())
            .expect_err("empty image should be rejected");

        assert!(error.contains("positive dimensions"));
    }

    #[test]
    fn build_question_region_prompt_requests_a_single_low_resolution_box() {
        let image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 1920,
            original_height: 1080,
            width: 960,
            height: 540,
            bytes: vec![1, 2, 3],
        };

        let prompt = build_question_region_prompt(&image);

        assert!(prompt.contains("single most likely algorithm problem region"));
        assert!(prompt.contains("960x540"));
        assert!(prompt.contains("boundingBox"));
        assert!(prompt.contains("questionText"));
        assert!(prompt.contains("top-left corner"));
        assert!(prompt.contains("Do not include markdown"));
    }

    #[test]
    fn build_question_region_recognition_request_packages_prompt_and_image() {
        let image = image::RgbaImage::from_pixel(8, 4, image::Rgba([10, 20, 30, 255]));

        let request = build_question_region_recognition_request(
            &image,
            RecognitionImageConfig {
                max_long_edge: 4,
                jpeg_quality: 80,
            },
        )
        .expect("build recognition request");

        assert_eq!(request.image.mime_type, RECOGNITION_IMAGE_MIME_TYPE);
        assert_eq!(request.image.original_width, 8);
        assert_eq!(request.image.original_height, 4);
        assert_eq!(request.image.width, 4);
        assert_eq!(request.image.height, 2);
        assert_eq!(request.image.scale_x(), 2.0);
        assert_eq!(request.image.scale_y(), 2.0);
        assert!(request.prompt.contains("Return only a JSON object"));
        assert!(request.prompt.contains("4x2"));
    }

    #[test]
    fn parse_question_recognition_result_reads_plain_json() {
        let response = r#"{
            "boundingBox": { "x": 24, "y": 48, "width": 640, "height": 360 },
            "confidence": 0.84,
            "title": "Two Sum",
            "questionText": "Given an array of integers, return indices of the two numbers that add up to a target.",
            "reason": "The main problem statement is centered in the page body."
        }"#;

        let parsed = parse_question_recognition_result(response).expect("parse recognition");

        assert_eq!(
            parsed,
            QuestionRecognitionResult {
                bounding_box: QuestionBoundingBox {
                    x: 24,
                    y: 48,
                    width: 640,
                    height: 360,
                },
                confidence: 0.84,
                title: Some("Two Sum".to_string()),
                question_text: Some(
                    "Given an array of integers, return indices of the two numbers that add up to a target."
                        .to_string(),
                ),
                reason: "The main problem statement is centered in the page body.".to_string(),
            }
        );
    }

    #[test]
    fn parse_question_recognition_result_accepts_fenced_json() {
        let response = r#"```json
        {
            "boundingBox": { "x": 12, "y": 18, "width": 320, "height": 180 },
            "confidence": 0.73,
            "title": null,
            "questionText": null,
            "reason": "The panel at the top of the page looks like the most likely question block."
        }
        ```"#;

        let parsed = parse_question_recognition_result(response).expect("parse fenced response");

        assert_eq!(parsed.bounding_box.x, 12);
        assert_eq!(parsed.bounding_box.y, 18);
        assert_eq!(parsed.bounding_box.width, 320);
        assert_eq!(parsed.bounding_box.height, 180);
        assert_eq!(parsed.title, None);
        assert_eq!(parsed.question_text, None);
        assert!(parsed.reason.contains("most likely question block"));
    }

    #[test]
    fn parse_question_recognition_result_rejects_non_integer_coordinates() {
        let response = r#"{
            "boundingBox": { "x": 12.5, "y": 18, "width": 320, "height": 180 },
            "confidence": 0.73,
            "title": null,
            "questionText": null,
            "reason": "The model returned a fractional x coordinate."
        }"#;

        let error = parse_question_recognition_result(response)
            .expect_err("fractional coordinates should be rejected");

        assert!(error.contains("invalid type"));
    }

    #[test]
    fn parsed_coordinates_still_must_fit_the_recognition_image() {
        let image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 1920,
            original_height: 1080,
            width: 960,
            height: 540,
            bytes: vec![1, 2, 3],
        };
        let response = r#"{
            "boundingBox": { "x": 920, "y": 500, "width": 80, "height": 60 },
            "confidence": 0.78,
            "title": "Out of bounds",
            "questionText": null,
            "reason": "The model returned a box that spills past the screenshot."
        }"#;
        let recognition = parse_question_recognition_result(response).expect("parse response");

        let error = validate_question_recognition_result(recognition, &image)
            .expect_err("parsed out-of-bounds boxes should be rejected");

        assert!(error.contains("bounds"));
    }

    #[test]
    fn validate_question_recognition_result_accepts_boxes_inside_the_image() {
        let image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 1920,
            original_height: 1080,
            width: 960,
            height: 540,
            bytes: vec![1, 2, 3],
        };
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 120,
                y: 80,
                width: 360,
                height: 180,
            },
            confidence: 0.91,
            title: Some("Two Sum".to_string()),
            question_text: Some("Given an array of integers...".to_string()),
            reason: "The center panel contains the whole problem statement.".to_string(),
        };

        let validated =
            validate_question_recognition_result(recognition.clone(), &image).expect("validate");

        assert_eq!(validated, recognition);
    }

    #[test]
    fn validate_question_recognition_result_rejects_negative_coordinates() {
        let image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 1920,
            original_height: 1080,
            width: 960,
            height: 540,
            bytes: vec![1, 2, 3],
        };
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: -1,
                y: 20,
                width: 300,
                height: 200,
            },
            confidence: 0.51,
            title: None,
            question_text: None,
            reason: "The box starts before the image origin.".to_string(),
        };

        let error = validate_question_recognition_result(recognition, &image)
            .expect_err("negative coordinates should be rejected");

        assert!(error.contains("bounds"));
    }

    #[test]
    fn validate_question_recognition_result_rejects_boxes_that_extend_beyond_the_image() {
        let image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 1920,
            original_height: 1080,
            width: 960,
            height: 540,
            bytes: vec![1, 2, 3],
        };
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 800,
                y: 460,
                width: 200,
                height: 100,
            },
            confidence: 0.38,
            title: None,
            question_text: None,
            reason: "The box spills past the bottom-right edge.".to_string(),
        };

        let error = validate_question_recognition_result(recognition, &image)
            .expect_err("overflowing box should be rejected");

        assert!(error.contains("bounds"));
    }

    #[test]
    fn validate_question_recognition_result_rejects_zero_sized_boxes() {
        let image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 1920,
            original_height: 1080,
            width: 960,
            height: 540,
            bytes: vec![1, 2, 3],
        };
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 120,
                y: 80,
                width: 0,
                height: 100,
            },
            confidence: 0.2,
            title: None,
            question_text: None,
            reason: "The box is missing width.".to_string(),
        };

        let error = validate_question_recognition_result(recognition, &image)
            .expect_err("zero-sized box should be rejected");

        assert!(error.contains("positive size"));
    }

    #[test]
    fn crop_original_question_region_maps_low_resolution_box_to_high_resolution_crop() {
        let source = image::RgbaImage::from_fn(8, 4, |x, y| {
            image::Rgba([(x * 20) as u8, (y * 60) as u8, 180, 255])
        });
        let recognition_image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 8,
            original_height: 4,
            width: 4,
            height: 2,
            bytes: vec![1, 2, 3],
        };
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 1,
                y: 0,
                width: 2,
                height: 1,
            },
            confidence: 0.88,
            title: Some("Two Sum".to_string()),
            question_text: Some("Find two numbers that add up to the target.".to_string()),
            reason: "The center band contains the visible problem statement.".to_string(),
        };

        let cropped =
            crop_original_question_region(&source, recognition.clone(), &recognition_image)
                .expect("crop high-resolution question region");

        assert_eq!(cropped.recognition, recognition);
        assert_eq!(
            cropped.crop_box,
            QuestionBoundingBox {
                x: 2,
                y: 0,
                width: 4,
                height: 2,
            }
        );
        assert_eq!(cropped.image.width(), 4);
        assert_eq!(cropped.image.height(), 2);
        assert_eq!(cropped.image.get_pixel(0, 0), source.get_pixel(2, 0));
        assert_eq!(cropped.image.get_pixel(3, 1), source.get_pixel(5, 1));
    }

    #[test]
    fn crop_original_question_region_uses_floor_and_ceil_for_fractional_scaling() {
        let source = image::RgbaImage::from_fn(10, 5, |x, y| {
            image::Rgba([(x * 15) as u8, (y * 40) as u8, 120, 255])
        });
        let recognition_image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 10,
            original_height: 5,
            width: 3,
            height: 2,
            bytes: vec![1, 2, 3],
        };
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 1,
                y: 0,
                width: 1,
                height: 1,
            },
            confidence: 0.7,
            title: None,
            question_text: None,
            reason: "The middle block looks like the question area.".to_string(),
        };

        let cropped = crop_original_question_region(&source, recognition, &recognition_image)
            .expect("crop fractional scale region");

        assert_eq!(
            cropped.crop_box,
            QuestionBoundingBox {
                x: 3,
                y: 0,
                width: 4,
                height: 3,
            }
        );
        assert_eq!(cropped.image.width(), 4);
        assert_eq!(cropped.image.height(), 3);
        assert_eq!(cropped.image.get_pixel(0, 0), source.get_pixel(3, 0));
        assert_eq!(cropped.image.get_pixel(3, 2), source.get_pixel(6, 2));
    }

    #[test]
    fn crop_original_question_region_rejects_source_dimension_mismatches() {
        let source = image::RgbaImage::from_pixel(7, 4, image::Rgba([10, 20, 30, 255]));
        let recognition_image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 8,
            original_height: 4,
            width: 4,
            height: 2,
            bytes: vec![1, 2, 3],
        };
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 1,
                y: 0,
                width: 2,
                height: 1,
            },
            confidence: 0.88,
            title: None,
            question_text: None,
            reason: "The box is valid but the source size is wrong.".to_string(),
        };

        let error = crop_original_question_region(&source, recognition, &recognition_image)
            .expect_err("source dimension mismatch should be rejected");

        assert!(error.contains("dimensions"));
    }

    #[test]
    fn crop_original_question_region_rejects_invalid_recognition_boxes() {
        let source = image::RgbaImage::from_pixel(8, 4, image::Rgba([10, 20, 30, 255]));
        let recognition_image = RecognitionImageInput {
            mime_type: RECOGNITION_IMAGE_MIME_TYPE,
            original_width: 8,
            original_height: 4,
            width: 4,
            height: 2,
            bytes: vec![1, 2, 3],
        };
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 3,
                y: 0,
                width: 2,
                height: 1,
            },
            confidence: 0.3,
            title: None,
            question_text: None,
            reason: "The low-resolution box extends past the right edge.".to_string(),
        };

        let error = crop_original_question_region(&source, recognition, &recognition_image)
            .expect_err("invalid recognition box should be rejected");

        assert!(error.contains("bounds"));
    }

    #[test]
    fn recognition_confidence_thresholds_use_stable_defaults() {
        let thresholds = RecognitionConfidenceThresholds::default();

        assert_eq!(
            thresholds.auto_accept_min_confidence,
            DEFAULT_RECOGNITION_AUTO_ACCEPT_CONFIDENCE
        );
        assert_eq!(
            thresholds.confirmation_min_confidence,
            DEFAULT_RECOGNITION_CONFIRMATION_MIN_CONFIDENCE
        );
    }

    #[test]
    fn recognition_confidence_thresholds_repair_invalid_values() {
        let thresholds = RecognitionConfidenceThresholds {
            auto_accept_min_confidence: 1.5,
            confirmation_min_confidence: 0.9,
        };

        let sanitized = thresholds.sanitized();

        assert_eq!(sanitized.auto_accept_min_confidence, 1.0);
        assert_eq!(sanitized.confirmation_min_confidence, 0.9);
    }

    #[test]
    fn recognition_confidence_thresholds_cap_confirmation_at_auto_accept() {
        let thresholds = RecognitionConfidenceThresholds {
            auto_accept_min_confidence: 0.75,
            confirmation_min_confidence: 0.92,
        };

        let sanitized = thresholds.sanitized();

        assert_eq!(sanitized.auto_accept_min_confidence, 0.75);
        assert_eq!(sanitized.confirmation_min_confidence, 0.75);
    }

    #[test]
    fn classification_routes_high_medium_and_low_confidence_scores() {
        let thresholds = RecognitionConfidenceThresholds {
            auto_accept_min_confidence: 0.85,
            confirmation_min_confidence: 0.55,
        };

        assert_eq!(
            super::classify_question_recognition_confidence(0.91, thresholds),
            RecognitionConfidenceRoute::AutoAccept
        );
        assert_eq!(
            super::classify_question_recognition_confidence(0.71, thresholds),
            RecognitionConfidenceRoute::NeedsConfirmation
        );
        assert_eq!(
            super::classify_question_recognition_confidence(0.31, thresholds),
            RecognitionConfidenceRoute::ManualFallback
        );
    }

    #[test]
    fn classification_routes_boundary_scores_to_the_expected_paths() {
        let thresholds = RecognitionConfidenceThresholds {
            auto_accept_min_confidence: 0.85,
            confirmation_min_confidence: 0.55,
        };

        assert_eq!(
            super::classify_question_recognition_confidence(0.85, thresholds),
            RecognitionConfidenceRoute::AutoAccept
        );
        assert_eq!(
            super::classify_question_recognition_confidence(0.55, thresholds),
            RecognitionConfidenceRoute::NeedsConfirmation
        );
        assert_eq!(
            super::classify_question_recognition_confidence(0.5499, thresholds),
            RecognitionConfidenceRoute::ManualFallback
        );
    }

    #[test]
    fn classification_clamps_invalid_confidence_scores_before_routing() {
        let thresholds = RecognitionConfidenceThresholds {
            auto_accept_min_confidence: 0.85,
            confirmation_min_confidence: 0.55,
        };

        assert_eq!(
            super::classify_question_recognition_confidence(1.4, thresholds),
            RecognitionConfidenceRoute::AutoAccept
        );
        assert_eq!(
            super::classify_question_recognition_confidence(-0.2, thresholds),
            RecognitionConfidenceRoute::ManualFallback
        );
        assert_eq!(
            super::classify_question_recognition_confidence(f64::NAN, thresholds),
            RecognitionConfidenceRoute::ManualFallback
        );
    }

    #[test]
    fn recognition_result_exposes_the_same_route_classification_helper() {
        let recognition = QuestionRecognitionResult {
            bounding_box: QuestionBoundingBox {
                x: 0,
                y: 0,
                width: 128,
                height: 64,
            },
            confidence: 0.68,
            title: None,
            question_text: None,
            reason: "The centered block looks like a question statement.".to_string(),
        };
        let thresholds = RecognitionConfidenceThresholds::default();

        assert_eq!(
            recognition.confidence_route(thresholds),
            RecognitionConfidenceRoute::NeedsConfirmation
        );
    }

    #[test]
    fn recognition_confidence_route_serializes_to_camel_case() {
        assert_eq!(
            serde_json::to_value(RecognitionConfidenceRoute::AutoAccept).expect("serialize route"),
            json!("autoAccept")
        );
        assert_eq!(
            serde_json::to_value(RecognitionConfidenceRoute::NeedsConfirmation)
                .expect("serialize route"),
            json!("needsConfirmation")
        );
        assert_eq!(
            serde_json::to_value(RecognitionConfidenceRoute::ManualFallback)
                .expect("serialize route"),
            json!("manualFallback")
        );
    }
}
