//! App menu: the same as Tauri's `Menu::default` on macOS, with text from the catalog.

use marks_i18n::{Localize, MenuText};
use tauri::menu::{
    AboutMetadata, HELP_SUBMENU_ID, Menu, PredefinedMenuItem, Submenu, WINDOW_SUBMENU_ID,
};
use tauri::{AppHandle, Runtime};

/// Which `PredefinedMenuItem` each item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
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
    Separator,
}

/// An item: its kind and its text (`None` only for separators).
type Item = (Kind, Option<MenuText>);

/// The submenus in order, with their title (`None`: the app one, set by macOS) and their items.
pub fn items() -> Vec<(Option<MenuText>, Vec<Item>)> {
    use Kind as K;
    use MenuText as T;
    const SEP: Item = (K::Separator, None);
    vec![
        (
            None,
            vec![
                (K::About, Some(T::About)),
                SEP,
                (K::Services, Some(T::Services)),
                SEP,
                (K::Hide, Some(T::Hide)),
                (K::HideOthers, Some(T::HideOthers)),
                SEP,
                (K::Quit, Some(T::Quit)),
            ],
        ),
        (Some(T::File), vec![(K::CloseWindow, Some(T::CloseWindow))]),
        (
            Some(T::Edit),
            vec![
                (K::Undo, Some(T::Undo)),
                (K::Redo, Some(T::Redo)),
                SEP,
                (K::Cut, Some(T::Cut)),
                (K::Copy, Some(T::Copy)),
                (K::Paste, Some(T::Paste)),
                (K::SelectAll, Some(T::SelectAll)),
            ],
        ),
        (Some(T::View), vec![(K::Fullscreen, Some(T::Fullscreen))]),
        (
            Some(T::Window),
            vec![
                (K::Minimize, Some(T::Minimize)),
                (K::Maximize, Some(T::Maximize)),
                SEP,
                (K::CloseWindow, Some(T::CloseWindow)),
            ],
        ),
        (Some(T::Help), vec![]),
    ]
}

/// Builds the `items()` menu in the active language.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    for (title, entries) in items() {
        let text = title.map_or_else(|| app.package_info().name.clone(), |t| t.localize());
        // With these ids macOS treats them as its Window and Help menus, as in `Menu::default`.
        let submenu = match title {
            Some(MenuText::Window) => Submenu::with_id(app, WINDOW_SUBMENU_ID, text, true)?,
            Some(MenuText::Help) => Submenu::with_id(app, HELP_SUBMENU_ID, text, true)?,
            _ => Submenu::new(app, text, true)?,
        };
        for (kind, text) in entries {
            let text = text.map(|t| t.localize());
            submenu.append(&predefined(app, kind, text.as_deref())?)?;
        }
        menu.append(&submenu)?;
    }
    Ok(menu)
}

fn predefined<R: Runtime>(
    app: &AppHandle<R>,
    kind: Kind,
    text: Option<&str>,
) -> tauri::Result<PredefinedMenuItem<R>> {
    match kind {
        Kind::About => PredefinedMenuItem::about(app, text, Some(about(app))),
        Kind::Services => PredefinedMenuItem::services(app, text),
        Kind::Hide => PredefinedMenuItem::hide(app, text),
        Kind::HideOthers => PredefinedMenuItem::hide_others(app, text),
        Kind::Quit => PredefinedMenuItem::quit(app, text),
        Kind::CloseWindow => PredefinedMenuItem::close_window(app, text),
        Kind::Undo => PredefinedMenuItem::undo(app, text),
        Kind::Redo => PredefinedMenuItem::redo(app, text),
        Kind::Cut => PredefinedMenuItem::cut(app, text),
        Kind::Copy => PredefinedMenuItem::copy(app, text),
        Kind::Paste => PredefinedMenuItem::paste(app, text),
        Kind::SelectAll => PredefinedMenuItem::select_all(app, text),
        Kind::Fullscreen => PredefinedMenuItem::fullscreen(app, text),
        Kind::Minimize => PredefinedMenuItem::minimize(app, text),
        Kind::Maximize => PredefinedMenuItem::maximize(app, text),
        Kind::Separator => PredefinedMenuItem::separator(app),
    }
}

/// The "About" data, as `Menu::default` builds it.
fn about<R: Runtime>(app: &AppHandle<R>) -> AboutMetadata<'static> {
    let info = app.package_info();
    let bundle = &app.config().bundle;
    AboutMetadata {
        name: Some(info.name.clone()),
        version: Some(info.version.to_string()),
        copyright: bundle.copyright.clone(),
        authors: bundle.publisher.clone().map(|p| vec![p]),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tauri 2.12.5's `Menu::default` on macOS (`tauri/src/menu/menu.rs`), constructor by
    /// constructor. Compare with theirs when upgrading Tauri. Cannot be built in a test: muda only
    /// creates menus on the main thread.
    const TAURI_DEFAULT: &[(&str, &[Kind])] = {
        use Kind::*;
        &[
            (
                "",
                &[
                    About, Separator, Services, Separator, Hide, HideOthers, Separator, Quit,
                ],
            ),
            ("File", &[CloseWindow]),
            (
                "Edit",
                &[Undo, Redo, Separator, Cut, Copy, Paste, SelectAll],
            ),
            ("View", &[Fullscreen]),
            ("Window", &[Minimize, Maximize, Separator, CloseWindow]),
            ("Help", &[]),
        ]
    };

    #[test]
    fn menu_has_every_default_item() {
        let ours = items();
        assert_eq!(ours.len(), TAURI_DEFAULT.len());
        for ((title, entries), (tauri_title, kinds)) in ours.iter().zip(TAURI_DEFAULT) {
            let title = title.map_or_else(String::new, |t| format!("{t:?}"));
            assert_eq!(&title, tauri_title);
            let our_kinds: Vec<Kind> = entries.iter().map(|(k, _)| *k).collect();
            assert_eq!(&our_kinds, kinds, "{title}");
            // Each item has its own text, and the separator none.
            for (kind, text) in entries {
                match (kind, text) {
                    (Kind::Separator, None) => {}
                    (kind, Some(text)) => assert_eq!(format!("{kind:?}"), format!("{text:?}")),
                    other => panic!("{other:?}"),
                }
            }
        }
    }
}
