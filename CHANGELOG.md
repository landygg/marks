# Changelog

All notable changes to marks are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and marks uses [Semantic Versioning](https://semver.org/). The section of each version is also its release notes.

## [Unreleased]

## [1.2.0] - 2026-11-26

**Requires macOS 15 or later. On macOS 14, don't install this update: marks would no longer open. If you already did, reinstall marks 1.1.0 from the releases page; your bookmarks are untouched.**

### Fixed

- With an input method (Japanese, Chinese, Korean), the Enter, Esc and arrow keys that pick a candidate no longer run the palette, add half-typed tags, rename a group or leave the title.
- A middle click, the “Open Link” menu or a link dropped on the window no longer loads a web page in place of marks.
- Wide code blocks and tables in the reader can be reached and scrolled with the keyboard.
- Tab in the command palette no longer walks through every result, and “No results” and search errors are read out by screen readers.
- A disabled “Extract with AI” tells keyboard and screen reader users why.
- Closing a focused notification with the keyboard moves the focus to the next one, or to the bell, instead of losing it.
- When adding a URL fails, the URL field is marked with the error and gets the focus.
- Copying to the clipboard says so when it fails.
- Tag, URL, group name, initials and endpoint fields are no longer autocorrected or capitalized.
- On the light theme, the selected row, links, fields and “Warning” notices have enough contrast.
- A pinned theme no longer flashes the system one when marks starts.
- On the extension's options page, every result is read out, also the same one twice, and Enter saves the token.

### Changed

- marks needs macOS 15 or later. From this version on, it won't offer an update that needs a newer macOS than the one your Mac has.
- The reader keeps lines to about 70 characters.
- The command palette closes with a click outside it.
- The Chromium extension no longer asks for native messaging, which only Safari uses, and needs Chrome 123 or later.
- From source, the unpacked extension now loads from `apps/extension/dist/chromium`; reload it from there if you loaded `apps/extension/dist`.

## [1.1.0] - 2026-11-24

### Added

- A “Failed” filter in the sidebar shows every bookmark that failed in the active profile, and clicking “N failed” in the status bar opens it.
- “Retry all” in that filter puts every failed bookmark back in the queue at once, after asking. It is also in the command palette.
- The status bar shows whether AI is off, on (with its model) or failing, and whether the local API for the extension is listening or its port is in use. Each one opens its section of Settings.

### Fixed

- A page nested too deeply for the reader now shows as plain text instead of failing to display.

### Changed

- Dependencies are up to date, including memory-safety fixes in the SQLite bindings and fixes in the app framework.

## [1.0.1] - 2026-11-17

### Fixed

- If the database file isn't an SQLite database or is damaged, marks now says so in plain words when it starts, leaves the file untouched and tells you where its backups are, instead of showing SQLite's raw error.
- marks only builds the path of a profile's file from a profile that exists in its registry, so no request can make it create or remove a file for a profile that doesn't exist.

### Changed

- Dependencies and build tools are up to date.

## [1.0.0] - 2026-11-16

The first public release, for macOS 14.5 or later. Apple silicon is the only tested platform; the Intel build is untested.

marks is local-first: your library is a set of SQLite files on your Mac, with no account, no marks server, no sync and no telemetry. The user guide is in [`docs/en`](https://github.com/landygg/marks/tree/v1.0.0/docs/en/README.md) (and in Spanish in [`docs/es`](https://github.com/landygg/marks/tree/v1.0.0/docs/es/README.md)).

### Added

- Save a URL and marks downloads the page, turns it into Markdown and keeps it in a SQLite file on your Mac, with full-text search.
- Reading view that loads nothing remote: images and embeds are links you open yourself.
- Optional AI, off by default: a summary, a category and tags, and semantic search. It runs on [Ollama](https://ollama.com) on your Mac, or on OpenAI, Anthropic and OpenAI-compatible servers. “Extract with AI” rescues pages the regular extractor gets wrong.
- Profiles, each in its own SQLite file, and tag groups.
- Browser extension that saves the page as you see it: for Chromium browsers (recommended, `marks-chromium-<version>.zip` in each release) and for Safari (experimental, inside marks.app). The app and the extension check that their versions are compatible.
- Import and export of the `bookmarks.html` file Chrome and Safari use.
- Command palette (⌘K), notifications, a status bar and resizable panels.
- English and Spanish, and light and dark themes.
- Automatic updates from GitHub Releases on Apple silicon, which you can turn off in Settings → Updates.
- Database safety: marks refuses a database from a newer version or one it didn't create, and backs up each database before migrating it.
- The `marks` command-line tool (built from source) to queue URLs, search, and export.

### Known limitations

- marks is not notarized by Apple: the first launch needs System Settings → Privacy & Security → “Open Anyway”.
- Safari only loads the extension with “Allow unsigned extensions”, which it turns off every time it quits.
- After each update, marks asks once for your login password to read the AI provider's API key from the keychain.

## [1.0.0-rc.2] - 2026-11-16

Second release candidate of 1.0.0, published as a prerelease so that 1.0.0-rc.1 can update to it. It has everything listed under 1.0.0.

## [1.0.0-rc.1] - 2026-11-16

Release candidate of 1.0.0, published as a prerelease to test automatic updates. It has everything listed under 1.0.0.

[Unreleased]: https://github.com/landygg/marks/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/landygg/marks/releases/tag/v1.2.0
[1.1.0]: https://github.com/landygg/marks/releases/tag/v1.1.0
[1.0.1]: https://github.com/landygg/marks/releases/tag/v1.0.1
[1.0.0]: https://github.com/landygg/marks/releases/tag/v1.0.0
[1.0.0-rc.1]: https://github.com/landygg/marks/releases/tag/v1.0.0-rc.1
[1.0.0-rc.2]: https://github.com/landygg/marks/releases/tag/v1.0.0-rc.2
