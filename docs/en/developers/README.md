# Developer documentation

[Español](../../es/developers/README.md) · **English**

This section is for anyone who wants to build marks, understand how it works or contribute. The user documentation starts at the [main index](../README.md).

marks v1.0 ships for macOS only. Apple Silicon is the only tested platform; Intel builds are untested. Windows and Linux will come later.

## Pages

| Page | What it covers |
|---|---|
| [Architecture](architecture.md) | The parts of the repository and the life of a bookmark |
| [Building and checking](building.md) | Requirements, development, release builds and `scripts/check.sh` |
| [Repository rules](rules.md) | What every change must respect |
| [Languages and text](i18n.md) | How to add a string or a language |
| [Migrations](migrations.md) | How the database schema changes |
| [Local API](local-api.md) | The HTTP API the browser extension uses |
| [Contributing](contributing.md) | Issues, pull requests, commits and verification |
| [Releasing](release.md) | Skeleton of the release process |

## Other documents in the repository

- [`AGENTS.md`](../../../AGENTS.md): a short map of the repository and its rules. It is written in Spanish.
- [`DESIGN.md`](../../../DESIGN.md): the visual design and its tokens. Read it before touching the interface.
