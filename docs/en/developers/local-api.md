# Local API

[Español](../../es/developers/local-api.md) · **English**

The desktop app opens an HTTP API for the browser extension. The code is in `apps/desktop/src-tauri/src/api.rs`. The API decides nothing: it checks the request and calls `core::capture`.

- It listens on `127.0.0.1:47821` only, while the app is open.
- It has no CORS, on purpose: no web page should be able to use it.
- If the port is taken at startup, the app keeps working without the API and leaves a notification.

## Checks

Every request goes through a guard before reaching its route, even when the route does not exist. It checks, in this order:

1. **`Host`.** It must be `127.0.0.1:47821` or `localhost:47821`. Otherwise, `403` with the code `host`.
2. **`Origin`.** If the header is present, its scheme must be an extension's: `safari-web-extension`, `chrome-extension` or `moz-extension`. Otherwise, `403` with the code `origin`. A request with no `Origin`, such as one from `curl`, passes.
3. **Token.** It travels in `Authorization: Bearer <token>`, never in the URL. If it is missing or belongs to no profile, `401` with the code `token`.

Then it checks the size declared in `Content-Length`.

## Token and profile

Each marks profile has its own token, 32 random bytes. The token decides which profile the page is saved to: the request never names the profile.

- core compares the token in constant time and never accepts an empty one.
- Regenerating the token in Settings invalidates the old one.
- The token of a deleted profile gets a `401`.

## Routes

### `GET /v1/ping`

Checks the connection and the token.

```json
{ "app": "marks", "version": "1.0.0", "api": 1, "profile": "Personal" }
```

`version` is the app version. `profile` is the name of the profile that owns the token.

`api` is the API level (`API_LEVEL` in `api.rs`). The extension declares the lowest one it works with (`MIN_API` in `apps/extension/client.js`):

- A missing `api` counts as 0. If `api` is below `MIN_API`, the extension says the app is too old (`appTooOld`).
- A route that answers `404` means the app no longer has it: the extension says it is the one that is too old (`extensionTooOld`).
- `API_LEVEL` is raised only by **additive** API changes an extension may need (a new field or route). A breaking change goes under another prefix (`/v2/…`), and old routes are removed once no published extension uses them.
- `MIN_API` is raised only when the extension starts using something from a new level. A test (`extension_files_agree_with_the_app`) requires `1 <= MIN_API <= API_LEVEL`.

### `POST /v1/bookmarks`

Saves a page. The body is JSON:

| Field | Type | Required | Content |
|---|---|---|---|
| `url` | string | yes | The page URL. `http` or `https` only |
| `title` | string | no | The tab title |
| `page` | object | no | What Defuddle extracted in the browser |
| `text` | string | no | The document's `innerText`. Used when `page` has little content |

`page` holds `title` (optional), `markdown` (required) and `meta` (optional). `meta` accepts `description`, `author`, `site`, `published`, `language`, `image`, `favicon`, `word_count` and `extractor`. All of it comes from a third-party page, so core trims and validates it before storing it.

Response:

```json
{ "id": 42, "created": true, "status": "pending", "blocked": false, "profile": "Personal" }
```

| Field | Meaning |
|---|---|
| `id` | The bookmark's `id` within its profile |
| `created` | `true` if it is new; `false` if the URL was already there |
| `status` | The bookmark's [state](architecture.md#bookmark-states) |
| `blocked` | `true` if the page was an anti-bot check. Its content is not stored |
| `profile` | The name of the profile it was saved to |

If the URL was already there, the bookmark only takes the content when it had none or was `failed`. Otherwise only the missing metadata is filled in. Capturing a URL in the `imported` state queues it.

## Errors

An error returns `{ "error": "<code>" }`. The API returns no text: the extension translates each code through the `error_<code>` key of its `_locales`, with dashes replaced by `_`.

| Code | HTTP | Cause |
|---|---|---|
| `host` | 403 | The `Host` header is not the expected one |
| `origin` | 403 | The `Origin` is not an extension's |
| `token` | 401 | The token is missing or not valid |
| `bad-request` | 400 | The body is not the expected JSON |
| `too-large` | 413 | The body is over 8 MB |
| `invalid-url` | 400 | The URL is not valid |
| `unsupported-scheme` | 400 | The URL is neither `http` nor `https` |
| `internal` | 500 | The app failed. The detail goes to the log, not to the response |

`invalid-url` and `unsupported-scheme` are `Msg` codes, the same ones core uses in stored data.

## Body size

The limit is 8 MB (`BODY_LIMIT`). On a `413`, the extension retries once without the `text` field.

When it rejects a request, the API reads and discards the body before answering, capped in bytes and at 5 seconds. Without that, the client would get a connection reset instead of the error.

## Safari

In Safari, a `fetch` from the extension to loopback fails. So the extension uses native messaging:

1. `client.js` picks `nativeTransport` when the extension's protocol is `safari-web-extension:`. In other browsers it uses `fetchTransport`.
2. The Swift handler (`SafariWebExtensionHandler.swift`) receives `{method, path, token, body?}`.
3. It checks the shape of the message and that the route and method are on its allowlist: `GET /v1/ping` and `POST /v1/bookmarks`.
4. It forwards the request to `http://127.0.0.1:47821` and returns `{status, body}`.

If it cannot reach the app, it returns `{error: "unreachable"}`. If the message is not valid, `{error: "bad_request"}`. The allowlist is the trust boundary on the native side.

## Trying it with curl

With the app running on a scratch database (`MARKS_DB`) and the token copied from Settings:

```sh
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:47821/v1/ping

curl -X POST http://127.0.0.1:47821/v1/bookmarks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "title": "Example"}'
```

## Adding a route

1. Add the route and its handler to the `router` in `api.rs`. The handler receives the `Profile` the guard left and calls core; blocking work goes in `spawn_blocking`.
2. Add it to `ROUTES` in `apps/extension/client.js`.
3. Add it to `allowedRoutes` in `SafariWebExtensionHandler.swift`.
4. If it returns a new error code, add it to `API_ERROR_CODES` and add its `error_<code>` key to `_locales/es` and `_locales/en`.
5. Write the tests: in `api.rs`, with `tower::ServiceExt::oneshot` on the router, without opening the port; and in `client.test.js`, with a fake transport.

## Known limits

- A web page can tell that marks is running, because the port is open. It cannot read, add or detect bookmarks.
- The server does not identify itself to the extension. A local process that takes port 47821 before marks would receive the token and the content of every saved page.
- The extension has no queue: with marks closed, it does not save.
