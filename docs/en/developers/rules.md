# Repository rules

[Español](../../es/developers/rules.md) · **English**

Every change must respect these rules. `scripts/check.sh` enforces most of them; the rest are checked in review.

## Where the logic goes

- **Logic goes in `crates/core`.** Tauri commands (3 to 10 lines), the CLI and the local API decide nothing: they take their arguments and call core.
- **`core` depends on neither Tauri nor the interface.** Nor on `marks-i18n`.
- **The extension decides nothing either.** It extracts the page and sends it; `core::capture` decides what is stored.

## Bookmark states

The only states are `pending`, `pending_embed`, `ready`, `failed` and `imported`. Any other value is a bug.

`imported` means imported and not extracted. The worker does not pick it up. It becomes `pending` through "Extract", `add` or a capture.

## Locks

- **Never hold the `Mutex<Store>`** during a network request or a model call.
- **Never hold two stores at once.**
- Never hold a lock across an `.await` either. Clone the `Arc` and let go.

## Profiles

- **Bookmark and group `id`s repeat across profiles.** Anything that names a bookmark, or writes, carries the `id` of its profile: the profile the data was loaded in or the edit was opened in, not the one active when it is sent.
- Only reads and `add` use the active profile.
- The first profile cannot be deleted, because it holds the profile registry and the global settings.
- There is no search across profiles.
- AI settings and provider keys are global. There is one key per role: summary and extraction.
- The API token belongs to each profile.

## Artificial intelligence

- AI can be off, and it is off by default on a new install. Without a model, the worker only extracts.
- The worker reads the model before each bookmark: turning AI off stops it at the next one.
- Manual re-extraction ("Extract again" and "Extract with AI") is only started by the user from the main window. Neither the API nor the worker starts it.

## User-facing text

**No text the user can see is written in the code.** It all goes in the language files, in Spanish and English at the same time:

| Layer | Files |
|---|---|
| Rust (core, `src-tauri`, CLI) | `crates/i18n/i18n/{es,en}/marks_i18n.ftl` |
| Frontend | `apps/desktop/src/lib/i18n/{es,en}.ts` |
| Extension | `apps/extension/_locales/{es,en}/messages.json` |
| App name | `apps/desktop/src-tauri/infoplist/{es,en}.lproj/InfoPlist.strings` |
| Safari extension name | `apps/extension/safari/{es,en}.lproj/InfoPlist.strings` |

Two guards in `scripts/check.sh` fail when they find a user-facing literal outside those files. See [Languages and text](i18n.md).

## Code

- **No inline magic values.** Ports, routes, limits and timeouts are named constants in the module that uses them.
- **No `unwrap()` outside tests.**
- core's errors are a `Msg` wrapped in a `Failure`, with the cause in `source()`.
- Tests live next to the code: `#[cfg(test)] mod tests` in Rust, and `*.test.ts` or `*.test.js` beside the file.

## Interface

- **Page content is always shown through `ContentView` and its allowlist** of tags and attributes (`features/library/reader.ts`). The app never uses `{@html}`.
- Outside `lib/ui` there are no `<button>`, `<input>`, `<select>` or `<textarea>` elements, and no colors, fonts or sizes. Screens carry layout styles only.
- `invoke` is used only in `lib/api.ts`.
- `window.alert` and `window.confirm` are not used: questions go through the `InlineConfirm` component.
- Read [`DESIGN.md`](../../../DESIGN.md) before touching the interface.

## Security

- The hidden `extractor` and `parser` windows have no Tauri permissions and must never get any.
- A new command needs its `allow-<command>` permission in `capabilities/default.json`, for the `main` window only.
- The CSP allows no remote origins. Images are `self` and `data:` only.
- The extension asks for the minimum permissions: `activeTab`, `scripting`, `storage`, `nativeMessaging` and `http://127.0.0.1:47821/*`. Do not add `<all_urls>`, `tabs` or broad host permissions.
- The extension token lives only in `storage.local` and in the `Authorization` header. It is never logged and never passed to the page.

## Git

- Commits go on the `main` branch.
- `scripts/check.sh` must pass before every commit.

## The identifier `dev.landygg.marks`

These files carry it. If you add or remove one, update the list in [`AGENTS.md`](../../../AGENTS.md):

- `apps/desktop/src-tauri/tauri.conf.json` (the bundle `identifier`)
- `crates/core/src/settings.rs` (`KEYRING_SERVICE`, the keychain service)
- `apps/extension/client.js` (`NATIVE_APP_ID`)
- `apps/extension/safari/Info.plist` (`CFBundleIdentifier`, with the `.safari-extension` suffix)

The log directory, `~/Library/Logs/dev.landygg.marks/`, is derived from the bundle identifier.

## Port `47821`

The local API listens on `127.0.0.1` only. The port appears in four files, which always change together:

- `apps/desktop/src-tauri/src/api.rs` (`API_PORT`)
- `apps/extension/manifest.json` (`host_permissions`)
- `apps/extension/client.js` (`API_PORT`)
- `apps/extension/safari/SafariWebExtensionHandler.swift` (`apiBase`)

The test `extension_files_agree_with_the_app` checks that they match.
