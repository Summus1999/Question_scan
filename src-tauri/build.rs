// Registers command permissions so Tauri can expose the Rust API to the frontend.
fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "load_app_state",
            "save_settings",
            "reset_settings",
            "show_main_window",
            "hide_main_window",
            "toggle_main_window",
            "set_tray_status",
            "send_ai_request",
            "clear_cache",
            "list_history",
            "delete_history_entry",
            "clear_history",
        ]),
    ))
    .expect("failed to run tauri build script");
}
