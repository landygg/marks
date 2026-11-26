---
name: marks-extension
description: Arquitectura, organización de ficheros, patrones y reglas de seguridad de la extensión de navegador de marks (apps/extension, MV3 en JS plano para Safari y Chromium, con el manejador Swift del appex de Safari). Usar al leer, cambiar, revisar o depurar cualquier fichero de apps/extension, al tocar la API loopback (puerto 47821, /v1/ping, /v1/bookmarks) desde el lado de la extensión, o al añadir permisos, pantallas, mensajes o rutas nuevas.
---

# Extensión de marks

MV3 en JS plano, empaquetado con esbuild (`build.mjs`) en `apps/extension/dist/chromium/` (lo que se carga en Chromium) y `apps/extension/dist/safari/` (lo que copia el appex). Un único código para Safari y Chromium; solo cambia el manifiesto, que `build.mjs` escribe para cada uno desde `manifest.json` (`manifestFor`): Chromium sin `nativeMessaging` y con `minimum_chrome_version` (`CHROME_MIN`, 123), Safari con `nativeMessaging`. La extensión no decide nada sobre el bookmark: extrae con Defuddle y envía `{url, title, page?, text?}` a la app; `crates/core::capture` decide qué se guarda.

`build.mjs` empaqueta `popup.js`, `options.js` y `background.js` (ESM, con `client.js` e `i18n.js` dentro) y `page-extract.ts` (IIFE con Defuddle, que define `globalThis.marksExtract`), y copia los ficheros estáticos. `build.test.js` comprueba la lista exacta de `dist/chromium/` y que `dist/safari/` solo difiere en el manifiesto: no añadir ficheros de entrada sin actualizarlo.

## Mapa

| Fichero | Papel | Puede usar |
|---|---|---|
| `client.js` | Cliente de la API: rutas, transportes, interpretación de respuestas | Nada del DOM ni de `chrome`/`browser` (corre en Node) |
| `i18n.js` | Único lector del namespace (`ext = browser ?? chrome`), `t`, `translatePage` | Namespace de extensión |
| `popup.js` / `popup.html` | Lee el token, envía `{type: SAVE_MESSAGE, token, tabId, url, title}` al background y muestra el resultado; no inyecta nada; al abrirse limpia el badge | `client.js`, `i18n.js` |
| `background.js` | Service worker (módulo): recibe `{type: SAVE_MESSAGE, token, tabId, url, title}` de la propia extensión, inyecta `page-extract.js` (`extractTab`), arma el cuerpo con `buildCapture`, llama a `saveTab` y responde; si falla, pone `!` en el badge | `client.js`, `i18n.js` |
| `options.js` / `options.html` | Guarda el token en `storage.local` y hace ping | `client.js`, `i18n.js` |
| `_locales/{es,en}/messages.json` | Todos los textos de usuario, con las mismas claves en `es` y `en`; `lang` es el código del idioma de ese locale (lo pone `translatePage` en `<html lang>`) | — |
| `extension.css` | Aspecto "Marcador" de `DESIGN.md`: tokens copiados tal cual de `apps/desktop/src/app.css`, ficha de la página con barra amarilla de 2px al guardarse, `status-dot` cuadrado, `kbd` con el atajo real en una caja y sus teclas separadas por `·` (como `ui/Keys.svelte` en la app) | — |
| `style.test.js` | Comprueba que los tokens de `extension.css` son los de `app.css` y que cada `data-i18n` existe en el locale | — |
| `client.test.js` | Tests de `client.js` con transportes falsos (`node --test`) | — |
| `safari/SafariWebExtensionHandler.swift` | Relé nativo: valida forma y ruta, reenvía a `127.0.0.1:47821` y devuelve `{status, body}` o `{error}` | Solo Foundation |

El popup solo manda el mensaje; el service worker inyecta, extrae y guarda (red), para que cerrar el popup no corte la petición. Sin `page-extract.js` (página restringida o fallo de inyección) cae a `innerText` y, si tampoco, guarda solo URL y título. Ninguno guarda estado en memoria: el token está en `storage.local`. No hay content scripts declarados.

## Reglas

1. **La lógica va en `client.js`** y se prueba en `client.test.js` con `fakeTransport`. Los ficheros de pantalla (`popup.js`, `options.js`) solo conectan el DOM con el cliente.
2. **Un transporte es `({method, path, token, body?}) => {status, body}`** y lanza solo cuando marks no responde. Safari usa `nativeTransport` (protocolo `safari-web-extension:`); el resto, `fetchTransport`. No llamar a `fetch` fuera de `fetchTransport`.
3. **Resultados como claves de mensaje** (`outcomeKey`, `saveTab`, `ping` devuelven `{key, profile?}`); la pantalla traduce con `t(key, profile)`. Ningún texto de usuario escrito en el código: texto nuevo = la misma clave en `_locales/es/messages.json` y `_locales/en/messages.json` (el nombre del appex, en `safari/{es,en}.lproj/InfoPlist.strings`, que `scripts/build-appex.sh` copia a `Contents/Resources/` del appex y `CFBundleLocalizations` declara). Una guarda en `scripts/check.sh` (`apps/desktop/src/i18n.guard.test.ts`) lo hace cumplir; lo que no es texto de usuario lleva `i18n-allow: <motivo>`. La API devuelve códigos de error, no texto: `client.js` los vuelve la clave `error_<código>` (guiones a `_`) y el test `extension_translates_every_api_error` (`api.rs`) exige una por cada código de `API_ERROR_CODES` (solo en tests); `_locales.test.js` exige las mismas claves y marcadores en `es` y `en`; una clave de error sin traducción cae en `serverError`. Código nuevo en la API = su `error_<código>` en `es` y en `en`.
4. **Constantes con nombre** para puertos, rutas, estados HTTP, claves de storage y timeouts (como `STATUS_TOO_LARGE`, `REQUEST_TIMEOUT_MS`).
5. **Texto al DOM con `textContent`**, nunca `innerHTML`.
6. **Ruta nueva de la API**: añadirla a `ROUTES` en `client.js` **y** a `allowedRoutes` en el manejador Swift (la lista blanca es la frontera de confianza del lado nativo), con su test.
7. **Contrato `api`/`MIN_API`**: `/v1/ping` devuelve `api` (`API_LEVEL` en `api.rs`) y `client.js` exporta `MIN_API`. `ping` da `appTooOld` si `api` falta o es menor que `MIN_API`; `outcomeKey` da `extensionTooOld` con un `404`, antes que la rama `error_`. `MIN_API` solo sube cuando la extensión usa algo de un nivel nuevo; `extension_files_agree_with_the_app` exige `1 <= MIN_API <= API_LEVEL`. La versión de la extensión (la de `manifest.json`, propia) se ve al pie de las opciones.
8. **Puerto `47821`**: vive en `manifest.json` (`host_permissions`), `client.js` (`API_PORT`), el `.swift` (`apiBase`) y `apps/desktop/src-tauri/src/api.rs` (`API_PORT`). Cambiarlos juntos; lo comprueba `extension_files_agree_with_the_app`.
9. **Permisos mínimos**: hoy `activeTab`, `scripting`, `storage`, `nativeMessaging` (solo en Safari: `build.mjs` lo quita del manifiesto de Chromium) y solo `http://127.0.0.1:47821/*`. No añadir `<all_urls>`, `tabs` ni host permissions amplios; si algo nuevo los parece necesitar, plantearlo antes.
10. **El token** solo vive en `storage.local` y en la cabecera `Authorization`; nunca se pasa a la función inyectada en la página, ni se registra en logs.
11. **Se lee sobre un clon, nunca sobre la página viva.** `marksExtract` pasa `document.cloneNode(true)` a Defuddle, así la página del usuario no cambia a la vista. Lo extraído es no confiable y lo trata la app: core lo acota y `ContentView` lo lee con sus listas blancas (ADR 0011).
12. **Cuerpo grande**: la app limita a 8 MB (`BODY_LIMIT` en `api.rs`); ante 413, `saveTab` reintenta una vez sin `text`. El Swift serializa con `.withoutEscapingSlashes` para que el tamaño coincida con Chromium.
13. **El manejador Swift siempre completa la petición**, también en los caminos de error (`bad_request`, `unreachable`). Si se añade una rama, debe terminar en `completion`.
14. **Trabajo que dura (red)** va en `background.js`, nunca en el popup; el listener comprueba `sender.id === ext.runtime.id` y el tipo de mensaje, y devuelve `true` para responder en asíncrono.
15. **Aspecto**: seguir `DESIGN.md` (amarillo solo como relleno con texto `on-primary` o indicador de 2px, nunca texto; datos en mono; sin sombras ni esquinas de más de 4px). Token nuevo: primero en `app.css`, luego copiado igual en `extension.css`. El color del punto de estado sale de `stateOf(key)`; una clave nueva de mensaje que no sea error debe ir en sus listas. `blocked`/`blockedIn` (la app dice `blocked: true`: la página pidió una verificación) cuentan como error.
16. **Idioma**: un locale nuevo lleva su mensaje `lang`; el `default_locale` es `en` (el idioma de reserva si el navegador no está en `es` ni `en`) y el HTML trae `lang="en"` hasta que `translatePage` lo sustituye.
17. **Identificador `dev.landygg.marks`**: en `client.js` (`NATIVE_APP_ID`) y `safari/Info.plist` (con `.safari-extension`). Ver la lista en `CLAUDE.md`.

## Verificación

- `node --test "apps/extension/*.test.js"`
- `xcrun swiftc -typecheck -parse-as-library -application-extension apps/extension/safari/SafariWebExtensionHandler.swift`
- Antes de commit: `scripts/check.sh` (incluye ambos).
- A mano: Chromium cargando `apps/extension/dist/chromium` (tras `node build.mjs`) con `pnpm --dir apps/desktop tauri dev`; Safari solo con `scripts/build-app.sh` (que llama a `scripts/build-appex.sh`; `scripts/verify-bundle.sh` comprueba el resultado). En la v1.0 Safari es experimental: sin Developer ID, solo carga la extensión con «Permitir extensiones no firmadas» (Ajustes → Desarrollador), que se apaga al cerrar Safari. Chromium es la recomendada y se publica como `marks-chromium-<versión>.zip` en cada release.
- Versión: subir `version` en `manifest.json` en cada release que cambie `apps/extension` o `packages/extract` (el appex y el zip de Chromium llevan `page-extract.js`, construido desde `packages/extract`).
- Bucle de Safari en desarrollo: subir `version` en `manifest.json` (el appex toma su versión de ahí), volver a registrar el appex con `lsregister -f` sobre `marks.app` y `pluginkit -a` sobre el appex, y reiniciar Safari; si no, Safari sigue con la extensión antigua.

## Más contexto

- Buenas prácticas externas, estado actual frente a ellas y fuentes: [references/best-practices.md](references/best-practices.md)
- Decisiones: `docs/adr/0005` (API loopback con token), `0010` (Defuddle en el navegador), `0007` (appex de Safari a mano).
- Guía web: antes de añadir o cambiar un componente, un formulario, un manejador de teclado o un estilo, consultar `modern-web-guidance` (`search` y luego `retrieve`) y filtrar cada recomendación con la política de motores de `AGENTS.md` («Navegadores»: Safari 18.0 y Chrome 123, sin polyfills).
