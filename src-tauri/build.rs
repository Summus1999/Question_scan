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
            "regenerate_with_language",
            "clear_cache",
            "list_history",
            "save_history_entry",
            "delete_history_entry",
            "clear_history",
            "list_leetcode_problem_index",
            "import_rag_documents",
            "list_rag_imports",
            "delete_rag_import",
            "clear_rag_index",
            "rebuild_rag_index",
            "search_rag_context",
            "select_rag_template_context",
            "build_rag_prompt_context",
        ]),
    ))
    .expect("failed to run tauri build script");
}
