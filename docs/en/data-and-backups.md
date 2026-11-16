# Data and backups

[Español](../es/data-and-backups.md) · **English**

Everything you save in marks lives in files on your computer. There is no copy on any server: **backups are up to you**.

## Where everything is

| What | Where |
|---|---|
| First profile, list of profiles and AI settings | `~/Library/Application Support/marks/marks.db` |
| The other profiles | `~/Library/Application Support/marks/marks-profiles/<n>.db` |
| API keys for AI providers | macOS Keychain, service `dev.landygg.marks` |
| Chosen language | The app's macOS preferences (`dev.landygg.marks`) |
| Theme, corners and window widths | Local storage of the app's window |
| Logs | `~/Library/Logs/dev.landygg.marks/` |
| Extension token | In the database (one per profile) and in the browser extension's storage |

Each database is a SQLite file. It holds the bookmarks with their content, notes, tags, groups, summaries, favicons and search vectors.

Next to each `.db` there may be two helper files, `-wal` and `-shm` (for example `marks.db-wal`). They are part of the database while marks is open: don't delete them.

`<n>` is the profile's internal number. `marks profiles` lists the numbers and names; see [Command line](cli.md).

## Making a backup

1. **Quit marks** (menu bar icon → “Quit”, or ⌘Q). Closing the window isn't enough: the app stays open.
2. Copy the whole `~/Library/Application Support/marks/` folder to another disk.

From the terminal, with marks closed:

```sh
cp -R ~/Library/Application\ Support/marks ~/Desktop/marks-backup-$(date +%F)
```

If you use Time Machine or any other whole-disk backup, that folder is already covered.

The backup does **not** include:

- API keys (they're in the Keychain). You'll have to type them again.
- Theme, corners, widths and language. Choose them again in Settings.

## Restoring a backup

1. Quit marks.
2. Move the current folder aside, just in case: rename `~/Library/Application Support/marks` to `marks-previous`.
3. Copy your backup folder to `~/Library/Application Support/marks`.
4. Open marks.

Always restore the whole folder. `marks.db` holds the list of profiles, and each profile is a file in `marks-profiles/`: mixing files from different backups can leave profiles without a file or files without a profile.

After restoring, each profile's token is the one in the backup. If you regenerated it later, paste the current one into the extension.

## Automatic copies before a migration

A new version of marks may change the database format. That's called a migration.

- marks makes a **copy of each database before migrating it**. v1.0 is the first version, so the first copy will come from the first version that changes the format.
- An older version of marks **refuses to open** a database it can't read, with a clear message, instead of damaging it.
- **There is no automatic way back.** To return to an earlier version of marks you have to restore the copy made before the migration. Anything you saved after updating won't be in it.

The copies go to a `backups` folder next to each database:

- the main database, to `~/Library/Application Support/marks/backups/marks-v<n>-<date>.db`;
- profile `<p>`, to `~/Library/Application Support/marks/marks-profiles/backups/<p>-v<n>-<date>.db`.

`<n>` is the format version before migrating, and `<date>` the UTC time of the copy (for example `20271102T091500Z`). The **3** newest of each database are kept; older ones are deleted on their own. If the copy can't be made (for example, with a full disk), marks doesn't migrate the database and says so when it opens.

To restore a copy:

1. Quit marks (marks menu → Quit).
2. Install the version of marks you used before updating.
3. Copy the backup file over `marks.db` (or over `marks-profiles/<p>.db`, for a profile), with that file's name. If there are `-wal` or `-shm` files next to it, delete them first.
4. Open marks.

## Exporting is not a full backup

“Export bookmarks…” writes a `bookmarks.html` with URLs, titles, dates and tags. It carries no notes, summaries or content. It's for taking your bookmarks to a browser, not for restoring marks. See [Import and export](import-export.md).

`marks export <DIR>` writes one Markdown file per bookmark, with its content, summary and notes. Any editor can read it, but marks doesn't import it back. See [Command line](cli.md).

## Moving marks to another Mac

1. On the old Mac: quit marks and copy `~/Library/Application Support/marks/`.
2. On the new one: install marks, **don't open it yet**, and put the folder in the same place.
3. Open marks. Type your API keys again, if you used any, and pair the extension again.

marks doesn't sync between computers. Don't put the folder in a sync service to use it from two Macs at once: two copies of marks writing to the same database can corrupt it.

## Using another location

The `MARKS_DB` environment variable changes the path of the main database, for both the app and the command line. The other profiles go in a folder next to it, named like the database without its extension plus `-profiles`: with `MARKS_DB=/tmp/test.db`, in `/tmp/test-profiles/`.

It's meant for testing and development. An app opened from the Finder or the Dock doesn't see your terminal's variables.

## Deleting all your data

With marks closed, remove:

- `~/Library/Application Support/marks/`: all your bookmarks.
- `~/Library/Logs/dev.landygg.marks/`: the logs.
- In **Keychain Access**, the entries of the `dev.landygg.marks` service: the API keys.
