//! Bookmarks core. The Tauri app, the MCP server and the CLI only wrap it.

pub mod ai;
pub mod ai_health;
pub mod blocked;
pub mod capture;
pub mod chunk;
pub mod download;
pub mod extract;
pub mod genai_model;
pub mod groups;
pub mod msg;
pub mod netscape;
pub mod notifications;
pub mod pipeline;
pub mod profiles;
pub mod search;
pub mod settings;
pub mod store;
pub mod transfer;
pub mod vectors;

use crate::msg::{Msg, MsgExt};
use std::path::PathBuf;
use url::Url;

/// Environment variable that overrides the database path.
pub const DB_ENV: &str = "MARKS_DB";
const DATA_DIR: &str = "marks";
const DB_FILE: &str = "marks.db";

/// `<local data>/marks/marks.db`, or the current directory if the OS has none.
pub fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DATA_DIR)
        .join(DB_FILE)
}

/// Normalizes a URL for deduplication: no fragment (except hash routes `#/…` and `#!…`), no `utm_*`, lowercase host.
/// Keeps the trailing slash: on some sites `/a` and `/a/` are different resources.
pub fn normalize_url(raw: &str) -> anyhow::Result<String> {
    let mut url = Url::parse(raw.trim()).msg(Msg::InvalidUrl)?;
    if !url.fragment().is_some_and(|f| f.starts_with(['/', '!'])) {
        url.set_fragment(None);
    }
    let kept: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(k, _)| !k.starts_with("utm_"))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    if kept.is_empty() {
        url.set_query(None);
    } else {
        url.query_pairs_mut().clear().extend_pairs(kept);
    }
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::normalize_url;
    use crate::msg::{Msg, failure_msg};

    #[test]
    fn normalizes_for_dedupe() {
        let a = normalize_url("HTTPS://Example.com/Post?utm_source=x#top").unwrap();
        let b = normalize_url("https://example.com/Post").unwrap();
        assert_eq!(a, b);
        assert_eq!(
            normalize_url("https://e.com/?q=1&utm_medium=m").unwrap(),
            "https://e.com/?q=1"
        );
        let err = normalize_url("not a url").unwrap_err();
        assert_eq!(failure_msg(&err), Some(&Msg::InvalidUrl));
    }

    #[test]
    fn normalize_keeps_hash_routes() {
        for kept in ["https://app.com/#/inbox", "https://app.com/#!/a"] {
            assert_eq!(normalize_url(kept).unwrap(), kept);
        }
        assert_eq!(
            normalize_url("https://app.com/doc#ancla").unwrap(),
            "https://app.com/doc"
        );
    }
}
