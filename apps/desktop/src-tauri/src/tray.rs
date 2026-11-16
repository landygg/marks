//! Menu bar icon, and a main window that hides on close.

use marks_i18n::{AppText, Localize};
use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Window, WindowEvent};

/// Main window label; same value as `label` in `tauri.conf.json`.
pub const MAIN_WINDOW: &str = "main";
/// Payload-less event asking to open the add dialog; same value as `OPEN_ADD_DIALOG` in `src/lib/api.ts`.
pub const OPEN_ADD_DIALOG: &str = "open-add-dialog";

const ID_OPEN: &str = "open";
const ID_ADD: &str = "add";
const ID_QUIT: &str = "quit";
const NO_ICON_WARNING: &str = // i18n-allow: log
    "bandeja: la app no tiene icono por defecto; el icono de la bandeja no se verá";

/// Creates the tray icon with its menu.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &item(app, ID_OPEN, AppText::TrayOpen)?,
            &item(app, ID_ADD, AppText::TrayAdd)?,
            &item(app, ID_QUIT, AppText::TrayQuit)?,
        ],
    )?;
    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(on_menu_event);
    match app.default_window_icon().cloned() {
        Some(icon) => tray = tray.icon(icon),
        None => log::warn!("{NO_ICON_WARNING}"),
    }
    tray.build(app)?;
    Ok(())
}

fn item(app: &AppHandle, id: &str, text: AppText) -> tauri::Result<MenuItem<tauri::Wry>> {
    MenuItem::with_id(app, id, text.localize(), true, None::<&str>)
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    log::info!("bandeja: opción {:?}", event.id().as_ref());
    match event.id().as_ref() {
        ID_OPEN => show_main(app),
        ID_ADD => {
            show_main(app);
            if let Err(e) = app.emit(OPEN_ADD_DIALOG, ()) {
                log::warn!("bandeja: no se pudo emitir {OPEN_ADD_DIALOG}: {e}");
            }
        }
        ID_QUIT => app.exit(0),
        _ => {}
    }
}

/// Shows and focuses the main window; a failure is only logged.
pub(crate) fn show_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        log::warn!("bandeja: no existe la ventana {MAIN_WINDOW}");
        return;
    };
    if let Err(e) = window.show().and_then(|()| window.set_focus()) {
        log::warn!("bandeja: no se pudo mostrar la ventana {MAIN_WINDOW}: {e}");
    }
}

/// Closing the main window hides it instead of destroying it; the app lives on in the tray.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW {
        return;
    }
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        log::info!("ventana {MAIN_WINDOW}: cierre interceptado, se oculta");
        if let Err(e) = window.hide() {
            log::warn!("ventana {MAIN_WINDOW}: no se pudo ocultar: {e}");
        }
    }
}
