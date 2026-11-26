# Reglas del repositorio

**Español** · [English](../../en/developers/rules.md)

Todo cambio debe respetar estas reglas. La mayoría las comprueba `scripts/check.sh`; las demás se miran en la revisión.

## Dónde va la lógica

- **La lógica va en `crates/core`.** Los comandos Tauri (de 3 a 10 líneas), el CLI y la API local no deciden nada: toman sus argumentos y llaman a core.
- **`core` no depende de Tauri ni de la interfaz.** Tampoco de `marks-i18n`.
- **La extensión tampoco decide.** Extrae la página y la envía; `core::capture` decide qué se guarda.

## Estados de un bookmark

Solo existen `pending`, `pending_embed`, `ready`, `failed` e `imported`. Cualquier otro valor es un bug.

`imported` significa importado y sin extraer. El worker no lo toma. Pasa a `pending` con "Extraer", con `add` o con una captura.

## Cerrojos

- **Nunca se retiene el `Mutex<Store>`** durante una petición de red o una llamada a un modelo.
- **Nunca se toman dos almacenes a la vez.**
- Tampoco se retiene un cerrojo a través de un `.await`. Clona el `Arc` y suéltalo.

## Perfiles

- **Los `id` de bookmark y de grupo se repiten entre perfiles.** Todo lo que nombra un bookmark o escribe lleva el `id` de su perfil: el del perfil en el que se cargó el dato o se abrió la edición, no el activo en el momento de enviar.
- Solo las lecturas y `add` usan el perfil activo.
- El primer perfil no se elimina, porque guarda el registro de perfiles y los ajustes globales.
- No hay búsqueda entre perfiles.
- Los ajustes de IA y las claves del proveedor son globales. Hay una clave por rol: resumen y extracción.
- El token de la API es de cada perfil.

## Inteligencia artificial

- La IA puede estar desactivada, y lo está por defecto en una instalación nueva. Sin modelo, el worker solo extrae.
- El worker lee el modelo antes de cada bookmark: desactivar la IA la corta en el siguiente.
- La reextracción manual ("Volver a extraer" y "Extraer con IA") solo la lanza el usuario desde la ventana principal. Ni la API ni el worker la inician.

## Texto de usuario

**Ningún texto que pueda ver el usuario se escribe en el código.** Todo va a los ficheros de idioma, en español e inglés a la vez:

| Capa | Ficheros |
|---|---|
| Rust (core, `src-tauri`, CLI) | `crates/i18n/i18n/{es,en}/marks_i18n.ftl` |
| Frontend | `apps/desktop/src/lib/i18n/{es,en}.ts` |
| Extensión | `apps/extension/_locales/{es,en}/messages.json` |
| Nombre de la app | `apps/desktop/src-tauri/infoplist/{es,en}.lproj/InfoPlist.strings` |
| Nombre de la extensión de Safari | `apps/extension/safari/{es,en}.lproj/InfoPlist.strings` |

Dos guardas de `scripts/check.sh` fallan si encuentran un literal de usuario fuera de esos ficheros. El detalle está en [Idiomas y textos](i18n.md).

## Código

- **Sin valores mágicos escritos en línea.** Puertos, rutas, topes y esperas son constantes con nombre en el módulo que las usa.
- **Sin `unwrap()` fuera de los tests.**
- Los errores de core son un `Msg` envuelto en `Failure`, con su causa en `source()`.
- Los tests van junto al código: `#[cfg(test)] mod tests` en Rust y `*.test.ts` o `*.test.js` al lado del fichero.

## Interfaz

- **El contenido de una página se muestra siempre con `ContentView` y su lista blanca** de etiquetas y atributos (`features/library/reader.ts`). No se usa `{@html}` en la app.
- Fuera de `lib/ui` no hay `<button>`, `<input>`, `<select>`, `<textarea>`, colores, tipografías ni tamaños. Las pantallas solo llevan estilos de disposición.
- `invoke` solo se usa en `lib/api.ts`.
- No se usan `window.alert` ni `window.confirm`: las preguntas van con el componente `InlineConfirm`.
- Lee [`DESIGN.md`](../../../DESIGN.md) antes de tocar la interfaz.

## Seguridad

- Las ventanas ocultas `extractor` y `parser` no tienen permisos de Tauri y no deben tenerlos.
- Un comando nuevo necesita su permiso `allow-<comando>` en `capabilities/default.json`, y solo para la ventana `main`.
- La política CSP no admite orígenes remotos. Las imágenes son solo `self` y `data:`.
- La extensión pide los permisos mínimos: `activeTab`, `scripting`, `storage`, `nativeMessaging` (solo en Safari: `build.mjs` lo quita del manifiesto de Chromium) y `http://127.0.0.1:47821/*`. No añadas `<all_urls>`, `tabs` ni permisos de host amplios.
- El token de la extensión solo vive en `storage.local` y en la cabecera `Authorization`. No se registra en logs ni se pasa a la página.

## Git

- Los commits van en la rama `main`.
- `scripts/check.sh` tiene que pasar antes de cada commit.

## Identificador `dev.landygg.marks`

Lo llevan estos ficheros. Si añades o quitas uno, actualiza la lista de [`AGENTS.md`](../../../AGENTS.md):

- `apps/desktop/src-tauri/tauri.conf.json` (`identifier` del bundle)
- `crates/core/src/settings.rs` (`KEYRING_SERVICE`, el servicio del llavero)
- `apps/extension/client.js` (`NATIVE_APP_ID`)
- `apps/extension/safari/Info.plist` (`CFBundleIdentifier`, con el sufijo `.safari-extension`)

El directorio de logs, `~/Library/Logs/dev.landygg.marks/`, se deriva del identificador del bundle.

## Puerto `47821`

La API local escucha solo en `127.0.0.1`. El puerto está en cuatro ficheros, que se cambian siempre juntos:

- `apps/desktop/src-tauri/src/api.rs` (`API_PORT`)
- `apps/extension/manifest.json` (`host_permissions`)
- `apps/extension/client.js` (`API_PORT`)
- `apps/extension/safari/SafariWebExtensionHandler.swift` (`apiBase`)

El test `extension_files_agree_with_the_app` comprueba que coinciden.
