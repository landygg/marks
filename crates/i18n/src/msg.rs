//! Translation of core's `Msg` codes.

use crate::loader::{Localize, chain, loader};
use i18n_embed_fl::fl;
use marks_core::msg::{BookmarkError, ErrorPart, Msg, Stage, parts};

impl Localize for Msg {
    fn localize(&self) -> String {
        let l = loader();
        match self {
            Msg::InvalidLocale => fl!(l, "error-invalid-locale"),
            Msg::NoContent => fl!(l, "error-no-content"),
            Msg::BookmarkNotFound => fl!(l, "error-bookmark-not-found"),
            Msg::StorePoisoned => fl!(l, "error-store-poisoned"),
            Msg::StoresPoisoned => fl!(l, "error-stores-poisoned"),
            Msg::UnknownStatus { value } => fl!(l, "error-unknown-status", value = value.as_str()),
            Msg::UnknownErrorKind { value } => {
                fl!(l, "error-unknown-error-kind", value = value.as_str())
            }
            Msg::OpenDatabase { path } => fl!(l, "error-open-database", path = path.as_str()),
            Msg::DatabaseTooNew { path } => {
                fl!(l, "error-database-too-new", path = path.as_str())
            }
            Msg::DatabaseUnknown { path } => {
                fl!(l, "error-database-unknown", path = path.as_str())
            }
            Msg::Backup { path } => fl!(l, "error-backup", path = path.as_str()),
            Msg::InvalidUrl => fl!(l, "error-invalid-url"),
            Msg::GroupNameEmpty => fl!(l, "error-group-name-empty"),
            Msg::GroupExists => fl!(l, "error-group-exists"),
            Msg::GroupNotFound => fl!(l, "error-group-not-found"),
            Msg::ProfileNotFound => fl!(l, "error-profile-not-found"),
            Msg::ProfileName { max } => {
                let max = *max as u64;
                fl!(l, "error-profile-name", max = max)
            }
            Msg::ProfileInitials { max } => {
                let max = *max as u64;
                fl!(l, "error-profile-initials", max = max)
            }
            Msg::ProfileExists => fl!(l, "error-profile-exists"),
            Msg::DefaultProfile => fl!(l, "error-default-profile"),
            Msg::SameProfile => fl!(l, "error-same-profile"),
            Msg::UrlInProfile { profile } => {
                fl!(l, "error-url-in-profile", profile = profile.as_str())
            }
            Msg::FileTooLarge { max_mb } => {
                let max_mb = *max_mb;
                fl!(l, "error-file-too-large", max_mb = max_mb)
            }
            Msg::NotUtf8 => fl!(l, "error-not-utf8"),
            Msg::Blocked => fl!(l, "error-blocked"),
            Msg::BlockedBookmark => fl!(l, "error-blocked-bookmark"),
            Msg::UnsupportedScheme => fl!(l, "error-unsupported-scheme"),
            Msg::Destination => fl!(l, "error-destination"),
            Msg::NotHtml => fl!(l, "error-not-html"),
            Msg::TooManyRedirects => fl!(l, "error-too-many-redirects"),
            Msg::NotImage => fl!(l, "error-not-image"),
            Msg::FaviconTooLarge => fl!(l, "error-favicon-too-large"),
            Msg::NoLocation => fl!(l, "error-no-location"),
            Msg::FetchTask => fl!(l, "error-fetch-task"),
            Msg::ThinResult => fl!(l, "error-thin-result"),
            Msg::ExtractTimeout => fl!(l, "error-extract-timeout"),
            Msg::EmbedChunkCount => fl!(l, "error-embed-chunk-count"),
            Msg::EmbedDim => fl!(l, "error-embed-dim"),
            Msg::VectorsLenMismatch => fl!(l, "error-vectors-len-mismatch"),
            Msg::VectorsDimMismatch => fl!(l, "error-vectors-dim-mismatch"),
            Msg::VectorsNoIndex => fl!(l, "error-vectors-no-index"),
            Msg::VectorsBadDim => fl!(l, "error-vectors-bad-dim"),
            Msg::NoJson => fl!(l, "error-no-json"),
            Msg::InvalidJson => fl!(l, "error-invalid-json"),
            Msg::ApiKeyRequired => fl!(l, "error-api-key-required"),
            Msg::AnthropicNoEmbeddings => fl!(l, "error-anthropic-no-embeddings"),
            Msg::ClientBuild => fl!(l, "error-client-build"),
            Msg::ChatFailed => fl!(l, "error-chat-failed"),
            Msg::EmbedFailed => fl!(l, "error-embed-failed"),
            Msg::EmptyChat => fl!(l, "error-empty-chat"),
            Msg::EmbedBatchCount => fl!(l, "error-embed-batch-count"),
            Msg::OllamaTooLong => fl!(l, "error-ollama-too-long"),
            Msg::OllamaStatus { status, detail } => {
                let status = u64::from(*status);
                fl!(
                    l,
                    "error-ollama-status",
                    status = status,
                    detail = detail.as_str()
                )
            }
            Msg::OllamaBadResponse => fl!(l, "error-ollama-bad-response"),
            Msg::EndpointRequired => fl!(l, "error-endpoint-required"),
            Msg::EmptyModel => fl!(l, "error-empty-model"),
            Msg::InvalidSettings => fl!(l, "error-invalid-settings"),
            Msg::EmptyExtractionModel => fl!(l, "error-empty-extraction-model"),
            Msg::KeyringEntry => fl!(l, "error-keyring-entry"),
            Msg::KeyringRead => fl!(l, "error-keyring-read"),
            Msg::KeyringWrite => fl!(l, "error-keyring-write"),
            Msg::KeyringDelete => fl!(l, "error-keyring-delete"),
            Msg::TokenRandom => fl!(l, "error-token-random"),
            Msg::WebviewUrl => fl!(l, "error-webview-url"),
            Msg::WebviewTimeout { stage } => {
                fl!(l, "error-webview-timeout", stage = stage_text(*stage))
            }
            Msg::WebviewDecode => fl!(l, "error-webview-decode"),
            Msg::WebviewClosedLoading => fl!(l, "error-webview-closed-loading"),
            Msg::WebviewClosedHtml => fl!(l, "error-webview-closed-html"),
            Msg::ParserPage => fl!(l, "error-parser-page"),
            Msg::ParserClean => fl!(l, "error-parser-clean"),
            Msg::ParserTimeout { stage } => {
                fl!(l, "error-parser-timeout", stage = stage_text(*stage))
            }
            Msg::ParserClosedLoading => fl!(l, "error-parser-closed-loading"),
            Msg::ParserClosedPage => fl!(l, "error-parser-closed-page"),
            Msg::NoPendingImport => fl!(l, "error-no-pending-import"),
            Msg::ImportOtherProfile => fl!(l, "error-import-other-profile"),
            Msg::ImportRead { path } => fl!(l, "error-import-read", path = path.as_str()),
            Msg::NoModel => fl!(l, "error-no-model"),
            Msg::NoExtractionModel => fl!(l, "error-no-extraction-model"),
            Msg::ExportWrite => fl!(l, "error-export-write"),
        }
    }
}

/// Each stage is its own message: never pasted in as a fragment of another.
fn stage_text(stage: Stage) -> String {
    let l = loader();
    match stage {
        Stage::Open => fl!(l, "error-stage-open"),
        Stage::Load => fl!(l, "error-stage-load"),
        Stage::Settle => fl!(l, "error-stage-settle"),
        Stage::Eval => fl!(l, "error-stage-eval"),
        Stage::Ready => fl!(l, "error-stage-ready"),
    }
}

/// Same as `Localize`, as a function.
pub fn msg(m: &Msg) -> String {
    m.localize()
}

/// Each error layer translated (`Msg`) or verbatim (`Raw`), joined with `error-chain`.
pub fn error(e: &anyhow::Error) -> String {
    join(&parts(e))
}

/// Same, for the error stored on a bookmark.
pub fn bookmark_error(e: &BookmarkError) -> String {
    join(&e.0)
}

fn join(parts: &[ErrorPart]) -> String {
    parts
        .iter()
        .map(|p| match p {
            ErrorPart::Msg(m) => m.localize(),
            ErrorPart::Raw { raw } => raw.clone(),
        })
        .rev()
        .reduce(|rest, head| chain(&head, &rest))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, localize_in};
    use marks_core::msg::{BookmarkError, Msg, MsgExt, Stage};

    struct ErrorOf<'a>(&'a anyhow::Error);
    impl Localize for ErrorOf<'_> {
        fn localize(&self) -> String {
            error(self.0)
        }
    }

    fn all() -> Vec<Msg> {
        vec![
            Msg::InvalidLocale,
            Msg::NoContent,
            Msg::BookmarkNotFound,
            Msg::StorePoisoned,
            Msg::StoresPoisoned,
            Msg::UnknownStatus { value: "x".into() },
            Msg::UnknownErrorKind { value: "x".into() },
            Msg::OpenDatabase { path: "/x".into() },
            Msg::DatabaseTooNew { path: "/x".into() },
            Msg::DatabaseUnknown { path: "/x".into() },
            Msg::Backup { path: "/x".into() },
            Msg::InvalidUrl,
            Msg::GroupNameEmpty,
            Msg::GroupExists,
            Msg::GroupNotFound,
            Msg::ProfileNotFound,
            Msg::ProfileName { max: 40 },
            Msg::ProfileInitials { max: 2 },
            Msg::ProfileExists,
            Msg::DefaultProfile,
            Msg::SameProfile,
            Msg::UrlInProfile {
                profile: "P".into(),
            },
            Msg::FileTooLarge { max_mb: 50 },
            Msg::NotUtf8,
            Msg::Blocked,
            Msg::BlockedBookmark,
            Msg::UnsupportedScheme,
            Msg::Destination,
            Msg::NotHtml,
            Msg::TooManyRedirects,
            Msg::NotImage,
            Msg::FaviconTooLarge,
            Msg::NoLocation,
            Msg::FetchTask,
            Msg::ThinResult,
            Msg::ExtractTimeout,
            Msg::EmbedChunkCount,
            Msg::EmbedDim,
            Msg::VectorsLenMismatch,
            Msg::VectorsDimMismatch,
            Msg::VectorsNoIndex,
            Msg::VectorsBadDim,
            Msg::NoJson,
            Msg::InvalidJson,
            Msg::ApiKeyRequired,
            Msg::AnthropicNoEmbeddings,
            Msg::ClientBuild,
            Msg::ChatFailed,
            Msg::EmbedFailed,
            Msg::EmptyChat,
            Msg::EmbedBatchCount,
            Msg::OllamaTooLong,
            Msg::OllamaStatus {
                status: 400,
                detail: "x".into(),
            },
            Msg::OllamaBadResponse,
            Msg::EndpointRequired,
            Msg::EmptyModel,
            Msg::InvalidSettings,
            Msg::EmptyExtractionModel,
            Msg::KeyringEntry,
            Msg::KeyringRead,
            Msg::KeyringWrite,
            Msg::KeyringDelete,
            Msg::TokenRandom,
            Msg::WebviewUrl,
            Msg::WebviewTimeout { stage: Stage::Load },
            Msg::WebviewDecode,
            Msg::WebviewClosedLoading,
            Msg::WebviewClosedHtml,
            Msg::ParserPage,
            Msg::ParserClean,
            Msg::ParserTimeout { stage: Stage::Load },
            Msg::ParserClosedLoading,
            Msg::ParserClosedPage,
            Msg::NoPendingImport,
            Msg::ImportOtherProfile,
            Msg::ImportRead { path: "/x".into() },
            Msg::NoModel,
            Msg::NoExtractionModel,
            Msg::ExportWrite,
        ]
    }

    #[test]
    fn every_variant_renders_in_both_languages() {
        for v in all() {
            let es = localize_in(Locale::Es, &v);
            let en = localize_in(Locale::En, &v);
            assert!(!es.is_empty() && !es.contains("Unknown"), "{v:?}: {es}");
            assert!(!en.is_empty() && !en.contains("Unknown"), "{v:?}: {en}");
            assert_ne!(es, en, "{v:?}: en falls back to es");
        }
    }

    #[test]
    fn exact_texts() {
        let es = Msg::UrlInProfile {
            profile: "Casa".into(),
        };
        assert_eq!(
            localize_in(Locale::Es, &es),
            "ese bookmark ya existe en el perfil «Casa»"
        );
        let en = Msg::UrlInProfile {
            profile: "Home".into(),
        };
        assert_eq!(
            localize_in(Locale::En, &en),
            "that bookmark is already in the profile “Home”"
        );
        assert_eq!(
            localize_in(Locale::Es, &Msg::ProfileName { max: 40 }),
            "el nombre del perfil debe tener entre 1 y 40 caracteres"
        );
    }

    #[test]
    fn webview_timeout_names_its_stage() {
        let es = |stage| localize_in(Locale::Es, &Msg::WebviewTimeout { stage });
        assert!(es(Stage::Load).contains("esperando a que termine de cargar"));
        assert!(es(Stage::Ready).contains("esperando a que el módulo de extracción esté listo"));
        let all = [
            Stage::Open,
            Stage::Load,
            Stage::Settle,
            Stage::Eval,
            Stage::Ready,
        ];
        for locale in [Locale::Es, Locale::En] {
            let texts: Vec<String> = all
                .iter()
                .map(|&stage| localize_in(locale, &Msg::ParserTimeout { stage }))
                .collect();
            for (i, t) in texts.iter().enumerate() {
                assert!(!t.contains("Unknown"), "{t}");
                assert!(texts.iter().enumerate().all(|(j, o)| i == j || o != t));
            }
        }
        assert_eq!(
            localize_in(
                Locale::En,
                &Msg::OllamaStatus {
                    status: 400,
                    detail: "x".into()
                }
            ),
            "Ollama responded with error 400: x"
        );
    }

    struct BookmarkErrorOf(BookmarkError);
    impl Localize for BookmarkErrorOf {
        fn localize(&self) -> String {
            bookmark_error(&self.0)
        }
    }

    #[test]
    fn bookmark_error_translates_codes_and_keeps_raw() {
        let raw = |s: &str| ErrorPart::Raw { raw: s.into() };
        let e = BookmarkErrorOf(BookmarkError(vec![
            ErrorPart::Msg(Msg::ChatFailed),
            raw("timeout"),
        ]));
        assert_eq!(
            localize_in(Locale::Es, &e),
            "falló la llamada al modelo de chat: timeout"
        );
        assert_eq!(
            localize_in(Locale::En, &e),
            "the chat model call failed: timeout"
        );
        let only_raw = BookmarkErrorOf(BookmarkError(vec![raw("se rompió")]));
        assert_eq!(localize_in(Locale::En, &only_raw), "se rompió");
    }

    #[test]
    fn error_joins_the_chain() {
        let io: anyhow::Result<()> = Err(std::io::Error::other("denied").into());
        let e = io.msg(Msg::OpenDatabase { path: "/x".into() }).unwrap_err();
        assert_eq!(
            localize_in(Locale::Es, &ErrorOf(&e)),
            "no se pudo abrir /x: denied"
        );
    }
}
