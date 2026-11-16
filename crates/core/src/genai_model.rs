//! `Model` over genai: Ollama, oMLX, OpenAI, Anthropic and OpenAI-compatible servers. Chat with
//! Ollama goes through its native API (`/api/chat`), the only one that accepts `num_ctx` (ADR 0013).

use crate::ai::{Model, SummarizeInput, Summary, extract_prompt, parse_summary, summarize_prompt};
use crate::msg::{Msg, MsgExt};
use crate::settings::{AiConnection, Provider};
use anyhow::Result;
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatResponseFormat, ReasoningEffort};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget, WebConfig};
use serde_json::{Value, json};
use std::time::Duration;

/// Texts per embeddings request.
pub const EMBED_BATCH_SIZE: usize = 32;
/// Local servers ignore the key, but the OpenAI adapter requires one.
const LOCAL_PLACEHOLDER_KEY: &str = "ollama";
const API_VERSION_PATH: &str = "/v1";
/// Timeout for connecting to the model server, so an endpoint with no listener fails in seconds.
/// It does not bound the response, which can take minutes with 60 000 characters of input.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

const OLLAMA_CHAT_PATH: &str = "/api/chat";
/// Context requested from Ollama, in tokens. Without it Ollama uses 4 096 below 24 GiB of VRAM and
/// silently drops the start of the prompt, instructions included. Measured with qwen3:8b (Ollama
/// 0.40.14): 60 000 characters of HTML are ~19 000 tokens, and the output Markdown also takes
/// context, so extraction (`MAX_AI_SOURCE_CHARS`) asks for 32 768.
const OLLAMA_EXTRACT_NUM_CTX: u32 = 32_768;
/// 12 000 characters of text (`MAX_PROMPT_CHARS`) plus the prompt are ~3 000 tokens, but with 300
/// existing tags it is already ~4 900, and other scripts spend more tokens per character.
const OLLAMA_SUMMARY_NUM_CTX: u32 = 16_384;
/// Ollama answers 400 with, e.g., "request (6017 tokens) exceeds the available context size
/// (1024 tokens), try increasing it" when `truncate: false` and the input does not fit.
const OLLAMA_TOO_LONG_MARKERS: [&str; 2] = ["exceeds", "context"];

/// The two chat calls, each with its own output format and context.
#[derive(Debug, Clone, Copy)]
enum ChatRole {
    Summary,
    Extraction,
}

pub struct GenaiModel {
    client: Client,
    chat_model: String,
    embed_model: String,
    embedding_id: String,
    provider: Provider,
    /// `/api/chat` URL, Ollama only.
    ollama_chat_url: Option<String>,
}

impl GenaiModel {
    /// Summary and extraction are both built here, each with its own connection, key and models.
    pub fn new(
        connection: &AiConnection,
        api_key: Option<String>,
        chat_model: &str,
        embed_model: &str,
    ) -> Result<Self> {
        let provider = connection.provider;
        let auth = auth_key(provider, api_key)?;
        let endpoint = connection.effective_endpoint();
        // Without an endpoint genai would use OpenAI's host and the content would leave the machine.
        if endpoint.is_none() && provider.requires_endpoint() {
            return Err(Msg::EndpointRequired.err());
        }
        let ollama_chat_url = endpoint
            .as_deref()
            .filter(|_| provider == Provider::Ollama)
            .map(|base| format!("{}{OLLAMA_CHAT_PATH}", unversioned_endpoint(base)));
        let endpoint = endpoint.map(|base| Endpoint::from_owned(versioned_endpoint(&base)));
        Ok(Self {
            client: client(endpoint, auth)?,
            chat_model: chat_model.to_string(),
            embed_model: embed_model.to_string(),
            embedding_id: format!("{provider}:{embed_model}"),
            provider,
            ollama_chat_url,
        })
    }

    fn model_iden(&self, name: &str) -> ModelIden {
        ModelIden::new(adapter_kind(self.provider), name)
    }

    async fn chat(&self, prompt: String, role: ChatRole) -> Result<String> {
        if let Some(url) = &self.ollama_chat_url {
            let body = ollama_chat_body(&self.chat_model, prompt, role);
            return ollama_chat(url.clone(), body).await;
        }
        let options = match role {
            ChatRole::Summary => chat_options(),
            ChatRole::Extraction => extract_options(),
        };
        let response = self
            .client
            .exec_chat(
                self.model_iden(&self.chat_model),
                ChatRequest::from_user(prompt),
                Some(&options),
            )
            .await
            .msg(Msg::ChatFailed)?;
        response
            .first_text()
            .map(str::to_string)
            .msg(Msg::EmptyChat)
    }

    async fn embed_batch(&self, batch: &[String]) -> Result<Vec<Vec<f32>>> {
        let response = self
            .client
            .embed_batch(self.model_iden(&self.embed_model), batch.to_vec(), None)
            .await
            .msg(Msg::EmbedFailed)?;
        let mut embeddings = response.embeddings;
        if embeddings.len() != batch.len() {
            return Err(Msg::EmbedBatchCount.err());
        }
        embeddings.sort_by_key(|e| e.index);
        Ok(embeddings.into_iter().map(|e| e.vector).collect())
    }
}

fn adapter_kind(provider: Provider) -> AdapterKind {
    match provider {
        Provider::Anthropic => AdapterKind::Anthropic,
        Provider::Ollama | Provider::Omlx | Provider::Openai | Provider::OpenaiCompatible => {
            AdapterKind::OpenAI
        }
    }
}

fn auth_key(provider: Provider, api_key: Option<String>) -> Result<String> {
    match (provider, api_key) {
        (_, Some(key)) => Ok(key),
        (Provider::Openai | Provider::Anthropic, None) => Err(Msg::ApiKeyRequired.err()),
        (Provider::Ollama | Provider::Omlx | Provider::OpenaiCompatible, None) => {
            Ok(LOCAL_PLACEHOLDER_KEY.to_string())
        }
    }
}

/// `http://h:1`, `http://h:1/` and `http://h:1/v1` ⇒ `http://h:1/v1/`.
fn versioned_endpoint(base: &str) -> String {
    format!("{}{API_VERSION_PATH}/", unversioned_endpoint(base))
}

/// `http://h:1/`, `http://h:1/v1` and `http://h:1/v1/` ⇒ `http://h:1`.
fn unversioned_endpoint(base: &str) -> &str {
    let base = base.trim_end_matches('/');
    base.strip_suffix(API_VERSION_PATH).unwrap_or(base)
}

/// Same as `chat_options` and `extract_options`, plus the role's context and `truncate: false`,
/// so an input that does not fit fails instead of arriving truncated.
fn ollama_chat_body(model: &str, prompt: String, role: ChatRole) -> Value {
    let num_ctx = match role {
        ChatRole::Summary => OLLAMA_SUMMARY_NUM_CTX,
        ChatRole::Extraction => OLLAMA_EXTRACT_NUM_CTX,
    };
    let mut body = json!({
        "model": model,
        "messages": [{ "role": "user", "content": prompt }],
        "stream": false,
        "think": false,
        "truncate": false,
        "options": { "num_ctx": num_ctx },
    });
    if matches!(role, ChatRole::Summary) {
        body["format"] = json!("json");
    }
    body
}

/// `ureq` is blocking, so it runs on its own thread. Same limits as genai: 5 s to connect and none
/// for the response (`pipeline` sets the extraction one). No error carries the prompt.
async fn ollama_chat(url: String, body: Value) -> Result<String> {
    tokio::task::spawn_blocking(move || ollama_chat_blocking(&url, &body))
        .await
        .msg(Msg::ChatFailed)?
}

fn ollama_chat_blocking(url: &str, body: &Value) -> Result<String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .http_status_as_error(false)
        .build()
        .into();
    let mut response = agent
        .post(url)
        .header("Content-Type", "application/json")
        .send(body.to_string())
        .msg(Msg::ChatFailed)?;
    let status = response.status().as_u16();
    let text = response.body_mut().read_to_string().msg(Msg::ChatFailed)?;
    let reply: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    if !(200..300).contains(&status) {
        let detail = reply["error"].as_str().unwrap_or_default();
        return ollama_status_failure(status, detail);
    }
    let content = reply["message"]["content"]
        .as_str()
        .msg(Msg::OllamaBadResponse)?;
    if content.trim().is_empty() {
        return Err(Msg::EmptyChat.err());
    }
    Ok(content.to_string())
}

/// Always `Err`: Ollama's status, under "does not fit" if the detail says so, else under the chat failure.
fn ollama_status_failure(status: u16, detail: &str) -> Result<String> {
    let cause = Msg::OllamaStatus {
        status,
        detail: detail.to_string(),
    }
    .err();
    let wrapper = if OLLAMA_TOO_LONG_MARKERS.iter().all(|m| detail.contains(m)) {
        Msg::OllamaTooLong
    } else {
        Msg::ChatFailed
    };
    Err(cause).msg(wrapper)
}

/// `endpoint` `None` keeps the adapter's default.
fn client(endpoint: Option<Endpoint>, auth: String) -> Result<Client> {
    let resolver = ServiceTargetResolver::from_resolver_fn(
        move |target: ServiceTarget| -> genai::resolver::Result<ServiceTarget> {
            Ok(ServiceTarget {
                endpoint: endpoint.unwrap_or(target.endpoint),
                auth: AuthData::from_single(auth),
                model: target.model,
            })
        },
    );
    Client::builder()
        .with_service_target_resolver(resolver)
        .with_web_config(WebConfig::default().with_connect_timeout(CONNECT_TIMEOUT))
        .build()
        .msg(Msg::ClientBuild)
}

fn chat_options() -> ChatOptions {
    ChatOptions::default()
        .with_reasoning_effort(ReasoningEffort::Zero)
        .with_response_format(ChatResponseFormat::JsonMode)
}

/// Extraction answers Markdown, not JSON.
fn extract_options() -> ChatOptions {
    ChatOptions::default().with_reasoning_effort(ReasoningEffort::Zero)
}

#[async_trait::async_trait]
impl Model for GenaiModel {
    async fn summarize(&self, input: SummarizeInput<'_>) -> Result<Summary> {
        let text = self
            .chat(summarize_prompt(&input), ChatRole::Summary)
            .await?;
        parse_summary(text.trim())
    }

    /// With the chat model: for the extraction role the app builds it with the extraction model.
    async fn extract_markdown(&self, input: &str) -> Result<String> {
        self.chat(extract_prompt(input), ChatRole::Extraction).await
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if self.provider == Provider::Anthropic {
            return Err(Msg::AnthropicNoEmbeddings.err());
        }
        let mut vectors = Vec::with_capacity(texts.len());
        for batch in texts.chunks(EMBED_BATCH_SIZE) {
            vectors.extend(self.embed_batch(batch).await?);
        }
        Ok(vectors)
    }

    fn embedding_id(&self) -> String {
        self.embedding_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{ErrorPart, parts};
    use crate::settings::AiSettings;

    fn build(settings: &AiSettings, api_key: Option<String>) -> Result<GenaiModel> {
        GenaiModel::new(
            &settings.connection,
            api_key,
            &settings.chat_model,
            &settings.embed_model,
        )
    }

    fn with_connection(provider: Provider, endpoint: Option<&str>) -> AiSettings {
        AiSettings {
            connection: AiConnection {
                provider,
                endpoint: endpoint.map(String::from),
            },
            ..AiSettings::default()
        }
    }

    #[test]
    fn embedding_id_uses_provider_and_model() {
        let settings = with_connection(Provider::OpenaiCompatible, Some("http://localhost:8000"));
        let model = build(&settings, None).unwrap();
        assert_eq!(
            model.embedding_id(),
            format!("openai_compatible:{}", settings.embed_model)
        );
    }

    #[test]
    fn openai_without_key_is_error() {
        for provider in [Provider::Openai, Provider::Anthropic] {
            let err = build(&with_connection(provider, None), None).err().unwrap();
            assert_eq!(err.to_string(), Msg::ApiKeyRequired.to_string());
        }
    }

    #[test]
    fn openai_compatible_without_endpoint_is_error() {
        for provider in [Provider::Omlx, Provider::OpenaiCompatible] {
            let err = build(&with_connection(provider, None), None).err().unwrap();
            assert_eq!(err.to_string(), Msg::EndpointRequired.to_string());
        }
    }

    #[test]
    fn endpoint_gets_single_api_version_suffix() {
        for base in [
            "http://h:1",
            "http://h:1/",
            "http://h:1/v1",
            "http://h:1/v1/",
        ] {
            assert_eq!(versioned_endpoint(base), "http://h:1/v1/");
        }
    }

    #[tokio::test]
    async fn anthropic_embed_is_error() {
        let settings = with_connection(Provider::Anthropic, None);
        let model = build(&settings, Some("k".into())).unwrap();
        let err = model.embed(&["hola".to_string()]).await.unwrap_err();
        assert_eq!(err.to_string(), Msg::AnthropicNoEmbeddings.to_string());
    }

    /// Single-request HTTP server: answers `status` and `body`, and returns the request line and
    /// the received body.
    fn one_shot_server(
        status: &'static str,
        body: &'static str,
    ) -> (String, std::thread::JoinHandle<(String, Value)>) {
        use std::io::{BufRead, BufReader, Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                if let Some((name, value)) = line.split_once(':')
                    && name.eq_ignore_ascii_case("content-length")
                {
                    length = value.trim().parse().unwrap();
                }
            }
            let mut received = vec![0; length];
            reader.read_exact(&mut received).unwrap();
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            reader.get_mut().write_all(response.as_bytes()).unwrap();
            let request_line = request_line
                .split(' ')
                .take(2)
                .collect::<Vec<_>>()
                .join(" ");
            (request_line, serde_json::from_slice(&received).unwrap())
        });
        (base, handle)
    }

    fn model_at(provider: Provider, endpoint: &str) -> GenaiModel {
        let connection = AiConnection {
            provider,
            endpoint: Some(endpoint.to_string()),
        };
        GenaiModel::new(&connection, None, "reader", "embed").unwrap()
    }

    const OLLAMA_OK: &str = r##"{"message":{"role":"assistant","content":"# Hola"},"done":true}"##;

    #[test]
    fn ollama_body_sets_context_and_disables_truncation_per_role() {
        let extract = ollama_chat_body("reader", "p".into(), ChatRole::Extraction);
        assert_eq!(extract["model"], "reader");
        assert_eq!(extract["messages"][0]["content"], "p");
        assert_eq!(extract["options"]["num_ctx"], OLLAMA_EXTRACT_NUM_CTX);
        assert_eq!(extract["truncate"], false);
        assert_eq!(extract["stream"], false);
        assert_eq!(extract["think"], false);
        assert!(extract.get("format").is_none());

        let summary = ollama_chat_body("qwen", "p".into(), ChatRole::Summary);
        assert_eq!(summary["model"], "qwen");
        assert_eq!(summary["options"]["num_ctx"], OLLAMA_SUMMARY_NUM_CTX);
        assert_eq!(summary["truncate"], false);
        assert_eq!(summary["format"], "json");
    }

    #[test]
    fn unversioned_endpoint_strips_api_version() {
        for base in [
            "http://h:1",
            "http://h:1/",
            "http://h:1/v1",
            "http://h:1/v1/",
        ] {
            assert_eq!(unversioned_endpoint(base), "http://h:1");
        }
    }

    #[tokio::test]
    async fn ollama_extraction_goes_to_the_native_api() {
        let (base, server) = one_shot_server("200 OK", OLLAMA_OK);
        let model = model_at(Provider::Ollama, &format!("{base}/v1"));
        let markdown = model.extract_markdown("<p>hola</p>").await.unwrap();
        assert_eq!(markdown, "# Hola");
        let (request_line, body) = server.join().unwrap();
        assert_eq!(request_line, "POST /api/chat");
        assert_eq!(body["model"], "reader");
        assert_eq!(body["options"]["num_ctx"], OLLAMA_EXTRACT_NUM_CTX);
        assert_eq!(body["truncate"], false);
    }

    #[tokio::test]
    async fn ollama_input_too_long_is_a_readable_error() {
        let (base, server) = one_shot_server(
            "400 Bad Request",
            r#"{"error":"request (6017 tokens) exceeds the available context size (1024 tokens), try increasing it"}"#,
        );
        let err = model_at(Provider::Ollama, &base)
            .extract_markdown("<p>hola</p>")
            .await
            .unwrap_err();
        server.join().unwrap();
        assert_eq!(err.to_string(), Msg::OllamaTooLong.to_string());
        assert!(format!("{err:#}").contains("6017 tokens"));
    }

    #[tokio::test]
    async fn ollama_other_status_is_a_chat_error_with_the_detail() {
        let (base, server) =
            one_shot_server("404 Not Found", r#"{"error":"model 'reader' not found"}"#);
        let err = model_at(Provider::Ollama, &base)
            .extract_markdown("<p>hola</p>")
            .await
            .unwrap_err();
        server.join().unwrap();
        assert_eq!(
            parts(&err),
            vec![
                ErrorPart::Msg(Msg::ChatFailed),
                ErrorPart::Msg(Msg::OllamaStatus {
                    status: 404,
                    detail: "model 'reader' not found".into()
                })
            ]
        );
    }

    #[tokio::test]
    async fn ollama_unreachable_is_a_chat_error() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        let err = model_at(Provider::Ollama, &base)
            .extract_markdown("<p>hola</p>")
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), Msg::ChatFailed.to_string());
    }

    #[tokio::test]
    async fn other_providers_still_go_through_genai() {
        let (base, server) = one_shot_server(
            "200 OK",
            r##"{"id":"x","object":"chat.completion","created":0,"model":"reader","choices":[{"index":0,"message":{"role":"assistant","content":"# Hola"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"##,
        );
        let model = model_at(Provider::OpenaiCompatible, &base);
        assert_eq!(
            model.extract_markdown("<p>hola</p>").await.unwrap(),
            "# Hola"
        );
        let (request_line, body) = server.join().unwrap();
        assert_eq!(request_line, "POST /v1/chat/completions");
        assert!(body.get("num_ctx").is_none() && body.get("options").is_none());
    }

    #[ignore = "requiere Ollama local con los modelos por defecto"]
    #[tokio::test]
    async fn ollama_roundtrip() {
        let model = build(&AiSettings::default(), None).unwrap();
        let vectors = model.embed(&["hola".to_string()]).await.unwrap();
        assert_eq!(vectors.len(), 1);
        assert!(!vectors[0].is_empty());
        let summary = model
            .summarize(SummarizeInput {
                url: "https://example.com",
                title: "Ejemplo",
                markdown: "# Pan casero\n\nReceta de pan con harina, agua, sal y levadura.",
                existing_categories: &[],
                existing_tags: &[],
            })
            .await
            .unwrap();
        assert!(!summary.summary.is_empty());
    }

    /// Through the real extraction path: a key at the start of `MAX_AI_SOURCE_CHARS` of HTML
    /// survives, which it did not with Ollama's default context.
    #[ignore = "requiere Ollama local con qwen3:8b; tarda ~30 s"]
    #[tokio::test]
    async fn ollama_long_input_keeps_its_start() {
        let row = "<tr><td class=\"cell\">harina</td><td class=\"cell\">500 g</td></tr>\n";
        let filler = row.repeat(crate::ai::MAX_AI_SOURCE_CHARS / row.len());
        let prompt = format!(
            "The secret code is ZEBRA-4417.\n```html\n<table>\n{filler}</table>\n```\n\
             What is the secret code stated in the very first sentence of this message? \
             Reply with only the code."
        );
        let model = build(&AiSettings::default(), None).unwrap();
        let reply = model.chat(prompt, ChatRole::Extraction).await.unwrap();
        assert!(reply.contains("ZEBRA-4417"), "{reply}");
    }
}
