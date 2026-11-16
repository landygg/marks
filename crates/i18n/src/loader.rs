use crate::Locale;
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use rust_embed::RustEmbed;
use std::cell::Cell;
use std::sync::{LazyLock, Mutex};

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

/// Pseudo-locale: accented, ~35 % longer and wrapped in `⟦ ⟧`.
fn pseudo(s: &str) -> std::borrow::Cow<'_, str> {
    let t = fluent_pseudo::transform(s, true, true);
    format!("⟦{t}⟧").into()
}

/// One loader per language, built once and never mutated afterwards.
fn build(locale: Locale) -> FluentLanguageLoader {
    // The domain is `marks_i18n` (the .ftl name), not the crate's.
    let l = FluentLanguageLoader::new("marks_i18n", "en".parse().unwrap_or_default());
    if let Ok(lang) = locale.tag().parse() {
        // The `en` fallback is embedded, so this cannot fail.
        let _ = i18n_embed::select(&l, &Localizations, &[lang]);
    }
    // After `select`, which recreates the bundles with the default value.
    l.set_use_isolating(false);
    if locale == Locale::Pseudo {
        l.with_bundles_mut(|b| b.set_transform(Some(pseudo)));
    }
    l
}

static ES: LazyLock<FluentLanguageLoader> = LazyLock::new(|| build(Locale::Es));
static EN: LazyLock<FluentLanguageLoader> = LazyLock::new(|| build(Locale::En));
static PSEUDO: LazyLock<FluentLanguageLoader> = LazyLock::new(|| build(Locale::Pseudo));

static ACTIVE: Mutex<Locale> = Mutex::new(Locale::En);

thread_local! {
    /// Language forced on this thread by `localize_in`.
    static OVERRIDE: Cell<Option<Locale>> = const { Cell::new(None) };
}

fn loader_for(locale: Locale) -> &'static FluentLanguageLoader {
    match locale {
        Locale::Es => &ES,
        Locale::En => &EN,
        Locale::Pseudo => &PSEUDO,
    }
}

/// The thread's loader (during a `localize_in`) or the active language's.
pub(crate) fn loader() -> &'static FluentLanguageLoader {
    loader_for(OVERRIDE.with(Cell::get).unwrap_or_else(active))
}

pub fn init(locale: Locale) {
    *ACTIVE.lock().unwrap_or_else(|e| e.into_inner()) = locale;
}

pub fn active() -> Locale {
    *ACTIVE.lock().unwrap_or_else(|e| e.into_inner())
}

pub trait Localize {
    fn localize(&self) -> String;
}

/// Restores the previously forced language, even if rendering panics.
struct Restore(Option<Locale>);
impl Drop for Restore {
    fn drop(&mut self) {
        OVERRIDE.with(|o| o.set(self.0));
    }
}

/// Renders in `locale` without touching the active language or other threads.
pub fn localize_in<T: Localize>(locale: Locale, item: &T) -> String {
    let _restore = Restore(OVERRIDE.with(|o| o.replace(Some(locale))));
    item.localize()
}

/// Joins our part and its cause with `error-chain`.
pub fn chain(head: &str, rest: &str) -> String {
    fl!(loader(), "error-chain", head = head, rest = rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoContent;
    impl Localize for NoContent {
        fn localize(&self) -> String {
            loader().get("error-no-content")
        }
    }
    struct Chain;
    impl Localize for Chain {
        fn localize(&self) -> String {
            chain("a", "b")
        }
    }

    // `init` is global: every test that uses it lives in this one.
    #[test]
    fn locales_render_and_do_not_leak() {
        init(Locale::En);
        assert_eq!(NoContent.localize(), "no content");
        assert_eq!(localize_in(Locale::Es, &NoContent), "sin contenido");
        assert_eq!(localize_in(Locale::En, &NoContent), "no content");
        assert_eq!(active(), Locale::En);
        assert_eq!(localize_in(Locale::Es, &Chain), "a: b");
        assert_eq!(localize_in(Locale::En, &Chain), "a: b");
        let p = localize_in(Locale::Pseudo, &NoContent);
        assert!(
            p.starts_with('⟦') && p.ends_with('⟧') && p.len() > 10,
            "{p}"
        );
        // Reentrant: the nested call returns to the outer one.
        struct Nest;
        impl Localize for Nest {
            fn localize(&self) -> String {
                format!(
                    "{}|{}",
                    localize_in(Locale::En, &NoContent),
                    NoContent.localize()
                )
            }
        }
        assert_eq!(localize_in(Locale::Es, &Nest), "no content|sin contenido");
        // A panic while rendering clears the override.
        struct Boom;
        impl Localize for Boom {
            fn localize(&self) -> String {
                panic!("boom")
            }
        }
        let _ = std::panic::catch_unwind(|| localize_in(Locale::Es, &Boom));
        assert_eq!(NoContent.localize(), "no content");
        // Another thread during a localize_in uses the active language.
        struct Other;
        impl Localize for Other {
            fn localize(&self) -> String {
                std::thread::spawn(|| NoContent.localize())
                    .join()
                    .unwrap_or_default()
            }
        }
        assert_eq!(localize_in(Locale::Es, &Other), "no content");
    }
}
