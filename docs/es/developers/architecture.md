# Arquitectura

**Español** · [English](../../en/developers/architecture.md)

marks es un workspace de Cargo con un workspace de pnpm al lado. Toda la lógica vive en una librería Rust, `marks-core`. La app de escritorio, el CLI y la API local solo la llaman.

## Piezas

| Ruta | Paquete | Papel |
|---|---|---|
| `crates/core` | `marks-core` (lib `marks_core`) | Almacén SQLite, extracción, IA, búsqueda, perfiles, importar y exportar |
| `crates/i18n` | `marks-i18n` | Idiomas y catálogo Fluent para todo lo que escribe Rust |
| `crates/cli` | `marks-cli` (binario `marks`) | CLI: solo encola y consulta |
| `apps/desktop/src-tauri` | `marks-desktop` | Backend Tauri 2: IPC, worker, API local, bandeja, menú |
| `apps/desktop/src` | `marks-desktop` (pnpm) | Frontend Svelte 5 + TypeScript + Vite |
| `apps/extension` | — | Extensión MV3 para Safari y Chromium, con el manejador Swift de Safari |
| `packages/extract` | `marks-extract` | Envoltorio de Defuddle: HTML → Markdown y metadatos |

`marks-core` no depende de Tauri, de la interfaz ni de `marks-i18n`. Solo el CLI y `src-tauri` dependen de `marks-i18n`.

## `crates/core`

| Módulo | Qué hace |
|---|---|
| `store` | SQLite, migraciones y FTS5. `url` es la URL normalizada y la que deduplica; `original_url` es la URL tal como llegó |
| `profiles` | `Library`: el registro de perfiles, un `Store` por perfil, el perfil activo, el token de cada perfil, eliminar y mover bookmarks |
| `msg` | `Msg` (códigos de mensaje con argumentos tipados), `Failure` y `BookmarkError`. core no lleva texto de usuario |
| `groups` | Grupos de etiquetas |
| `notifications` | Registro de notificaciones en memoria, con un tope de 200. Sin E/S ni reloj |
| `chunk` | Trozos de unos 1.500 caracteres, 40 como máximo por bookmark |
| `ai` | El trait `Model` (resumen, embeddings, extracción), los prompts y `FakeModel` para tests |
| `genai_model` | `Model` sobre la librería `genai`. El chat con Ollama va a su API nativa `/api/chat` |
| `settings` | Ajustes de IA por rol y claves en el llavero del sistema |
| `vectors` | Índice vectorial con sqlite-vec |
| `search` | Búsqueda híbrida: FTS5 y vecinos más cercanos, fusionados con RRF (`k = 60`) |
| `pipeline` | `process_next`, la reextracción manual y la descarga de favicons |
| `extract` | Los tipos `Page` y `PageMeta`, `is_thin` y los `Fetcher`. No convierte HTML |
| `download` | Descarga HTTP con destinos validados, tope de tamaño y charset declarado |
| `blocked` | Detecta páginas de verificación antibots |
| `capture` | Decide qué se guarda de una captura que envía la extensión |
| `netscape` | Lee y escribe el formato `bookmarks.html` |
| `transfer` | Importar (`read_file`, `preview`, `apply`) y `export_html` |

`normalize_url`, en `lib.rs`, quita el fragmento y los parámetros `utm_`. Conserva los fragmentos que empiezan por `#/` o `#!`, porque son rutas de una aplicación web.

## `crates/i18n`

core devuelve códigos (`Msg`) y `marks-i18n` los convierte en texto. El crate tiene:

- `Locale` (`Es`, `En` y `Pseudo`, este último solo para desarrollo) y `resolve`, que elige el idioma.
- El catálogo Fluent, en `crates/i18n/i18n/{es,en}/marks_i18n.ftl`.
- Los enums `AppText`, `MenuText`, `CliText` y `DataText`, para los textos que no son un `Msg`.

El detalle está en [Idiomas y textos](i18n.md).

## `crates/cli`

El binario `marks`. Subcomandos: `add`, `list`, `search`, `export`, `import`, `export-html`, `stats` y `profiles`.

- Solo encola: no descarga páginas ni llama a modelos. La app extrae lo encolado al abrirse.
- `--profile <nombre>` elige el perfil. Sin la opción usa el primer perfil, no el activo de la app.
- `--db <ruta>` o la variable `MARKS_DB` eligen la base de datos.
- No hay subcomando `help`: la ayuda es `--help`.

## `apps/desktop`

Dos mitades con una frontera estrecha.

**`src-tauri` (Rust).** Expone core por IPC y corre lo que funciona en segundo plano.

| Fichero | Papel |
|---|---|
| `lib.rs` | Arranque: plugins, estado, worker, API, bandeja y la lista de comandos |
| `state.rs` | `AppState`: la `Library`, los modelos, la lista de notificaciones y los cerrojos |
| `commands.rs` | Los comandos IPC. Cada uno toma el estado y llama a core |
| `worker.rs` | El bucle que procesa la cola |
| `api.rs` | La [API local](local-api.md) |
| `webview_fetcher.rs`, `parser_window.rs` | Las ventanas ocultas `extractor` y `parser` |
| `extracting.rs` | Qué bookmark se está extrayendo |
| `notifications.rs` | Guarda una notificación y avisa a la ventana |
| `tray.rs` | Icono de la barra de menús; cerrar la ventana la oculta |
| `menu.rs` | Menú nativo de la app. Solo macOS |
| `locale.rs` | Idioma de la app, guardado en `AppleLanguages`. Solo macOS |

**`src` (Svelte 5).** Solo pinta y llama a `lib/api.ts`, el único fichero que usa `invoke`.

| Ruta | Papel |
|---|---|
| `lib/api.ts` | Envoltorios tipados de cada comando y nombres de los eventos |
| `lib/features/<zona>/` | Una carpeta por zona: `layout`, `library`, `notifications`, `palette`, `profiles`, `settings` y `statusbar` |
| `lib/ui/` | Los componentes de `DESIGN.md`. Reciben texto y datos por props |
| `lib/i18n/` | Los catálogos `es.ts` y `en.ts` |
| `lib/shortcuts.ts` | Los atajos de teclado |

## `apps/extension`

Extensión MV3 en JavaScript plano, con el mismo código para Safari y Chromium. `build.mjs` la empaqueta con esbuild en `apps/extension/dist/chromium/`, que es lo que carga Chromium, y la copia a `apps/extension/dist/safari/`, que va en el appex. Solo cambia el manifiesto, escrito para cada navegador desde `manifest.json`: el de Chromium quita `nativeMessaging` (solo el appex de Safari habla con la app de forma nativa) y declara `minimum_chrome_version` 123, el suelo de motores.

- `popup.js` envía un mensaje al service worker y muestra el resultado.
- `background.js` inyecta `page-extract.js` en la pestaña, extrae sobre un clon del documento y envía la captura a la app.
- `client.js` es el cliente de la API. No usa el DOM ni las API de extensión, así que sus tests corren en Node.
- `safari/SafariWebExtensionHandler.swift` reenvía los mensajes nativos de Safari a la API local.

La extensión no decide qué se guarda: eso lo hace `core::capture`.

## `packages/extract`

`marks-extract` es el único paquete que depende de Defuddle. Lo cargan dos sitios:

- `apps/desktop/extract.html`, en la ventana oculta `parser` de la app.
- `apps/extension/page-extract.ts`, que esbuild empaqueta para inyectarlo en la página.

Defuddle corre sin red: el paquete rechaza cualquier petición de sus extractores de sitio.

## Recorrido de un bookmark

1. **Entrada.** El bookmark llega por el diálogo de la app, por `marks add`, por una importación o por una captura de la extensión (`POST /v1/bookmarks`). Una captura puede traer ya el contenido extraído en el navegador.
2. **Cola.** El worker toma primero lo último encolado. Sondea cada 15 segundos y también despierta cuando alguien le avisa. Se turna entre perfiles: un bookmark y un favicon de cada uno por vuelta.
3. **Descarga.** Si el bookmark no tiene contenido, `HttpFetcher` baja el HTML por HTTP.
4. **Respaldo.** Si HTTP falla o el contenido es escaso (menos de 200 caracteres), `FallbackFetcher` prueba `WebviewFetcher`: la ventana oculta `extractor` carga la página y devuelve el HTML ya renderizado. No se intenta con 404, 410 ni 5xx.
5. **Extracción.** La ventana oculta `parser` pasa el HTML por Defuddle y devuelve Markdown y metadatos.
6. **Resumen.** Con la IA activada, el modelo de chat devuelve resumen, categoría y etiquetas. El bookmark pasa a `pending_embed`. Sin modelo, el recorrido termina aquí y el bookmark se queda en `pending`, con su contenido.
7. **Embeddings.** El contenido se trocea, se calculan los vectores y el bookmark pasa a `ready`.
8. **Favicon.** Se descarga aparte, solo de destinos públicos. Los fallos pasajeros se reintentan con espera creciente, hasta 4 intentos.

Un fallo de red o de modelo se guarda en el bookmark, que pasa a `failed`.

## Estados de un bookmark

| Estado | Significado |
|---|---|
| `pending` | En cola, o extraído y sin resumir |
| `pending_embed` | Resumido; faltan los embeddings |
| `ready` | Listo |
| `failed` | Falló; el error queda guardado en el bookmark |
| `imported` | Importado y sin extraer. El worker no lo toma |

Un bookmark `imported` pasa a `pending` con "Extraer", con `add` o con una captura. Cualquier otro valor es un bug.

## Ventanas ocultas

- `extractor` carga páginas remotas, que no son de confianza.
- `parser` carga solo `extract.html` y recibe HTML ajeno como dato.

Ninguna de las dos tiene permisos (capabilities) de Tauri: solo la ventana `main` los tiene. Las dos comparten un único cerrojo, así que nunca hay dos ventanas ocultas abiertas a la vez. Cada una se destruye siempre al terminar, con éxito, con error o al vencer su tope de tiempo.

## Hilos y cerrojos

- Cada perfil tiene su `Store` detrás de un `Mutex`. Nunca se retiene durante una petición de red o una llamada a un modelo, y nunca se toman dos almacenes a la vez.
- La base se abre en modo WAL, porque el CLI y la app pueden abrirla a la vez.
- Un comando Tauri síncrono corre en el hilo principal y congela la ventana mientras dura. Las lecturas y el trabajo largo van en comandos `command(async)`; la red y los modelos, en `async fn`.
- Los eventos de Rust a la ventana no llevan datos: solo avisan de que algo cambió y la ventana recarga.
