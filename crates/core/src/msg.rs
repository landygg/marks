//! Message codes: core produces no user text; it returns `Msg` and another layer translates it.

use std::fmt;

use serde::{Deserialize, Serialize};

/// User message. The serialized code is stable: changing it changes the data format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "code", content = "args", rename_all = "kebab-case")]
pub enum Msg {
    InvalidLocale,
    NoContent,
    BookmarkNotFound,
    StorePoisoned,
    StoresPoisoned,
    UnknownStatus { value: String },
    UnknownErrorKind { value: String },
    OpenDatabase { path: String },
    DatabaseTooNew { path: String },
    DatabaseUnknown { path: String },
    Backup { path: String },
    InvalidUrl,
    GroupNameEmpty,
    GroupExists,
    GroupNotFound,
    ProfileNotFound,
    ProfileName { max: usize },
    ProfileInitials { max: usize },
    ProfileExists,
    DefaultProfile,
    SameProfile,
    UrlInProfile { profile: String },
    FileTooLarge { max_mb: u64 },
    NotUtf8,
    Blocked,
    BlockedBookmark,
    UnsupportedScheme,
    Destination,
    NotHtml,
    TooManyRedirects,
    NotImage,
    FaviconTooLarge,
    NoLocation,
    FetchTask,
    ThinResult,
    ExtractTimeout,
    EmbedChunkCount,
    EmbedDim,
    VectorsLenMismatch,
    VectorsDimMismatch,
    VectorsNoIndex,
    VectorsBadDim,
    NoJson,
    InvalidJson,
    ApiKeyRequired,
    AnthropicNoEmbeddings,
    ClientBuild,
    ChatFailed,
    EmbedFailed,
    EmptyChat,
    EmbedBatchCount,
    OllamaTooLong,
    OllamaStatus { status: u16, detail: String },
    OllamaBadResponse,
    EndpointRequired,
    EmptyModel,
    InvalidSettings,
    EmptyExtractionModel,
    KeyringEntry,
    KeyringRead,
    KeyringWrite,
    KeyringDelete,
    TokenRandom,
    WebviewUrl,
    WebviewTimeout { stage: Stage },
    WebviewDecode,
    WebviewClosedLoading,
    WebviewClosedHtml,
    ParserPage,
    ParserClean,
    ParserTimeout { stage: Stage },
    ParserClosedLoading,
    ParserClosedPage,
    NoPendingImport,
    ImportOtherProfile,
    ImportRead { path: String },
    NoModel,
    NoExtractionModel,
    ExportWrite,
}

/// Stage of a hidden load; each is its own message, never a text fragment.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    Open,
    Load,
    Settle,
    Eval,
    Ready,
}

impl fmt::Display for Msg {
    /// Technical form (code and arguments) for logs and tests; never shown to the user.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Should serialization fail (it cannot with these types), write a fixed value instead of panicking.
        let value = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        write!(
            f,
            "{}",
            value
                .get("code")
                .and_then(|c| c.as_str())
                .unwrap_or("unknown")
        )?;
        match value.get("args") {
            Some(args) => write!(f, " {args}"),
            None => Ok(()),
        }
    }
}

/// Error with a `Msg` and an optional cause. It is an error type (not a `.context`) so `parts` can
/// recognize the layer: anyhow cannot recover a context through `downcast_ref`.
#[derive(Debug)]
pub struct Failure {
    pub msg: Msg,
    source: Option<anyhow::Error>,
}

impl Failure {
    pub fn new(msg: Msg) -> Self {
        Failure { msg, source: None }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl std::error::Error for Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

impl Msg {
    pub fn err(self) -> anyhow::Error {
        Failure::new(self).into()
    }
}

pub trait MsgExt<T> {
    fn msg(self, m: Msg) -> anyhow::Result<T>;
}

impl<T, E: Into<anyhow::Error>> MsgExt<T> for Result<T, E> {
    fn msg(self, m: Msg) -> anyhow::Result<T> {
        self.map_err(|e| {
            Failure {
                msg: m,
                source: Some(e.into()),
            }
            .into()
        })
    }
}

impl<T> MsgExt<T> for Option<T> {
    fn msg(self, m: Msg) -> anyhow::Result<T> {
        self.ok_or_else(|| m.err())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorPart {
    Msg(Msg),
    Raw { raw: String },
}

/// The error's `Msg`, if it is ours.
pub fn failure_msg(e: &anyhow::Error) -> Option<&Msg> {
    e.downcast_ref::<Failure>().map(|f| &f.msg)
}

/// Our part (`Msg`, translated) and third-party parts (`Raw`, verbatim), layer by layer.
pub fn parts(e: &anyhow::Error) -> Vec<ErrorPart> {
    e.chain()
        .map(|layer| match layer.downcast_ref::<Failure>() {
            Some(f) => ErrorPart::Msg(f.msg.clone()),
            None => ErrorPart::Raw {
                raw: layer.to_string(),
            },
        })
        .collect()
}

/// What is stored on a failed bookmark.
#[derive(Debug, Clone, PartialEq)]
pub struct BookmarkError(pub Vec<ErrorPart>);

impl BookmarkError {
    pub fn from_anyhow(e: &anyhow::Error) -> Self {
        BookmarkError(parts(e))
    }

    /// An error that is only third-party text.
    #[cfg(test)]
    pub(crate) fn raw(text: &str) -> Self {
        BookmarkError(vec![ErrorPart::Raw { raw: text.into() }])
    }

    pub fn to_json(&self) -> String {
        // A `Vec<ErrorPart>` always serializes; the empty one only backs the `Result`.
        serde_json::to_string(&self.0).unwrap_or_else(|_| "[]".into())
    }

    /// Never fails: an unreadable item (e.g. a code from another version) becomes `Raw` with its JSON.
    pub fn from_json(s: &str) -> Self {
        let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(s) else {
            return BookmarkError(vec![ErrorPart::Raw { raw: s.into() }]);
        };
        BookmarkError(
            items
                .into_iter()
                .map(|v| {
                    serde_json::from_value(v.clone())
                        .unwrap_or(ErrorPart::Raw { raw: v.to_string() })
                })
                .collect(),
        )
    }
}

impl fmt::Display for BookmarkError {
    /// Like `Msg`'s: technical form, layer by layer, for logs and tests.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, part) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(": ")?;
            }
            match part {
                ErrorPart::Msg(m) => m.fmt(f)?,
                ErrorPart::Raw { raw } => f.write_str(raw)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_keeps_messages_and_raw_causes() {
        let io: anyhow::Result<()> = Err(std::io::Error::other("disk").into());
        let e = io.msg(Msg::NoContent).unwrap_err();
        assert_eq!(
            parts(&e),
            vec![
                ErrorPart::Msg(Msg::NoContent),
                ErrorPart::Raw { raw: "disk".into() }
            ]
        );
    }

    #[test]
    fn json_round_trip() {
        let b = BookmarkError(vec![
            ErrorPart::Msg(Msg::NoContent),
            ErrorPart::Raw { raw: "x".into() },
        ]);
        assert_eq!(b.to_json(), r#"[{"code":"no-content"},{"raw":"x"}]"#);
        assert_eq!(BookmarkError::from_json(&b.to_json()), b);
    }

    #[test]
    fn unknown_code_reads_as_raw() {
        let b = BookmarkError::from_json(r#"[{"code":"from-the-future","args":{"a":1}}]"#);
        assert_eq!(
            b.0,
            vec![ErrorPart::Raw {
                raw: r#"{"code":"from-the-future","args":{"a":1}}"#.into()
            }]
        );
    }

    #[test]
    fn broken_json_reads_as_raw() {
        assert_eq!(
            BookmarkError::from_json("no json").0,
            vec![ErrorPart::Raw {
                raw: "no json".into()
            }]
        );
    }

    #[test]
    fn display_and_option_msg() {
        assert_eq!(Msg::NoContent.to_string(), "no-content");
        assert_eq!(
            parts(&None::<u8>.msg(Msg::NoContent).unwrap_err()),
            vec![ErrorPart::Msg(Msg::NoContent)]
        );
    }

    #[test]
    fn nested_context_keeps_every_layer() {
        let inner = anyhow::anyhow!("inner").context("ctx");
        let e = Err::<(), _>(inner).msg(Msg::NoContent).unwrap_err();
        assert_eq!(
            parts(&e),
            vec![
                ErrorPart::Msg(Msg::NoContent),
                ErrorPart::Raw { raw: "ctx".into() },
                ErrorPart::Raw {
                    raw: "inner".into()
                }
            ]
        );
    }
}
