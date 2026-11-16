# Releasing

[Español](../../es/developers/release.md) · **English**

## 1. Version

The app's version lives in three files, which say the same:

- `Cargo.toml` (`[workspace.package]`, inherited by every crate)
- `apps/desktop/package.json`
- `packages/extract/package.json`

`Cargo.lock` updates itself on build. `tauri.conf.json` has no version: Tauri takes it from Cargo. `scripts/check-version.sh`, which runs at the start of `scripts/check.sh`, checks that both `package.json` files repeat the workspace version; given a tag (`scripts/check-version.sh v1.0.0`), it also checks that the tag names it and that `tauri.conf.json` no longer has the provisional updater public key.

The extension has its own version in `apps/extension/manifest.json`, separate from the app's, and `scripts/build-appex.sh` reads it from there for the Safari appex. It is bumped when anything in `apps/extension` or `packages/extract` changes, because the appex and the Chromium zip bundle the extractor (Defuddle); if it doesn't go up, Safari keeps the old extension. `scripts/check-version.sh` does not compare it.

## 2. Verification

```sh
scripts/check.sh
```

Then the manual tests for the release.

Manual tests before publishing. The updater can only be tested with two published versions, because `releases/latest` skips drafts and prereleases, and a draft's files can't be downloaded without signing in. Use two prereleases (`1.0.0-rc.1` and `1.0.0-rc.2`):

1. For each one, version `1.0.0-rc.N` in the three version files and a `## [1.0.0-rc.N]` section in `CHANGELOG.md` (without it the release fails). Tag `v1.0.0-rc.N`.
2. Publish the draft as a prerelease (don't leave it as a draft).
3. Install rc.1 and open it with `MARKS_UPDATE_ENDPOINT=https://github.com/landygg/marks/releases/download/v1.0.0-rc.2/latest-darwin-aarch64.json` and a scratch database in `MARKS_DB`.

With those two versions, check:

- A database created by the previous version, with data, opens with the new one. If the new one brings migrations, a copy lands in `backups/`; if not, none. `user_version` and `meta.min_reader` end up as [Migrations](migrations.md) says (in v1.0, `1` and `"1"`).
- Save a page from Chromium (the recommended one) and from Safari with “Allow unsigned extensions” (experimental).
- First launch from the DMG downloaded with a browser, through “Open Anyway” ([Installation](../installation.md)).
- Import and export a `bookmarks.html`.
- AI with Ollama: “Test connection” and a summary.
- Update from the previous version with the updater: the new app opens without going through “Open Anyway” again, and `xattr -l /Applications/marks.app` shows no `com.apple.quarantine`.

## 3. Build

```sh
scripts/build-app.sh
```

This leaves `target/release/bundle/macos/marks.app`, with the Safari extension inside. See [Building and checking](building.md#the-app-with-the-safari-extension).

With `--target aarch64-apple-darwin` or `--target x86_64-apple-darwin`, the app lands in `target/<triple>/release/bundle/macos/`. The Intel one is built but not tested on an Intel Mac.

`scripts/build-appex.sh` builds the Safari appex first: universal (`arm64` and `x86_64`, joined with `lipo`), with the version from `apps/extension/manifest.json`, and signed with the hardened runtime and its entitlements, without `--timestamp`. `tauri.release.conf.json` puts it into the bundle through `bundle.macOS.files`, so the updater's `.app.tar.gz` carries it. Tauri then signs the app with the same identity.

Checked with Tauri 2.12 and the self-signed certificate: Tauri does **not** re-sign the appex (the CDHash inside `marks.app` is the one in `target/appex`, and the entitlements stay) and does not add `--timestamp` either. With the hardened runtime the app extracts (the `extractor` and `parser` windows), uses Ollama, saves the API key to the keychain and reads it back after a restart, and answers on the local API, with no exception entitlements.

`scripts/verify-bundle.sh <marks.app>` checks the result: the appex is in `Contents/PlugIns/`, sandboxed and universal, and the app and the appex carry the hardened runtime and a valid signature (`codesign --verify --strict --deep`).

With `--dmg`, `scripts/build-app.sh` also builds the DMG: `target/<triple>/release/bundle/dmg/marks_<version>_aarch64.dmg` (or `_x64.dmg`). With `CI=true`, Tauri doesn't drive Finder through AppleScript, so the DMG builds without a GUI session, as in CI.

## 4. Signing, without notarization

The release signs the app, the appex and the DMG with `marks Code Signing`, a self-signed code-signing certificate (`APPLE_SIGNING_IDENTITY="marks Code Signing" scripts/build-app.sh --target … --dmg`). The app and the appex carry the hardened runtime, and only the appex has entitlements (sandbox and network). With no `APPLE_API_*` or `APPLE_ID` in the environment, Tauri doesn't notarize.

Without a Developer ID or notarization, Gatekeeper rejects the app, as expected:

```sh
spctl -a -vv target/aarch64-apple-darwin/release/bundle/macos/marks.app
# …: rejected
# origin=marks Code Signing
```

Users open it the first time with “Open Anyway” ([Installation](../installation.md)).

With the self-signed certificate, macOS does not treat a new build as the same app for the keychain: after an update, the app asks once for the keychain password to read the AI provider's key (checked with two builds signed with `marks Code Signing`).

## 5. Automatic updates

The app updates with `tauri-plugin-updater` from GitHub Releases:

- **Endpoint:** `https://github.com/landygg/marks/releases/latest/download/latest-{{target}}-{{arch}}.json`. `releases/latest` skips drafts and prereleases.
- **Channel:** one, the stable one, and only Apple silicon: each release publishes `latest-darwin-aarch64.json`. The Intel build doesn't update itself.
- **Format:** Tauri v2's static manifest, which `scripts/updater-manifest.mjs` writes with the notes from the `CHANGELOG.md` section, and the `marks_<version>_aarch64.app.tar.gz` package with its `.sig` signature.
- **Keys:** the public one is in `tauri.conf.json` (`plugins.updater.pubkey`). The owner keeps the private one and its password in a password manager and in GitHub's `release` environment; they never enter the repo.
- `MARKS_UPDATE_ENDPOINT` replaces the endpoint, for testing only; the signature is still required.

## 6. Safari extension

In v1.0 it ships inside `marks.app`.

The app declares `API_LEVEL` and returns it from `ping` (`api`); the extension declares `MIN_API`. If the app answers a lower `api`, or none, the extension shows `appTooOld`; if a route answers `404`, `extensionTooOld`. On each release: if the API gained something the extension uses, raise `API_LEVEL` and, in the extension that uses it, `MIN_API`. See [Local API](local-api.md).

Later, if the Apple Developer Program is paid for (the Mac App Store requires it), a separate app on the Mac App Store that contains the extension.

## 7. Chromium extension

Each release attaches `marks-chromium-<version>.zip`, with the contents of `apps/extension/dist` and the version from `apps/extension/manifest.json`. It is loaded unpacked. `release.yml` builds it with:

```sh
cd apps/extension/dist && zip -qr "$OUT/marks-chromium-$EXT_VERSION.zip" .
```

## 8. Migrations and data

From v1.1 on, every release that brings a migration checks, with a copy of a database from the previous release that has data in it, that the pre-migration copy lands in `backups/` and that `meta.min_reader` is right; and that the previous release opens or refuses the migrated database according to its `Compat`. See [Migrations](migrations.md).

## 9. Release notes

`CHANGELOG.md`, in English and in the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. New entries go under `## [Unreleased]`; when a version is prepared, that section becomes `## [<version>] - <date>`. That section is both the GitHub release notes and the `notes` of `latest-darwin-aarch64.json` (`scripts/updater-manifest.mjs`), and without it the release fails.

## 10. Publishing

`.github/workflows/release.yml` makes the release when a `v*` tag is pushed:

1. With the version bumped in the files and `## [<version>]` in `CHANGELOG.md`, create and push the tag: `git tag v1.0.0 && git push origin v1.0.0`.
2. The `build` job, in the `release` environment, checks the tag with `scripts/check-version.sh`, imports the certificate into a temporary keychain and, for `aarch64` and `x86_64`, runs `scripts/build-app.sh --target … --dmg` with the updater files and `scripts/verify-bundle.sh` on the app and on the one inside the `.app.tar.gz`. On `aarch64` it also prepares `latest-darwin-aarch64.json` and the Chromium zip. At the end it deletes the temporary keychain.
3. The `publish` job creates a **draft** release with every file. If the tag has a `-` (`v1.0.0-rc.1`), it marks it as a prerelease.
4. Review the draft: the files, the notes, and an install from the DMG.
5. Publish it by hand. From then on, `releases/latest` serves the new `latest-darwin-aarch64.json` and older installs update.

The secrets live in GitHub's `release` environment, not in the repository's secrets, and that environment only accepts `v*` tags:

| Secret | What it is |
|---|---|
| `APPLE_CERTIFICATE` | the signing certificate's `.p12`, in base64 (`base64 -i cert.p12 \| pbcopy`) |
| `APPLE_CERTIFICATE_PASSWORD` | the `.p12` password |
| `APPLE_SIGNING_IDENTITY` | `marks Code Signing` |
| `TAURI_SIGNING_PRIVATE_KEY` | the contents of the updater's private key |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | its password |

The temporary keychain's password isn't a secret: the job generates it with `openssl rand -base64 24`.

## Later

Windows and Linux are not part of v1.0. When they arrive, this page will get their steps.
