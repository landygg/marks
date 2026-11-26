# Architecture

[Español](../../es/developers/architecture.md) · **English**

marks is a Cargo workspace with a pnpm workspace beside it. All the logic lives in one Rust library, `marks-core`. The desktop app, the CLI and the local API only call it.

## Parts

| Path | Package | Role |
|---|---|---|
| `crates/core` | `marks-core` (lib `marks_core`) | SQLite store, extraction, AI, search, profiles, import and export |
| `crates/i18n` | `marks-i18n` | Languages and the Fluent catalog for everything Rust prints |
| `crates/cli` | `marks-cli` (binary `marks`) | CLI: queues and queries only |
| `apps/desktop/src-tauri` | `marks-desktop` | Tauri 2 backend: IPC, worker, local API, tray, menu |
| `apps/desktop/src` | `marks-desktop` (pnpm) | Svelte 5 + TypeScript + Vite frontend |
| `apps/extension` | — | MV3 extension for Safari and Chromium, with Safari's Swift handler |
| `packages/extract` | `marks-extract` | Wrapper around Defuddle: HTML → Markdown and metadata |

`marks-core` does not depend on Tauri, on the interface or on `marks-i18n`. Only the CLI and `src-tauri` depend on `marks-i18n`.

## `crates/core`

| Module | What it does |
|---|---|
| `store` | SQLite, migrations and FTS5. `url` is the normalized URL, used for deduplication; `original_url` is the URL as it arrived |
| `profiles` | `Library`: the profile registry, one `Store` per profile, the active profile, each profile's token, deleting and moving bookmarks |
| `msg` | `Msg` (message codes with typed arguments), `Failure` and `BookmarkError`. core holds no user-facing text |
| `groups` | Tag groups |
| `notifications` | In-memory notification log, capped at 200 entries. No I/O and no clock |
| `chunk` | Chunks of about 1,500 characters, 40 at most per bookmark |
| `ai` | The `Model` trait (summary, embeddings, extraction), the prompts and `FakeModel` for tests |
| `genai_model` | `Model` on top of the `genai` library. Chat with Ollama goes to its native `/api/chat` API |
| `settings` | AI settings per role, and keys in the system keychain |
| `vectors` | Vector index with sqlite-vec |
| `search` | Hybrid search: FTS5 and nearest neighbors, merged with RRF (`k = 60`) |
| `pipeline` | `process_next`, manual re-extraction and favicon downloads |
| `extract` | The `Page` and `PageMeta` types, `is_thin` and the `Fetcher`s. It does not convert HTML |
| `download` | HTTP download with validated destinations, a size cap and the declared charset |
| `blocked` | Detects anti-bot check pages |
| `capture` | Decides what is stored from a capture sent by the extension |
| `netscape` | Reads and writes the `bookmarks.html` format |
| `transfer` | Import (`read_file`, `preview`, `apply`) and `export_html` |

`normalize_url`, in `lib.rs`, drops the fragment and the `utm_` parameters. It keeps fragments that start with `#/` or `#!`, because those are routes of a web app.

## `crates/i18n`

core returns codes (`Msg`) and `marks-i18n` turns them into text. The crate holds:

- `Locale` (`Es`, `En` and `Pseudo`, the last one for development only) and `resolve`, which picks the language.
- The Fluent catalog, in `crates/i18n/i18n/{es,en}/marks_i18n.ftl`.
- The `AppText`, `MenuText`, `CliText` and `DataText` enums, for text that is not a `Msg`.

See [Languages and text](i18n.md) for the details.

## `crates/cli`

The `marks` binary. Subcommands: `add`, `list`, `search`, `export`, `import`, `export-html`, `stats` and `profiles`.

- It only queues: it never downloads pages or calls models. The app extracts what was queued the next time it opens.
- `--profile <name>` picks the profile. Without it, the CLI uses the first profile, not the one active in the app.
- `--db <path>` or the `MARKS_DB` variable pick the database.
- There is no `help` subcommand: help is `--help`.

## `apps/desktop`

Two halves with a narrow boundary between them.

**`src-tauri` (Rust).** Exposes core over IPC and runs the background work.

| File | Role |
|---|---|
| `lib.rs` | Startup: plugins, state, worker, API, tray and the command list |
| `state.rs` | `AppState`: the `Library`, the models, the notification list and the locks |
| `commands.rs` | The IPC commands. Each one takes the state and calls core |
| `worker.rs` | The loop that processes the queue |
| `api.rs` | The [local API](local-api.md) |
| `webview_fetcher.rs`, `parser_window.rs` | The hidden `extractor` and `parser` windows |
| `extracting.rs` | Which bookmark is being extracted |
| `notifications.rs` | Stores a notification and tells the window |
| `tray.rs` | Menu bar icon; closing the window hides it |
| `menu.rs` | The app's native menu. macOS only |
| `locale.rs` | The app language, stored in `AppleLanguages`. macOS only |

**`src` (Svelte 5).** Only renders and calls `lib/api.ts`, the one file that uses `invoke`.

| Path | Role |
|---|---|
| `lib/api.ts` | Typed wrappers for every command, and the event names |
| `lib/features/<zone>/` | One folder per zone: `layout`, `library`, `notifications`, `palette`, `profiles`, `settings` and `statusbar` |
| `lib/ui/` | The components described in `DESIGN.md`. They take text and data as props |
| `lib/i18n/` | The `es.ts` and `en.ts` catalogs |
| `lib/shortcuts.ts` | Keyboard shortcuts |

## `apps/extension`

An MV3 extension in plain JavaScript, with the same code for Safari and Chromium. `build.mjs` bundles it with esbuild into `apps/extension/dist/chromium/`, which Chromium loads, and copies it to `apps/extension/dist/safari/`, which goes into the appex. Only the manifest differs, written for each browser from `manifest.json`: Chromium's drops `nativeMessaging` (only the Safari appex talks to the app natively) and declares `minimum_chrome_version` 123, the engine floor.

- `popup.js` sends a message to the service worker and shows the result.
- `background.js` injects `page-extract.js` into the tab, extracts from a clone of the document and sends the capture to the app.
- `client.js` is the API client. It uses neither the DOM nor the extension APIs, so its tests run under Node.
- `safari/SafariWebExtensionHandler.swift` relays Safari's native messages to the local API.

The extension does not decide what gets stored: `core::capture` does.

## `packages/extract`

`marks-extract` is the only package that depends on Defuddle. Two places load it:

- `apps/desktop/extract.html`, in the app's hidden `parser` window.
- `apps/extension/page-extract.ts`, which esbuild bundles so it can be injected into the page.

Defuddle runs offline: the package rejects every request its site extractors try to make.

## The life of a bookmark

1. **Entry.** The bookmark arrives from the app's dialog, from `marks add`, from an import or from a capture sent by the extension (`POST /v1/bookmarks`). A capture may already carry the content extracted in the browser.
2. **Queue.** The worker takes the most recently queued bookmark first. It polls every 15 seconds and also wakes up when told to. It takes turns between profiles: one bookmark and one favicon from each per pass.
3. **Download.** If the bookmark has no content, `HttpFetcher` downloads the HTML over HTTP.
4. **Fallback.** If HTTP fails or the content is thin (under 200 characters), `FallbackFetcher` tries `WebviewFetcher`: the hidden `extractor` window loads the page and returns the rendered HTML. It is not tried after a 404, a 410 or a 5xx.
5. **Extraction.** The hidden `parser` window runs the HTML through Defuddle and returns Markdown and metadata.
6. **Summary.** With AI turned on, the chat model returns a summary, a category and tags. The bookmark moves to `pending_embed`. Without a model the journey ends here and the bookmark stays `pending`, with its content.
7. **Embeddings.** The content is chunked, the vectors are computed and the bookmark becomes `ready`.
8. **Favicon.** Downloaded separately, from public destinations only. Transient failures are retried with a growing delay, up to 4 attempts.

A network or model failure is stored on the bookmark, which becomes `failed`.

## Bookmark states

| State | Meaning |
|---|---|
| `pending` | Queued, or extracted and not yet summarized |
| `pending_embed` | Summarized; embeddings still to do |
| `ready` | Done |
| `failed` | Failed; the error is stored on the bookmark |
| `imported` | Imported and not extracted. The worker does not pick it up |

An `imported` bookmark becomes `pending` through "Extract", `add` or a capture. Any other value is a bug.

## Hidden windows

- `extractor` loads remote pages, which are untrusted.
- `parser` loads only `extract.html` and receives third-party HTML as data.

Neither has any Tauri permissions (capabilities): only the `main` window does. Both share a single lock, so two hidden windows are never open at once. Each one is always destroyed when it finishes, whether it succeeded, failed or hit its time limit.

## Threads and locks

- Each profile has its `Store` behind a `Mutex`. It is never held during a network request or a model call, and two stores are never held at once.
- The database is opened in WAL mode, because the CLI and the app may open it at the same time.
- A synchronous Tauri command runs on the main thread and freezes the window while it lasts. Reads and long work go in `command(async)` commands; network and model calls go in `async fn`s.
- Events from Rust to the window carry no data: they only say that something changed, and the window reloads.
