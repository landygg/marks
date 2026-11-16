# Changelog

All notable changes to marks are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and marks uses [Semantic Versioning](https://semver.org/). The section of each version is also its release notes.

## [Unreleased]

## [1.0.0] - 2026-11-16

The first public release, for macOS 14.5 or later. Apple silicon is the only tested platform; the Intel build is untested.

### Added

- Save a URL and marks downloads the page, turns it into Markdown and keeps it in a SQLite file on your Mac, with full-text search.
- Reading view that loads nothing remote: images and embeds are links you open yourself.
- Optional AI, off by default: a summary, a category and tags, and semantic search. It runs on [Ollama](https://ollama.com) on your Mac, or on OpenAI, Anthropic and OpenAI-compatible servers. “Extract with AI” rescues pages the regular extractor gets wrong.
- Profiles, each in its own SQLite file, and tag groups.
- Browser extension that saves the page as you see it: for Chromium browsers (recommended, `marks-chromium-<version>.zip` in each release) and for Safari (experimental, inside marks.app). The app and the extension check that their versions are compatible.
- Import and export of the `bookmarks.html` file Chrome and Safari use.
- Command palette (⌘K), notifications, a status bar and resizable panels.
- English and Spanish.
- Automatic updates from GitHub Releases on Apple silicon, which you can turn off in Settings → Updates.
- Database safety: marks refuses a database from a newer version or one it didn't create, and backs up each database before migrating it.
- The `marks` command-line tool (built from source) to queue URLs, search, and export.

### Known limitations

- marks is not notarized by Apple: the first launch needs System Settings → Privacy & Security → “Open Anyway”.
- Safari only loads the extension with “Allow unsigned extensions”, which it turns off every time it quits.

## [1.0.0-rc.1] - 2026-11-16

Release candidate of 1.0.0, published as a prerelease to test automatic updates. It has everything listed under 1.0.0.

[Unreleased]: https://github.com/landygg/marks/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/landygg/marks/releases/tag/v1.0.0
[1.0.0-rc.1]: https://github.com/landygg/marks/releases/tag/v1.0.0-rc.1
