//! Keeps the `main` window on the app's own pages: a middle click, WebKit's "Open Link" or a
//! dropped link would otherwise load a remote page in place of the interface.

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Runtime, Url};

use crate::tray::MAIN_WINDOW;

/// Origin of the bundled app.
const APP_URL: &str = "tauri://localhost";
/// Origin of the Vite server under `tauri dev` (`devUrl` in `tauri.conf.json`).
const APP_DEV_URL: &str = "http://127.0.0.1:1420";
const PLUGIN_NAME: &str = "navigation";
const LOG_BLOCKED: &str = "main: navegación rechazada"; // i18n-allow: log

/// Same origin (scheme, host and port) as the app; the Vite server only in development.
fn is_app_url(url: &Url) -> bool {
    let same_origin = |base: &str| {
        Url::parse(base).is_ok_and(|base| {
            // By hand: `Url::origin` is opaque for `tauri://`, so two of them never compare equal.
            base.scheme() == url.scheme()
                && base.host() == url.host()
                && base.port_or_known_default() == url.port_or_known_default()
        })
    };
    same_origin(APP_URL) || (cfg!(debug_assertions) && same_origin(APP_DEV_URL))
}

/// The plugin; registered in `run`.
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(PLUGIN_NAME)
        .on_navigation(|webview, url| {
            // `extractor` and `parser` run their own hook first (`open_hidden`); this one is for `main`.
            if webview.label() != MAIN_WINDOW || is_app_url(url) {
                return true;
            }
            log::warn!("{LOG_BLOCKED}: {url}");
            false
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_app_origin_is_allowed() {
        let url = |s: &str| Url::parse(s).expect("URL válida");
        assert!(is_app_url(&url("tauri://localhost/")));
        assert!(is_app_url(&url("tauri://localhost/index.html")));
        // Tests run in debug: the Vite server is allowed too.
        assert!(is_app_url(&url("http://127.0.0.1:1420/")));
        for bad in [
            "https://example.com/",
            "http://127.0.0.1:47821/",
            "http://127.0.0.1:1421/",
            "about:blank",
            "data:text/html,x",
            "file:///etc/hosts",
            "tauri://otro/",
        ] {
            assert!(!is_app_url(&url(bad)), "{bad}");
        }
    }
}
