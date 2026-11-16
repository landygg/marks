# Migrations

[Español](../../es/developers/migrations.md) · **English**

The database schema changes through numbered migrations. They live in `crates/core/src/store.rs`.

## How they work

- `MIGRATIONS` is an ordered list. `MIGRATIONS[n - 1]` takes the database from version `n - 1` to `n`.
- A database's version is its `PRAGMA user_version`.
- A migration is one of two kinds:
  - `Migration::Sql(&str)`: a batch of SQL.
  - `Migration::Rust(fn(&Transaction) -> Result<()>)`: Rust code, for when rows have to be read and rewritten.
- `migrate` runs every time a store is opened (`Store::open`). It applies the pending migrations, each in its own transaction together with the `user_version` change.
- The transaction is immediate and rereads the version inside it. If the CLI and the app migrate at the same time, the one that arrives second sees the version already advanced and does not repeat the migration.

## One file per profile

Each profile is a separate SQLite file, and they all share the same schema and the same migrations.

- The first profile is the main database, `marks.db`. It is migrated at startup.
- Profile `n` (n > 1) lives in `marks-profiles/<n>.db`. It is migrated the first time its store is opened in that run.

The `profiles` table, with the profile registry and the tokens, exists only in the main database. It is not part of `MIGRATIONS`: `profiles.rs` creates it with `CREATE TABLE IF NOT EXISTS`.

## Current migrations

| No. | Kind | What it does |
|---|---|---|
| 1 | SQL | The v1.0 initial schema; the migrations of the 0.x test versions were merged into this one |

## Adding a migration

1. Write the migration as a `MIGRATION_<n>` constant or a `migration_<n>` function, with a comment saying what changes and why.
2. Append it **at the end** of `MIGRATIONS`.
3. Add a test that starts from a database at the previous version, with data in it, and checks the result. `store.rs` has examples with a test migration 2 (`ADDITIVE_V2`), such as `opens_newer_additive_database_untouched` and `backs_up_before_migrating`.
4. If the change adds or removes a value of `bookmarks.status`, review the [rule about states](rules.md#bookmark-states).
5. Declare its `Compat` and add a test like `migrating_to_v2_updates_min_reader` that checks the `min_reader` it leaves.

Rules:

- **Append only.** The order is the version.
- **A shipped migration is never edited.** Users' databases have already applied it. Any fix goes in a new migration.
- **`crates/core/fixtures/schema-initial.txt` is never touched.** It is the schema of migration 1, and `initial_schema_matches_reference` compares it with what `SCHEMA_1` creates.
- A migration must succeed on any database the previous version could have left behind, including an empty one.
- If it is slow on a large database, measure it and say so in the pull request.

## Backup and compatibility

### Pre-migration copy

Before applying pending migrations to a database file, `Store::open` copies it with `backup_before_migrating`:

- only if `check` accepts it and `1 <= user_version < MIGRATIONS.len()`: a new database, an up-to-date one, a refused one and an in-memory one are never copied;
- with `VACUUM INTO`, which includes what is committed in the WAL, to `<database folder>/backups/<stem>-v<from>-<YYYYMMDDTHHMMSSZ>.db` (the time comes from SQLite, in UTC). The copy is first written under a temporary name and then renamed: if it fails half-way (a full disk, for example), the partial file is deleted, and if another process copies in the same second, both openings go on. If that file already exists, it is not copied again;
- then it prunes the folder: of the `<stem>-v<n>-<date>.db` files it keeps the `BACKUPS_KEPT` (3) newest by date, and leaves the others alone;
- if the copy or the pruning fails, opening fails with `Msg::Backup` and nothing is migrated.

In v1.0 it never really runs: the tests exercise it with the test migration `ADDITIVE_V2`.

### Minimum reader level

Each database stores in the `meta` table, key `min_reader`, the oldest schema version an app must know to read it. Migration 1 creates it and every migration updates it, in the same transaction as `user_version`.

- Each entry of `MIGRATIONS` declares its compatibility: `Compat::Additive` or `Compat::Breaking`.
- `min_reader` is the version of the last `Breaking` migration applied. Migration 1 is `Breaking`, so a v1.0 database has `min_reader = 1`.
- A migration is `Breaking` if it drops or renames columns or tables, changes what a value means, or would make an older app write wrong data. If it only adds (a column that can be ignored, a new table), it is `Additive`.
- When a database opens, `check` decides before writing anything:
  - an empty database (`user_version = 0` and nothing in `sqlite_master`) is created by migration 1;
  - a database without `min_reader` is refused as unknown (`Msg::DatabaseUnknown`): it comes from a 0.x build or is not from marks. It is left untouched;
  - if `min_reader` is greater than the number of migrations the app knows, it is refused with `Msg::DatabaseTooNew`;
  - otherwise it opens. A database newer than the app, but with a known `min_reader`, opens without migrating and without writing anything.

There are no down-migrations. Going back to an older version of the app with an already migrated database is not supported; the intended path is to restore the pre-migration copy. The user guide is in [Data and backups](../data-and-backups.md).
