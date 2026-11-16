# Migraciones

**Español** · [English](../../en/developers/migrations.md)

El esquema de la base de datos cambia con migraciones numeradas. Viven en `crates/core/src/store.rs`.

## Cómo funcionan

- `MIGRATIONS` es una lista ordenada. `MIGRATIONS[n - 1]` lleva la base de la versión `n - 1` a la `n`.
- La versión de una base es su `PRAGMA user_version`.
- Una migración es de uno de dos tipos:
  - `Migration::Sql(&str)`: un lote de SQL.
  - `Migration::Rust(fn(&Transaction) -> Result<()>)`: código Rust, para cuando hay que leer filas y reescribirlas.
- `migrate` se ejecuta cada vez que se abre un almacén (`Store::open`). Aplica las migraciones pendientes, cada una en su transacción junto con el cambio de `user_version`.
- La transacción es inmediata y relee la versión dentro. Si el CLI y la app migran a la vez, quien llega segundo ve la versión ya avanzada y no repite la migración.

## Un fichero por perfil

Cada perfil es un fichero SQLite aparte, y todos comparten el mismo esquema y las mismas migraciones.

- El primer perfil es la base principal, `marks.db`. Se migra al arrancar.
- El perfil `n` (n > 1) vive en `marks-profiles/<n>.db`. Se migra la primera vez que se abre su almacén en esa ejecución.

La tabla `profiles`, con el registro de perfiles y sus tokens, solo existe en la base principal. No es parte de `MIGRATIONS`: la crea `profiles.rs` con `CREATE TABLE IF NOT EXISTS`.

## Migraciones actuales

| N.º | Tipo | Qué hace |
|---|---|---|
| 1 | SQL | Esquema inicial de la v1.0; las migraciones de las versiones de prueba 0.x se juntaron en esta |

## Añadir una migración

1. Escribe la migración como una constante `MIGRATION_<n>` o una función `migration_<n>`, con un comentario que diga qué cambia y por qué.
2. Añádela **al final** de `MIGRATIONS`.
3. Añade un test que parta de una base en la versión anterior, con datos, y compruebe el resultado. `store.rs` tiene ejemplos con una migración 2 de prueba (`ADDITIVE_V2`), como `opens_newer_additive_database_untouched` y `backs_up_before_migrating`.
4. Si el cambio añade o quita un valor de `bookmarks.status`, revisa la [regla de los estados](rules.md#estados-de-un-bookmark).
5. Declara su `Compat` y añade un test como `migrating_to_v2_updates_min_reader`, que compruebe el `min_reader` que deja.

Reglas:

- **Solo se añade al final.** El orden es la versión.
- **Una migración publicada no se edita.** Las bases de los usuarios ya la aplicaron. Lo que haya que corregir va en una migración nueva.
- **`crates/core/fixtures/schema-initial.txt` no se toca nunca.** Es el esquema de la migración 1, y `initial_schema_matches_reference` lo compara con lo que crea `SCHEMA_1`.
- Una migración debe terminar bien sobre cualquier base que haya dejado la versión anterior, también una vacía.
- Si es lenta sobre una base grande, mídela y anótalo en el pull request.

## Copia previa y compatibilidad

### Copia previa

Antes de aplicar migraciones pendientes a una base en fichero, `Store::open` la copia con `backup_before_migrating`:

- solo si `check` la acepta y `1 <= user_version < MIGRATIONS.len()`: ni una base nueva, ni una al día, ni una rechazada, ni una en memoria se copian;
- con `VACUUM INTO`, que incluye lo confirmado en el WAL, en `<carpeta de la base>/backups/<stem>-v<desde>-<AAAAMMDDTHHMMSSZ>.db` (la hora sale de SQLite, en UTC). La copia se escribe primero con un nombre temporal y luego se renombra: si falla a medias (con el disco lleno, por ejemplo), se borra el fichero parcial, y si otro proceso copia en el mismo segundo, las dos aperturas siguen. Si ese fichero ya existe, no se copia otra vez;
- después poda la carpeta: de los ficheros `<stem>-v<n>-<fecha>.db` deja los `BACKUPS_KEPT` (3) más nuevos por fecha, y no toca los demás;
- si la copia o la poda fallan, la apertura falla con `Msg::Backup` y no se migra nada.

En la v1.0 no se ejecuta nunca de verdad: lo prueban los tests con la migración de prueba `ADDITIVE_V2`.

### Nivel mínimo de lectura

Cada base guarda en la tabla `meta`, clave `min_reader`, la versión más antigua del esquema que tiene que conocer una app para leerla. La crea la migración 1 y la actualiza cada migración, en la misma transacción que `user_version`.

- Cada entrada de `MIGRATIONS` declara su compatibilidad: `Compat::Additive` o `Compat::Breaking`.
- `min_reader` es la versión de la última migración `Breaking` aplicada. La migración 1 es `Breaking`, así que una base de la v1.0 tiene `min_reader = 1`.
- Una migración es `Breaking` si quita o renombra columnas o tablas, cambia el significado de un valor, o hace que una app anterior escriba datos incorrectos. Si solo añade (una columna que se puede ignorar, una tabla nueva), es `Additive`.
- Al abrir una base, `check` decide antes de escribir nada:
  - una base vacía (`user_version = 0` y nada en `sqlite_master`) se crea con la migración 1;
  - una base sin `min_reader` se rechaza como desconocida (`Msg::DatabaseUnknown`): es de una build 0.x o no es de marks. No se toca;
  - si `min_reader` es mayor que el número de migraciones que conoce la app, se rechaza con `Msg::DatabaseTooNew`;
  - si no, se abre. Una base más nueva que la app, pero con `min_reader` conocido, se abre sin migrar y sin escribir nada.

No hay migraciones hacia atrás. Volver a una versión anterior de la app con una base ya migrada no está soportado; el camino previsto es restaurar la copia previa a la migración. La guía para usuarios está en [Datos y copias de seguridad](../data-and-backups.md).
