# Buenas prácticas de extensiones MV3 y estado de marks

Investigado el 2026-11-10. Las fuentes oficiales (Chrome, Apple, agentskills.io) pesan más que los blogs.

## Lo que recomiendan y cómo está marks

| Práctica | Fuente | Estado en `apps/extension` |
|---|---|---|
| Pedir solo los permisos que se usan; `activeTab` en vez de host permissions amplios | Chrome, WWDC20 | Cumple: `activeTab` + `scripting`, host solo `127.0.0.1:47821` |
| Declarar `content_security_policy` explícita (`default-src 'self'`) | Chrome | **No declarada.** MV3 aplica por defecto `script-src 'self'; object-src 'self'` a las páginas de la extensión, así que hoy no hay hueco; declararla solo lo haría explícito |
| No usar `innerHTML` / `document.write`; insertar texto plano | Chrome | Cumple: `textContent` |
| Tratar como no confiable lo que viene de la página | Chrome | Cumple: `marksExtract` solo lee, sobre un clon del documento; core decide qué se guarda y la app sanea el contenido al pintarlo |
| No pasar secretos a contexto de página | Chrome | Cumple: el token no entra en la función inyectada |
| HTTPS, no HTTP | Chrome | Excepción razonada: HTTP solo a loopback `127.0.0.1`, con token Bearer (ADR 0005) |
| Validar mensajes en el lado nativo y completar siempre la petición | Apple (guía general) | Cumple: lista blanca de rutas y método; todas las ramas llaman a `completion` |
| El estado no vive en memoria (los service workers mueren); usar `storage` | Chrome, guías MV3 | Cumple: `background.js` (service worker) no guarda nada en memoria; el token está en `storage.local` y llega con cada mensaje |
| Mismo código para varios navegadores con un shim de namespace | Ejemplos open source (Save In) | Cumple: `ext = browser ?? chrome` solo en `i18n.js`; transporte según protocolo |
| `externally_connectable` y `web_accessible_resources` mínimos | Chrome | Cumple: no declara ninguno |

## Riesgo resuelto: el guardado vivía en el popup

Si el popup se cerraba antes de que respondiera la app, su página moría y la petición se cortaba. Desde 2026-11-10 el popup solo manda el mensaje y la extracción y el guardado los hace `background.js` (service worker), que sobrevive al popup; un fallo deja `!` en el badge hasta que se vuelve a abrir el popup.

Pendiente de comprobar a mano en Safari: que acepta `"type": "module"` en `background` (un hilo antiguo del foro de Apple decía que no; un bug de WebKit de 2023 lo usa) y que `sendNativeMessage` funciona desde el worker.

## Organización de ficheros

Las guías coinciden en separar por contexto de ejecución (popup, opciones, background, content scripts) y aislar la lógica pura para probarla fuera del navegador. marks lo hace sin carpetas: un fichero por contexto (popup, opciones, background) y `client.js` como núcleo probado. Con el tamaño actual (~640 líneas) no hace falta `src/` ni TypeScript. El bundler llegó en v0.6: esbuild, con salida en `dist/chromium/` y `dist/safari/` desde la 1.2.0 (el navegador carga esas carpetas, no la de fuentes; solo cambia el manifiesto), porque el script que se inyecta para extraer con Defuddle tiene dependencias de npm (ver `docs/superpowers/specs/2026-11-12-v0.6-defuddle-extraction-design.md`).

## Fuentes

- Chrome, Stay secure: https://developer.chrome.com/docs/extensions/develop/security-privacy/stay-secure
- Apple, Messaging between the app and JavaScript in a Safari web extension: https://developer.apple.com/documentation/safariservices/messaging-between-the-app-and-javascript-in-a-safari-web-extension
- WWDC20, Meet Safari Web Extensions (notas): https://wwdcnotes.com/notes/wwdc20/10665/
- Foro de Apple, Browser Extension Native Messaging Security: https://developer.apple.com/forums/thread/760972
- Save In, Manifest V3 (código compartido entre navegadores): https://github.com/gyng/save-in/wiki/Manifest-V3
- Agent Skills, especificación: https://agentskills.io/specification
