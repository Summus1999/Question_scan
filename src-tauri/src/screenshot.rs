//! Screenshot backend selection for stage 3.
//!
//! This wrapper keeps the Windows-capable crate in one place so later capture
//! and temporary-file work can build on a stable entry point.

use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

pub(crate) const SCREENSHOT_BACKEND: &str = "screenshots";

pub(crate) use screenshots::{display_info::DisplayInfo, image, Screen};

/// Chooses whether the capture flow should use the current display or every display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenSelection {
    CurrentDisplay,
    AllDisplays,
}

/// A capture result mapped into the shared virtual desktop coordinate space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// The origin and extent of the unified virtual desktop layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopLayout {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
}

impl DesktopLayout {
    /// Builds the shared desktop layout from a set of screens.
    pub fn from_screens(screens: &[Screen]) -> Option<Self> {
        let first = screens.first()?;
        let mut min_x = first.display_info.x;
        let mut min_y = first.display_info.y;
        let mut max_x = i64::from(first.display_info.x) + i64::from(first.display_info.width);
        let mut max_y = i64::from(first.display_info.y) + i64::from(first.display_info.height);

        for screen in &screens[1..] {
            let display_info = screen.display_info;
            min_x = min_x.min(display_info.x);
            min_y = min_y.min(display_info.y);
            max_x = max_x.max(i64::from(display_info.x) + i64::from(display_info.width));
            max_y = max_y.max(i64::from(display_info.y) + i64::from(display_info.height));
        }

        Some(Self {
            origin_x: min_x,
            origin_y: min_y,
            width: (max_x - i64::from(min_x)).clamp(0, i64::from(u32::MAX)) as u32,
            height: (max_y - i64::from(min_y)).clamp(0, i64::from(u32::MAX)) as u32,
        })
    }

    /// Converts a virtual desktop point into the normalized layout space.
    pub fn normalize_point(&self, point: (i32, i32)) -> (i32, i32) {
        (
            translate_coordinate(point.0, self.origin_x),
            translate_coordinate(point.1, self.origin_y),
        )
    }

    /// Converts a screen's global coordinates into the shared desktop layout space.
    pub fn normalize_screen_rect(&self, display_info: DisplayInfo) -> DesktopRect {
        let (x, y) = self.normalize_point((display_info.x, display_info.y));

        DesktopRect {
            x,
            y,
            width: display_info.width,
            height: display_info.height,
        }
    }
}

/// Controls how screenshots are re-encoded before being sent to the AI service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenshotCompressionConfig {
    pub max_long_edge: u32,
    pub jpeg_quality: u8,
}

impl ScreenshotCompressionConfig {
    /// Repairs invalid compression settings so downstream encoding stays predictable.
    pub fn sanitized(self) -> Self {
        Self {
            max_long_edge: self.max_long_edge.max(1),
            jpeg_quality: self.jpeg_quality.clamp(1, 100),
        }
    }
}

/// A compressed image payload ready for AI upload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedImage {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

/// A crop rectangle normalized into local capture coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// A serializable screenshot summary for frontend state and AI request handoff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotMetadata {
    pub image_path: String,
    pub width: u32,
    pub height: u32,
    pub display_id: u32,
    pub captured_at_unix_ms: u64,
}

/// Captures a screen together with its display metadata.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CapturedScreen {
    pub display_info: DisplayInfo,
    pub desktop_rect: DesktopRect,
    pub captured_at: SystemTime,
    pub temp_path: PathBuf,
    pub image: image::RgbaImage,
}

impl CapturedScreen {
    /// Converts the in-memory capture result into the small object exposed to other layers.
    pub fn metadata(&self) -> ScreenshotMetadata {
        ScreenshotMetadata {
            image_path: self.temp_path.display().to_string(),
            width: self.image.width(),
            height: self.image.height(),
            display_id: self.display_info.id,
            captured_at_unix_ms: system_time_to_unix_ms(self.captured_at),
        }
    }
}

#[allow(dead_code)]
pub type ScreenshotResult<T> = Result<T, String>;

/// Captures the configured set of displays and returns raw images for later cropping and saving.
#[allow(dead_code)]
pub fn capture_screens(
    selection: ScreenSelection,
    anchor: Option<(i32, i32)>,
) -> ScreenshotResult<Vec<CapturedScreen>> {
    let screens = Screen::all().map_err(|error| error.to_string())?;
    let desktop_layout = DesktopLayout::from_screens(&screens)
        .ok_or_else(|| "No screens are available to capture".to_string())?;
    let selected_screens = select_screens(&screens, selection, anchor);
    let capture_root = ensure_screenshot_temp_root(&std::env::temp_dir())?;
    let captured_at = SystemTime::now();

    selected_screens
        .into_iter()
        .enumerate()
        .map(|(index, screen)| {
            capture_screen(screen, &desktop_layout, captured_at, index, &capture_root)
        })
        .collect()
}

/// Captures the configured displays and returns only serializable screenshot metadata.
#[allow(dead_code)]
pub fn capture_screens_metadata(
    selection: ScreenSelection,
    anchor: Option<(i32, i32)>,
) -> ScreenshotResult<Vec<ScreenshotMetadata>> {
    Ok(capture_screens(selection, anchor)?
        .into_iter()
        .map(|captured_screen| captured_screen.metadata())
        .collect())
}

/// Deletes the temporary files for a capture batch after the AI request has finished.
pub(crate) fn cleanup_captured_screens(
    captured_screens: &[CapturedScreen],
) -> ScreenshotResult<()> {
    for captured_screen in captured_screens {
        if let Err(error) = fs::remove_file(&captured_screen.temp_path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                return Err(error.to_string());
            }
        }
    }

    Ok(())
}

/// Converts a virtual-desktop crop rectangle into capture-local coordinates after bounds checks.
#[allow(dead_code)]
pub(crate) fn normalize_crop_rect(
    captured_screen: &CapturedScreen,
    crop_rect: DesktopRect,
) -> ScreenshotResult<CropRect> {
    if crop_rect.width == 0 || crop_rect.height == 0 {
        return Err("Crop rectangle must have a positive size".to_string());
    }

    let capture_left = i64::from(captured_screen.desktop_rect.x);
    let capture_top = i64::from(captured_screen.desktop_rect.y);
    let capture_right = capture_left + i64::from(captured_screen.desktop_rect.width);
    let capture_bottom = capture_top + i64::from(captured_screen.desktop_rect.height);

    let crop_left = i64::from(crop_rect.x);
    let crop_top = i64::from(crop_rect.y);
    let crop_right = crop_left + i64::from(crop_rect.width);
    let crop_bottom = crop_top + i64::from(crop_rect.height);

    if crop_left < capture_left
        || crop_top < capture_top
        || crop_right > capture_right
        || crop_bottom > capture_bottom
    {
        return Err("Crop rectangle exceeds capture bounds".to_string());
    }

    Ok(CropRect {
        x: (crop_left - capture_left) as u32,
        y: (crop_top - capture_top) as u32,
        width: crop_rect.width,
        height: crop_rect.height,
    })
}

/// Picks the screens that match the requested selection policy.
pub(crate) fn select_screens(
    screens: &[Screen],
    selection: ScreenSelection,
    anchor: Option<(i32, i32)>,
) -> Vec<Screen> {
    match selection {
        ScreenSelection::AllDisplays => screens.to_vec(),
        ScreenSelection::CurrentDisplay => {
            select_current_screen(screens, anchor).into_iter().collect()
        }
    }
}

#[allow(dead_code)]
fn capture_screen(
    screen: Screen,
    desktop_layout: &DesktopLayout,
    captured_at: SystemTime,
    index: usize,
    capture_root: &Path,
) -> ScreenshotResult<CapturedScreen> {
    let image = screen.capture().map_err(|error| error.to_string())?;
    let temp_path = temp_capture_path(capture_root, screen.display_info.id, captured_at, index);
    save_image_to_temp_file(&image, &temp_path)?;

    Ok(CapturedScreen {
        display_info: screen.display_info,
        desktop_rect: desktop_layout.normalize_screen_rect(screen.display_info),
        captured_at,
        temp_path,
        image,
    })
}

fn select_current_screen(screens: &[Screen], anchor: Option<(i32, i32)>) -> Option<Screen> {
    let anchor = anchor.unwrap_or((0, 0));

    screens
        .iter()
        .copied()
        .find(|screen| screen_contains_point(screen, anchor))
        .or_else(|| screens.first().copied())
}

fn screen_contains_point(screen: &Screen, (x, y): (i32, i32)) -> bool {
    let display_info = screen.display_info;
    let right = i64::from(display_info.x) + i64::from(display_info.width);
    let bottom = i64::from(display_info.y) + i64::from(display_info.height);
    let x = i64::from(x);
    let y = i64::from(y);

    x >= i64::from(display_info.x) && x < right && y >= i64::from(display_info.y) && y < bottom
}

fn translate_coordinate(value: i32, origin: i32) -> i32 {
    (i64::from(value) - i64::from(origin)).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn system_time_to_unix_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

pub(crate) fn screenshot_temp_root(base_dir: impl AsRef<Path>) -> PathBuf {
    base_dir.as_ref().join("question-scan")
}

pub(crate) fn temp_capture_path(
    capture_root: impl AsRef<Path>,
    display_id: u32,
    captured_at: SystemTime,
    index: usize,
) -> PathBuf {
    let timestamp = captured_at
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    capture_root.as_ref().join(format!(
        "capture-{timestamp}-display-{display_id}-{index}.png"
    ))
}

pub(crate) fn ensure_screenshot_temp_root(base_dir: impl AsRef<Path>) -> ScreenshotResult<PathBuf> {
    let capture_root = screenshot_temp_root(base_dir);
    fs::create_dir_all(&capture_root).map_err(|error| error.to_string())?;
    Ok(capture_root)
}

pub(crate) fn save_image_to_temp_file(
    image: &image::RgbaImage,
    temp_path: impl AsRef<Path>,
) -> ScreenshotResult<()> {
    let mut buffer = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image.clone())
        .write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|error| error.to_string())?;
    fs::write(temp_path.as_ref(), buffer.into_inner()).map_err(|error| error.to_string())
}

/// Re-encodes a screenshot into a compact JPEG payload for AI requests.
#[allow(dead_code)]
pub(crate) fn compress_image_for_ai(
    image: &image::RgbaImage,
    config: ScreenshotCompressionConfig,
) -> ScreenshotResult<CompressedImage> {
    let config = config.sanitized();
    let resized = resize_for_ai(image, config.max_long_edge);
    let width = resized.width();
    let height = resized.height();
    let mut buffer = Cursor::new(Vec::new());

    resized
        .write_to(
            &mut buffer,
            image::ImageOutputFormat::Jpeg(config.jpeg_quality),
        )
        .map_err(|error| error.to_string())?;

    Ok(CompressedImage {
        width,
        height,
        bytes: buffer.into_inner(),
    })
}

fn resize_for_ai(image: &image::RgbaImage, max_long_edge: u32) -> image::DynamicImage {
    let source = image::DynamicImage::ImageRgba8(image.clone());
    let width = source.width();
    let height = source.height();
    let long_edge = width.max(height);

    if long_edge == 0 || long_edge <= max_long_edge {
        return source;
    }

    let scale = max_long_edge as f64 / long_edge as f64;
    let target_width = ((width as f64 * scale).round() as u32).max(1);
    let target_height = ((height as f64 * scale).round() as u32).max(1);

    source.resize_exact(
        target_width,
        target_height,
        image::imageops::FilterType::Triangle,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        cleanup_captured_screens, compress_image_for_ai, image, normalize_crop_rect,
        save_image_to_temp_file, screenshot_temp_root, select_screens, temp_capture_path,
        CapturedScreen, CropRect, DesktopLayout, DesktopRect, DisplayInfo, Screen, ScreenSelection,
        ScreenshotCompressionConfig, ScreenshotMetadata, SCREENSHOT_BACKEND,
    };
    use serde_json::json;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn fake_display_info(
        id: u32,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        is_primary: bool,
    ) -> DisplayInfo {
        DisplayInfo {
            id,
            raw_handle: unsafe { std::mem::zeroed() },
            x,
            y,
            width,
            height,
            rotation: 0.0,
            scale_factor: 1.0,
            frequency: 60.0,
            is_primary,
        }
    }

    fn fake_screen(id: u32, x: i32, y: i32, width: u32, height: u32, is_primary: bool) -> Screen {
        Screen {
            display_info: fake_display_info(id, x, y, width, height, is_primary),
        }
    }

    #[test]
    fn selects_the_screenshots_backend() {
        assert_eq!(SCREENSHOT_BACKEND, "screenshots");
    }

    #[test]
    fn reexports_the_image_types() {
        let image = image::RgbaImage::new(1, 1);
        assert_eq!(image.width(), 1);
        assert_eq!(image.height(), 1);
    }

    #[test]
    fn reexports_the_screen_type() {
        let screen: Option<Screen> = None;
        assert!(screen.is_none());
    }

    #[test]
    fn current_display_selection_chooses_the_screen_covering_the_anchor_point() {
        let screens = vec![
            fake_screen(1, 0, 0, 1920, 1080, true),
            fake_screen(2, 1920, 0, 1600, 900, false),
        ];

        let selected = select_screens(&screens, ScreenSelection::CurrentDisplay, Some((2000, 40)));

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].display_info.id, 2);
    }

    #[test]
    fn current_display_selection_falls_back_to_the_first_screen() {
        let screens = vec![
            fake_screen(1, 0, 0, 1920, 1080, true),
            fake_screen(2, 1920, 0, 1600, 900, false),
        ];

        let selected = select_screens(&screens, ScreenSelection::CurrentDisplay, Some((5000, 40)));

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].display_info.id, 1);
    }

    #[test]
    fn all_display_selection_keeps_the_full_set() {
        let screens = vec![
            fake_screen(1, 0, 0, 1920, 1080, true),
            fake_screen(2, 1920, 0, 1600, 900, false),
        ];

        let selected = select_screens(&screens, ScreenSelection::AllDisplays, None);

        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].display_info.id, 1);
        assert_eq!(selected[1].display_info.id, 2);
    }

    #[test]
    fn desktop_layout_uses_the_virtual_desktop_origin() {
        let screens = vec![
            fake_screen(1, -1920, 0, 1920, 1080, true),
            fake_screen(2, 0, -900, 1600, 900, false),
            fake_screen(3, 0, 0, 2560, 1440, false),
        ];

        let layout = DesktopLayout::from_screens(&screens).expect("layout");

        assert_eq!(layout.origin_x, -1920);
        assert_eq!(layout.origin_y, -900);
        assert_eq!(layout.width, 4480);
        assert_eq!(layout.height, 2340);
    }

    #[test]
    fn desktop_layout_normalizes_screen_rectangles_against_the_same_origin() {
        let screens = vec![
            fake_screen(1, -1920, 0, 1920, 1080, true),
            fake_screen(2, 0, -900, 1600, 900, false),
        ];

        let layout = DesktopLayout::from_screens(&screens).expect("layout");
        let left = layout.normalize_screen_rect(screens[0].display_info);
        let top = layout.normalize_screen_rect(screens[1].display_info);

        assert_eq!(
            left,
            DesktopRect {
                x: 0,
                y: 900,
                width: 1920,
                height: 1080,
            }
        );
        assert_eq!(
            top,
            DesktopRect {
                x: 1920,
                y: 0,
                width: 1600,
                height: 900,
            }
        );
        assert_eq!(layout.normalize_point((15, 20)), (1935, 920));
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();

        std::env::current_dir()
            .expect("cwd")
            .join("target")
            .join("question-scan-tests")
            .join(format!("{name}-{nonce}"))
    }

    #[test]
    fn screenshot_temp_root_uses_the_question_scan_workspace() {
        let root = screenshot_temp_root("C:/Temp");
        assert!(root.ends_with("question-scan"));
    }

    #[test]
    fn temp_capture_path_uses_png_and_keeps_display_identity() {
        let capture_root = std::path::PathBuf::from(r"C:\Temp\question-scan");
        let captured_at = UNIX_EPOCH + std::time::Duration::from_secs(123);

        let path = temp_capture_path(&capture_root, 77, captured_at, 2);

        assert_eq!(
            path,
            capture_root.join("capture-123000000000-display-77-2.png")
        );
    }

    #[test]
    fn save_image_to_temp_file_writes_a_valid_png() {
        let root = temp_root("temp-image");
        fs::create_dir_all(&root).expect("create root");

        let temp_path = root.join("capture.png");
        let image = image::RgbaImage::from_pixel(2, 1, image::Rgba([12, 34, 56, 255]));

        save_image_to_temp_file(&image, &temp_path).expect("save temp image");

        let bytes = fs::read(&temp_path).expect("read temp image");
        let decoded = image::load_from_memory(&bytes).expect("decode temp image");

        assert_eq!(decoded.width(), 2);
        assert_eq!(decoded.height(), 1);

        fs::remove_file(&temp_path).expect("remove temp image");
        fs::remove_dir_all(&root).expect("remove temp root");
    }

    #[test]
    fn screenshot_compression_config_repairs_invalid_values() {
        let config = ScreenshotCompressionConfig {
            max_long_edge: 0,
            jpeg_quality: 250,
        };

        let sanitized = config.sanitized();

        assert_eq!(sanitized.max_long_edge, 1);
        assert_eq!(sanitized.jpeg_quality, 100);
    }

    #[test]
    fn compress_image_for_ai_scales_down_and_encodes_jpeg() {
        let image = image::RgbaImage::from_fn(4, 2, |x, y| {
            image::Rgba([(x * 40) as u8, (y * 120) as u8, 180, 255])
        });

        let compressed = compress_image_for_ai(
            &image,
            ScreenshotCompressionConfig {
                max_long_edge: 2,
                jpeg_quality: 80,
            },
        )
        .expect("compress image");

        assert_eq!(compressed.width, 2);
        assert_eq!(compressed.height, 1);
        assert!(!compressed.bytes.is_empty());

        let decoded = image::load_from_memory(&compressed.bytes).expect("decode jpeg");
        assert_eq!(decoded.width(), 2);
        assert_eq!(decoded.height(), 1);
    }

    #[test]
    fn captured_screen_metadata_contains_path_size_display_and_timestamp() {
        let captured_at = UNIX_EPOCH + std::time::Duration::from_secs(123);
        let temp_path = std::path::PathBuf::from(r"C:\Temp\question-scan\capture.png");
        let captured_screen = CapturedScreen {
            display_info: fake_display_info(77, 0, 0, 4, 3, true),
            desktop_rect: DesktopRect {
                x: 0,
                y: 0,
                width: 4,
                height: 3,
            },
            captured_at,
            temp_path: temp_path.clone(),
            image: image::RgbaImage::from_pixel(4, 3, image::Rgba([1, 2, 3, 255])),
        };

        let metadata = captured_screen.metadata();

        assert_eq!(
            metadata,
            ScreenshotMetadata {
                image_path: temp_path.display().to_string(),
                width: 4,
                height: 3,
                display_id: 77,
                captured_at_unix_ms: 123_000,
            }
        );

        let serialized = serde_json::to_value(&metadata).expect("serialize metadata");
        assert_eq!(
            serialized,
            json!({
                "imagePath": temp_path.display().to_string(),
                "width": 4,
                "height": 3,
                "displayId": 77,
                "capturedAtUnixMs": 123_000u64,
            })
        );
    }

    #[test]
    fn cleanup_captured_screens_deletes_temp_files() {
        let root = temp_root("cleanup-batch");
        fs::create_dir_all(&root).expect("create root");

        let first_path = root.join("capture-1.png");
        let second_path = root.join("capture-2.png");
        fs::write(&first_path, b"first temp file").expect("write first temp file");
        fs::write(&second_path, b"second temp file").expect("write second temp file");

        let captured_screens = vec![
            CapturedScreen {
                display_info: fake_display_info(11, 0, 0, 1, 1, true),
                desktop_rect: DesktopRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                captured_at: UNIX_EPOCH + std::time::Duration::from_secs(1),
                temp_path: first_path.clone(),
                image: image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255])),
            },
            CapturedScreen {
                display_info: fake_display_info(12, 0, 0, 1, 1, false),
                desktop_rect: DesktopRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                captured_at: UNIX_EPOCH + std::time::Duration::from_secs(2),
                temp_path: second_path.clone(),
                image: image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255])),
            },
        ];

        cleanup_captured_screens(&captured_screens).expect("cleanup captured screens");

        assert!(!first_path.exists());
        assert!(!second_path.exists());

        fs::remove_dir_all(&root).expect("remove temp root");
    }

    #[test]
    fn cleanup_captured_screens_ignores_missing_files() {
        let root = temp_root("cleanup-missing");
        fs::create_dir_all(&root).expect("create root");

        let existing_path = root.join("existing.png");
        let missing_path = root.join("missing.png");
        fs::write(&existing_path, b"temporary content").expect("write temp file");

        let captured_screens = vec![
            CapturedScreen {
                display_info: fake_display_info(21, 0, 0, 1, 1, true),
                desktop_rect: DesktopRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                captured_at: UNIX_EPOCH + std::time::Duration::from_secs(3),
                temp_path: existing_path.clone(),
                image: image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255])),
            },
            CapturedScreen {
                display_info: fake_display_info(22, 0, 0, 1, 1, false),
                desktop_rect: DesktopRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                captured_at: UNIX_EPOCH + std::time::Duration::from_secs(4),
                temp_path: missing_path.clone(),
                image: image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255])),
            },
        ];

        cleanup_captured_screens(&captured_screens).expect("cleanup captured screens");

        assert!(!existing_path.exists());
        assert!(!missing_path.exists());

        fs::remove_dir_all(&root).expect("remove temp root");
    }

    #[test]
    fn normalize_crop_rect_translates_capture_coordinates_into_local_space() {
        let captured_screen = CapturedScreen {
            display_info: fake_display_info(31, -1920, 0, 1920, 1080, true),
            desktop_rect: DesktopRect {
                x: -1920,
                y: 0,
                width: 1920,
                height: 1080,
            },
            captured_at: UNIX_EPOCH + std::time::Duration::from_secs(5),
            temp_path: std::path::PathBuf::from(r"C:\Temp\question-scan\capture.png"),
            image: image::RgbaImage::from_pixel(1920, 1080, image::Rgba([0, 0, 0, 255])),
        };

        let normalized = normalize_crop_rect(
            &captured_screen,
            DesktopRect {
                x: -1800,
                y: 120,
                width: 640,
                height: 360,
            },
        )
        .expect("normalize crop rect");

        assert_eq!(
            normalized,
            CropRect {
                x: 120,
                y: 120,
                width: 640,
                height: 360,
            }
        );
    }

    #[test]
    fn normalize_crop_rect_rejects_out_of_bounds_rectangles() {
        let captured_screen = CapturedScreen {
            display_info: fake_display_info(32, 0, 0, 1280, 720, true),
            desktop_rect: DesktopRect {
                x: 0,
                y: 0,
                width: 1280,
                height: 720,
            },
            captured_at: UNIX_EPOCH + std::time::Duration::from_secs(6),
            temp_path: std::path::PathBuf::from(r"C:\Temp\question-scan\capture.png"),
            image: image::RgbaImage::from_pixel(1280, 720, image::Rgba([0, 0, 0, 255])),
        };

        let error = normalize_crop_rect(
            &captured_screen,
            DesktopRect {
                x: 1200,
                y: 680,
                width: 120,
                height: 60,
            },
        )
        .expect_err("reject out-of-bounds crop");

        assert!(error.contains("bounds"));
    }

    #[test]
    fn ensure_screenshot_temp_root_errors_when_root_is_a_file() {
        let root = temp_root("occupied-root");
        fs::create_dir_all(&root).expect("create root");

        let occupied_path = root.join("question-scan");
        fs::write(&occupied_path, b"not a directory").expect("write occupied path");

        let error = super::ensure_screenshot_temp_root(&root).expect_err("root file should fail");

        assert!(!error.is_empty());

        fs::remove_file(&occupied_path).expect("remove occupied path");
        fs::remove_dir_all(&root).expect("remove temp root");
    }
}
