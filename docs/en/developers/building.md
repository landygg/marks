# Building and checking

[Español](../../es/developers/building.md) · **English**

## Requirements

- macOS 14.5 or later. That is the minimum the bundle declares (`minimumSystemVersion` in `tauri.conf.json`).
- The Xcode command line tools. `scripts/check.sh` and `scripts/build-app.sh` use `xcrun swiftc`, `codesign` and `PlistBuddy`.
- Rust (2024 edition), with `rustfmt` and `clippy`.
- Node.js and pnpm.
- Optional, for local AI: [Ollama](https://ollama.com) with `qwen3:8b` and `nomic-embed-text`.

> The tool versions are pinned in the repository: Rust in `rust-toolchain.toml`, Node.js in `.node-version` and pnpm in the `packageManager` field of the root `package.json`.

## Setting up the repository

```sh
git clone https://github.com/landygg/marks.git
cd marks
pnpm install
```

`pnpm install` at the root installs the three workspace packages: `apps/desktop`, `apps/extension` and `packages/extract`.

## The desktop app in development

```sh
MARKS_DB=/tmp/marks-scratch.db pnpm --dir apps/desktop tauri dev
```

Always set `MARKS_DB` while developing. Without it, the app opens your real database.

- `MARKS_DB` works for both the app and the CLI. Additional profiles are stored next to the database, in a folder named after it plus `-profiles`: with the example above, `/tmp/marks-scratch-profiles/`.
- `MARKS_LOCALE=es|en|pseudo` forces the language of the app, the CLI and the tests. `pseudo` accents and lengthens every string and wraps it in `⟦ ⟧`, so you can spot text that never went through a catalog.
- Changing the language in Settings relaunches the app and ends `tauri dev`. Start it again.
- `tauri dev` does not include the Safari extension.
- If you use a separate `target` directory for tests, put it under `/private/tmp`, not `/tmp`: Tauri will not relaunch a binary whose path goes through a symlink.

## CLI

```sh
MARKS_DB=/tmp/marks-scratch.db cargo run -p marks-cli -- add https://example.com --tag demo
MARKS_DB=/tmp/marks-scratch.db cargo run -p marks-cli -- list
cargo run -p marks-cli -- --help
```

The CLI only queues. The app downloads and extracts when it opens the same database.

## Release build

```sh
pnpm --dir apps/desktop tauri build --bundles app
```

This leaves `target/release/bundle/macos/marks.app`, ad-hoc signed. Without `--bundles app` it also builds the DMG, in `target/release/bundle/dmg/`. The DMG step drives Finder through AppleScript and fails in a session with no GUI or without Automation permission.

## The app with the Safari extension

```sh
scripts/build-app.sh
```

The script:

1. Picks the signing identity.
2. Calls `scripts/build-appex.sh`, which builds the extension, compiles the Swift handler for `arm64` and `x86_64`, joins them with `lipo`, and signs the appex with the hardened runtime into `target/appex/marks Safari Extension.appex`. It takes the version from `apps/extension/manifest.json` and the minimum macOS from `tauri.conf.json`.
3. Builds `marks.app` with `tauri build --bundles app --config src-tauri/tauri.release.conf.json`, which copies the appex into `Contents/PlugIns/`. Tauri signs the app with the same identity and the hardened runtime.
4. Verifies the signature.

With `--target <triple>` it builds for that architecture (`aarch64-apple-darwin` or `x86_64-apple-darwin`) and leaves the app in `target/<triple>/release/bundle/macos/`. With `--dmg` it also builds the DMG, in `target/<triple>/release/bundle/dmg/` (with `CI=true`, without driving Finder). Any other argument (for example `--config '<json>'`) goes to `tauri build`; `--target` and `--dmg` are recognized in any position.

The signing identity comes from, in this order:

1. The `APPLE_SIGNING_IDENTITY` variable, the one releases use (`marks Code Signing`, the self-signed certificate).
2. The `CODESIGN_IDENTITY` variable.
3. The first valid "Apple Development" identity in the keychain.
4. Ad-hoc signing, with a warning.

Safari only loads the extension with “Allow unsigned extensions”, with any of them short of a Developer ID.

To check the result:

```sh
scripts/verify-bundle.sh target/release/bundle/macos/marks.app
```

It exits with 1 if the appex is missing from `Contents/PlugIns/`, lacks the sandbox or isn't universal, if the app or the appex lacks the hardened runtime, or if `codesign --verify --strict --deep` fails.

Register a single copy of `marks.app` with Safari. If the one in `target/` and an installed one coexist, Safari may load the wrong extension.

## The extension in Chromium

```sh
pnpm --dir apps/extension build
```

Then open `chrome://extensions`, turn on developer mode, click "Load unpacked" and choose `apps/extension/dist`. The app has to be running (`tauri dev` is fine). Pairing with the token is covered in [Browser extension](../browser-extension.md).

`pnpm --dir apps/extension watch` rebuilds on save.

## `scripts/check.sh`

This is the full check. It must pass before every commit. It runs, in this order:

| Step | Command |
|---|---|
| Rust formatting | `cargo fmt --all --check` |
| Rust lints | `cargo clippy --workspace --all-targets -- -D warnings` |
| Rust tests | `cargo test --workspace` |
| JS dependencies | `pnpm --dir apps/desktop install --frozen-lockfile` |
| Extension build | `pnpm --dir apps/extension build` |
| Extension tests | `node --test "apps/extension/*.test.js"` |
| `packages/extract` types | `pnpm --dir packages/extract check` |
| `packages/extract` tests | `pnpm --dir packages/extract test` |
| Frontend formatting | `pnpm --dir apps/desktop format:check` (Prettier) |
| Frontend lints | `pnpm --dir apps/desktop lint` (ESLint) |
| Frontend types | `pnpm --dir apps/desktop check` (svelte-check) |
| Frontend tests | `pnpm --dir apps/desktop test` (vitest) |
| Swift handler types | `xcrun swiftc -typecheck …`, on macOS only |

To iterate on one part:

```sh
cargo test -p marks-core
cargo test -p marks-desktop
pnpm --dir apps/desktop test
node --test "apps/extension/*.test.js"
pnpm --dir apps/desktop format      # applies the formatting
```

## Tests that need Ollama

Two integration tests are ignored by default, because they need Ollama running:

```sh
cargo test -p marks-core ollama_roundtrip -- --ignored
cargo test -p marks-core ollama_long_input_keeps_its_start -- --ignored
```

The first needs `qwen3:8b` and `nomic-embed-text`. The second takes over half a minute and checks that a 60,000-character input reaches the model whole.

## Trying the local API

With the app running on a scratch database, copy the token from Settings and run:

```sh
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:47821/v1/ping
```

The rest is in [Local API](local-api.md).

## Logs

The app writes its logs to stdout and to `~/Library/Logs/dev.landygg.marks/`. The folder name comes from the bundle identifier.
