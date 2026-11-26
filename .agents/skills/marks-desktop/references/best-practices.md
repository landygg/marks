# Buenas prácticas de Tauri 2 + Svelte 5 y estado de marks

Investigado el 2026-11-11 sobre la documentación oficial (v2.tauri.app, svelte.dev), que pesa más que blogs y skills de terceros. Versiones en el repo: `tauri` 2.12.5, Svelte 5, Vite 8.

## Backend (Tauri 2)

| Práctica | Fuente | Estado en `src-tauri` |
|---|---|---|
| Comandos en un módulo propio, `pub`, registrados con ruta en un único `generate_handler!` | Tauri, Calling Rust | Cumple: `commands.rs`, un solo `invoke_handler` en `lib.rs` |
| Comandos sin `async` corren en el hilo principal; trabajo pesado en `async` o `#[tauri::command(async)]` | Tauri, Calling Rust; código de `tauri-macros` (`sync_threadpool`) | Cumple desde 2026-11-11: red/modelo son `async fn`; lecturas, llavero, ficheros de perfil y escrituras masivas, `command(async)`; en el hilo principal solo escrituras puntuales, para conservar su orden |
| `async` con referencias prestadas (`State`) → devolver `Result` | Tauri, Calling Rust | Cumple: todos devuelven `Result<_, String>` |
| Errores serializables; enum con `thiserror` y `kind` cuando el frontend debe distinguirlos | Tauri, Calling Rust | Parcial a propósito: `String` con contexto (`to_ipc`). Basta mientras la interfaz solo muestre el mensaje |
| Estado con `.manage()`; no hace falta `Arc` alrededor; `std::Mutex` vale en async si no cruza `.await` | Tauri, State Management | Cumple. `Arc<Library>` existe porque worker y API la comparten fuera de Tauri |
| Fuera de comandos, mover `AppHandle` y usar `app.state()` | Tauri, State Management | Cumple: `worker::spawn`, `api::spawn` |
| Tipo de `State` equivocado compila y falla en ejecución | Tauri, State Management | Un solo tipo (`AppState`): riesgo bajo |
| Eventos: sin tipo, sin respuesta, solo JSON; para flujos, `Channel`. Siempre `unlisten` | Tauri, Calling Rust | Cumple: eventos sin carga como aviso de recarga |
| Capabilities por etiqueta de ventana, mínimas, un fichero por capability | Tauri, Capabilities | Cumple: `default.json` solo para `main`; `extractor` y `parser` sin ninguna (test). `tauri-plugin-dialog` se usa solo desde Rust, sin permisos `dialog:` |
| Ventana sin capability = sin IPC; contenido remoto solo con `remote.urls` | Tauri, Capabilities | Cumple: no hay `remote.urls` |
| Comandos propios accesibles desde todas las ventanas salvo `AppManifest::commands` en `build.rs` | Tauri, Capabilities; `tauri` 2.12.5 `webview/mod.rs` | Cumple desde 2026-11-11: `AppManifest` con los 42 comandos y un `allow-*` por comando solo para `main`. Un comando no concedido se rechaza (falla cerrado); un test compara `generate_handler!`, `build.rs` y la capability |
| CSP lo más restrictiva posible; nada de scripts remotos; Tauri añade nonces/hashes | Tauri, CSP | Cumple: `default-src 'self'`, imágenes `self`/`data:`, y `form-action`, `base-uri`, `frame-src` y `object-src` a `'none'` (las dos primeras no heredan de `default-src`). `style-src 'unsafe-inline'` lo pide Svelte para estilos en línea |

## Frontend (Svelte 5)

| Práctica | Fuente | Estado en `src` |
|---|---|---|
| Estado compartido en `.svelte.ts` exportando objetos que se mutan (no reasignar) | Svelte, `$state` | Cumple: `palette`, `profileDialog`; `lib` es una instancia de clase con campos de runas, porque `$state({...})` no admite `$derived` |
| `$state.raw` para datos grandes que solo se reemplazan | Svelte, Best practices | No se usa, a propósito: con `$state.raw` las mutaciones (`push`, `sort`, `obj.x = …`) no se pintan ni dan error, como con `shallowRef` en Vue o `useState` en React. Vue solo lo recomienda por encima de ~100.000 propiedades por render. Lo que pesaba era otra cosa: cada `refresh` traía objetos nuevos y repintaba todas las filas. `keepUnchanged` conserva los de las filas sin cambios. Medido en jsdom con 2.000 bookmarks: ~230 ms antes, ~27 ms con `$state` profundo y `keepUnchanged`, ~2 ms con `$state.raw` y `keepUnchanged`. `lib` es una clase para que `visible`, `categories` y `tags` sean `$derived` |
| `$derived` para valores calculados; `$effect` como último recurso y sin asignar estado | Svelte, Best practices | Cumple desde 2026-11-11: los efectos que reiniciaban campos al cambiar de perfil se quitaron (`{#key}` en `Settings.svelte` y `$derived` reasignable en `ProfileSettings`). Desde 2026-11-12, el nombre del grupo en la barra lateral es un `$derived` reasignable, el título pasa a `TitleEditor` y el DOM, a attachments. Quedan tres efectos (eran once): dos `$effect.pre` que siguen un valor guardado salvo cambios sin guardar (`NotesEditor`, `TitleEditor`) y la carga del contenido en `Detail`. |
| `<svelte:window>` en vez de `addEventListener` en `onMount` | Svelte, Best practices | Cumple: `App.svelte` usa `<svelte:window onkeydown>` desde 2026-11-11 |
| `{#each}` con clave única, nunca el índice | Svelte, Best practices | Cumple desde 2026-11-12, también en las listas estáticas. Los trozos de un texto resaltado van sin clave: no tienen identidad propia |
| `{@attach}` para el DOM de un elemento en vez de `bind:this` + `$effect` | Svelte, `{@attach}`; `$effect` | Cumple desde 2026-11-12: diálogos (`lib/modal.ts`), filas que se desplazan a la vista y fin de un arrastre cuyo botón desaparece. `bind:this` queda donde un manejador necesita el elemento (`dialog.close()` de la paleta) |
| `class={{ ... }}` en vez de la directiva `class:` | Svelte, `class` | Cumple desde 2026-11-12: ninguna directiva `class:` |
| Contexto (`createContext`) en vez de módulos compartidos, por SSR | Svelte, Best practices | No aplica: app de una sola ventana sin SSR; los módulos son la opción más simple |
| Pasar getters, no valores, a funciones que deben ver cambios | Svelte, `$state` | Ten en cuenta al extraer lógica de componentes |

## Plataforma web (WebKit del sistema)

| Práctica | Fuente | Estado en `src` |
|---|---|---|
| HTML, CSS y JS de cliente según la guía, filtrados por el suelo de motores (`AGENTS.md`, «Navegadores») | `modern-web-guidance` (skill `2026_11_16-268fb9ea`) | Auditada el 2026-11-25 (auditoría del 2026-11-25, notas del proyecto). Arreglado en la 1.2.0: Enter, Esc y flechas con un método de entrada (`isComposing`), la ventana `main` limitada a la app (`navigation.rs`), campos sin autocorrector (`identifier`), código y tablas desplazables con el teclado, opciones de la paleta fuera del Tab y su estado en `role="status"`, la paleta se cierra con un clic fuera (`lightDismiss`), el motivo de un botón deshabilitado (`reason`), el foco al cerrar un toast, el error de URL en el campo, los títulos de diálogo en `h1`, el fallo al copiar, el indicador y el borde de campo con contraste en claro, `--warn` claro a 4,5:1, el lector a 70 `ch`, el tema antes del primer fotograma, `overflow: clip` y propiedades lógicas. Pendiente: el zoom (A16/C9), C4 y C16 (tras perfilar), C15, A13, C13, C14, A8, A9, A11, A12 y A17 |

## Organización de ficheros

Las guías oficiales no imponen carpetas: Tauri propone `src-tauri/src/{lib.rs,main.rs}` y módulos al crecer; Svelte, componentes y módulos `.svelte.ts`. Desde el 2026-11-11, marks agrupa por **zona** (colocation): `lib/features/{library,palette,profiles,settings}/` con pantalla, estado, lógica y tests juntos. En la raíz de `lib` queda solo lo compartido: `api.ts` (frontera IPC y sus tipos), `i18n/` (catálogos `es` y `en`, con `strings.ts` como reexportación de `t`), `shortcuts.ts`, `format.ts` y `ui/` (sistema de diseño). Se descartaron las carpetas por tipo técnico (`types/`, `utils/`, `services/`): separan lo que cambia junto y `utils/` acaba de cajón de sastre. En `src-tauri/src` se mantienen módulos planos (8 ficheros); `commands.rs` se partiría en `commands/<zona>.rs` si pasa de ~500 líneas, y una carpeta `services/` invitaría a meter lógica que pertenece a `crates/core`.

## Fuentes

- Tauri, Calling Rust from the Frontend: https://v2.tauri.app/develop/calling-rust/
- Tauri, State Management: https://v2.tauri.app/develop/state-management/
- Tauri, Capabilities: https://v2.tauri.app/security/capabilities/
- Tauri, Content Security Policy: https://v2.tauri.app/security/csp/
- Tauri, Runtime authority: https://v2.tauri.app/security/runtime-authority/
- Tauri 2.11.1 (ACL para orígenes remotos): https://v2.tauri.app/release/tauri/v2.11.1/
- Svelte, Best practices: https://svelte.dev/docs/svelte/best-practices
- Svelte, `$state`: https://svelte.dev/docs/svelte/$state
- Vue, Performance (`shallowRef` para estructuras grandes): https://vuejs.org/guide/best-practices/performance.html
- React, Updating Arrays in State: https://react.dev/learn/updating-arrays-in-state
- Agent Skills, especificación: https://agentskills.io/specification
