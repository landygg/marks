# marks

Bookmarks local-first: SQLite + FTS5 + sqlite-vec, extracción a Markdown y resumen/embeddings con IA local. Workspace Cargo: `crates/core` (`marks-core`, lib `marks_core`), `crates/cli` (binario `marks`), `apps/desktop/src-tauri` (`marks-desktop`, Tauri 2.12 + Svelte 5 + Vite).

Este fichero lleva lo que vale para todo el repo. Lo propio de cada app vive en su skill, que hay que leer antes de tocarla:

- `apps/desktop` (`src` y `src-tauri`): skill `marks-desktop`. Incluye el mapa de ficheros, comandos Tauri, hilos, ventanas y capabilities, CSP, Svelte 5, `lib/ui`, `DESIGN.md` y la verificación.
- `apps/extension`: skill `marks-extension`. Incluye el mapa, transportes, permisos, token, manejador Swift y aspecto.

Las skills están en `.agents/skills`. `.claude/skills` es un enlace a esa carpeta, porque Claude Code no lee `.agents`.

## Estructura

- `crates/core/src`:
  - `store`: SQLite, migraciones (`Migration::Sql` o `Rust`; la historia empieza en la v1.0 con una sola, `SCHEMA_1`, fijada por `crates/core/fixtures/schema-initial.txt`) y FTS5. Antes de migrar, `check` decide sin escribir: la tabla `meta` guarda `min_reader`, y una base que necesita una app más nueva da `Msg::DatabaseTooNew` y una que no es de marks, `Msg::DatabaseUnknown`. Antes de cada migración, `VACUUM INTO` deja una copia en `<carpeta de la base>/backups/` (se conservan 3 por base); si la copia falla, no se migra (`Msg::Backup`). El error de un bookmark va en `bookmarks.error_msg`. `url` es la normalizada y la que deduplica; `original_url` es la URL tal como llegó, y la usan el export y "Abrir enlace". La cola toma primero lo último encolado (`updated_at DESC`, `pending` antes que `pending_embed`). `retry_failed` pasa todos los `failed` del almacén (bloqueados incluidos) a `pending` en un solo `UPDATE` y borra su error.
  - `profiles`: `Library`, con el registro de perfiles en la base principal, un `Store` por perfil, el perfil activo, el token por perfil, y eliminar y mover bookmarks.
  - `msg`: `Msg` (códigos con argumentos tipados y código serializado estable), `Failure` (un `Msg` con su causa en `source()`), `ErrorPart` y `BookmarkError` (lo que se guarda en `bookmarks.error_msg`, JSON). core no tiene texto de usuario ni idioma: traduce `marks-i18n`.
  - `groups`: grupos de etiquetas (`TagGroup` y sus operaciones sobre `Store`).
  - `notifications`: registro de notificaciones en memoria: agrupa por perfil, gravedad y texto, tope de 200, y la notificación de un bookmark fallido (`bookmark_failure`). Sin E/S ni reloj: quien llama pasa la hora.
  - `chunk`: trozos de ~1.500 caracteres, máx. 40.
  - `ai`: trait `Model` (resumen, embeddings y `extract_markdown`), prompts, parser y `FakeModel`.
  - `ai_health`: si el modelo de resumen funciona. `AiStatus` (`Off`, `Ready { model }` o `Failed`, que no se construyó o cuya última llamada falló, con el error en `ErrorPart`), `AiHealth` (el estado con una generación por modelo construido: una llamada de un modelo anterior no cuenta; el aviso `on_change` corre sin el cerrojo y solo si el estado cambia) y `Watched`, un `Model` que anota el resultado de cada `summarize`, `embed` y `extract_markdown`. Es de la sesión, sin reloj ni contador.
  - `genai_model`: `Model` sobre genai, salvo el chat con Ollama, que va a su `/api/chat` con `ureq` para pedir `num_ctx` por rol y `truncate: false` (ADR 0013).
  - `settings`: ajustes de IA por rol (resumen y búsqueda en `ai`; extracción en `ai_extraction`, que sin nada guardado usa los del resumen) y llavero, con una entrada por rol en el mismo servicio (`api_key` y `extraction_api_key`). El token de la API local vive en `profiles`. `UpdateSettings` (global, clave `updates`) guarda si se buscan actualizaciones solas.
  - `vectors`: sqlite-vec.
  - `search`: búsqueda híbrida, RRF `k = 60`.
  - `pipeline`: `Fetcher`, `process_next`, la reextracción manual (`reextract` con Defuddle y `extract_with_ai` con el modelo de extracción, que solo sustituyen el contenido si no es escaso ni una página de verificación antibots y rechazan los bookmarks bloqueados; si una reextracción cambia el contenido mientras el worker resume o indexa, el worker no guarda lo viejo y lo vuelve a procesar) y `favicon_step`, que guarda en la base el favicon de cada bookmark nuevo o vuelto a extraer (los fallos pasajeros se reintentan con espera creciente hasta `FAVICON_MAX_ATTEMPTS`, los permanentes no). El detalle lo muestra como `data:` URL, sin pedir nada a la red; la `og:image` solo se guarda como URL.
  - `extract`: tipos `Page` y `PageMeta` (con topes), `is_thin` y los `Fetcher`. No convierte HTML: el Markdown lo saca Defuddle (ADR 0010). `FallbackFetcher` prueba un segundo `Fetcher` solo si el primero falla o devuelve contenido escaso.
  - `download`: descarga HTTP con destinos validados, tope de tamaño y charset declarado. `download_favicon` baja el favicon solo de destinos públicos (sin redes privadas ni de enlace local, también tras resolver el nombre), con tope y tipo sacado de los bytes (SVG solo si es XML bien formado con raíz `<svg>`, sin DOCTYPE, scripts ni `on…`); `is_transient` distingue los fallos que se reintentan.
  - `blocked`: detecta páginas de verificación antibots.
  - `capture`: decide qué se guarda en una captura del navegador.
  - `netscape`: lee y escribe el formato `bookmarks.html` (ADR 0014), con un lector propio y tolerante. Títulos y carpetas a una línea y con el tope `extract::MAX_NAME_CHARS`; anidamiento hasta `MAX_DEPTH`.
  - `transfer`: importar (`read_file`, con tope de 50 MB y UTF-8; `preview`, sin escribir; `apply`, en una transacción) y `export_html`. Lo importado entra en `imported`; a un bookmark que ya estaba solo se le añaden etiquetas, sin tocar `updated_at`.
- `crates/i18n` (`marks-i18n`): idiomas `Locale` (`Es`, `En`, y `Pseudo` solo para desarrollo), `resolve`, el catálogo Fluent (`i18n/{es,en}/marks_i18n.ftl`), `msg`/`error`/`bookmark_error` y los enums `AppText`, `MenuText`, `CliText` y `DataText`. Solo dependen de él el CLI y `src-tauri`; core no. Sus tests (`catalogs.rs`, `no_user_literals.rs`) son las guardas.
- `crates/core/src/lib.rs`: `normalize_url` quita los `utm_` y el fragmento, salvo los que empiezan por `#/` o `#!` (rutas de una SPA).
- `crates/cli/src/main.rs`: el CLI. Solo encola: ni descarga ni llama a modelos (la app extrae al abrirse). `--profile <nombre>` elige el perfil (sin él, el primero, no el activo de la app); `profiles` los lista. El idioma sale de `MARKS_LOCALE`, `LC_ALL`, `LC_MESSAGES` y `LANG`, y la ayuda, los errores y la salida están traducidos; no hay subcomando `help` (usar `--help`). `import <FICHERO>` importa sin preguntar, en `imported` (la app no lo extrae sola), y `export-html <FICHERO>` exporta el perfil.
- `apps/desktop`:
  - `src-tauri` expone core por IPC y corre el worker, la API loopback, las ventanas ocultas `extractor` (carga) y `parser` (Defuddle sobre `extract.html`), `extracting` (qué bookmark se extrae), la bandeja y la lista de notificaciones en `AppState` (`notifications.rs`: `notify` y `warn`, y el evento `NOTIFICATIONS_CHANGED`, sin carga). `AppState` guarda también `ai_health` (el modelo de resumen va envuelto en `Watched`) y `api_status` (`Starting`, `Listening`, `Busy` o `Stopped`); `status.rs` los da a la barra inferior con `app_status` y avisa de cada cambio con `APP_STATUS_CHANGED`, sin carga. Notifican el worker, el arranque (IA activada sin modelo), la API (puerto ocupado) y la ventana (`push_notification`). `parser` y `extractor` no tienen capabilities. El worker se turna entre perfiles (un bookmark y un favicon de cada uno por vuelta); un perfil cuyo almacén falla se salta hasta el siguiente aviso.
  - Si la base principal no abre al arrancar, una alerta nativa (`rfd`) lo explica y la app sale.
  - `updater.rs`: actualizaciones desde GitHub Releases con `tauri-plugin-updater`, solo desde Rust y sin permisos `updater:`. Comprueba a los 30 s y cada 24 h, solo en `aarch64` y en release; `MARKS_UPDATE_ENDPOINT` sustituye el endpoint para probar. La clave pública va en `tauri.conf.json`; la privada nunca entra en el repo.
  - Idioma de la app: `locale.rs` lee y escribe `AppleLanguages` solo en el dominio de preferencias propio (el id del bundle; bajo `tauri dev`, el nombre del proceso, `marks-desktop`). `locale_info` da el ajuste (`system`, `es` o `en`) y el idioma con el que arrancó; `set_locale` lo guarda y reinicia la app. `menu.rs` construye en macOS el menú entero (como `Menu::default` de Tauri 2.12, con Archivo) con el catálogo, vía `Builder::menu`. La API loopback devuelve solo códigos de error y la extensión los traduce (`API_ERROR_CODES`, solo en tests). El `Bookmark.error` de core no se serializa: la interfaz recibe `LocalBookmark`, con el error ya traducido.
  - `tauri-plugin-dialog` se usa solo desde Rust: los comandos de importar y exportar abren la ventana nativa y la ruta nunca viene de la webview, así que la capability no tiene permisos `dialog:`. Ojo: el script de inicio del plugin sustituye `window.alert` y `window.confirm` en todas las webviews, y sin esos permisos fallan. El frontend no usa ninguno de los dos.
  - `src` es el frontend Svelte, con `lib/api.ts`, `lib/i18n` (catálogos `es` y `en`; `lib/strings.ts` reexporta `t`), `lib/features/<zona>` (con `layout`: anchos, plegado de la barra lateral y su botón) y `lib/ui`.
- `packages/extract`: `marks-extract`, el paquete que envuelve Defuddle (única dependencia que lo nombra); lo cargan `extract.html` (ventana `parser`) y `apps/extension/page-extract.ts` (esbuild).
- `apps/extension`: extensión MV3 en JS plano, compartida por Safari y Chromium. `build.mjs` la construye en `dist/chromium` (sin `nativeMessaging` y con `minimum_chrome_version`) y `dist/safari` (con `nativeMessaging`, la que copia el appex), cada una con su manifiesto sacado de `manifest.json`. `apps/extension/safari` lleva el manejador Swift del appex. Tiene su versión propia en `manifest.json`, distinta de la de la app, que se sube cuando cambia `apps/extension` o `packages/extract` (el appex y el zip llevan el extractor); la compatibilidad la da `api` en `/v1/ping` (`API_LEVEL` en `api.rs`) frente a `MIN_API` en `client.js`.
- `scripts/build-appex.sh`: el appex de Safari universal y firmado en `target/appex`. `scripts/build-app.sh [--target <triple>] [--dmg]`: el appex y luego `tauri build` con `tauri.release.conf.json`, que lo mete en el bundle (`bundle.macOS.files`); Tauri firma la app con `APPLE_SIGNING_IDENTITY`. `scripts/verify-bundle.sh <marks.app>` comprueba appex, sandbox, arquitecturas, hardened runtime y firma.
- `scripts/check-version.sh [TAG]`: las versiones de `Cargo.toml` y los `package.json`, y con un tag, que lo nombra y que la clave del updater no es la provisional. `scripts/updater-manifest.mjs` escribe `latest-darwin-aarch64.json` con la sección de `CHANGELOG.md`.
- `.github/workflows/ci.yml` corre `scripts/check.sh`; `release.yml`, con un tag `v*`, construye, firma y deja una release en borrador (secrets en el Environment `release`).

## Comandos

- Verificación completa, antes de cada commit: `scripts/check.sh`. Ejecuta:
  - fmt y clippy `-D warnings`;
  - `cargo test`;
  - `pnpm install --frozen-lockfile`;
  - el build de la extensión y sus tests;
  - la comprobación de tipos y los tests de `packages/extract`;
  - Prettier, ESLint, svelte-check y vitest del frontend;
  - en macOS, la comprobación de tipos del manejador Swift.
- Tests de Rust: `cargo test --workspace` (o `-p marks-core`).
- Test de integración con Ollama, ignorado por defecto: `cargo test -p marks-core ollama_roundtrip -- --ignored`. Necesita Ollama con `qwen3:8b` y `nomic-embed-text`. `ollama_long_input_keeps_its_start` (también ignorado, ~40 s) comprueba que una entrada de 60 000 caracteres llega entera.
- CLI: `cargo run -p marks-cli -- <add|list|search|export|stats> …` (`--help` para opciones).
- Base de datos de pruebas, sin tocar la real (CLI y app): `MARKS_DB=/tmp/marks-scratch.db cargo run -p marks-cli -- list`.
- App de escritorio en desarrollo: `pnpm --dir apps/desktop tauri dev`, con `MARKS_DB=…` delante para una base de pruebas.
- Build de release: `pnpm --dir apps/desktop tauri build`. Deja `target/release/bundle/macos/marks.app` (firma ad-hoc) y `target/release/bundle/dmg/`.
  - El DMG automatiza Finder por AppleScript y falla en sesiones sin GUI o sin permiso de Automatización.
  - Solo el `.app`: `pnpm --dir apps/desktop tauri build --bundles app`.
- App con la extensión de Safari: `scripts/build-app.sh`, que deja `target/release/bundle/macos/marks.app` (con `--target <triple>`, en `target/<triple>/release/…`; con `--dmg`, también el DMG). Después, `scripts/verify-bundle.sh <ruta a marks.app>`.
  - Identidad de firma: `$APPLE_SIGNING_IDENTITY` (la release usa `marks Code Signing`, autofirmado); si no, `$CODESIGN_IDENTITY`; si no, la primera "Apple Development" válida; si no, ad-hoc con aviso.
  - Solo el appex: `scripts/build-appex.sh`.
- Versiones: `scripts/check-version.sh` (y `scripts/check-version.sh v1.0.0` antes de etiquetar). Manifiesto del updater: `node scripts/updater-manifest.mjs --version … --sig … --url … --notes CHANGELOG.md --out …` (tests: `node --test scripts/*.test.mjs`).
  - `tauri dev` no incluye la extensión de Safari. La de Chromium funciona cargando `apps/extension/dist/chromium` sin empaquetar, tras `pnpm --dir apps/extension build`, mientras corre `tauri dev`.
- Idioma en desarrollo: `MARKS_LOCALE=es|en|pseudo` manda sobre el del sistema (app, CLI y tests). `pseudo` acentúa y alarga cada texto entre `⟦ ⟧`, para ver lo que no pasó por un catálogo. Bajo `tauri dev`, cambiar el idioma en Ajustes relanza la app y termina `tauri dev`: hay que volver a lanzarlo. Los directorios `target` de pruebas van en `/private/tmp`, no en `/tmp`: Tauri no relanza un binario cuya ruta pasa por un enlace simbólico.
- Tests de la extensión: `node --test "apps/extension/*.test.js"`.
- Probar la API, con `MARKS_DB=…` y la app en marcha (el token está en Ajustes): `curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:47821/v1/ping`.
- Frontend: `pnpm --dir apps/desktop check`, `test`, `lint`, `format` y `format:check` (detalle en la skill `marks-desktop`).
- TypeScript 7 y 6 a la vez, como recomienda el anuncio de TS 7.0: `@typescript/native` (`npm:typescript@^7`) da el `tsc` de `packages/extract` y el `svelte-check --tsgo`; `typescript` es `npm:@typescript/typescript6`, porque TS 7.0 no tiene API de JS y la usan `typescript-eslint` y `svelte-check`. Quitar el alias de 6 cuando esas dos soporten la API de TS 7 (prevista en 7.1). `--tsgo` deja su caché en `apps/desktop/.svelte-check/`, ignorada por git, ESLint y Prettier.

**Base real:** `~/Library/Application Support/marks/marks.db`.

- Es el perfil 1, y guarda también el registro de perfiles y los ajustes globales.
- El perfil `n` > 1 vive en `marks-profiles/<n>.db`, junto a ella. El nombre de la carpeta es el de la base principal sin extensión más `-profiles`: con `MARKS_DB=/tmp/marks-scratch.db`, `/tmp/marks-scratch-profiles/`.

**Logs de la app** (tauri-plugin-log, destino por defecto): `~/Library/Logs/dev.landygg.marks/` y stdout.

## Reglas

- **La lógica va en `crates/core`.** Los comandos Tauri (3-10 líneas), el CLI y la API (`api.rs`, que llama a `core::capture`) no deciden nada: solo llaman a core.
- **`core` no depende de Tauri ni de la interfaz.**
- **Estados de bookmark:** `pending`, `pending_embed`, `ready`, `failed` e `imported` (importado y sin extraer: el worker no lo toma; pasa a `pending` con "Extraer", `add` o `capture`). Cualquier otro valor es un bug.
- **Nunca se retiene el `Mutex<Store>`** durante red o modelo, ni dos almacenes a la vez.
- **Los `id` de bookmark y de grupo se repiten entre perfiles.**
  - Lo que nombra un bookmark o escribe lleva el `id` de su perfil: el del perfil en el que se cargó o se abrió la edición, no el activo al enviar.
  - Solo las lecturas y `add` usan el perfil activo.
- **Perfiles.**
  - El primer perfil no se elimina, porque guarda el registro.
  - No hay búsqueda entre perfiles.
  - Los ajustes de IA y las claves del proveedor (una por rol: resumen y extracción) son globales; el token de la API es por perfil.
  - La IA puede estar desactivada (`AiSettings.enabled`), y lo está por defecto en una instalación nueva: sin modelo, el worker solo extrae. Un JSON guardado sin el campo cuenta como desactivada. El worker lee el modelo antes de cada bookmark: desactivarla corta la IA en el siguiente.
- **Ningún texto de usuario escrito en el código, en ningún sitio.** Todo texto que pueda ver el usuario va a los ficheros de idioma, en español (`es`) e inglés (`en`) a la vez, sin excepciones aplazadas (spec `docs/superpowers/specs/2026-11-14-i18n-es-en-design.md`, ADR 0015):
  - Rust (core, `src-tauri` y CLI): `crates/i18n/i18n/{es,en}/marks_i18n.ftl`, del crate `marks-i18n`. core no lleva texto: devuelve códigos (`Msg`, envuelto en `Failure`) y `marks-i18n` los traduce con `fl!(loader(), …)`.
  - Frontend: `apps/desktop/src/lib/i18n/{es,en}.ts` (`en` con `satisfies Messages`); `lib/strings.ts` solo reexporta `t`.
  - Extensión: `apps/extension/_locales/{es,en}/messages.json`.
  - Bundle: `apps/desktop/src-tauri/infoplist/{es,en}.lproj/InfoPlist.strings` y `apps/extension/safari/{es,en}.lproj/InfoPlist.strings`.
  - La guarda corre en `scripts/check.sh` (`crates/i18n/tests/no_user_literals.rs` para Rust y `apps/desktop/src/i18n.guard.test.ts` para TS, Svelte, JS, HTML y Swift) y falla con un literal de usuario fuera de esos ficheros. Lo que no es texto de usuario y la guarda confundiría lleva `i18n-allow: <motivo>` en la línea del literal o en la de la `const`, `static` o `let` que lo declara; el motivo es obligatorio (`temporal` no vale), una marca sin uso también falla, y la guarda imprime cuántas hay en stderr.
- **Sin valores mágicos inline.** Van como constantes con nombre en el módulo que los usa.
- **Sin `unwrap()` fuera de tests.**
- **Contenido siempre con `ContentView` y su lista blanca** (`features/library/reader.ts`, ADR 0011). Sin `{@html}` en la app.
- **Commits en la rama `main`, en Conventional Commits** (`tipo(ámbito): resumen`, en inglés; tipos y ámbitos en `CONTRIBUTING.md`). No hacer push sin pedirlo.

## Identificador `dev.landygg.marks`

Ficheros que lo llevan (actualizar esta lista al añadir o quitar uno):

- `apps/desktop/src-tauri/tauri.conf.json` (`identifier` del bundle)
- `crates/core/src/settings.rs` (`KEYRING_SERVICE`, servicio del llavero)
- `apps/extension/client.js` (`NATIVE_APP_ID`)
- `apps/extension/safari/Info.plist` (`CFBundleIdentifier` del appex, con sufijo `.safari-extension`)

Derivado del identificador del bundle, sin escribirlo en código: el directorio de logs `~/Library/Logs/dev.landygg.marks/`.

## Puerto `47821`

Solo en `127.0.0.1`. Ficheros que lo llevan (cambiarlos siempre juntos; el test `extension_files_agree_with_the_app` comprueba que coinciden):

- `apps/desktop/src-tauri/src/api.rs` (`API_PORT`)
- `apps/extension/manifest.json` (`host_permissions`)
- `apps/extension/client.js` (`API_PORT`)
- `apps/extension/safari/SafariWebExtensionHandler.swift` (`apiBase`)

## Documentos

- **Skills de agente.** `.agents/skills/marks-desktop` y `.agents/skills/marks-extension`, cada una con `references/best-practices.md`: fuentes externas y estado del repo frente a ellas.
- **Diseño visual.** `DESIGN.md`, en el formato DESIGN.md de Google (versión alpha), con los tokens en el front matter. Leerlo antes de tocar la interfaz.
- **v0.2.** Diseño `docs/superpowers/specs/2026-11-09-v0.2-desktop-app-design.md`; plan `docs/superpowers/plans/2026-11-09-v0.2-desktop-app.md`.
- **v0.3.** Diseño `docs/superpowers/specs/2026-11-09-v0.3-extension-api-design.md`; plan `docs/superpowers/plans/2026-11-09-v0.3-extension-api.md`.
- **v0.3.1.** Diseño `docs/superpowers/specs/2026-11-10-v0.3.1-webview-fallback-design.md`; plan `docs/superpowers/plans/2026-11-10-v0.3.1-webview-fallback.md`.
- **v0.4 (interfaz Marcador).** Plan `docs/superpowers/plans/2026-11-10-v0.4-marcador-ui.md`.
- **v0.5 (perfiles).** Diseño `docs/superpowers/specs/2026-11-10-v0.5-profiles-design.md`; plan `docs/superpowers/plans/2026-11-10-v0.5-profiles.md`.
- **v0.6 (Defuddle y svelte-markdown).** Diseño `docs/superpowers/specs/2026-11-12-v0.6-defuddle-extraction-design.md`, aprobado y con los resultados de los spikes; plan `docs/superpowers/plans/2026-11-12-v0.6-defuddle.md`.
- **Notificaciones y barra inferior.** Diseño `docs/superpowers/specs/2026-11-13-notifications-design.md`; plan `docs/superpowers/plans/2026-11-13-notifications.md`.
- **v0.7 (importar y exportar).** Diseño `docs/superpowers/specs/2026-11-13-v0.7-import-export-design.md`; plan `docs/superpowers/plans/2026-11-13-v0.7-import-export.md`.
- **Idiomas (es y en).** Diseño `docs/superpowers/specs/2026-11-14-i18n-es-en-design.md`; plan `docs/superpowers/plans/2026-11-14-i18n-es-en.md`.
- **Paneles.** Diseño `docs/superpowers/specs/2026-11-14-panels-design.md`; plan `docs/superpowers/plans/2026-11-14-panels.md`.
- **v1.0 (release pública).** Plan `docs/superpowers/plans/2026-11-16-v1.0-release.md`. Proceso de release: `docs/{es,en}/developers/release.md`.
- **Decisiones.** `docs/adr/`:
  - 0001 a 0004: Svelte, genai, SQLite y sqlite-vec;
  - 0005: API loopback con token;
  - 0006: HTML crudo a core;
  - 0007: appex de Safari a mano;
  - 0008: webview oculta como respaldo;
  - 0009: un fichero SQLite por perfil;
  - 0010: Defuddle en el navegador (sustituye a 0006).
  - 0011: lectura con componentes y listas blancas, sin `{@html}` ni DOMPurify.
  - 0012: extracción con IA con proveedor propio y HTML limpio como entrada.
  - 0013: el chat con Ollama, por su API nativa con `num_ctx` (se aparta en parte de 0002).
  - 0014: intercambio con los navegadores por `bookmarks.html`, con carpetas ↔ grupos y etiquetas.
  - 0015: dos idiomas con un catálogo por capa (Fluent en Rust, diccionario tipado en TS, `_locales`) y `Msg` en core.
