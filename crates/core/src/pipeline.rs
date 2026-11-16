//! Processes a pending bookmark: download, summary and embeddings, plus its favicon separately.
//! Also manual re-extraction, with Defuddle or the extraction model.
//! Never holds the `Mutex<Store>` during network or model calls.

use crate::ai::{Model, SummarizeInput, Summary, clean_extracted};
use crate::blocked::{BlockSignals, is_blocked, is_blocked_error};
use crate::chunk::chunk;
use crate::download::{Image, is_transient};
use crate::extract::{Fetcher, Page, PageMeta, is_thin};
use crate::msg::{BookmarkError, Msg, MsgExt};
use crate::store::{Bookmark, ErrorKind, Status, Store, lock};
use anyhow::Result;
use std::future::Future;
use std::sync::Mutex;
use std::time::Duration;

/// `extractor` prefix when a model extracted the content.
pub const AI_EXTRACTOR_PREFIX: &str = "ai:";
/// Timeout for the extraction model call: a remote one with `MAX_AI_SOURCE_CHARS` of input is
/// slow. Loading the page has its own limits and does not count here.
pub const EXTRACT_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Attempts per favicon, the first included, as long as failures are transient
/// (`download::is_transient`). A permanent failure is not retried.
pub const FAVICON_MAX_ATTEMPTS: u32 = 4;
/// Delay before the first retry; it doubles on each one, up to `FAVICON_RETRY_MAX`.
pub const FAVICON_RETRY_BASE: Duration = Duration::from_secs(60);
pub const FAVICON_RETRY_MAX: Duration = Duration::from_secs(30 * 60);

/// Favicon download: `download::download_favicon` outside tests.
pub type DownloadFavicon<'a> = &'a dyn Fn(&str) -> Result<Image>;

/// Guard dropped when extraction ends; the caller decides what happens on creation and drop.
pub type OnExtract<'a> = &'a (dyn Fn(i64) -> Box<dyn std::any::Any + Send> + Sync);

/// Processes the next pending bookmark and returns its id, also when it ends up `failed`.
/// Without a model it only extracts the content and the bookmark stays `pending`.
/// Network and model failures are stored on the bookmark; an `Err` is a store failure.
pub async fn process_next(
    store: &Mutex<Store>,
    fetcher: &dyn Fetcher,
    model: Option<&dyn Model>,
    on_extract: OnExtract<'_>,
) -> Result<Option<i64>> {
    let next = {
        let guard = lock(store)?;
        match model {
            Some(_) => guard.next_pending()?,
            None => guard.next_without_content()?,
        }
    };
    let Some(bookmark) = next else {
        return Ok(None);
    };
    let id = bookmark.id;
    let content = match bookmark.status {
        Status::Pending => {
            let Some(content) = extract_step(store, fetcher, on_extract, &bookmark).await? else {
                return Ok(Some(id));
            };
            let Some(model) = model else {
                return Ok(Some(id));
            };
            match summarize_step(store, model, &bookmark, content).await? {
                Some(content) => content,
                None => return Ok(Some(id)),
            }
        }
        _ => bookmark.content.unwrap_or_default(),
    };
    if let Some(model) = model {
        embed_step(store, model, id, &content).await?;
    }
    Ok(Some(id))
}

/// The bookmark to re-extract. Not a blocked one: those are rescued from the extension.
fn reextractable(store: &Mutex<Store>, id: i64) -> Result<Bookmark> {
    let bookmark = lock(store)?.get(id)?.msg(Msg::BookmarkNotFound)?;
    if bookmark.error_kind == Some(ErrorKind::Blocked) {
        return Err(Msg::BlockedBookmark.err());
    }
    Ok(bookmark)
}

/// Re-extracts the URL with `fetcher` and replaces the page (with its metadata) only if it is not
/// thin; on failure or thin content, the content is unchanged.
pub async fn reextract(store: &Mutex<Store>, id: i64, fetcher: &dyn Fetcher) -> Result<()> {
    let bookmark = reextractable(store, id)?;
    let page = fetcher.fetch(&bookmark.url).await?;
    if is_thin(&page.markdown) {
        return Err(Msg::ThinResult.err());
    }
    lock(store)?.replace_page(id, &page)
}

/// Extracts with `model` the input given by `load` (the final URL and the page's clean HTML) or,
/// if that fails, the stored content. Replaces only the content, unless it is thin or an anti-bot
/// check page: stored metadata is kept, except `word_count`, which is recounted, and `extractor`,
/// which becomes `ai:<model_name>`.
pub async fn extract_with_ai<F>(
    store: &Mutex<Store>,
    id: i64,
    load: impl FnOnce(String) -> F,
    model: &dyn Model,
    model_name: &str,
) -> Result<()>
where
    F: Future<Output = Result<(String, String)>>,
{
    let bookmark = reextractable(store, id)?;
    let (final_url, input) = match load(bookmark.url).await {
        Ok((final_url, html)) => (Some(final_url), html),
        Err(err) => (
            None,
            bookmark
                .content
                .filter(|c| !c.trim().is_empty())
                .ok_or(err)?,
        ),
    };
    // Before the model, so we do not wait on it for a page that will not be stored.
    ensure_not_blocked(final_url.as_deref(), &input)?;
    let extracted = tokio::time::timeout(EXTRACT_TIMEOUT, model.extract_markdown(&input))
        .await
        .msg(Msg::ExtractTimeout)??;
    let markdown = clean_extracted(&extracted);
    if is_thin(&markdown) {
        return Err(Msg::ThinResult.err());
    }
    ensure_not_blocked(final_url.as_deref(), &markdown)?;
    let word_count = u32::try_from(markdown.split_whitespace().count()).ok();
    let page = Page {
        title: None,
        markdown,
        meta: PageMeta {
            word_count,
            extractor: Some(format!("{AI_EXTRACTOR_PREFIX}{model_name}")),
            ..bookmark.meta
        },
    };
    lock(store)?.replace_page(id, &page)
}

/// `Blocked` if `text` (or the URL the load ended at) is an anti-bot check page.
fn ensure_not_blocked(final_url: Option<&str>, text: &str) -> Result<()> {
    let signals = BlockSignals {
        status: None,
        headers: &[],
        final_url,
        markdown: text,
        word_count: None,
    };
    if is_blocked(&signals) {
        return Err(Msg::Blocked.err());
    }
    Ok(())
}

/// Whether the stored content is still `content`: a manual re-extraction may have replaced it
/// while the model was working, in which case the bookmark is already `pending` with the new one.
fn content_unchanged(store: &Store, id: i64, content: &str) -> Result<bool> {
    Ok(store.get(id)?.and_then(|b| b.content).as_deref() == Some(content))
}

/// Downloads if there is no content and stores it with its metadata. `None` if the bookmark ended
/// up `failed`. The store is not locked during the download.
async fn extract_step(
    store: &Mutex<Store>,
    fetcher: &dyn Fetcher,
    on_extract: OnExtract<'_>,
    bookmark: &Bookmark,
) -> Result<Option<String>> {
    if let Some(content) = bookmark.content.as_deref().filter(|c| !c.trim().is_empty()) {
        return Ok(Some(content.to_string()));
    }
    let id = bookmark.id;
    let guard = on_extract(id);
    let fetched = fetcher.fetch(&bookmark.url).await;
    drop(guard);
    match fetched {
        Ok(page) if page.markdown.trim().is_empty() => {
            fail_locked(store, id, &Msg::NoContent.err()).map(|()| None)
        }
        Ok(page) => {
            lock(store)?.set_page(id, &page)?;
            Ok(Some(page.markdown))
        }
        Err(err) if is_blocked_error(&err) => {
            lock(store)?.set_blocked(id)?;
            Ok(None)
        }
        Err(err) => fail_locked(store, id, &err).map(|()| None),
    }
}

/// Summarizes the content. `None` if the bookmark ended up `failed` or was re-extracted meanwhile.
async fn summarize_step(
    store: &Mutex<Store>,
    model: &dyn Model,
    bookmark: &Bookmark,
    content: String,
) -> Result<Option<String>> {
    let id = bookmark.id;
    let (categories, tags) = taxonomy(&*lock(store)?)?;
    let input = SummarizeInput {
        url: &bookmark.url,
        title: &bookmark.title,
        markdown: &content,
        existing_categories: &categories,
        existing_tags: &tags,
    };
    let summarized = model.summarize(input).await;
    let guard = lock(store)?;
    // Re-extracted meanwhile: the summary is of the old content and is not stored.
    if !content_unchanged(&guard, id, &content)? {
        return Ok(None);
    }
    match summarized.and_then(|summary| save_summary(&guard, id, &summary)) {
        Ok(()) => Ok(Some(content)),
        Err(err) => fail(&guard, id, &err).map(|()| None),
    }
}

/// Chunks, embeds and marks the bookmark `ready`.
async fn embed_step(store: &Mutex<Store>, model: &dyn Model, id: i64, content: &str) -> Result<()> {
    let chunks = chunk(content);
    if chunks.is_empty() {
        return fail_locked(store, id, &Msg::NoContent.err());
    }
    let embedded = model.embed(&chunks).await;
    let guard = lock(store)?;
    // Re-extracted meanwhile: the vectors are of the old content and are not stored.
    if !content_unchanged(&guard, id, content)? {
        return Ok(());
    }
    // Any failure is stored on the bookmark; otherwise `next_pending` would retry it forever.
    embedded
        .and_then(|vectors| save_vectors(&guard, model, id, &chunks, &vectors))
        .or_else(|err| fail(&guard, id, &err))
}

fn taxonomy(store: &Store) -> Result<(Vec<String>, Vec<String>)> {
    Ok((store.categories()?, store.tags()?))
}

fn save_summary(store: &Store, id: i64, summary: &Summary) -> Result<()> {
    store.apply_summary(id, summary)?;
    store.set_status(id, Status::PendingEmbed, None)
}

fn save_vectors(
    store: &Store,
    model: &dyn Model,
    id: i64,
    chunks: &[String],
    vectors: &[Vec<f32>],
) -> Result<()> {
    let dim = uniform_dim(chunks.len(), vectors)?;
    store.ensure_vectors(&model.embedding_id(), dim)?;
    store.replace_chunks(id, chunks, vectors)?;
    store.set_status(id, Status::Ready, None)
}

/// Common dimension of the vectors. Checked before `ensure_vectors`: a dimension different from
/// the index's would wipe the whole index.
fn uniform_dim(chunk_count: usize, vectors: &[Vec<f32>]) -> Result<usize> {
    if vectors.len() != chunk_count {
        return Err(Msg::EmbedChunkCount.err());
    }
    let dim = vectors.first().map_or(0, Vec::len);
    if dim == 0 || vectors.iter().any(|v| v.len() != dim) {
        return Err(Msg::EmbedDim.err());
    }
    Ok(dim)
}

/// Downloads the favicon of the next bookmark that needs one and returns its id; `None` if none
/// is left. Blocking, and never holds the store during network calls. A transient failure is
/// retried later, up to `FAVICON_MAX_ATTEMPTS`; a permanent one, or the last, leaves the bookmark
/// without a favicon and the UI shows none.
pub fn favicon_step(store: &Mutex<Store>, download: DownloadFavicon<'_>) -> Result<Option<i64>> {
    let Some(bookmark) = lock(store)?.next_without_favicon()? else {
        return Ok(None);
    };
    let fetched = bookmark.meta.favicon.as_deref().map(download);
    let favicon = match fetched {
        Some(Ok(image)) => Some(image),
        Some(Err(err)) if is_transient(&err) => {
            let store = lock(store)?;
            let failures = store.favicon_failures(bookmark.id)? + 1;
            if failures < FAVICON_MAX_ATTEMPTS {
                store.retry_favicon(&bookmark, favicon_backoff(failures))?;
                return Ok(Some(bookmark.id));
            }
            None
        }
        _ => None,
    };
    lock(store)?.set_favicon(&bookmark, &favicon)?;
    Ok(Some(bookmark.id))
}

/// Delay after transient failure number `failures` (from 1): `FAVICON_RETRY_BASE` doubled each
/// time, at most `FAVICON_RETRY_MAX`.
pub fn favicon_backoff(failures: u32) -> Duration {
    let doublings = failures.saturating_sub(1).min(u32::BITS - 1);
    FAVICON_RETRY_BASE
        .saturating_mul(1 << doublings)
        .min(FAVICON_RETRY_MAX)
}

fn fail_locked(store: &Mutex<Store>, id: i64, err: &anyhow::Error) -> Result<()> {
    fail(&*lock(store)?, id, err)
}

/// Marks the bookmark `failed`, storing the full error chain as codes.
fn fail(store: &Store, id: i64, err: &anyhow::Error) -> Result<()> {
    store.set_failed(id, &BookmarkError::from_anyhow(err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{FAKE_EMBED_ERROR, FAKE_SUMMARIZE_ERROR, FakeModel};
    use crate::extract::{ERR_FAKE_UNKNOWN_URL, FakeFetcher};
    use crate::store::NewBookmark;
    use async_trait::async_trait;
    use std::sync::Arc;

    const URL: &str = "https://example.com/post";
    const MARKDOWN: &str = "Primer párrafo sobre rust.\n\nSegundo párrafo sobre tauri.";
    const USER_TITLE: &str = "Mi título";
    const USER_TAG: &str = "mio";
    const OTHER_URL: &str = "https://example.com/otro";
    /// Two paragraphs that do not fit together in one chunk.
    const PARAGRAPH_CHARS: usize = 1000;

    fn store_with(
        title: Option<&str>,
        tags: &[&str],
        content: Option<&str>,
    ) -> (Mutex<Store>, i64) {
        let s = Store::open_in_memory().unwrap();
        let id = add(&s, URL, title, tags, content);
        (Mutex::new(s), id)
    }

    fn add(s: &Store, url: &str, title: Option<&str>, tags: &[&str], content: Option<&str>) -> i64 {
        s.add(NewBookmark {
            url: url.into(),
            title: title.map(Into::into),
            notes: String::new(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            content: content.map(Into::into),
            ..Default::default()
        })
        .unwrap()
        .id
    }

    fn error_of(store: &Mutex<Store>, id: i64) -> String {
        let b = get(store, id);
        assert_eq!(b.status, Status::Failed);
        b.error.unwrap().to_string()
    }

    fn fetcher(markdown: &str) -> FakeFetcher {
        let page = Page {
            title: Some(URL.into()),
            markdown: markdown.into(),
            ..Page::default()
        };
        FakeFetcher {
            pages: [(URL.to_string(), page)].into(),
        }
    }

    fn get(store: &Mutex<Store>, id: i64) -> crate::store::Bookmark {
        store.lock().unwrap().get(id).unwrap().unwrap()
    }

    fn chunk_count(store: &Mutex<Store>, id: i64) -> i64 {
        store
            .lock()
            .unwrap()
            .conn
            .query_row(
                "SELECT COUNT(*) FROM chunks WHERE bookmark_id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap()
    }

    fn no_guard(_: i64) -> Box<dyn std::any::Any + Send> {
        Box::new(())
    }

    /// Tauri requires `Send` futures: this fails to compile if a guard crosses an `.await`.
    fn assert_send<T: Send>(_: &T) {}

    #[tokio::test]
    async fn nothing_pending_returns_none() {
        let store = Mutex::new(Store::open_in_memory().unwrap());
        let next = process_next(
            &store,
            &FakeFetcher::default(),
            Some(&FakeModel::default()),
            &no_guard,
        )
        .await;
        assert_eq!(next.unwrap(), None);
    }

    #[tokio::test]
    async fn happy_path_ends_ready_with_chunks_and_summary() {
        let (store, id) = store_with(None, &[], None);
        let (fetcher, model) = (fetcher(MARKDOWN), FakeModel::default());
        let future = process_next(&store, &fetcher, Some(&model), &no_guard);
        assert_send(&future);
        assert_eq!(future.await.unwrap(), Some(id));
        let b = get(&store, id);
        assert_eq!(b.status, Status::Ready);
        assert_eq!(b.error, None);
        assert_eq!(b.content.as_deref(), Some(MARKDOWN));
        assert!(b.summary.is_some() && b.category.is_some());
        assert_eq!(b.embedding_id, Some(model.embedding_id()));
        assert!(chunk_count(&store, id) > 0);
    }

    #[tokio::test]
    async fn empty_page_fails_with_sin_contenido() {
        let (store, id) = store_with(None, &[], None);
        let next = process_next(
            &store,
            &fetcher(" \n\n "),
            Some(&FakeModel::default()),
            &no_guard,
        )
        .await;
        assert_eq!(next.unwrap(), Some(id));
        let b = get(&store, id);
        assert_eq!(b.status, Status::Failed);
        assert_eq!(
            b.error.as_ref().map(ToString::to_string).as_deref(),
            Some(Msg::NoContent.to_string().as_str())
        );
    }

    #[tokio::test]
    async fn fetch_failure_marks_failed() {
        let (store, id) = store_with(None, &[], None);
        let next = process_next(
            &store,
            &FakeFetcher::default(),
            Some(&FakeModel::default()),
            &no_guard,
        )
        .await;
        assert_eq!(next.unwrap(), Some(id));
        let b = get(&store, id);
        assert_eq!(b.status, Status::Failed);
        assert!(
            b.error
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                .unwrap()
                .contains(ERR_FAKE_UNKNOWN_URL)
        );
    }

    #[tokio::test]
    async fn summarize_failure_keeps_content() {
        let (store, id) = store_with(None, &[], None);
        let model = FakeModel {
            fail_summarize: true,
            ..FakeModel::default()
        };
        let next = process_next(&store, &fetcher(MARKDOWN), Some(&model), &no_guard).await;
        assert_eq!(next.unwrap(), Some(id));
        let b = get(&store, id);
        assert_eq!(b.status, Status::Failed);
        assert!(
            b.error
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                .unwrap()
                .contains(FAKE_SUMMARIZE_ERROR)
        );
        assert_eq!(b.content.as_deref(), Some(MARKDOWN));
        assert_eq!(b.summary, None);
    }

    #[tokio::test]
    async fn embed_failure_keeps_summary() {
        let (store, id) = store_with(None, &[], None);
        let model = FakeModel {
            fail_embed: true,
            ..FakeModel::default()
        };
        let next = process_next(&store, &fetcher(MARKDOWN), Some(&model), &no_guard).await;
        assert_eq!(next.unwrap(), Some(id));
        let b = get(&store, id);
        assert_eq!(b.status, Status::Failed);
        assert!(
            b.error
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                .unwrap()
                .contains(FAKE_EMBED_ERROR)
        );
        assert!(b.summary.is_some());
        assert_eq!(chunk_count(&store, id), 0);
    }

    #[tokio::test]
    async fn pending_embed_skips_summarize() {
        let (store, id) = store_with(None, &[], Some(MARKDOWN));
        let summary = Summary {
            title: "Puesto a mano".into(),
            summary: "Resumen a mano".into(),
            category: "manual".into(),
            tags: vec![],
        };
        {
            let s = store.lock().unwrap();
            s.apply_summary(id, &summary).unwrap();
            s.set_status(id, Status::PendingEmbed, None).unwrap();
        }
        let model = FakeModel {
            fail_summarize: true,
            ..FakeModel::default()
        };
        let next = process_next(&store, &FakeFetcher::default(), Some(&model), &no_guard).await;
        assert_eq!(next.unwrap(), Some(id));
        let b = get(&store, id);
        assert_eq!(b.status, Status::Ready);
        assert_eq!(b.summary.as_deref(), Some("Resumen a mano"));
        assert!(chunk_count(&store, id) > 0);
    }

    #[tokio::test]
    async fn user_title_and_tags_survive_processing() {
        let (store, id) = store_with(Some(USER_TITLE), &[USER_TAG], None);
        let next = process_next(
            &store,
            &fetcher(MARKDOWN),
            Some(&FakeModel::default()),
            &no_guard,
        )
        .await;
        assert_eq!(next.unwrap(), Some(id));
        let b = get(&store, id);
        assert_eq!(b.status, Status::Ready);
        assert_eq!(b.title, USER_TITLE);
        assert_eq!(b.tags, vec![USER_TAG]);
        assert!(b.summary.is_some());
    }

    #[tokio::test]
    async fn zero_dim_vectors_fail() {
        let (store, id) = store_with(None, &[], Some(MARKDOWN));
        let model = FakeModel {
            dim: 0,
            ..FakeModel::default()
        };
        let next = process_next(&store, &FakeFetcher::default(), Some(&model), &no_guard).await;
        assert_eq!(next.unwrap(), Some(id));
        assert_eq!(error_of(&store, id), Msg::EmbedDim.to_string());
    }

    #[tokio::test]
    async fn mixed_dims_fail_without_stalling_the_queue() {
        let two_chunks = ["a".repeat(PARAGRAPH_CHARS), "b".repeat(PARAGRAPH_CHARS)].join("\n\n");
        let s = Store::open_in_memory().unwrap();
        // The queue takes the most recently queued first.
        let second = add(&s, OTHER_URL, None, &[], Some(MARKDOWN));
        let first = add(&s, URL, None, &[], Some(&two_chunks));
        let store = Mutex::new(s);
        let (fetcher, model) = (
            FakeFetcher::default(),
            FakeModel {
                mixed_dims: true,
                ..FakeModel::default()
            },
        );
        let next = process_next(&store, &fetcher, Some(&model), &no_guard).await;
        assert_eq!(next.unwrap(), Some(first));
        assert_eq!(error_of(&store, first), Msg::EmbedDim.to_string());
        let next = process_next(&store, &fetcher, Some(&model), &no_guard).await;
        assert_eq!(next.unwrap(), Some(second));
    }

    #[tokio::test]
    async fn without_model_extracts_and_stays_pending() {
        let (store, id) = store_with(None, &[], None);
        let next = process_next(&store, &fetcher(MARKDOWN), None, &no_guard).await;
        assert_eq!(next.unwrap(), Some(id));
        let b = get(&store, id);
        assert_eq!(b.status, Status::Pending);
        assert_eq!(b.content.as_deref(), Some(MARKDOWN));
        assert_eq!(b.summary, None);
    }

    #[tokio::test]
    async fn without_model_does_not_repeat_the_same_bookmark() {
        let (store, id) = store_with(None, &[], None);
        let fetcher = fetcher(MARKDOWN);
        let first = process_next(&store, &fetcher, None, &no_guard).await;
        assert_eq!(first.unwrap(), Some(id));
        let second = process_next(&store, &fetcher, None, &no_guard).await;
        assert_eq!(second.unwrap(), None);
    }

    #[tokio::test]
    async fn without_model_failure_is_recorded() {
        let (store, id) = store_with(None, &[], None);
        let next = process_next(&store, &FakeFetcher::default(), None, &no_guard).await;
        assert_eq!(next.unwrap(), Some(id));
        assert!(error_of(&store, id).contains(ERR_FAKE_UNKNOWN_URL));
    }

    /// Records the order of events: the download, and when the guard is created and dropped.
    struct LoggedFetcher(Arc<Mutex<Vec<&'static str>>>);

    #[async_trait]
    impl Fetcher for LoggedFetcher {
        async fn fetch(&self, _: &str) -> Result<Page> {
            self.0.lock().unwrap().push("fetch");
            Ok(Page {
                markdown: MARKDOWN.into(),
                ..Page::default()
            })
        }
    }

    struct DropLog(Arc<Mutex<Vec<&'static str>>>);

    impl Drop for DropLog {
        fn drop(&mut self) {
            self.0.lock().unwrap().push("drop");
        }
    }

    #[tokio::test]
    async fn on_extract_runs_once_before_fetch_and_its_guard_drops_after() {
        let (store, id) = store_with(None, &[], None);
        let log = Arc::new(Mutex::new(Vec::new()));
        let on_extract = |got: i64| -> Box<dyn std::any::Any + Send> {
            assert_eq!(got, id);
            log.lock().unwrap().push("start");
            Box::new(DropLog(log.clone()))
        };
        let next = process_next(&store, &LoggedFetcher(log.clone()), None, &on_extract).await;
        assert_eq!(next.unwrap(), Some(id));
        assert_eq!(*log.lock().unwrap(), ["start", "fetch", "drop"]);
    }

    #[tokio::test]
    async fn on_extract_skipped_when_content_exists() {
        let (store, _) = store_with(None, &[], Some(MARKDOWN));
        let called = Mutex::new(false);
        let on_extract = |_| -> Box<dyn std::any::Any + Send> {
            *called.lock().unwrap() = true;
            Box::new(())
        };
        let model = FakeModel::default();
        process_next(&store, &FakeFetcher::default(), Some(&model), &on_extract)
            .await
            .unwrap();
        assert!(!*called.lock().unwrap());
    }

    struct BlockedFetcher;

    #[async_trait]
    impl Fetcher for BlockedFetcher {
        async fn fetch(&self, _: &str) -> Result<Page> {
            Err(Msg::Blocked.err())
        }
    }

    #[tokio::test]
    async fn blocked_fetch_marks_blocked() {
        let (store, id) = store_with(None, &[], None);
        let next = process_next(&store, &BlockedFetcher, None, &no_guard).await;
        assert_eq!(next.unwrap(), Some(id));
        let b = get(&store, id);
        assert_eq!(b.status, Status::Failed);
        assert_eq!(
            b.error.as_ref().map(ToString::to_string).as_deref(),
            Some(Msg::Blocked.to_string().as_str())
        );
    }

    #[tokio::test]
    async fn page_title_replaces_url_title() {
        let (store, id) = store_with(None, &[], None);
        let page = Page {
            title: Some("Título real".into()),
            markdown: MARKDOWN.into(),
            ..Page::default()
        };
        let fetcher = FakeFetcher {
            pages: [(URL.to_string(), page)].into(),
        };
        process_next(&store, &fetcher, None, &no_guard)
            .await
            .unwrap();
        assert_eq!(get(&store, id).title, "Título real");
    }

    fn with_favicon_url(store: &Mutex<Store>) -> i64 {
        let s = store.lock().unwrap();
        let id = add(&s, URL, None, &[], None);
        let page = Page {
            markdown: MARKDOWN.into(),
            meta: crate::extract::PageMeta {
                image: Some(format!("{URL}/og.png")),
                favicon: Some(format!("{URL}/favicon.ico")),
                ..Default::default()
            },
            ..Page::default()
        };
        s.set_page(id, &page).unwrap();
        id
    }

    fn png() -> Image {
        Image {
            mime: "image/png".into(),
            bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
        }
    }

    #[test]
    fn favicon_step_stores_only_the_favicon() {
        let store = Mutex::new(Store::open_in_memory().unwrap());
        let id = with_favicon_url(&store);
        let asked = Mutex::new(vec![]);
        let download = |url: &str| {
            asked.lock().unwrap().push(url.to_string());
            Ok(png())
        };
        assert_eq!(favicon_step(&store, &download).unwrap(), Some(id));
        assert_eq!(
            *asked.lock().unwrap(),
            [format!("{URL}/favicon.ico")],
            "nunca la og:image"
        );
        assert_eq!(
            store.lock().unwrap().stored_favicon(id).unwrap(),
            Some(png())
        );
        assert_eq!(favicon_step(&store, &download).unwrap(), None);
    }

    /// Moves the retry time into the past, as if time had passed.
    fn retry_now(store: &Mutex<Store>) {
        store
            .lock()
            .unwrap()
            .conn
            .execute("UPDATE bookmarks SET favicon_retry_at = '2000-01-01T00:00:00Z' WHERE favicon_retry_at IS NOT NULL", [])
            .unwrap();
    }

    fn failures(store: &Mutex<Store>, id: i64) -> u32 {
        store.lock().unwrap().favicon_failures(id).unwrap()
    }

    #[test]
    fn a_permanent_failure_is_tried_once_until_the_page_is_extracted_again() {
        let permanent: [fn() -> anyhow::Error; 4] = [
            || ureq::Error::StatusCode(404).into(),
            || Msg::NotImage.err(),
            || Msg::FaviconTooLarge.err(),
            || Msg::Destination.err(),
        ];
        for error in permanent {
            let store = Mutex::new(Store::open_in_memory().unwrap());
            let id = with_favicon_url(&store);
            let calls = Mutex::new(0);
            let failing = |_: &str| {
                *calls.lock().unwrap() += 1;
                Err(error())
            };
            assert_eq!(favicon_step(&store, &failing).unwrap(), Some(id));
            retry_now(&store);
            assert_eq!(
                favicon_step(&store, &failing).unwrap(),
                None,
                "{:#}",
                error()
            );
            assert_eq!(*calls.lock().unwrap(), 1);
            let b = get(&store, id);
            assert!(b.favicon_at.is_some());
            assert_eq!(failures(&store, id), 0);
            assert_eq!(store.lock().unwrap().stored_favicon(id).unwrap(), None);

            // Re-extracting the page makes it pending again.
            let page = Page {
                markdown: MARKDOWN.into(),
                meta: b.meta,
                ..Page::default()
            };
            store.lock().unwrap().set_page(id, &page).unwrap();
            assert_eq!(favicon_step(&store, &|_| Ok(png())).unwrap(), Some(id));
            assert_eq!(
                store.lock().unwrap().stored_favicon(id).unwrap(),
                Some(png())
            );
        }
    }

    #[test]
    fn a_transient_failure_waits_and_is_retried() {
        let store = Mutex::new(Store::open_in_memory().unwrap());
        let id = with_favicon_url(&store);
        let down = |_: &str| Err(ureq::Error::StatusCode(503).into());
        assert_eq!(favicon_step(&store, &down).unwrap(), Some(id));
        let b = get(&store, id);
        assert!(b.favicon_at.is_none(), "sigue pendiente");
        assert_eq!(failures(&store, id), 1);
        let ok = |_: &str| Ok(png());
        assert_eq!(favicon_step(&store, &ok).unwrap(), None, "aún en espera");

        retry_now(&store);
        assert_eq!(favicon_step(&store, &ok).unwrap(), Some(id));
        assert_eq!(
            store.lock().unwrap().stored_favicon(id).unwrap(),
            Some(png())
        );
        assert!(get(&store, id).favicon_at.is_some());

        // A new extraction restarts the count.
        let page = Page {
            markdown: MARKDOWN.into(),
            meta: b.meta,
            ..Page::default()
        };
        assert_eq!(favicon_step(&store, &down).unwrap(), None);
        store.lock().unwrap().set_page(id, &page).unwrap();
        assert_eq!(favicon_step(&store, &down).unwrap(), Some(id));
        assert_eq!(failures(&store, id), 1);
        store.lock().unwrap().set_page(id, &page).unwrap();
        assert_eq!(failures(&store, id), 0);
    }

    #[test]
    fn transient_failures_give_up_after_the_last_attempt() {
        let store = Mutex::new(Store::open_in_memory().unwrap());
        let id = with_favicon_url(&store);
        let calls = Mutex::new(0);
        let offline = |_: &str| {
            *calls.lock().unwrap() += 1;
            Err(ureq::Error::ConnectionFailed.into())
        };
        for attempt in 1..=FAVICON_MAX_ATTEMPTS {
            assert_eq!(
                favicon_step(&store, &offline).unwrap(),
                Some(id),
                "{attempt}"
            );
            retry_now(&store);
        }
        assert_eq!(favicon_step(&store, &offline).unwrap(), None, "se rinde");
        assert_eq!(*calls.lock().unwrap(), FAVICON_MAX_ATTEMPTS);
        assert_eq!(failures(&store, id), FAVICON_MAX_ATTEMPTS - 1);
        assert!(get(&store, id).favicon_at.is_some());
        assert_eq!(store.lock().unwrap().stored_favicon(id).unwrap(), None);
    }

    #[test]
    fn the_backoff_doubles_and_is_bounded() {
        assert_eq!(favicon_backoff(1), FAVICON_RETRY_BASE);
        assert_eq!(favicon_backoff(2), FAVICON_RETRY_BASE * 2);
        assert_eq!(favicon_backoff(3), FAVICON_RETRY_BASE * 4);
        for failures in [10, 64, u32::MAX] {
            assert_eq!(favicon_backoff(failures), FAVICON_RETRY_MAX);
        }
    }

    #[test]
    fn favicon_step_skips_bookmarks_without_a_favicon_url() {
        let (store, _) = store_with(None, &[], Some(MARKDOWN));
        let download = |_: &str| -> Result<Image> { panic!("no hay nada que descargar") };
        assert_eq!(favicon_step(&store, &download).unwrap(), None);
    }

    /// Non-thin text: `MIN_CONTENT_CHARS` letters in a row.
    fn long_text(letter: char) -> String {
        std::iter::repeat_n(letter, crate::extract::MIN_CONTENT_CHARS).collect()
    }

    #[tokio::test]
    async fn reextract_replaces_only_a_good_page() {
        let old = long_text('v');
        let (store, id) = store_with(Some(USER_TITLE), &[USER_TAG], Some(&old));
        let err = reextract(&store, id, &fetcher("escaso")).await.unwrap_err();
        assert_eq!(err.to_string(), Msg::ThinResult.to_string());
        assert_eq!(get(&store, id).content.as_deref(), Some(old.as_str()));

        let err = reextract(&store, id, &FakeFetcher::default())
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), ERR_FAKE_UNKNOWN_URL);
        assert_eq!(get(&store, id).content.as_deref(), Some(old.as_str()));

        let new = long_text('n');
        reextract(&store, id, &fetcher(&new)).await.unwrap();
        let b = get(&store, id);
        assert_eq!(b.content.as_deref(), Some(new.as_str()));
        assert_eq!(b.status, Status::Pending);
        assert_eq!(b.title, USER_TITLE);
        assert_eq!(b.tags, vec![USER_TAG.to_string()]);
    }

    #[tokio::test]
    async fn reextract_refuses_a_blocked_bookmark() {
        let (store, id) = store_with(None, &[], None);
        store.lock().unwrap().set_blocked(id).unwrap();
        let err = reextract(&store, id, &fetcher(&long_text('n')))
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), Msg::BlockedBookmark.to_string());
        let err = extract_with_ai(
            &store,
            id,
            |_| async { Ok((URL.to_string(), long_text('n'))) },
            &FakeModel::default(),
            "m",
        )
        .await
        .unwrap_err();
        assert_eq!(err.to_string(), Msg::BlockedBookmark.to_string());
    }

    #[tokio::test]
    async fn extract_with_ai_keeps_content_when_the_result_is_thin() {
        let old = long_text('v');
        let (store, id) = store_with(None, &[], Some(&old));
        let model = FakeModel::default();
        let err = extract_with_ai(
            &store,
            id,
            |_| async { Ok((URL.to_string(), "<p>x</p>".to_string())) },
            &model,
            "m",
        )
        .await
        .unwrap_err();
        assert_eq!(err.to_string(), Msg::ThinResult.to_string());
        let b = get(&store, id);
        assert_eq!(b.content.as_deref(), Some(old.as_str()));
        assert_eq!(b.meta.extractor, None);
    }

    #[tokio::test]
    async fn extract_with_ai_replaces_content_and_keeps_the_metadata() {
        let (store, id) = store_with(Some(USER_TITLE), &[USER_TAG], None);
        let page = Page {
            markdown: long_text('v'),
            meta: crate::extract::PageMeta {
                author: Some("Ana".into()),
                extractor: Some("defuddle@0.19.6".into()),
                ..Default::default()
            },
            ..Page::default()
        };
        store.lock().unwrap().set_page(id, &page).unwrap();
        let fenced = format!("```markdown\n{}\n```", long_text('n'));
        let seen = Mutex::new(String::new());
        extract_with_ai(
            &store,
            id,
            |url| {
                *seen.lock().unwrap() = url;
                async move { Ok((URL.to_string(), fenced)) }
            },
            &FakeModel::default(),
            "reader",
        )
        .await
        .unwrap();
        assert_eq!(*seen.lock().unwrap(), URL);
        let b = get(&store, id);
        assert_eq!(b.content, Some(long_text('n')));
        assert_eq!(b.meta.author.as_deref(), Some("Ana"));
        assert_eq!(b.meta.extractor.as_deref(), Some("ai:reader"));
        assert_eq!(b.status, Status::Pending);
        assert_eq!(b.title, USER_TITLE);
        assert_eq!(b.tags, vec![USER_TAG.to_string()]);
    }

    #[tokio::test]
    async fn extract_with_ai_falls_back_to_the_stored_content() {
        let old = long_text('v');
        let (store, id) = store_with(None, &[], Some(&old));
        let failing = |_| async { Err(anyhow::anyhow!("sin red")) };
        extract_with_ai(&store, id, failing, &FakeModel::default(), "m")
            .await
            .unwrap();
        let b = get(&store, id);
        // `FakeModel` returns its input: the input was the stored content.
        assert_eq!(b.content.as_deref(), Some(old.as_str()));
        assert_eq!(b.meta.extractor.as_deref(), Some("ai:m"));

        let (empty, id) = store_with(None, &[], None);
        let err = extract_with_ai(&empty, id, failing, &FakeModel::default(), "m")
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), "sin red");
    }

    /// `FakeModel` that, the first time it summarizes (`on_embed: false`) or embeds (`on_embed: true`),
    /// replaces the page as a concurrent manual re-extraction would.
    struct ReextractsMidCall<'a> {
        store: &'a Mutex<Store>,
        id: i64,
        on_embed: bool,
        done: std::sync::atomic::AtomicBool,
    }

    impl ReextractsMidCall<'_> {
        fn replace_once(&self) {
            if !self.done.swap(true, std::sync::atomic::Ordering::SeqCst) {
                let page = Page {
                    markdown: long_text('n'),
                    ..Page::default()
                };
                self.store
                    .lock()
                    .unwrap()
                    .replace_page(self.id, &page)
                    .unwrap();
            }
        }
    }

    #[async_trait]
    impl Model for ReextractsMidCall<'_> {
        async fn summarize(&self, input: SummarizeInput<'_>) -> Result<Summary> {
            if !self.on_embed {
                self.replace_once();
            }
            FakeModel::default().summarize(input).await
        }

        async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            if self.on_embed {
                self.replace_once();
            }
            FakeModel::default().embed(texts).await
        }

        async fn extract_markdown(&self, input: &str) -> Result<String> {
            Ok(input.to_string())
        }

        fn embedding_id(&self) -> String {
            FakeModel::default().embedding_id()
        }
    }

    fn reextracts_mid_call(store: &Mutex<Store>, id: i64, on_embed: bool) -> ReextractsMidCall<'_> {
        ReextractsMidCall {
            store,
            id,
            on_embed,
            done: false.into(),
        }
    }

    #[tokio::test]
    async fn a_reextraction_during_the_summary_discards_the_old_summary() {
        let (store, id) = store_with(None, &[], Some(&long_text('v')));
        let model = reextracts_mid_call(&store, id, false);
        process_next(&store, &FakeFetcher::default(), Some(&model), &no_guard)
            .await
            .unwrap();
        let b = get(&store, id);
        assert_eq!(b.content, Some(long_text('n')));
        assert_eq!(b.summary, None);
        assert_eq!(b.status, Status::Pending);
        // The next pass summarizes and indexes the new content.
        process_next(&store, &FakeFetcher::default(), Some(&model), &no_guard)
            .await
            .unwrap();
        let b = get(&store, id);
        assert_eq!(b.status, Status::Ready);
        assert!(b.summary.is_some());
        assert!(chunk_count(&store, id) > 0);
    }

    #[tokio::test]
    async fn a_reextraction_during_the_embeddings_discards_the_old_vectors() {
        let (store, id) = store_with(None, &[], Some(&long_text('v')));
        let model = reextracts_mid_call(&store, id, true);
        process_next(&store, &FakeFetcher::default(), Some(&model), &no_guard)
            .await
            .unwrap();
        let b = get(&store, id);
        assert_eq!(b.content, Some(long_text('n')));
        assert_eq!(b.status, Status::Pending);
        assert_eq!(chunk_count(&store, id), 0);
        process_next(&store, &FakeFetcher::default(), Some(&model), &no_guard)
            .await
            .unwrap();
        assert_eq!(get(&store, id).status, Status::Ready);
        assert!(chunk_count(&store, id) > 0);
    }

    const CHALLENGE: &str = include_str!("../fixtures/cloudflare-challenge.html");
    const CHALLENGE_URL: &str = "https://www.example.com/post";

    #[tokio::test]
    async fn extract_with_ai_refuses_a_bot_check_page() {
        // Without the check the challenge page would pass: it is not thin.
        assert!(!is_thin(CHALLENGE));
        let old = long_text('v');
        let (store, id) = store_with(None, &[], Some(&old));
        let load = |_| async { Ok((CHALLENGE_URL.to_string(), CHALLENGE.to_string())) };
        let err = extract_with_ai(&store, id, load, &FakeModel::default(), "m")
            .await
            .unwrap_err();
        assert!(is_blocked_error(&err), "{err:#}");
        let b = get(&store, id);
        assert_eq!(b.content.as_deref(), Some(old.as_str()));
        assert_eq!(b.meta.extractor, None);
    }

    #[tokio::test]
    async fn extract_with_ai_counts_the_words_of_the_new_content() {
        let (store, id) = store_with(None, &[], None);
        let page = Page {
            markdown: long_text('v'),
            meta: crate::extract::PageMeta {
                word_count: Some(9_999),
                ..Default::default()
            },
            ..Page::default()
        };
        store.lock().unwrap().set_page(id, &page).unwrap();
        let text = format!("{} dos tres", long_text('n'));
        let load = |_| async move { Ok((URL.to_string(), text)) };
        extract_with_ai(&store, id, load, &FakeModel::default(), "m")
            .await
            .unwrap();
        assert_eq!(get(&store, id).meta.word_count, Some(3));
    }
}
