/// App commands. From this list Tauri generates an `allow-<command>` permission for each and
/// rejects any app command no capability grants: today only `capabilities/default.json` grants
/// them, to the `main` window; `extractor` reaches none.
/// Must match `generate_handler!` in `src/lib.rs` (a test in `lib.rs` checks it).
const COMMANDS: &[&str] = &[
    "list",
    "search",
    "get",
    "bookmark_favicon",
    "add",
    "update",
    "delete",
    "retry",
    "tag_groups",
    "create_tag_group",
    "rename_tag_group",
    "delete_tag_group",
    "set_tag_group",
    "get_settings",
    "save_settings",
    "save_extraction_settings",
    "test_connection",
    "reextract",
    "extract_with_ai",
    "has_extraction_model",
    "reindex",
    "process_unsummarized",
    "stats",
    "api_info",
    "regenerate_api_token",
    "profiles",
    "create_profile",
    "rename_profile",
    "delete_profile",
    "switch_profile",
    "move_bookmark",
    "extracting",
    "notifications",
    "push_notification",
    "mark_notifications_read",
    "dismiss_notification",
    "clear_notifications",
    "import_preview",
    "import_confirm",
    "import_cancel",
    "export_bookmarks",
    "extract_imported",
    "locale_info",
    "set_locale",
    "update_info",
    "check_update",
    "install_update",
    "set_update_auto",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))?;
    Ok(())
}
