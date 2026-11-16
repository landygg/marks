# API local

**Español** · [English](../../en/developers/local-api.md)

La app de escritorio abre una API HTTP para la extensión del navegador. El código está en `apps/desktop/src-tauri/src/api.rs`. La API no decide nada: comprueba la petición y llama a `core::capture`.

- Escucha solo en `127.0.0.1:47821`, mientras la app está abierta.
- No tiene CORS, a propósito: ninguna página web debe poder usarla.
- Si el puerto está ocupado al arrancar, la app sigue funcionando sin API y deja una notificación.

## Comprobaciones

Cada petición pasa por un guardián antes de llegar a su ruta, también si la ruta no existe. Comprueba, en este orden:

1. **`Host`.** Debe ser `127.0.0.1:47821` o `localhost:47821`. Si no, `403` con el código `host`.
2. **`Origin`.** Si la cabecera viene, su esquema debe ser el de una extensión: `safari-web-extension`, `chrome-extension` o `moz-extension`. Si no, `403` con el código `origin`. Una petición sin `Origin`, como la de `curl`, pasa.
3. **Token.** Va en `Authorization: Bearer <token>`, nunca en la URL. Si falta o no es de ningún perfil, `401` con el código `token`.

Después comprueba el tamaño declarado en `Content-Length`.

## Token y perfil

Cada perfil de marks tiene su token, de 32 bytes aleatorios. El token decide en qué perfil se guarda: la petición no nombra el perfil.

- core compara el token en tiempo constante y nunca acepta uno vacío.
- Regenerar el token en Ajustes invalida el anterior.
- El token de un perfil eliminado da `401`.

## Rutas

### `GET /v1/ping`

Comprueba la conexión y el token.

```json
{ "app": "marks", "version": "1.0.0", "api": 1, "profile": "Personal" }
```

`version` es la versión de la app. `profile` es el nombre del perfil dueño del token.

`api` es el nivel de la API (`API_LEVEL` en `api.rs`). La extensión declara el mínimo con el que funciona (`MIN_API` en `apps/extension/client.js`):

- `api` que falta cuenta como 0. Si `api` es menor que `MIN_API`, la extensión dice que la app es demasiado antigua (`appTooOld`).
- Una ruta que responde `404` quiere decir que la app ya no la tiene: la extensión dice que ella es demasiado antigua (`extensionTooOld`).
- `API_LEVEL` solo sube con cambios **aditivos** de la API que una extensión pueda necesitar (un campo o una ruta nuevos). Un cambio que rompe va con otro prefijo (`/v2/…`), y las rutas viejas se quitan cuando ya no las use ninguna extensión publicada.
- `MIN_API` solo sube cuando la extensión empieza a usar algo de un nivel nuevo. Un test (`extension_files_agree_with_the_app`) exige `1 <= MIN_API <= API_LEVEL`.

### `POST /v1/bookmarks`

Guarda una página. El cuerpo es JSON:

| Campo | Tipo | Obligatorio | Contenido |
|---|---|---|---|
| `url` | texto | sí | La URL de la página. Solo `http` o `https` |
| `title` | texto | no | El título de la pestaña |
| `page` | objeto | no | Lo que extrajo Defuddle en el navegador |
| `text` | texto | no | El `innerText` del documento. Se usa si `page` trae poco contenido |

`page` lleva `title` (opcional), `markdown` (obligatorio) y `meta` (opcional). `meta` admite `description`, `author`, `site`, `published`, `language`, `image`, `favicon`, `word_count` y `extractor`. Todo llega de una página ajena, así que core lo recorta y lo valida antes de guardarlo.

Respuesta:

```json
{ "id": 42, "created": true, "status": "pending", "blocked": false, "profile": "Personal" }
```

| Campo | Significado |
|---|---|
| `id` | El `id` del bookmark en su perfil |
| `created` | `true` si es nuevo; `false` si la URL ya estaba |
| `status` | El [estado](architecture.md#estados-de-un-bookmark) del bookmark |
| `blocked` | `true` si la página era una verificación antibots. Su contenido no se guarda |
| `profile` | El nombre del perfil en el que se guardó |

Si la URL ya estaba, el bookmark solo recibe el contenido cuando no tenía o estaba en `failed`. En otro caso solo se completan los metadatos que faltaban. Capturar una URL en estado `imported` la pone en cola.

## Errores

Un error devuelve `{ "error": "<código>" }`. La API no devuelve texto: la extensión traduce cada código con la clave `error_<código>` de sus `_locales`, con los guiones cambiados por `_`.

| Código | HTTP | Causa |
|---|---|---|
| `host` | 403 | La cabecera `Host` no es la esperada |
| `origin` | 403 | El `Origin` no es de una extensión |
| `token` | 401 | Falta el token o no es válido |
| `bad-request` | 400 | El cuerpo no es el JSON esperado |
| `too-large` | 413 | El cuerpo pasa de 8 MB |
| `invalid-url` | 400 | La URL no es válida |
| `unsupported-scheme` | 400 | La URL no es `http` ni `https` |
| `internal` | 500 | Fallo de la app. El detalle queda en el log, no en la respuesta |

`invalid-url` y `unsupported-scheme` son códigos de `Msg`, los mismos que core usa en los datos guardados.

## Tamaño del cuerpo

El límite es de 8 MB (`BODY_LIMIT`). Ante un `413`, la extensión reintenta una vez sin el campo `text`.

Cuando rechaza una petición, la API lee y descarta su cuerpo antes de responder, con un tope de bytes y de 5 segundos. Sin eso, el cliente recibiría un corte de conexión en lugar del error.

## Safari

En Safari, un `fetch` de la extensión a loopback falla. Por eso la extensión usa mensajería nativa:

1. `client.js` elige `nativeTransport` cuando el protocolo de la extensión es `safari-web-extension:`. En los demás navegadores usa `fetchTransport`.
2. El manejador Swift (`SafariWebExtensionHandler.swift`) recibe `{method, path, token, body?}`.
3. Comprueba la forma del mensaje y que la ruta y el método estén en su lista blanca: `GET /v1/ping` y `POST /v1/bookmarks`.
4. Reenvía la petición a `http://127.0.0.1:47821` y devuelve `{status, body}`.

Si no puede contactar con la app, devuelve `{error: "unreachable"}`. Si el mensaje no es válido, `{error: "bad_request"}`. La lista blanca es la frontera de confianza del lado nativo.

## Probar con curl

Con la app en marcha sobre una base de pruebas (`MARKS_DB`) y el token copiado de Ajustes:

```sh
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:47821/v1/ping

curl -X POST http://127.0.0.1:47821/v1/bookmarks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "title": "Example"}'
```

## Añadir una ruta

1. Añade la ruta y su manejador al `router` de `api.rs`. El manejador recibe el `Profile` que dejó el guardián y llama a core; el trabajo bloqueante va en `spawn_blocking`.
2. Añádela a `ROUTES` en `apps/extension/client.js`.
3. Añádela a `allowedRoutes` en `SafariWebExtensionHandler.swift`.
4. Si devuelve un código de error nuevo, añádelo a `API_ERROR_CODES` y añade su clave `error_<código>` a `_locales/es` y `_locales/en`.
5. Escribe los tests: en `api.rs`, con `tower::ServiceExt::oneshot` sobre el router, sin abrir el puerto; y en `client.test.js`, con un transporte falso.

## Límites conocidos

- Una página web puede detectar que marks está en marcha, porque el puerto está abierto. No puede leer, añadir ni detectar bookmarks.
- El servidor no se identifica ante la extensión. Un proceso local que ocupe el puerto 47821 antes que marks recibiría el token y el contenido de cada página guardada.
- La extensión no tiene cola: con marks cerrado no guarda.
