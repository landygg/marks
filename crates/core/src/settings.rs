//! AI settings (in the primary database's `settings` table) and provider API keys (in the system
//! keychain), one per role: summary and extraction. The local API token is per profile and lives
//! in the `profiles` registry.

use crate::msg::{Msg, MsgExt};
use crate::store::Store;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const KEYRING_SERVICE: &str = "dev.landygg.marks";
pub const KEYRING_USER: &str = "api_key";
pub const EXTRACTION_KEYRING_USER: &str = "extraction_api_key";
pub const OLLAMA_DEFAULT_ENDPOINT: &str = "http://localhost:11434";
pub const DEFAULT_CHAT_MODEL: &str = "qwen3:8b";
pub const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text";
const SETTINGS_KEY: &str = "ai";
pub const EXTRACTION_SETTINGS_KEY: &str = "ai_extraction";
pub const API_TOKEN_KEY: &str = "api_token";
const API_TOKEN_BYTES: usize = 32;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Ollama,
    Omlx,
    Openai,
    Anthropic,
    OpenaiCompatible,
}

impl Provider {
    /// snake_case name, the same serde uses.
    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Ollama => "ollama",
            Provider::Omlx => "omlx",
            Provider::Openai => "openai",
            Provider::Anthropic => "anthropic",
            Provider::OpenaiCompatible => "openai_compatible",
        }
    }

    pub(crate) fn requires_endpoint(self) -> bool {
        matches!(self, Provider::Omlx | Provider::OpenaiCompatible)
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Provider and endpoint for one role. Flattened into each setting, so the JSON stored under `ai`
/// does not change. A missing field takes its `Default` value.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct AiConnection {
    pub provider: Provider,
    pub endpoint: Option<String>,
}

impl Default for AiConnection {
    fn default() -> Self {
        Self {
            provider: Provider::Ollama,
            endpoint: None,
        }
    }
}

impl AiConnection {
    /// Endpoint to use; `None` means the provider's default.
    pub fn effective_endpoint(&self) -> Option<String> {
        match self.provider {
            Provider::Ollama => self
                .endpoint
                .clone()
                .or_else(|| Some(OLLAMA_DEFAULT_ENDPOINT.to_string())),
            _ => self.endpoint.clone(),
        }
    }

    /// The endpoint trimmed; empty counts as none.
    fn normalized(&self) -> AiConnection {
        AiConnection {
            provider: self.provider,
            endpoint: self
                .endpoint
                .as_deref()
                .map(str::trim)
                .filter(|e| !e.is_empty())
                .map(str::to_string),
        }
    }

    fn validated(&self) -> Result<AiConnection> {
        let connection = self.normalized();
        if connection.provider.requires_endpoint() && connection.endpoint.is_none() {
            return Err(Msg::EndpointRequired.err());
        }
        Ok(connection)
    }
}

/// Summary and search. A missing field takes its `Default` value.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct AiSettings {
    /// Without AI, bookmarks are only extracted. Also turns off AI extraction.
    pub enabled: bool,
    #[serde(flatten)]
    pub connection: AiConnection,
    pub chat_model: String,
    pub embed_model: String,
}

impl Default for AiSettings {
    /// A new install, with no stored settings, starts with AI disabled.
    fn default() -> Self {
        Self {
            enabled: false,
            connection: AiConnection::default(),
            chat_model: DEFAULT_CHAT_MODEL.to_string(),
            embed_model: DEFAULT_EMBED_MODEL.to_string(),
        }
    }
}

impl AiSettings {
    fn validated(&self) -> Result<AiSettings> {
        // Disabled is always saved; it is validated when turned back on.
        let connection = if self.enabled {
            self.connection.validated()?
        } else {
            self.connection.normalized()
        };
        let chat_model = self.chat_model.trim().to_string();
        let embed_model = self.embed_model.trim().to_string();
        if self.enabled && (chat_model.is_empty() || embed_model.is_empty()) {
            return Err(Msg::EmptyModel.err());
        }
        Ok(AiSettings {
            enabled: self.enabled,
            connection,
            chat_model,
            embed_model,
        })
    }
}

/// AI extraction (key `EXTRACTION_SETTINGS_KEY`). With nothing stored it uses the summary settings.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct ExtractionSettings {
    pub same_as_summary: bool,
    #[serde(flatten)]
    pub connection: AiConnection,
    pub model: String,
}

impl Default for ExtractionSettings {
    fn default() -> Self {
        Self {
            same_as_summary: true,
            connection: AiConnection::default(),
            model: String::new(),
        }
    }
}

impl ExtractionSettings {
    /// Effective connection and model: the summary ones if `same_as_summary`.
    pub fn resolve<'a>(&'a self, summary: &'a AiSettings) -> (&'a AiConnection, &'a str) {
        if self.same_as_summary {
            (&summary.connection, &summary.chat_model)
        } else {
            (&self.connection, &self.model)
        }
    }

    fn validated(&self) -> Result<ExtractionSettings> {
        let model = self.model.trim().to_string();
        if model.is_empty() {
            return Err(Msg::EmptyExtractionModel.err());
        }
        Ok(ExtractionSettings {
            same_as_summary: false,
            connection: self.connection.validated()?,
            model,
        })
    }
}

/// A model's role: picks its keychain entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiRole {
    Summary,
    Extraction,
}

pub fn load(store: &Store) -> Result<AiSettings> {
    match store.get_setting(SETTINGS_KEY)? {
        Some(json) => serde_json::from_str(&json).msg(Msg::InvalidSettings),
        None => Ok(AiSettings::default()),
    }
}

pub fn save(store: &Store, s: &AiSettings) -> Result<()> {
    store.set_setting(SETTINGS_KEY, &serde_json::to_string(&s.validated()?)?)
}

fn normalized_key(key: Option<&str>) -> Option<&str> {
    key.map(str::trim).filter(|k| !k.is_empty())
}

pub fn load_extraction(store: &Store) -> Result<ExtractionSettings> {
    match store.get_setting(EXTRACTION_SETTINGS_KEY)? {
        Some(json) => serde_json::from_str(&json).msg(Msg::InvalidSettings),
        None => Ok(ExtractionSettings::default()),
    }
}

/// With `same_as_summary` it is saved unvalidated: its connection and model are unused.
pub fn save_extraction(store: &Store, s: &ExtractionSettings) -> Result<()> {
    let json = if s.same_as_summary {
        serde_json::to_string(s)?
    } else {
        serde_json::to_string(&s.validated()?)?
    };
    store.set_setting(EXTRACTION_SETTINGS_KEY, &json)
}

/// Global key of `UpdateSettings`, in the main database like the AI settings.
const UPDATE_SETTINGS_KEY: &str = "updates";

/// Whether the app checks GitHub Releases for a new version on its own.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(default)]
pub struct UpdateSettings {
    pub auto: bool,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self { auto: true }
    }
}

/// A broken value reads as the default: a switch is not worth refusing to start the check.
pub fn load_update_settings(store: &Store) -> Result<UpdateSettings> {
    Ok(store
        .get_setting(UPDATE_SETTINGS_KEY)?
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default())
}

pub fn save_update_settings(store: &Store, s: UpdateSettings) -> Result<()> {
    store.set_setting(UPDATE_SETTINGS_KEY, &serde_json::to_string(&s)?)
}

/// Keychain entry for each role, within the same `KEYRING_SERVICE`.
fn keyring_user(role: AiRole) -> &'static str {
    match role {
        AiRole::Summary => KEYRING_USER,
        AiRole::Extraction => EXTRACTION_KEYRING_USER,
    }
}

fn keyring_entry(role: AiRole) -> Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, keyring_user(role)).msg(Msg::KeyringEntry)
}

pub fn api_key(role: AiRole) -> Result<Option<String>> {
    match keyring_entry(role)?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e).msg(Msg::KeyringRead),
    }
}

/// Stores the key trimmed; `None`, empty or whitespace-only deletes it, and a missing entry
/// counts as success.
pub fn set_api_key(role: AiRole, key: Option<&str>) -> Result<()> {
    let entry = keyring_entry(role)?;
    match normalized_key(key) {
        Some(key) => entry.set_password(key).msg(Msg::KeyringWrite),
        None => match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e).msg(Msg::KeyringDelete),
        },
    }
}

/// Random system bytes in lowercase hex; no fallback if it fails.
pub(crate) fn random_token() -> Result<String> {
    let mut bytes = [0u8; API_TOKEN_BYTES];
    getrandom::fill(&mut bytes).msg(Msg::TokenRandom)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> Store {
        Store::open_in_memory().unwrap()
    }

    #[test]
    fn api_key_is_trimmed_and_blank_means_delete() {
        assert_eq!(normalized_key(Some("  sk-1 \n")), Some("sk-1"));
        assert_eq!(normalized_key(Some("   ")), None);
        assert_eq!(normalized_key(Some("")), None);
        assert_eq!(normalized_key(None), None);
    }

    #[test]
    fn default_is_disabled() {
        assert!(!AiSettings::default().enabled);
    }

    #[test]
    fn ai_settings_without_enabled_are_off() {
        let s = store();
        s.set_setting(
            SETTINGS_KEY,
            r#"{"provider":"ollama","endpoint":null,"chat_model":"m","embed_model":"e"}"#,
        )
        .unwrap();
        assert!(!load(&s).unwrap().enabled);
    }

    #[test]
    fn disabled_saves_without_endpoint_or_models() {
        let s = store();
        let off = AiSettings {
            enabled: false,
            chat_model: " ".into(),
            embed_model: "".into(),
            ..omlx(None)
        };
        save(&s, &off).unwrap();
        let loaded = load(&s).unwrap();
        assert!(!loaded.enabled);
        assert_eq!(loaded.connection.provider, Provider::Omlx);
        assert_eq!(loaded.connection.endpoint, None);
        assert_eq!(loaded.chat_model, "");
    }

    #[test]
    fn disabled_keeps_the_other_fields() {
        let s = store();
        let off = AiSettings {
            enabled: false,
            ..omlx(Some("http://h:1"))
        };
        save(&s, &off).unwrap();
        assert_eq!(load(&s).unwrap(), off);
    }

    #[test]
    fn load_returns_default_when_unset() {
        assert_eq!(load(&store()).unwrap(), AiSettings::default());
    }

    #[test]
    fn save_then_load_roundtrip() {
        let s = store();
        let custom = AiSettings {
            enabled: true,
            connection: AiConnection {
                provider: Provider::OpenaiCompatible,
                endpoint: Some("http://localhost:8000/v1".into()),
            },
            chat_model: "m".into(),
            embed_model: "e".into(),
        };
        save(&s, &custom).unwrap();
        assert_eq!(load(&s).unwrap(), custom);
        assert_eq!(custom.connection.provider.to_string(), "openai_compatible");
    }

    fn omlx(endpoint: Option<&str>) -> AiSettings {
        AiSettings {
            enabled: true,
            connection: AiConnection {
                provider: Provider::Omlx,
                endpoint: endpoint.map(String::from),
            },
            ..AiSettings::default()
        }
    }

    #[test]
    fn save_rejects_omlx_without_endpoint() {
        let err = save(&store(), &omlx(None)).unwrap_err();
        assert_eq!(err.to_string(), Msg::EndpointRequired.to_string());
    }

    #[test]
    fn save_treats_blank_endpoint_as_missing() {
        let err = save(&store(), &omlx(Some("   "))).unwrap_err();
        assert_eq!(err.to_string(), Msg::EndpointRequired.to_string());
    }

    #[test]
    fn save_trims_models_and_endpoint() {
        let s = store();
        let padded = AiSettings {
            chat_model: "  m ".into(),
            ..omlx(Some(" http://x "))
        };
        save(&s, &padded).unwrap();
        let loaded = load(&s).unwrap();
        assert_eq!(loaded.chat_model, "m");
        assert_eq!(loaded.connection.endpoint.as_deref(), Some("http://x"));
    }

    #[test]
    fn save_rejects_empty_model_names() {
        for (chat, embed) in [("", "e"), ("  ", "e"), ("m", ""), ("m", " ")] {
            let bad = AiSettings {
                enabled: true,
                chat_model: chat.into(),
                embed_model: embed.into(),
                ..AiSettings::default()
            };
            let err = save(&store(), &bad).unwrap_err();
            assert_eq!(err.to_string(), Msg::EmptyModel.to_string());
        }
    }

    #[test]
    fn provider_names_match_serde() {
        for p in [
            Provider::Ollama,
            Provider::Omlx,
            Provider::Openai,
            Provider::Anthropic,
            Provider::OpenaiCompatible,
        ] {
            assert_eq!(
                serde_json::to_string(&p).unwrap(),
                format!("\"{}\"", p.as_str())
            );
        }
    }

    #[test]
    fn load_errors_on_corrupt_json() {
        let s = store();
        s.set_setting(SETTINGS_KEY, "{no es json").unwrap();
        assert_eq!(
            load(&s).unwrap_err().to_string(),
            Msg::InvalidSettings.to_string()
        );
    }

    #[test]
    fn effective_endpoint_defaults_for_ollama() {
        assert_eq!(
            AiSettings::default()
                .connection
                .effective_endpoint()
                .as_deref(),
            Some(OLLAMA_DEFAULT_ENDPOINT)
        );
    }

    /// Keys of the stored JSON object, sorted.
    fn keys(json: &str) -> Vec<String> {
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    }

    #[test]
    fn stored_ai_json_reads_unchanged() {
        let with_enabled = r#"{"enabled":false,"provider":"omlx","endpoint":"http://h:1","chat_model":"m","embed_model":"e"}"#;
        let without_enabled =
            r#"{"provider":"anthropic","endpoint":null,"chat_model":"m","embed_model":"e"}"#;
        for (json, enabled, provider, endpoint) in [
            (with_enabled, false, Provider::Omlx, Some("http://h:1")),
            (without_enabled, false, Provider::Anthropic, None),
        ] {
            let s = store();
            s.set_setting(SETTINGS_KEY, json).unwrap();
            let loaded = load(&s).unwrap();
            assert_eq!(
                loaded,
                AiSettings {
                    enabled,
                    connection: AiConnection {
                        provider,
                        endpoint: endpoint.map(String::from),
                    },
                    chat_model: "m".into(),
                    embed_model: "e".into(),
                }
            );
            save(&s, &loaded).unwrap();
            let saved = s.get_setting(SETTINGS_KEY).unwrap().unwrap();
            assert_eq!(keys(&saved), keys(with_enabled));
        }
    }

    #[test]
    fn extraction_defaults_to_summary() {
        let s = store();
        let extraction = load_extraction(&s).unwrap();
        assert!(extraction.same_as_summary);
        let summary = omlx(Some("http://h:1"));
        let (connection, model) = extraction.resolve(&summary);
        assert_eq!(connection, &summary.connection);
        assert_eq!(model, summary.chat_model);
    }

    fn own_extraction(
        provider: Provider,
        endpoint: Option<&str>,
        model: &str,
    ) -> ExtractionSettings {
        ExtractionSettings {
            same_as_summary: false,
            connection: AiConnection {
                provider,
                endpoint: endpoint.map(String::from),
            },
            model: model.into(),
        }
    }

    #[test]
    fn own_extraction_settings_validate() {
        let s = store();
        let err = save_extraction(&s, &own_extraction(Provider::OpenaiCompatible, None, "r"))
            .unwrap_err();
        assert_eq!(err.to_string(), Msg::EndpointRequired.to_string());
        let err = save_extraction(&s, &own_extraction(Provider::Ollama, None, "  ")).unwrap_err();
        assert_eq!(err.to_string(), Msg::EmptyExtractionModel.to_string());
        assert_eq!(s.get_setting(EXTRACTION_SETTINGS_KEY).unwrap(), None);

        let own = own_extraction(Provider::OpenaiCompatible, Some(" http://h:2 "), " reader ");
        save_extraction(&s, &own).unwrap();
        let loaded = load_extraction(&s).unwrap();
        assert_eq!(
            loaded,
            own_extraction(Provider::OpenaiCompatible, Some("http://h:2"), "reader")
        );
        let summary = AiSettings::default();
        let (connection, model) = loaded.resolve(&summary);
        assert_eq!(connection, &loaded.connection);
        assert_eq!(model, "reader");

        let same = ExtractionSettings {
            same_as_summary: true,
            ..own_extraction(Provider::OpenaiCompatible, None, "")
        };
        save_extraction(&s, &same).unwrap();
        assert_eq!(load_extraction(&s).unwrap(), same);
    }

    #[test]
    fn keyring_user_per_role() {
        assert_eq!(keyring_user(AiRole::Summary), KEYRING_USER);
        assert_eq!(keyring_user(AiRole::Extraction), EXTRACTION_KEYRING_USER);
    }

    #[test]
    fn update_settings_default_to_auto() {
        assert!(load_update_settings(&store()).unwrap().auto);
    }

    #[test]
    fn update_settings_round_trip() {
        let s = store();
        save_update_settings(&s, UpdateSettings { auto: false }).unwrap();
        assert!(!load_update_settings(&s).unwrap().auto);
    }

    #[test]
    fn broken_update_settings_fall_back() {
        let s = store();
        s.set_setting(UPDATE_SETTINGS_KEY, "{").unwrap();
        assert!(load_update_settings(&s).unwrap().auto);
    }
}
