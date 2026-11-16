# Command line

[Español](../es/cli.md) · **English**

`marks` is a terminal binary that works on the same database as the app. Use it to add, list, search, import and export from scripts.

The `marks` binary isn't distributed: you build it from source with `cargo build --release -p marks-cli`, which leaves `target/release/marks`. See [Building](developers/building.md).

## What it does and doesn't do

- **It only queues.** `marks add` stores the URL and nothing more: it doesn't download the page or call any model. The app extracts it the next time it's open.
- **It only does full-text search.** Semantic search is in the app.
- **It doesn't need the app to be open**, and it can be used while the app is open.

## Usage

```
marks [OPTIONS] <COMMAND>
```

| Command | What it does |
|---|---|
| `add <URL>` | Queues the URL |
| `list` | Lists all bookmarks |
| `search [QUERY]...` | Full-text search (up to 50 results) |
| `export <DIR>` | Writes one `.md` per bookmark |
| `import <FILE>` | Imports a `bookmarks.html` |
| `export-html <FILE>` | Exports the profile to a `bookmarks.html` |
| `stats` | Counts bookmarks by status |
| `profiles` | Lists the profiles |

| Option | What it does |
|---|---|
| `--profile <NAME>` | Profile to work on. Without it, **the first one**, not the app's active profile |
| `--db <PATH>` | Path to the database. Also settable with the `MARKS_DB` variable |
| `-h`, `--help` | Help, overall or for one command (`marks add --help`) |
| `-V`, `--version` | Version |

There is no `help` subcommand: help is `--help`.

## Examples

```sh
# Add, with tags and a note
marks add https://example.com --tag reading --tag rust --notes "for the weekend"

# With a title of your own
marks add https://example.com/post --title "My title"

# List and search
marks list
marks search rust ownership

# In another profile
marks profiles
marks --profile Work list

# Count by status
marks stats
```

Options for `add`: `--title <TITLE>` (otherwise the app uses the page's), `--notes <NOTES>` and `--tag <TAG>`, which can be repeated.

### Output

`list`, `search` and `add` print one line per bookmark:

```
#12  [ready]  Page title  https://example.com/post
```

A failed bookmark adds `(error: …)` at the end. The statuses are `pending`, `waiting to be indexed`, `ready`, `failed` and `not extracted`.

`profiles` prints `#id  [initials]  name`.

`stats` prints five lines: `total`, `pending`, `failed`, `unsummarized` and `not extracted`.

## Exporting to Markdown

```sh
marks export ./my-bookmarks
```

It creates the directory if it doesn't exist and writes one `<id>.md` file per bookmark in the profile. Each file starts with a metadata block (title, URL, category, tags, author, site, publication date, language, description and creation date), followed by the notes, the summary and the content.

This is how you take the **content** with you in a readable format. marks doesn't import these files back.

## Importing and exporting `bookmarks.html`

```sh
marks import bookmarks.html
marks export-html marks.html
```

- `import` imports **without asking** (the app shows a summary first) and prints the result: `12 new, 3 already present, 1 not importable`. What you import stays “not extracted” until you ask for it in the app.
- `export-html` writes the whole profile.

The rules for folders, tags and groups are the same as in the app. See [Import and export](import-export.md).

## Language

Help, errors and output come in Spanish or English. The language is taken, in this order, from the variables `MARKS_LOCALE` (`es` or `en`), `LC_ALL`, `LC_MESSAGES` and `LANG`, and then from the system language. If none of them is Spanish or English, English is used.

```sh
MARKS_LOCALE=en marks --help
```

In a script, set `MARKS_LOCALE` if you're going to read the output.

## A scratch database

To try things without touching your data:

```sh
MARKS_DB=/tmp/marks-scratch.db marks add https://example.com
MARKS_DB=/tmp/marks-scratch.db marks list
```

## Exit code

`0` when everything went well; non-zero on an error, which is written to standard error.
