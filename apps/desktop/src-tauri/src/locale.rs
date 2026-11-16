//! App language: resolved at startup; the setting is stored in `AppleLanguages` of the app's
//! domain, which macOS, AppKit and `sys-locale` honor on relaunch.

use anyhow::Result;
use marks_core::msg::Msg;
use marks_i18n::Locale;
use serde::Serialize;

/// Environment variable that forces the language (`es`, `en` or `pseudo`), for development and tests.
const LOCALE_ENV: &str = "MARKS_LOCALE";
const SETTING_SYSTEM: &str = "system";
const SETTING_ES: &str = "es";
const SETTING_EN: &str = "en";
const ACTIVE_PSEUDO: &str = "pseudo";
#[cfg(target_os = "macos")]
const APPLE_LANGUAGES: &str = "AppleLanguages";

/// The stored setting (`system`, `es` or `en`) and the language the app started with.
#[derive(Serialize)]
pub struct LocaleInfo {
    setting: &'static str,
    active: &'static str,
}

pub fn startup() -> Locale {
    let preferred: Vec<String> = sys_locale::get_locales().collect();
    marks_i18n::resolve(std::env::var(LOCALE_ENV).ok().as_deref(), &preferred)
}

pub fn info() -> LocaleInfo {
    let active = match marks_i18n::active() {
        Locale::Pseudo => ACTIVE_PSEUDO,
        l => l.tag(),
    };
    LocaleInfo {
        setting: setting_from_languages(&app_languages()),
        active,
    }
}

/// Saves the setting; it takes effect when the app relaunches.
pub fn save(setting: &str) -> Result<()> {
    write_app_languages(parse_setting(setting)?);
    Ok(())
}

/// The setting from the app domain's `AppleLanguages` list: empty means "same as the system".
fn setting_from_languages(languages: &[String]) -> &'static str {
    if languages.is_empty() {
        return SETTING_SYSTEM;
    }
    match marks_i18n::resolve(None, languages) {
        Locale::Es => SETTING_ES,
        Locale::En | Locale::Pseudo => SETTING_EN,
    }
}

/// `None` for `system` (delete the key); the language for `es` and `en`.
fn parse_setting(setting: &str) -> Result<Option<&'static str>> {
    match setting {
        SETTING_SYSTEM => Ok(None),
        SETTING_ES => Ok(Some(SETTING_ES)),
        SETTING_EN => Ok(Some(SETTING_EN)),
        _ => Err(Msg::InvalidLocale.err()),
    }
}

/// `AppleLanguages` from our own domain, without merged values: `standardUserDefaults` always
/// includes the system's global list and would never say "same as the system". The domain is the
/// bundle id or, without a bundle (`tauri dev`), the process name, which is where
/// `standardUserDefaults` writes.
#[cfg(target_os = "macos")]
fn app_languages() -> Vec<String> {
    use objc2_foundation::{NSArray, NSBundle, NSProcessInfo, NSString, NSUserDefaults};
    let domain = NSBundle::mainBundle()
        .bundleIdentifier()
        .unwrap_or_else(|| NSProcessInfo::processInfo().processName());
    let Some(values) = NSUserDefaults::standardUserDefaults().persistentDomainForName(&domain)
    else {
        return Vec::new();
    };
    let Some(Ok(list)) = values
        .objectForKey(&NSString::from_str(APPLE_LANGUAGES))
        .map(|v| v.downcast::<NSArray>())
    else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|v| v.downcast::<NSString>().ok())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(target_os = "macos")]
fn write_app_languages(language: Option<&str>) {
    use objc2_foundation::{NSArray, NSString, NSUserDefaults};
    let defaults = NSUserDefaults::standardUserDefaults();
    let key = NSString::from_str(APPLE_LANGUAGES);
    match language {
        Some(l) => {
            let list = NSArray::from_retained_slice(&[NSString::from_str(l)]);
            // SAFETY: an NSArray of NSString is a valid preferences value.
            unsafe { defaults.setObject_forKey(Some(&list), &key) }
        }
        None => defaults.removeObjectForKey(&key),
    }
    defaults.synchronize();
}

// shortcut: outside macOS there is no app setting and the app follows the system; implement it
// when packaging for another OS.
#[cfg(not(target_os = "macos"))]
fn app_languages() -> Vec<String> {
    Vec::new()
}

#[cfg(not(target_os = "macos"))]
fn write_app_languages(_language: Option<&str>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use marks_core::msg::failure_msg;

    #[test]
    fn setting_from_languages_maps_the_app_list() {
        assert_eq!(setting_from_languages(&[]), "system");
        assert_eq!(setting_from_languages(&["es".into()]), "es");
        assert_eq!(setting_from_languages(&["en-GB".into()]), "en");
    }

    #[test]
    fn rejects_unknown_setting() {
        let e = parse_setting("fr").unwrap_err();
        assert_eq!(failure_msg(&e), Some(&Msg::InvalidLocale));
        assert_eq!(parse_setting("system").unwrap(), None);
        assert_eq!(parse_setting("es").unwrap(), Some("es"));
        assert_eq!(parse_setting("en").unwrap(), Some("en"));
    }
}
