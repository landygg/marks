//! AI summary and extraction: the `Model` trait, prompts, parsing and a fake model for tests.

use crate::msg::{Msg, MsgExt};
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Summary {
    pub title: String,
    pub summary: String,
    pub category: String,
    pub tags: Vec<String>,
}

/// Max characters of Markdown sent to the model.
pub const MAX_PROMPT_CHARS: usize = 12_000;
const TRUNCATED_NOTE: &str = "[texto truncado]"; // i18n-allow: prompt
/// Max characters of HTML (or stored Markdown) sent to the extraction model.
pub const MAX_AI_SOURCE_CHARS: usize = 60_000;
/// ReaderLM-v2's documented template; generic chat models understand it too.
const EXTRACT_PROMPT: &str = // i18n-allow: prompt
    "Extract the main content from the given HTML and convert it to Markdown format.";
const FENCE: &str = "```";
/// Fence info strings that wrap Markdown; any other language is a real code block.
const MARKDOWN_FENCE_INFO: [&str; 3] = ["", "markdown", "md"];
pub(crate) const FAKE_SUMMARIZE_ERROR: &str = "fake summarize error"; // i18n-allow: test double
pub(crate) const FAKE_EMBED_ERROR: &str = "fake embed error"; // i18n-allow: test double
const FAKE_DEFAULT_DIM: usize = 8;
const FAKE_CATEGORY: &str = "pruebas";
const FAKE_TAG: &str = "fake";
const NONE_LISTED: &str = "(ninguna)";

/// Prompt rule that requires a category; `apply_summary` stores NULL if it comes back empty.
const CATEGORY_RULE: &str = // i18n-allow: prompt
    "- \"category\": una categoría corta y obligatoria (una o dos palabras); nunca vacía.";

const PROMPT_TEMPLATE: &str = // i18n-allow: prompt
    "\
Eres un asistente que resume páginas web para una biblioteca de bookmarks.
Responde únicamente con un objeto JSON con las claves \"title\", \"summary\", \"category\" y \"tags\".
- \"summary\": 2 o 3 frases, en el idioma de la página.
{category_rule}
- \"tags\": entre 2 y 6 etiquetas cortas.
- Reutiliza las categorías y etiquetas existentes cuando encajen.

Categorías existentes: {categories}
Etiquetas existentes: {tags}

URL: {url}
Título: {title}

Contenido:
{markdown}";

#[derive(Debug, Clone, Copy)]
pub struct SummarizeInput<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub markdown: &'a str,
    pub existing_categories: &'a [String],
    pub existing_tags: &'a [String],
}

#[async_trait::async_trait]
pub trait Model: Send + Sync {
    async fn summarize(&self, input: SummarizeInput<'_>) -> anyhow::Result<Summary>;
    async fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;
    /// Clean HTML (or the stored Markdown) → Markdown, using `extract_prompt`.
    async fn extract_markdown(&self, input: &str) -> anyhow::Result<String>;
    fn embedding_id(&self) -> String;
}

fn join_or_none(items: &[String]) -> String {
    if items.is_empty() {
        NONE_LISTED.to_string()
    } else {
        items.join(", ")
    }
}

/// Cuts to `max` characters (never mid-character) and appends the truncation note.
fn truncated(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((end, _)) => format!("{}\n{TRUNCATED_NOTE}", &text[..end]),
        None => text.to_string(),
    }
}

pub fn extract_prompt(input: &str) -> String {
    format!(
        "{EXTRACT_PROMPT}\n{FENCE}html\n{}\n{FENCE}",
        truncated(input, MAX_AI_SOURCE_CHARS)
    )
}

/// Strips a ``` fence (untagged, `markdown` or `md`) that wraps the whole output.
pub fn clean_extracted(text: &str) -> String {
    let text = text.trim();
    text.strip_prefix(FENCE)
        .and_then(|rest| rest.strip_suffix(FENCE))
        .and_then(|inner| inner.split_once('\n'))
        .filter(|(info, _)| MARKDOWN_FENCE_INFO.contains(&info.trim()))
        .map_or(text, |(_, body)| body)
        .trim()
        .to_string()
}

/// Fills placeholders in a single pass: values are never rescanned.
fn fill_template(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(pos) = rest.find('{') {
        out.push_str(&rest[..pos]);
        rest = &rest[pos..];
        match values.iter().find(|(key, _)| rest.starts_with(key)) {
            Some((key, value)) => {
                out.push_str(value);
                rest = &rest[key.len()..];
            }
            None => {
                out.push('{');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

pub fn summarize_prompt(input: &SummarizeInput<'_>) -> String {
    fill_template(
        PROMPT_TEMPLATE,
        &[
            ("{category_rule}", CATEGORY_RULE),
            ("{categories}", &join_or_none(input.existing_categories)),
            ("{tags}", &join_or_none(input.existing_tags)),
            ("{url}", input.url),
            ("{title}", input.title),
            ("{markdown}", &truncated(input.markdown, MAX_PROMPT_CHARS)),
        ],
    )
}

pub fn parse_summary(text: &str) -> anyhow::Result<Summary> {
    let (start, end) = text
        .find('{')
        .zip(text.rfind('}'))
        .filter(|(s, e)| s < e)
        .msg(Msg::NoJson)?;
    let mut summary: Summary = serde_json::from_str(&text[start..=end]).msg(Msg::InvalidJson)?;
    for field in [
        &mut summary.title,
        &mut summary.summary,
        &mut summary.category,
    ] {
        *field = field.trim().to_string();
    }
    Ok(summary)
}

/// Deterministic model for tests; no network.
#[derive(Debug, Clone)]
pub struct FakeModel {
    pub dim: usize,
    pub fail_summarize: bool,
    pub fail_embed: bool,
    /// Odd-indexed vectors get one extra dimension.
    pub mixed_dims: bool,
}

impl Default for FakeModel {
    fn default() -> Self {
        Self {
            dim: FAKE_DEFAULT_DIM,
            fail_summarize: false,
            fail_embed: false,
            mixed_dims: false,
        }
    }
}

impl FakeModel {
    fn embed_one(&self, text: &str) -> Vec<f32> {
        if self.dim == 0 {
            return Vec::new();
        }
        let mut vector = vec![0.0f32; self.dim];
        for word in text.to_lowercase().split_whitespace() {
            let mut hasher = DefaultHasher::new();
            word.hash(&mut hasher);
            vector[(hasher.finish() % self.dim as u64) as usize] += 1.0;
        }
        let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            vector.iter_mut().for_each(|v| *v /= norm);
        }
        vector
    }
}

#[async_trait::async_trait]
impl Model for FakeModel {
    async fn summarize(&self, input: SummarizeInput<'_>) -> anyhow::Result<Summary> {
        anyhow::ensure!(!self.fail_summarize, FAKE_SUMMARIZE_ERROR);
        Ok(Summary {
            title: format!("{} (IA)", input.title),
            summary: format!("Resumen de {}", input.url), // i18n-allow: test double
            category: FAKE_CATEGORY.to_string(),
            tags: vec![FAKE_TAG.to_string()],
        })
    }

    async fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        anyhow::ensure!(!self.fail_embed, FAKE_EMBED_ERROR);
        Ok(texts
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let mut vector = self.embed_one(t);
                if self.mixed_dims && i % 2 == 1 {
                    vector.push(0.0);
                }
                vector
            })
            .collect())
    }

    async fn extract_markdown(&self, input: &str) -> anyhow::Result<String> {
        Ok(input.to_string())
    }

    fn embedding_id(&self) -> String {
        format!("fake:{}", self.dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input<'a>(markdown: &'a str, cats: &'a [String], tags: &'a [String]) -> SummarizeInput<'a> {
        SummarizeInput {
            url: "https://e.com",
            title: "T",
            markdown,
            existing_categories: cats,
            existing_tags: tags,
        }
    }

    #[test]
    fn prompt_truncates_markdown() {
        let md = "€".repeat(20_000);
        let prompt = summarize_prompt(&input(&md, &[], &[]));
        assert_eq!(prompt.matches('€').count(), MAX_PROMPT_CHARS);
        assert!(prompt.contains(TRUNCATED_NOTE));
        let short = summarize_prompt(&input("corto", &[], &[]));
        assert!(!short.contains(TRUNCATED_NOTE));
    }

    #[test]
    fn prompt_requires_category() {
        let prompt = summarize_prompt(&input("md", &[], &[]));
        assert!(prompt.contains(CATEGORY_RULE));
        assert!(!prompt.contains("{category_rule}"));
    }

    #[test]
    fn prompt_lists_existing_taxonomy() {
        let cats = vec!["rust".to_string(), "cocina".to_string()];
        let tags = vec!["tauri".to_string(), "receta".to_string()];
        let prompt = summarize_prompt(&input("md", &cats, &tags));
        for word in cats.iter().chain(&tags) {
            assert!(prompt.contains(word.as_str()));
        }
    }

    #[test]
    fn parse_summary_accepts_fenced_json() {
        let text = "Claro:\n```json\n{\"title\":\" A \",\"summary\":\"B \",\"category\":\" c\",\"tags\":[\"x\"]}\n```\nlisto";
        let s = parse_summary(text).unwrap();
        assert_eq!(
            (s.title.as_str(), s.summary.as_str(), s.category.as_str()),
            ("A", "B", "c")
        );
        assert_eq!(s.tags, vec!["x"]);
    }

    #[test]
    fn parse_summary_rejects_bad_shapes() {
        assert!(parse_summary("{\"title\":\"a\",\"category\":\"c\",\"tags\":[]}").is_err());
        assert!(
            parse_summary("{\"title\":\"a\",\"summary\":\"s\",\"category\":\"c\",\"tags\":\"a\"}")
                .is_err()
        );
        let err = parse_summary("{\"title\":\"a\"}").unwrap_err();
        assert_eq!(err.to_string(), Msg::InvalidJson.to_string());
        assert_eq!(
            parse_summary("sin json").unwrap_err().to_string(),
            Msg::NoJson.to_string()
        );
    }

    #[tokio::test]
    async fn fake_embed_is_deterministic_and_normalized() {
        let model = FakeModel::default();
        let texts = vec!["Hola mundo hola".to_string(), String::new()];
        let a = model.embed(&texts).await.unwrap();
        assert_eq!(a, model.embed(&texts).await.unwrap());
        assert_eq!(a[0].len(), 8);
        let norm: f32 = a[0].iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
        assert!(a[1].iter().all(|v| *v == 0.0));
        assert_eq!(model.embedding_id(), "fake:8");
    }

    #[tokio::test]
    async fn fake_model_summarizes_and_fails_on_demand() {
        let s = FakeModel::default()
            .summarize(input("md", &[], &[]))
            .await
            .unwrap();
        assert_eq!(
            (s.title.as_str(), s.summary.as_str()),
            ("T (IA)", "Resumen de https://e.com")
        );
        let failing = FakeModel {
            fail_summarize: true,
            fail_embed: true,
            ..Default::default()
        };
        assert!(failing.summarize(input("md", &[], &[])).await.is_err());
        assert!(failing.embed(&[]).await.is_err());
    }

    #[test]
    fn prompt_does_not_rescan_values_for_placeholders() {
        let md = "€".repeat(20_000);
        let mut i = input(&md, &[], &[]);
        i.title = "{markdown}";
        let prompt = summarize_prompt(&i);
        assert_eq!(prompt.matches("{markdown}").count(), 1);
        assert_eq!(prompt.matches('€').count(), MAX_PROMPT_CHARS);
    }

    #[tokio::test]
    async fn fake_embed_mixed_dims_differ() {
        let model = FakeModel {
            mixed_dims: true,
            ..Default::default()
        };
        let vectors = model.embed(&["a".into(), "b".into()]).await.unwrap();
        assert_eq!(vectors[1].len(), vectors[0].len() + 1);
    }

    #[tokio::test]
    async fn fake_embed_with_zero_dim_returns_empty_vectors() {
        let model = FakeModel {
            dim: 0,
            ..Default::default()
        };
        assert_eq!(
            model.embed(&["a b".to_string()]).await.unwrap(),
            vec![Vec::<f32>::new()]
        );
    }

    #[test]
    fn extract_prompt_truncates_with_marker() {
        let html = "€".repeat(MAX_AI_SOURCE_CHARS + 10);
        let prompt = extract_prompt(&html);
        assert_eq!(prompt.matches('€').count(), MAX_AI_SOURCE_CHARS);
        assert!(prompt.contains(TRUNCATED_NOTE));
        let exact = extract_prompt(&"€".repeat(MAX_AI_SOURCE_CHARS));
        assert!(!exact.contains(TRUNCATED_NOTE));
    }

    #[test]
    fn extract_prompt_wraps_html_block() {
        assert_eq!(
            extract_prompt("<p>hola</p>"),
            format!("{EXTRACT_PROMPT}\n```html\n<p>hola</p>\n```")
        );
    }

    #[test]
    fn clean_extracted_strips_one_fence() {
        assert_eq!(
            clean_extracted("```markdown\n# T\n\ntexto\n```\n"),
            "# T\n\ntexto"
        );
        assert_eq!(clean_extracted("  ```\n# T\n```"), "# T");
        // Only one fence, and only if it wraps the whole output.
        assert_eq!(clean_extracted("```\n```js\nx\n```\n```"), "```js\nx\n```");
        let inner = "# T\n\n```js\nx\n```";
        assert_eq!(clean_extracted(inner), inner);
        assert_eq!(clean_extracted("  sin cercado \n"), "sin cercado");
        // A code block in another language is content, not a wrapper.
        assert_eq!(clean_extracted("```js\nx\n```"), "```js\nx\n```");
    }

    #[tokio::test]
    async fn fake_extract_markdown_returns_input() {
        let out = FakeModel::default()
            .extract_markdown("<p>x</p>")
            .await
            .unwrap();
        assert_eq!(out, "<p>x</p>");
    }
}
