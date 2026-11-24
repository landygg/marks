mod api;
mod commands;
mod extracting;
mod locale;
// Tauri only sets its default menu on macOS, and so do we.
#[cfg(target_os = "macos")]
mod menu;
mod notifications;
mod parser_window;
mod state;
mod status;
mod tray;
mod updater;
mod webview_fetcher;
mod worker;

use marks_i18n::{AppText, Localize};
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> anyhow::Result<()> {
    // First of all: the tray, the menu and the window are born with text in the resolved language.
    marks_i18n::init(locale::startup());
    // Before the builder: a failure opening the database returns to `main` with its context, no panic.
    let state = AppState::open().inspect_err(|e| {
        // Opened from Finder, nobody sees stderr: say it in a native alert before quitting.
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title(AppText::WindowTitle.localize())
            .set_description(marks_i18n::error(e))
            .show();
    })?;
    let builder = tauri::Builder::default();
    // Replaces Tauri's default; built in `build`, once the language is set.
    #[cfg(target_os = "macos")]
    let builder = builder.menu(menu::build);
    builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        // In `setup` the logger is already active and records the model and worker notices.
        .setup(|app| {
            if let Some(main) = app.get_webview_window(tray::MAIN_WINDOW) {
                main.set_title(&AppText::WindowTitle.localize())?;
            }
            let handle = app.handle().clone();
            app.state::<AppState>()
                .ai_health
                .on_change(move || status::emit_changed(&handle));
            app.state::<AppState>().init_model(app.handle());
            worker::spawn(app.handle().clone());
            api::spawn(app.handle().clone());
            tray::install(app.handle())?;
            if updater::auto_check_supported() {
                updater::spawn(app.handle().clone());
            }
            Ok(())
        })
        .on_window_event(tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::list,
            commands::search,
            commands::get,
            commands::bookmark_favicon,
            commands::add,
            commands::update,
            commands::delete,
            commands::retry,
            commands::tag_groups,
            commands::create_tag_group,
            commands::rename_tag_group,
            commands::delete_tag_group,
            commands::set_tag_group,
            commands::get_settings,
            commands::save_settings,
            commands::save_extraction_settings,
            commands::test_connection,
            commands::reextract,
            commands::extract_with_ai,
            commands::has_extraction_model,
            commands::reindex,
            commands::process_unsummarized,
            commands::stats,
            commands::api_info,
            commands::regenerate_api_token,
            commands::profiles,
            commands::create_profile,
            commands::rename_profile,
            commands::delete_profile,
            commands::switch_profile,
            commands::move_bookmark,
            commands::extracting,
            commands::notifications,
            commands::push_notification,
            commands::mark_notifications_read,
            commands::dismiss_notification,
            commands::clear_notifications,
            commands::import_preview,
            commands::import_confirm,
            commands::import_cancel,
            commands::export_bookmarks,
            commands::extract_imported,
            commands::retry_failed,
            commands::app_status,
            commands::locale_info,
            commands::set_locale,
            commands::update_info,
            commands::check_update,
            commands::install_update,
            commands::set_update_auto,
        ])
        .build(tauri::generate_context!())?
        .run(on_run_event);
    Ok(())
}

/// Dock click with the window hidden (macOS only): show it again.
fn on_run_event(app: &tauri::AppHandle, event: tauri::RunEvent) {
    #[cfg(target_os = "macos")]
    if let tauri::RunEvent::Reopen { .. } = event {
        tray::show_main(app);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, event);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    const LIB_RS: &str = include_str!("lib.rs");
    const BUILD_RS: &str = include_str!("../build.rs");
    const CAPABILITY: &str = include_str!("../capabilities/default.json");

    /// The quoted names on each `"…",` line of `COMMANDS` in `build.rs`.
    fn manifest_commands() -> BTreeSet<String> {
        BUILD_RS
            .lines()
            .filter_map(|l| l.trim().strip_prefix('"')?.strip_suffix("\","))
            .map(str::to_string)
            .collect()
    }

    /// A command missing from the manifest or the capability is rejected at runtime, not at compile
    /// time: this test catches it earlier.
    #[test]
    fn every_command_is_in_the_manifest_and_granted_to_main() {
        let handler: BTreeSet<String> = LIB_RS
            .lines()
            .filter_map(|l| l.trim().strip_prefix("commands::")?.strip_suffix(','))
            .map(str::to_string)
            .collect();
        let capability: serde_json::Value =
            serde_json::from_str(CAPABILITY).expect("default.json válido");
        let granted: BTreeSet<String> = capability["permissions"]
            .as_array()
            .expect("lista de permisos")
            .iter()
            .filter_map(|p| p.as_str()?.strip_prefix("allow-"))
            .map(|p| p.replace('-', "_"))
            .collect();
        assert!(!handler.is_empty(), "no se encontró generate_handler!");
        assert_eq!(
            handler,
            manifest_commands(),
            "generate_handler! frente a build.rs"
        );
        assert_eq!(
            handler, granted,
            "generate_handler! frente a capabilities/default.json"
        );
    }
}
