/**
 * Question Scan 会话上下文模块（阶段 6）。
 *
 * 职责：保存最近一次截图→AI 流水线的数据，支持语言切换和重新生成时复用。
 * 使用 RwLock 保护数据，支持多线程安全访问。
 * 关键用途：用户切换语言后无需重新截图，直接复用已保存的图片和识别文本。
 */
use crate::language::{LanguageId, PlatformFormat};
use std::sync::RwLock;

/** 保存最近一次成功流水线的数据，支持语言切换和重新生成时复用图片与识别文本，无需重新截图。 */
/// Holds the data from the most recent successful screenshot-to-AI pipeline so that
/// language switching and regeneration can reuse the same cropped image and recognized
/// text without re-capturing the screen.
#[derive(Debug, Default)]
pub struct SessionContext {
    /// The compressed image bytes that were sent to the AI in the last request.
    image_bytes: RwLock<Option<Vec<u8>>>,
    /// The MIME type of the compressed image.
    image_mime_type: RwLock<Option<String>>,
    /// The problem title extracted during recognition, if any.
    recognized_title: RwLock<Option<String>>,
    /// The problem text extracted during recognition, if any.
    recognized_text: RwLock<Option<String>>,
    /// The platform format used in the last request.
    platform_format: RwLock<Option<PlatformFormat>>,
    /// The language used in the last request.
    last_language: RwLock<Option<LanguageId>>,
}

impl SessionContext {
    /** 保存截图和识别流水线的数据。 */
    /// Stores the data from a completed screenshot-and-recognition pipeline.
    pub fn save_request_data(
        &self,
        image_bytes: Vec<u8>,
        image_mime_type: impl Into<String>,
        recognized_title: Option<impl Into<String>>,
        recognized_text: Option<impl Into<String>>,
        platform_format: PlatformFormat,
        language: LanguageId,
    ) {
        *self.image_bytes.write().expect("image bytes lock poisoned") = Some(image_bytes);
        *self.image_mime_type.write().expect("mime type lock poisoned") =
            Some(image_mime_type.into());
        *self.recognized_title.write().expect("title lock poisoned") =
            recognized_title.map(Into::into);
        *self.recognized_text.write().expect("text lock poisoned") =
            recognized_text.map(Into::into);
        *self.platform_format.write().expect("platform lock poisoned") = Some(platform_format);
        *self.last_language.write().expect("language lock poisoned") = Some(language);
    }

    /** 返回上次请求保存的图片字节（如果有）。 */
    /// Returns the image bytes if a previous request was saved.
    pub fn image_bytes(&self) -> Option<Vec<u8>> {
        self.image_bytes.read().expect("image bytes lock poisoned").clone()
    }

    /** 返回上次请求保存的图片 MIME 类型（如果有）。 */
    /// Returns the image MIME type if a previous request was saved.
    pub fn image_mime_type(&self) -> Option<String> {
        self.image_mime_type
            .read()
            .expect("mime type lock poisoned")
            .clone()
    }

    /** 返回上次请求保存的识别标题（如果有）。 */
    /// Returns the recognized title if a previous request was saved.
    pub fn recognized_title(&self) -> Option<String> {
        self.recognized_title
            .read()
            .expect("title lock poisoned")
            .clone()
    }

    /** 返回上次请求保存的识别文本（如果有）。 */
    /// Returns the recognized text if a previous request was saved.
    pub fn recognized_text(&self) -> Option<String> {
        self.recognized_text
            .read()
            .expect("text lock poisoned")
            .clone()
    }

    /** 返回上次请求保存的平台格式（如果有）。 */
    /// Returns the platform format if a previous request was saved.
    pub fn platform_format(&self) -> Option<PlatformFormat> {
        *self.platform_format.read().expect("platform lock poisoned")
    }

    /** 返回上次请求保存的语言（如果有）。 */
    /// Returns the last language if a previous request was saved.
    pub fn last_language(&self) -> Option<LanguageId> {
        *self.last_language.read().expect("language lock poisoned")
    }

    /** 清空所有保存的会话数据。 */
    /// Clears all saved session data.
    pub fn clear(&self) {
        *self.image_bytes.write().expect("image bytes lock poisoned") = None;
        *self.image_mime_type.write().expect("mime type lock poisoned") = None;
        *self.recognized_title.write().expect("title lock poisoned") = None;
        *self.recognized_text.write().expect("text lock poisoned") = None;
        *self.platform_format.write().expect("platform lock poisoned") = None;
        *self.last_language.write().expect("language lock poisoned") = None;
    }

    /** 判断是否拥有足够数据以使用新语言重新生成（需要图片字节和 MIME 类型）。 */
    /// Returns true if enough data is present to regenerate with a new language.
    pub fn can_regenerate(&self) -> bool {
        self.image_bytes().is_some() && self.image_mime_type().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_starts_empty() {
        let session = SessionContext::default();
        assert!(!session.can_regenerate());
        assert_eq!(session.image_bytes(), None);
        assert_eq!(session.image_mime_type(), None);
        assert_eq!(session.recognized_title(), None);
        assert_eq!(session.recognized_text(), None);
        assert_eq!(session.platform_format(), None);
        assert_eq!(session.last_language(), None);
    }

    #[test]
    fn save_and_retrieve_full_session() {
        let session = SessionContext::default();
        session.save_request_data(
            vec![1, 2, 3],
            "image/jpeg",
            Some("Two Sum"),
            Some("Find two numbers that add up to target."),
            PlatformFormat::Acm,
            LanguageId::Cpp20,
        );

        assert!(session.can_regenerate());
        assert_eq!(session.image_bytes(), Some(vec![1, 2, 3]));
        assert_eq!(session.image_mime_type(), Some("image/jpeg".to_string()));
        assert_eq!(session.recognized_title(), Some("Two Sum".to_string()));
        assert_eq!(
            session.recognized_text(),
            Some("Find two numbers that add up to target.".to_string())
        );
        assert_eq!(session.platform_format(), Some(PlatformFormat::Acm));
        assert_eq!(session.last_language(), Some(LanguageId::Cpp20));
    }

    #[test]
    fn save_with_none_title_and_text() {
        let session = SessionContext::default();
        session.save_request_data(
            vec![4, 5, 6],
            "image/png",
            None::<&str>,
            None::<&str>,
            PlatformFormat::LeetCode,
            LanguageId::Python,
        );

        assert!(session.can_regenerate());
        assert_eq!(session.recognized_title(), None);
        assert_eq!(session.recognized_text(), None);
        assert_eq!(session.platform_format(), Some(PlatformFormat::LeetCode));
        assert_eq!(session.last_language(), Some(LanguageId::Python));
    }

    #[test]
    fn clear_resets_all_fields() {
        let session = SessionContext::default();
        session.save_request_data(
            vec![1],
            "image/jpeg",
            Some("Title"),
            Some("Text"),
            PlatformFormat::Generic,
            LanguageId::Rust,
        );

        session.clear();

        assert!(!session.can_regenerate());
        assert_eq!(session.image_bytes(), None);
        assert_eq!(session.image_mime_type(), None);
        assert_eq!(session.recognized_title(), None);
        assert_eq!(session.recognized_text(), None);
        assert_eq!(session.platform_format(), None);
        assert_eq!(session.last_language(), None);
    }

    #[test]
    fn can_regenerate_requires_both_image_bytes_and_mime_type() {
        let session = SessionContext::default();
        assert!(!session.can_regenerate());

        // Only bytes, no mime type
        *session.image_bytes.write().unwrap() = Some(vec![1]);
        assert!(!session.can_regenerate());

        // Both present
        *session.image_mime_type.write().unwrap() = Some("image/jpeg".to_string());
        assert!(session.can_regenerate());
    }

    #[test]
    fn update_overwrites_previous_data() {
        let session = SessionContext::default();
        session.save_request_data(
            vec![1],
            "image/jpeg",
            Some("Old Title"),
            Some("Old Text"),
            PlatformFormat::Acm,
            LanguageId::Cpp17,
        );

        session.save_request_data(
            vec![2, 3],
            "image/png",
            Some("New Title"),
            None::<&str>,
            PlatformFormat::LeetCode,
            LanguageId::Java,
        );

        assert_eq!(session.image_bytes(), Some(vec![2, 3]));
        assert_eq!(session.image_mime_type(), Some("image/png".to_string()));
        assert_eq!(session.recognized_title(), Some("New Title".to_string()));
        assert_eq!(session.recognized_text(), None);
        assert_eq!(session.platform_format(), Some(PlatformFormat::LeetCode));
        assert_eq!(session.last_language(), Some(LanguageId::Java));
    }
}
