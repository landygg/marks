//! Desktop app texts that are not `Msg`: window, tray, menu, native dialogs and notices.

use crate::loader::{Localize, loader};
use i18n_embed_fl::fl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppText {
    WindowTitle,
    TrayOpen,
    TrayAdd,
    TrayQuit,
    ImportDialogTitle,
    ExportDialogTitle,
    HtmlFilter,
    /// Notices from `state`, `worker` and `api`; the cause follows through `chain`.
    ModelUnavailable,
    ProfilesFailed,
    StoreFailed,
    FaviconsFailed,
    ProfileOpenFailed,
    ApiPortBusy {
        port: u16,
    },
    ApiStopped,
    /// Notification that arrives without text.
    EmptyNotification,
    /// The updater (`updater.rs`): a new version found, and the manual check or install failing.
    UpdateAvailable {
        version: String,
    },
    UpdateCheckFailed,
    UpdateInstallFailed,
    /// The update found needs a newer macOS than this Mac's: it is not offered.
    UpdateNeedsNewerMacos {
        version: String,
        minimum: String,
        current: String,
    },
    Menu(MenuText),
}

/// One text per `PredefinedMenuItem` in Tauri's default menu, plus the submenu titles (macOS sets
/// the app one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuText {
    About,
    Services,
    Hide,
    HideOthers,
    Quit,
    CloseWindow,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Fullscreen,
    Minimize,
    Maximize,
    File,
    Edit,
    View,
    Window,
    Help,
}

impl Localize for AppText {
    fn localize(&self) -> String {
        let l = loader();
        match self {
            AppText::WindowTitle => fl!(l, "app-window-title"),
            AppText::TrayOpen => fl!(l, "tray-open"),
            AppText::TrayAdd => fl!(l, "tray-add"),
            AppText::TrayQuit => fl!(l, "tray-quit"),
            AppText::ImportDialogTitle => fl!(l, "app-import-dialog-title"),
            AppText::ExportDialogTitle => fl!(l, "app-export-dialog-title"),
            AppText::HtmlFilter => fl!(l, "app-html-filter"),
            AppText::ModelUnavailable => fl!(l, "app-model-unavailable"),
            AppText::ProfilesFailed => fl!(l, "app-profiles-failed"),
            AppText::StoreFailed => fl!(l, "app-store-failed"),
            AppText::FaviconsFailed => fl!(l, "app-favicons-failed"),
            AppText::ProfileOpenFailed => fl!(l, "app-profile-open-failed"),
            // As text: a Fluent number could get a thousands separator.
            AppText::ApiPortBusy { port } => {
                fl!(l, "app-api-port-busy", port = port.to_string())
            }
            AppText::ApiStopped => fl!(l, "app-api-stopped"),
            AppText::EmptyNotification => fl!(l, "app-empty-notification"),
            AppText::UpdateAvailable { version } => {
                fl!(l, "app-update-available", version = version.as_str())
            }
            AppText::UpdateCheckFailed => fl!(l, "app-update-check-failed"),
            AppText::UpdateInstallFailed => fl!(l, "app-update-install-failed"),
            AppText::UpdateNeedsNewerMacos {
                version,
                minimum,
                current,
            } => fl!(
                l,
                "app-update-needs-macos",
                version = version.as_str(),
                minimum = minimum.as_str(),
                current = current.as_str()
            ),
            AppText::Menu(m) => m.localize(),
        }
    }
}

impl Localize for MenuText {
    fn localize(&self) -> String {
        let l = loader();
        match self {
            MenuText::About => fl!(l, "menu-about"),
            MenuText::Services => fl!(l, "menu-services"),
            MenuText::Hide => fl!(l, "menu-hide"),
            MenuText::HideOthers => fl!(l, "menu-hide-others"),
            MenuText::Quit => fl!(l, "menu-quit"),
            MenuText::CloseWindow => fl!(l, "menu-close-window"),
            MenuText::Undo => fl!(l, "menu-undo"),
            MenuText::Redo => fl!(l, "menu-redo"),
            MenuText::Cut => fl!(l, "menu-cut"),
            MenuText::Copy => fl!(l, "menu-copy"),
            MenuText::Paste => fl!(l, "menu-paste"),
            MenuText::SelectAll => fl!(l, "menu-select-all"),
            MenuText::Fullscreen => fl!(l, "menu-fullscreen"),
            MenuText::Minimize => fl!(l, "menu-minimize"),
            MenuText::Maximize => fl!(l, "menu-maximize"),
            MenuText::File => fl!(l, "menu-file"),
            MenuText::Edit => fl!(l, "menu-edit"),
            MenuText::View => fl!(l, "menu-view"),
            MenuText::Window => fl!(l, "menu-window"),
            MenuText::Help => fl!(l, "menu-help"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, localize_in};

    #[test]
    fn app_texts_render_in_both_languages() {
        assert_eq!(localize_in(Locale::Es, &AppText::TrayAdd), "Añadir URL…");
        assert_eq!(localize_in(Locale::En, &AppText::TrayAdd), "Add URL…");
        assert_eq!(
            localize_in(Locale::En, &AppText::ApiPortBusy { port: 47821 }),
            "the local API for the extension could not open port 47821"
        );
        assert_eq!(
            localize_in(Locale::Es, &AppText::Menu(MenuText::Edit)),
            "Edición"
        );
        assert_eq!(localize_in(Locale::En, &MenuText::Quit), "Quit marks");
        assert_eq!(
            localize_in(Locale::En, &AppText::EmptyNotification),
            "error with no message"
        );
        assert_eq!(
            localize_in(
                Locale::Es,
                &AppText::UpdateAvailable {
                    version: "1.0.1".into()
                }
            ),
            "Hay una versión nueva de marks: 1.0.1."
        );
        assert_eq!(
            localize_in(
                Locale::En,
                &AppText::UpdateNeedsNewerMacos {
                    version: "1.3.0".into(),
                    minimum: "16.0".into(),
                    current: "15.4".into(),
                }
            ),
            "marks 1.3.0 needs macOS 16.0 or later, and this Mac has macOS 15.4. marks stays on this version."
        );
    }
}
