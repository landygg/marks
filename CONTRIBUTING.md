# Contributing to marks

[Español](docs/es/developers/contributing.md) · **English**

Thanks for wanting to improve marks.

## Before you start

- Read [Architecture](docs/en/developers/architecture.md) and [Repository rules](docs/en/developers/rules.md).
- Set up your environment with [Building and checking](docs/en/developers/building.md).
- Always develop against a scratch database (`MARKS_DB`), never your real one.

v1.0 is macOS only, and Apple Silicon is the only tested platform. Changes for Windows or Linux are not being merged yet.

## Issues

Issues go to [GitHub Issues](https://github.com/landygg/marks/issues), with a template for bugs and one for features.

When reporting a bug, include:

- The marks version, the macOS version, and whether the Mac is Apple Silicon or Intel.
- Steps to reproduce, what you expected and what happened.
- The text of the notification or error, if there is one. A bookmark's error can be copied from its detail.
- If the extension is involved, the browser and its version.

Review any log, database or `bookmarks.html` before attaching it: they contain your URLs and the content of your pages. Never paste the extension token or an API key.

For a security problem, do not open a public issue: report it privately through [GitHub's security advisories](https://github.com/landygg/marks/security/advisories/new). See [`SECURITY.md`](SECURITY.md).

To propose a feature, open an issue first and explain the problem it solves. A large change with no prior agreement may not be merged.

## Pull requests

1. Branch off `main`.
2. One change per pull request. A fix and a refactor do not travel together.
3. Add or update tests. When fixing a bug, add a test that fails without the fix.
4. Run `scripts/check.sh` and make sure all of it passes.
5. In the pull request, say what changes, why, and how you tested it. Say what you could not test, too.

What review looks for:

- Logic is in `crates/core`, not in a Tauri command, the CLI or the extension.
- All user-facing text is in the catalogs, in Spanish and English. See [Languages and text](docs/en/developers/i18n.md).
- No `unwrap()` outside tests and no inline magic values.
- A schema change is a new migration, appended to the list. See [Migrations](docs/en/developers/migrations.md).
- An interface change follows [`DESIGN.md`](DESIGN.md).
- A new permission, in the app or the extension, is explained and is the narrowest one that works.
- The documentation in `docs/es` and `docs/en` reflects the change, in both languages.

## Commit style

- [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): summary`, in English, in the imperative and with no trailing period. For example: `feat(desktop): add a Corners setting to Appearance`.
- Types: `feat`, `fix`, `docs`, `build`, `ci`, `chore`, `refactor`, `test` and `perf`.
- Scopes: `core`, `desktop`, `extension`, `cli`, `i18n` and `api`. Leave the scope out when the change spans several.
- The message describes the effect, not the files touched.
- Do not mention AI tools or assistants in commits, pull requests or issues.

## Verification

`scripts/check.sh` is the full check: formatting, lints and tests for Rust, the frontend, the extension and `packages/extract`, plus a type check of the Swift handler. Its steps are listed in [Building and checking](docs/en/developers/building.md#scriptschecksh).

Whatever the tests do not cover is tested by hand with `tauri dev` and a scratch database. If your change touches the Safari extension, test it with the app built by `scripts/build-app.sh`: `tauri dev` does not include it.

## License

marks is released under the MIT license, and contributions come in under the same license, with no DCO or CLA to sign.
