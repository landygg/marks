//! Local HTTP API for the browser extension: loopback only, with `Host`, `Origin` and token
//! checks. No CORS on purpose: no web page must be able to use it.

use crate::notifications::{notify, warn};
use crate::state::AppState;
use crate::status::ApiStatus;
use crate::worker;
use axum::body::{Body, HttpBody};
use axum::extract::rejection::JsonRejection;
use axum::extract::{DefaultBodyLimit, Extension, Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use marks_core::capture::{Capture, CaptureError, capture};
use marks_core::msg::Msg;
use marks_core::notifications::{NewNotification, Severity, Target};
use marks_core::profiles::{Library, Profile};
use marks_i18n::{AppText, Localize, chain};
use serde::Serialize;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::net::TcpListener;

pub const API_PORT: u16 = 47821;
/// The API's level, in `/v1/ping`. Raised only by additive changes that an extension may need
/// (`MIN_API` in `apps/extension/client.js`); a breaking change is a new route prefix.
const API_LEVEL: u32 = 1;
pub const BODY_LIMIT: usize = 8 * 1024 * 1024;
/// Max bytes read and discarded from a rejected body before answering; past the cap it answers
/// and closes, so a client without a token cannot keep the server reading forever.
const DRAIN_LIMIT: usize = 4 * BODY_LIMIT;
/// Max time draining a rejected body, so a slow client cannot hold the connection; once it
/// expires the rejection is sent anyway.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
const ROUTE_PING: &str = "/v1/ping";
const ROUTE_BOOKMARKS: &str = "/v1/bookmarks";
const APP_NAME: &str = "marks";
const ALLOWED_HOSTS: [&str; 2] = ["127.0.0.1", "localhost"];
const EXTENSION_SCHEMES: [&str; 3] = ["safari-web-extension", "chrome-extension", "moz-extension"];
const BEARER_PREFIX: &str = "Bearer "; // i18n-allow: format
const SCHEME_SEPARATOR: &str = "://";

// The API's own error codes; the extension translates them (`error_<code with _>` in `_locales`).
const CODE_HOST: &str = "host";
const CODE_ORIGIN: &str = "origin";
const CODE_TOKEN: &str = "token";
const CODE_BAD_REQUEST: &str = "bad-request";
const CODE_TOO_LARGE: &str = "too-large";
const CODE_INTERNAL: &str = "internal";

/// Every code the API can return. `Msg` ones come from core's serialized code.
#[cfg(test)] // only the tests against `_locales` check it
pub const API_ERROR_CODES: &[&str] = &[
    CODE_ORIGIN,
    CODE_HOST,
    CODE_TOKEN,
    CODE_BAD_REQUEST,
    CODE_TOO_LARGE,
    CODE_INTERNAL,
    "unsupported-scheme",
    "invalid-url",
];

#[derive(Clone)]
pub struct ApiContext {
    pub library: Arc<Library>,
    /// Called after each successful capture.
    pub on_change: Arc<dyn Fn() + Send + Sync>,
}

/// Knows nothing of Tauri: tested with `tower::ServiceExt::oneshot`.
pub fn router(ctx: ApiContext) -> Router {
    router_with_drain_timeout(ctx, DRAIN_TIMEOUT)
}

/// Variant with an injectable drain timeout so tests do not wait 5 s.
fn router_with_drain_timeout(ctx: ApiContext, drain_timeout: Duration) -> Router {
    Router::new()
        .route(ROUTE_PING, get(ping))
        .route(ROUTE_BOOKMARKS, axum::routing::post(post_bookmark))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        // `layer` (not `route_layer`): the guard also covers nonexistent routes.
        .layer(middleware::from_fn_with_state(
            (ctx.clone(), drain_timeout),
            guard,
        ))
        .with_state(ctx)
}

/// Listens on 127.0.0.1:API_PORT on Tauri's runtime. Port taken ⇒ `log::warn!` and the app goes on.
pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(serve(app));
}

async fn serve(app: AppHandle) {
    let state = app.state::<AppState>();
    let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, API_PORT)).await {
        Ok(listener) => listener,
        Err(e) => {
            warn(
                &app,
                extension_notice(
                    Severity::Warning,
                    chain(
                        &AppText::ApiPortBusy { port: API_PORT }.localize(),
                        &e.to_string(),
                    ),
                ),
            );
            state.set_api_status(&app, ApiStatus::Busy);
            return;
        }
    };
    state.set_api_status(&app, ApiStatus::Listening);
    let notifier = app.clone();
    let ctx = ApiContext {
        library: state.library.clone(),
        on_change: Arc::new(move || {
            notifier.state::<AppState>().wake.notify_one();
            worker::notify_changed(&notifier);
        }),
    };
    if let Err(e) = axum::serve(listener, router(ctx)).await {
        log::error!("api: el servidor se detuvo: {e}");
        state.set_api_status(&app, ApiStatus::Stopped);
        notify(
            &app,
            extension_notice(
                Severity::Error,
                chain(&AppText::ApiStopped.localize(), &e.to_string()),
            ),
        );
    }
}

/// Global API notice that leads to its settings.
fn extension_notice(severity: Severity, message: String) -> NewNotification {
    NewNotification {
        profile: None,
        severity,
        message,
        detail: None,
        target: Some(Target::Extension),
    }
}

#[derive(Serialize)]
struct Pong {
    app: &'static str,
    version: &'static str,
    api: u32,
    profile: String,
}

#[derive(Serialize)]
struct Saved {
    id: i64,
    created: bool,
    status: &'static str,
    blocked: bool,
    profile: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(ErrorBody { error: self.1 })).into_response()
    }
}

impl ApiError {
    fn code(status: StatusCode, code: &str) -> Self {
        Self(status, code.to_string())
    }

    /// A `Msg`'s serialized code (`invalid-url`, …), the same one stored data uses.
    fn msg(status: StatusCode, msg: &Msg) -> Self {
        let code = serde_json::to_value(msg)
            .ok()
            .and_then(|v| v.get("code")?.as_str().map(str::to_string))
            .unwrap_or_else(|| CODE_INTERNAL.to_string());
        Self(status, code)
    }

    fn internal(detail: impl std::fmt::Display) -> Self {
        log::error!("api: {detail:#}");
        Self::code(StatusCode::INTERNAL_SERVER_ERROR, CODE_INTERNAL)
    }
}

async fn guard(
    State((ctx, drain_timeout)): State<(ApiContext, Duration)>,
    mut req: Request,
    next: Next,
) -> Response {
    // `check` is synchronous: the registry lock is released before the first `.await`.
    let verdict = check(&ctx, &req).and_then(|p| check_declared_size(&req).map(|()| p));
    match verdict {
        Ok(profile) => {
            // Routes capture into the profile that owns the token.
            req.extensions_mut().insert(profile);
            next.run(req).await
        }
        Err(e) => {
            // Answering without reading the body makes the client get a reset instead of the error.
            // If the timeout expires the result is ignored: we answer anyway.
            let _ = tokio::time::timeout(drain_timeout, drain(req.into_body())).await;
            e.into_response()
        }
    }
}

/// 413 from `Content-Length`; the extractor's limit covers bodies without that header.
fn check_declared_size(req: &Request) -> Result<(), ApiError> {
    let declared = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    match declared {
        Some(len) if len > BODY_LIMIT as u64 => Err(ApiError::code(
            StatusCode::PAYLOAD_TOO_LARGE,
            CODE_TOO_LARGE,
        )),
        _ => Ok(()),
    }
}

/// Reads and discards the body up to `DRAIN_LIMIT` bytes.
async fn drain(body: Body) {
    let mut body = std::pin::pin!(body);
    let mut drained = 0;
    while drained <= DRAIN_LIMIT {
        match std::future::poll_fn(|cx| body.as_mut().poll_frame(cx)).await {
            Some(Ok(frame)) => drained += frame.data_ref().map_or(0, |data| data.len()),
            _ => break,
        }
    }
}

/// `Host` → `Origin` → token, in that order; returns the profile that owns the token.
fn check(ctx: &ApiContext, req: &Request) -> Result<Profile, ApiError> {
    let headers = req.headers();
    let host_ok = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split_once(':'))
        .is_some_and(|(name, port)| ALLOWED_HOSTS.contains(&name) && port == API_PORT.to_string());
    if !host_ok {
        return Err(ApiError::code(StatusCode::FORBIDDEN, CODE_HOST));
    }
    if let Some(origin) = headers.get(header::ORIGIN) {
        let origin_ok = origin
            .to_str()
            .ok()
            .and_then(|o| o.split_once(SCHEME_SEPARATOR))
            .is_some_and(|(scheme, _)| EXTENSION_SCHEMES.contains(&scheme));
        if !origin_ok {
            return Err(ApiError::code(StatusCode::FORBIDDEN, CODE_ORIGIN));
        }
    }
    let presented = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(BEARER_PREFIX))
        .unwrap_or_default();
    // Core compares in constant time and never accepts an empty token.
    ctx.library
        .profile_for_token(presented)
        .map_err(ApiError::internal)?
        .ok_or(ApiError::code(StatusCode::UNAUTHORIZED, CODE_TOKEN))
}

async fn ping(Extension(profile): Extension<Profile>) -> Json<Pong> {
    Json(Pong {
        app: APP_NAME,
        version: env!("CARGO_PKG_VERSION"),
        api: API_LEVEL,
        profile: profile.name,
    })
}

async fn post_bookmark(
    State(ctx): State<ApiContext>,
    Extension(profile): Extension<Profile>,
    body: Result<Json<Capture>, JsonRejection>,
) -> Result<Json<Saved>, ApiError> {
    let Json(body) = body.map_err(|e| match e.status() {
        StatusCode::PAYLOAD_TOO_LARGE => {
            ApiError::code(StatusCode::PAYLOAD_TOO_LARGE, CODE_TOO_LARGE)
        }
        _ => ApiError::code(StatusCode::BAD_REQUEST, CODE_BAD_REQUEST),
    })?;
    let store = ctx.library.store(profile.id).map_err(ApiError::internal)?;
    let captured = tokio::task::spawn_blocking(move || capture(&store, body))
        .await
        .map_err(ApiError::internal)?
        .map_err(|e| match e {
            CaptureError::InvalidUrl(msg) => ApiError::msg(StatusCode::BAD_REQUEST, &msg),
            CaptureError::Store(e) => ApiError::internal(e),
        })?;
    (ctx.on_change)();
    Ok(Json(Saved {
        id: captured.bookmark.id,
        created: captured.created,
        status: captured.bookmark.status.as_str(),
        blocked: captured.blocked,
        profile: profile.name,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use marks_core::profiles::{DEFAULT_PROFILE_ID, Library, ProfileSeed};
    use marks_core::store::lock;
    use serde_json::json;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use tower::ServiceExt;

    const TEST_SEED: ProfileSeed = ProfileSeed {
        name: "Personal",
        initials: "PE",
    };

    const HOST: &str = "127.0.0.1:47821";
    /// Extension files that repeat the port.
    const PORT_FILES: [&str; 3] = [
        "manifest.json",
        "client.js",
        "safari/SafariWebExtensionHandler.swift",
    ];
    const EXTENSION_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../extension/");

    static FIXTURE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct Fixture {
        ctx: ApiContext,
        token: String,
        changes: Arc<AtomicUsize>,
        dir: std::path::PathBuf,
    }

    /// Deletes the library directory when the test ends.
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn fixture() -> Fixture {
        let n = FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("marks-api-{}-{n}", std::process::id()));
        let library = Arc::new(Library::open(&dir.join("marks.db"), &TEST_SEED).unwrap());
        let token = library.token(DEFAULT_PROFILE_ID).unwrap();
        let changes = Arc::new(AtomicUsize::new(0));
        let counter = changes.clone();
        let ctx = ApiContext {
            library,
            on_change: Arc::new(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            }),
        };
        Fixture {
            ctx,
            token,
            changes,
            dir,
        }
    }

    fn request(method: &str, uri: &str, headers: &[(&str, &str)], body: String) -> Request<Body> {
        let mut req = Request::builder().method(method).uri(uri);
        if !headers.iter().any(|(k, _)| *k == "host") {
            req = req.header("host", HOST);
        }
        for (k, v) in headers {
            req = req.header(*k, *v);
        }
        req.body(Body::from(body)).unwrap()
    }

    async fn send(
        f: &Fixture,
        req: Request<Body>,
    ) -> (StatusCode, axum::http::HeaderMap, serde_json::Value) {
        let res = router(f.ctx.clone()).oneshot(req).await.unwrap();
        let (status, headers) = (res.status(), res.headers().clone());
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            headers,
            serde_json::from_slice(&bytes).unwrap_or_default(),
        )
    }

    fn bearer(token: &str) -> String {
        format!("{BEARER_PREFIX}{token}")
    }

    async fn ping_status(f: &Fixture, headers: &[(&str, &str)]) -> StatusCode {
        send(f, request("GET", ROUTE_PING, headers, String::new()))
            .await
            .0
    }

    async fn post(f: &Fixture, body: String) -> (StatusCode, serde_json::Value) {
        post_with(f, &f.token, body).await
    }

    async fn post_with(f: &Fixture, token: &str, body: String) -> (StatusCode, serde_json::Value) {
        let auth = bearer(token);
        let req = request(
            "POST",
            ROUTE_BOOKMARKS,
            &[
                ("authorization", &auth),
                ("content-type", "application/json"),
            ],
            body,
        );
        let (status, _, json) = send(f, req).await;
        (status, json)
    }

    const PAGE: &str = r#"{"url":"https://example.com/a","title":"A","text":"Texto largo de la página guardada. Texto largo de la página guardada. Texto largo de la página guardada. Texto largo de la página guardada. Texto largo de la página guardada. Texto largo de la página guardada. Texto largo de la página guardada."}"#;

    const FIXTURE_PAGE: &str = include_str!("../../../../packages/extract/fixtures/extracted.json");

    fn stored(f: &Fixture, url: &str) -> marks_core::store::Bookmark {
        let store = f.ctx.library.store(DEFAULT_PROFILE_ID).unwrap();
        lock(&store).unwrap().find_by_url(url).unwrap().unwrap()
    }

    #[tokio::test]
    async fn post_with_fixture_stores_all_metadata() {
        let f = fixture();
        let page: serde_json::Value = serde_json::from_str(FIXTURE_PAGE).unwrap();
        let body = json!({"url": "https://example.com/m", "page": page}).to_string();
        let (status, res) = post(&f, body).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(res["blocked"], false);
        let b = stored(&f, "https://example.com/m");
        let m = b.meta;
        assert_eq!(b.title, "Guía del contrato");
        assert_eq!(m.author.as_deref(), Some("Luis Gómez"));
        assert_eq!(m.site.as_deref(), Some("Sitio de Luis"));
        assert_eq!(m.language.as_deref(), Some("es"));
        assert_eq!(m.image.as_deref(), Some("https://cdn.example.com/i.png"));
        assert_eq!(
            m.favicon.as_deref(),
            Some("https://example.com/favicon.ico")
        );
        assert_eq!(m.word_count, Some(49));
        assert_eq!(m.extractor.as_deref(), Some("defuddle@0.19.6"));
        assert!(m.description.is_some() && m.published.is_some());
    }

    #[tokio::test]
    async fn long_metadata_is_bounded() {
        let f = fixture();
        let md = "palabra ".repeat(60);
        let long = "ñ".repeat(5_000);
        let page = json!({"markdown": md, "meta": {"description": long, "author": long}});
        let body = json!({"url": "https://example.com/l", "page": page}).to_string();
        assert_eq!(post(&f, body).await.0, StatusCode::OK);
        let m = stored(&f, "https://example.com/l").meta;
        assert_eq!(m.description.unwrap().chars().count(), 1_000);
        assert_eq!(m.author.unwrap().chars().count(), 300);
    }

    #[tokio::test]
    async fn javascript_image_is_dropped() {
        let f = fixture();
        let md = "palabra ".repeat(60);
        let page = json!({"markdown": md, "meta": {"image": "javascript:alert(1)", "favicon": "data:image/png;base64,AA"}});
        let body = json!({"url": "https://example.com/j", "page": page}).to_string();
        assert_eq!(post(&f, body).await.0, StatusCode::OK);
        let m = stored(&f, "https://example.com/j").meta;
        assert_eq!((m.image, m.favicon), (None, None));
    }

    #[tokio::test]
    async fn blocked_page_answers_blocked_true() {
        let f = fixture();
        let page = json!({"markdown": "Just a moment...\n\nVerify you are human"});
        let body = json!({"url": "https://example.com/b", "page": page}).to_string();
        let (status, res) = post(&f, body).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(res["blocked"], true);
        assert_eq!(res["status"], "failed");
    }

    #[tokio::test]
    async fn foreign_host_is_403() {
        let f = fixture();
        let auth = bearer(&f.token);
        let status = ping_status(&f, &[("host", "evil.example"), ("authorization", &auth)]).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn localhost_host_passes() {
        let f = fixture();
        let auth = bearer(&f.token);
        let status =
            ping_status(&f, &[("host", "localhost:47821"), ("authorization", &auth)]).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn allowed_host_with_wrong_port_is_403() {
        let f = fixture();
        let auth = bearer(&f.token);
        let status = ping_status(&f, &[("host", "127.0.0.1:80"), ("authorization", &auth)]).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn web_origin_is_403() {
        let f = fixture();
        let auth = bearer(&f.token);
        let status = ping_status(
            &f,
            &[("origin", "https://evil.example"), ("authorization", &auth)],
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn null_origin_is_403() {
        let f = fixture();
        let auth = bearer(&f.token);
        let status = ping_status(&f, &[("origin", "null"), ("authorization", &auth)]).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn extension_origins_pass() {
        let f = fixture();
        let auth = bearer(&f.token);
        for origin in [
            "safari-web-extension://abc",
            "chrome-extension://abc",
            "moz-extension://abc",
        ] {
            let status = ping_status(&f, &[("origin", origin), ("authorization", &auth)]).await;
            assert_eq!(status, StatusCode::OK, "{origin}");
        }
    }

    #[tokio::test]
    async fn missing_or_wrong_token_is_401() {
        let f = fixture();
        let basic = format!("Basic {}", f.token);
        assert_eq!(ping_status(&f, &[]).await, StatusCode::UNAUTHORIZED);
        for auth in ["Bearer nope", basic.as_str()] {
            assert_eq!(
                ping_status(&f, &[("authorization", auth)]).await,
                StatusCode::UNAUTHORIZED
            );
        }
    }

    #[tokio::test]
    async fn empty_bearer_token_is_401() {
        let f = fixture();
        for auth in ["Bearer ", "Bearer"] {
            assert_eq!(
                ping_status(&f, &[("authorization", auth)]).await,
                StatusCode::UNAUTHORIZED
            );
        }
    }

    #[tokio::test]
    async fn unknown_route_without_token_is_401() {
        let f = fixture();
        let (status, _, _) = send(&f, request("GET", "/nada", &[], String::new())).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn preflight_gets_no_cors_headers() {
        let f = fixture();
        let req = request(
            "OPTIONS",
            ROUTE_BOOKMARKS,
            &[("origin", "https://evil.example")],
            String::new(),
        );
        let (status, headers, _) = send(&f, req).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(!headers.contains_key("access-control-allow-origin"));
    }

    #[tokio::test]
    async fn ping_names_the_profile() {
        let f = fixture();
        let auth = bearer(&f.token);
        let (status, _, json) = send(
            &f,
            request(
                "GET",
                ROUTE_PING,
                &[("authorization", &auth)],
                String::new(),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            json,
            json!({
                "app": APP_NAME,
                "version": env!("CARGO_PKG_VERSION"),
                "api": API_LEVEL,
                "profile": TEST_SEED.name,
            })
        );
    }

    #[tokio::test]
    async fn token_saves_into_its_own_profile() {
        let f = fixture();
        let work = f.ctx.library.create("Trabajo", "TR").unwrap();
        let token = f.ctx.library.token(work.id).unwrap();
        let (status, json) = post_with(&f, &token, PAGE.to_string()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["profile"], "Trabajo");
        let id = json["id"].as_i64().unwrap();
        let work_store = f.ctx.library.store(work.id).unwrap();
        assert!(lock(&work_store).unwrap().get(id).unwrap().is_some());
        let primary = f.ctx.library.store(DEFAULT_PROFILE_ID).unwrap();
        assert!(lock(&primary).unwrap().list().unwrap().is_empty());

        // Profile 1's token, used once profile 2 exists, saves into profile 1.
        let (status, json) = post(&f, PAGE.to_string()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["profile"], TEST_SEED.name);
        assert_eq!(lock(&primary).unwrap().list().unwrap().len(), 1);
        assert_eq!(lock(&work_store).unwrap().list().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn deleted_profile_token_is_401() {
        let f = fixture();
        let work = f.ctx.library.create("Trabajo", "TR").unwrap();
        let auth = bearer(&f.ctx.library.token(work.id).unwrap());
        f.ctx.library.delete(work.id).unwrap();
        assert_eq!(
            ping_status(&f, &[("authorization", &auth)]).await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn post_creates_bookmark_and_notifies() {
        let f = fixture();
        let (status, json) = post(&f, PAGE.to_string()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["created"], true);
        assert_eq!(json["status"], "pending");
        assert_eq!(json["profile"], TEST_SEED.name);
        assert_eq!(f.changes.load(Ordering::SeqCst), 1);
        let id = json["id"].as_i64().unwrap();
        let store = f.ctx.library.store(DEFAULT_PROFILE_ID).unwrap();
        let saved = lock(&store).unwrap().get(id).unwrap().unwrap();
        assert!(saved.content.is_some_and(|c| c.contains("Texto largo")));
    }

    #[tokio::test]
    async fn post_duplicate_reports_not_created() {
        let f = fixture();
        post(&f, PAGE.to_string()).await;
        let (status, json) = post(&f, PAGE.to_string()).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["created"], false);
    }

    #[tokio::test]
    async fn post_invalid_url_is_400() {
        let f = fixture();
        let (status, json) = post(&f, r#"{"url":"ftp://a.b"}"#.to_string()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["error"], "unsupported-scheme");
        assert_eq!(f.changes.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn post_bad_json_is_400() {
        let f = fixture();
        let (status, json) = post(&f, r#"{"url":"#.to_string()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["error"], "bad-request");
    }

    #[tokio::test]
    async fn body_over_limit_is_413() {
        let f = fixture();
        let body = json!({ "url": "https://example.com", "html": "a".repeat(BODY_LIMIT + 1) });
        let (status, json) = post(&f, body.to_string()).await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(json["error"], "too-large");
    }

    #[tokio::test]
    async fn regenerated_token_applies_immediately() {
        let f = fixture();
        let old = bearer(&f.token);
        let new = f.ctx.library.regenerate_token(DEFAULT_PROFILE_ID).unwrap();
        assert_eq!(
            ping_status(&f, &[("authorization", &old)]).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ping_status(&f, &[("authorization", &bearer(&new))]).await,
            StatusCode::OK
        );
    }

    /// Minimal HTTP/1.1 client over a real socket (not `oneshot`): writes the request on one thread
    /// and reads the response on another, as URLSession and fetch do. Returns the write result and
    /// the status line.
    fn raw_exchange(
        addr: std::net::SocketAddr,
        auth: &str,
        declared_len: usize,
        sent_len: usize,
    ) -> (std::io::Result<()>, String) {
        use std::io::{BufRead, BufReader, Write};
        let stream = std::net::TcpStream::connect(addr).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(SOCKET_TIMEOUT_SECS)))
            .unwrap();
        let mut writer = stream.try_clone().unwrap();
        let head = format!(
            "POST {ROUTE_BOOKMARKS} HTTP/1.1\r\nHost: {HOST}\r\nAuthorization: {auth}\r\nContent-Type: application/json\r\nContent-Length: {declared_len}\r\n\r\n"
        );
        let write = std::thread::spawn(move || {
            writer.write_all(head.as_bytes())?;
            let chunk = vec![b'a'; WRITE_CHUNK];
            let mut left = sent_len;
            while left > 0 {
                let n = left.min(WRITE_CHUNK);
                writer.write_all(&chunk[..n])?;
                left -= n;
            }
            writer.flush()
        });
        let mut status = String::new();
        let _ = BufReader::new(&stream).read_line(&mut status);
        (write.join().unwrap(), status.trim_end().to_string())
    }

    async fn serve_on_ephemeral_port(app: Router) -> std::net::SocketAddr {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        addr
    }

    const SOCKET_TIMEOUT_SECS: u64 = 10;
    const TEST_DRAIN_TIMEOUT: Duration = Duration::from_millis(300);
    /// Margin over `TEST_DRAIN_TIMEOUT` for answering and for a loaded machine.
    const DRAIN_SLACK: Duration = Duration::from_secs(1);
    /// Body the slow client announces and never finishes sending.
    const SLOW_CLIENT_DECLARED_LEN: usize = 1024 * 1024;
    const SLOW_CLIENT_BYTES: usize = 16;
    const WRITE_CHUNK: usize = 64 * 1024;
    const SOCKET_ROUNDS: usize = 5;

    async fn socket_status(
        addr: std::net::SocketAddr,
        auth: String,
        declared_len: usize,
        sent_len: usize,
    ) -> (std::io::Result<()>, String) {
        tokio::task::spawn_blocking(move || raw_exchange(addr, &auth, declared_len, sent_len))
            .await
            .unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn wrong_token_with_large_body_is_a_clean_401_over_a_socket() {
        let f = fixture();
        let addr = serve_on_ephemeral_port(router(f.ctx.clone())).await;
        for len in [1024 * 1024, 3 * 1024 * 1024, 12 * 1024 * 1024] {
            for round in 0..SOCKET_ROUNDS {
                let (write, status) = socket_status(addr, bearer("nope"), len, len).await;
                assert!(write.is_ok(), "{len} B, ronda {round}: {write:?}");
                assert_eq!(
                    status, "HTTP/1.1 401 Unauthorized",
                    "{len} B, ronda {round}"
                );
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn over_limit_body_is_a_clean_413_over_a_socket() {
        let f = fixture();
        let addr = serve_on_ephemeral_port(router(f.ctx.clone())).await;
        for len in [BODY_LIMIT + 1, 2 * BODY_LIMIT] {
            for round in 0..SOCKET_ROUNDS {
                let (write, status) = socket_status(addr, bearer(&f.token), len, len).await;
                assert!(write.is_ok(), "{len} B, ronda {round}: {write:?}");
                assert_eq!(
                    status, "HTTP/1.1 413 Payload Too Large",
                    "{len} B, ronda {round}"
                );
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn drain_stops_at_its_cap() {
        let f = fixture();
        let addr = serve_on_ephemeral_port(router(f.ctx.clone())).await;
        // Declares a huge body but sends only a little more than the cap: the server must answer
        // without waiting for the rest (otherwise the read times out).
        let (_, status) = socket_status(
            addr,
            bearer("nope"),
            16 * DRAIN_LIMIT,
            DRAIN_LIMIT + WRITE_CHUNK,
        )
        .await;
        assert_eq!(status, "HTTP/1.1 401 Unauthorized");
    }

    /// Slow client: headers with a wrong token, a few body bytes, then silence without closing.
    /// Returns the status line and how long it took to arrive.
    fn slow_client(addr: std::net::SocketAddr) -> (String, Duration) {
        use std::io::{BufRead, BufReader, Write};
        let mut stream = std::net::TcpStream::connect(addr).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(SOCKET_TIMEOUT_SECS)))
            .unwrap();
        let head = format!(
            "POST {ROUTE_BOOKMARKS} HTTP/1.1\r\nHost: {HOST}\r\nAuthorization: {}\r\nContent-Length: {}\r\n\r\n",
            bearer("nope"),
            SLOW_CLIENT_DECLARED_LEN
        );
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(&[b'a'; SLOW_CLIENT_BYTES]).unwrap();
        let started = std::time::Instant::now();
        let mut status = String::new();
        let _ = BufReader::new(&stream).read_line(&mut status);
        (status.trim_end().to_string(), started.elapsed())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn slow_rejected_client_is_answered_within_the_drain_timeout() {
        let f = fixture();
        let addr =
            serve_on_ephemeral_port(router_with_drain_timeout(f.ctx.clone(), TEST_DRAIN_TIMEOUT))
                .await;
        let (status, elapsed) = tokio::task::spawn_blocking(move || slow_client(addr))
            .await
            .unwrap();
        assert_eq!(status, "HTTP/1.1 401 Unauthorized");
        assert!(elapsed < TEST_DRAIN_TIMEOUT + DRAIN_SLACK, "{elapsed:?}");
    }

    fn extension_file(name: &str) -> String {
        std::fs::read_to_string(format!("{EXTENSION_DIR}{name}")).unwrap()
    }

    #[test]
    fn extension_translates_every_api_error() {
        for lang in ["es", "en"] {
            let messages: serde_json::Value =
                serde_json::from_str(&extension_file(&format!("_locales/{lang}/messages.json")))
                    .unwrap();
            for code in API_ERROR_CODES {
                let key = format!("error_{}", code.replace('-', "_"));
                assert!(
                    messages[&key]["message"]
                        .as_str()
                        .is_some_and(|m| !m.is_empty()),
                    "{lang} no traduce {key}"
                );
            }
        }
    }

    #[tokio::test]
    async fn every_error_the_api_sends_is_listed() {
        let f = fixture();
        let mut codes = Vec::new();
        for body in [
            r#"{"url":"ftp://a.b"}"#,
            r#"{"url":"no es una url"}"#,
            r#"{"url":"#,
        ] {
            codes.push(post(&f, body.to_string()).await.1["error"].clone());
        }
        codes.push(post_with(&f, "mal", PAGE.to_string()).await.1["error"].clone());
        let big = json!({ "url": "https://example.com", "html": "a".repeat(BODY_LIMIT + 1) });
        codes.push(post(&f, big.to_string()).await.1["error"].clone());
        let auth = bearer(&f.token);
        for headers in [
            vec![
                ("host", "evil.example:47821"),
                ("authorization", auth.as_str()),
            ],
            vec![
                ("origin", "https://evil.example"),
                ("authorization", auth.as_str()),
            ],
        ] {
            let (_, _, json) = send(&f, request("GET", ROUTE_PING, &headers, String::new())).await;
            codes.push(json["error"].clone());
        }
        // `internal` has no simple request that triggers it.
        for code in codes {
            let code = code.as_str().unwrap_or_default();
            assert!(
                API_ERROR_CODES.contains(&code),
                "`{code}` no está en API_ERROR_CODES"
            );
        }
    }

    #[test]
    fn extension_files_agree_with_the_app() {
        let port = API_PORT.to_string();
        for name in PORT_FILES {
            assert!(
                extension_file(name).contains(&port),
                "{name} no lleva el puerto"
            );
        }
        let manifest: serde_json::Value =
            serde_json::from_str(&extension_file("manifest.json")).unwrap();
        assert!(manifest["version"].is_string());
        let client = extension_file("client.js");
        let marker = "export const MIN_API = ";
        let start = client.find(marker).expect("client.js no lleva MIN_API") + marker.len();
        let end = start + client[start..].find(';').expect("MIN_API sin ;");
        let min_api: u32 = client[start..end]
            .trim()
            .parse()
            .expect("MIN_API no es un número");
        assert!(
            (1..=API_LEVEL).contains(&min_api),
            "MIN_API {min_api} fuera de 1..={API_LEVEL}"
        );
        for name in ["client.js", "safari/SafariWebExtensionHandler.swift"] {
            let source = extension_file(name);
            for route in [ROUTE_PING, ROUTE_BOOKMARKS] {
                assert!(source.contains(route), "{name} no lleva {route}");
            }
        }
    }
}
