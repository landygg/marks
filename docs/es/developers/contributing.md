# Contribuir

**Español** · [English](../../en/developers/contributing.md)

Gracias por querer mejorar marks. Esta página es la versión en español de [`CONTRIBUTING.md`](../../../CONTRIBUTING.md).

## Antes de empezar

- Lee [Arquitectura](architecture.md) y [Reglas del repositorio](rules.md).
- Prepara el entorno con [Compilar y verificar](building.md).
- Desarrolla siempre con una base de pruebas (`MARKS_DB`), nunca con tu base real.

La v1.0 es solo para macOS, y Apple Silicon es la única plataforma probada. Los cambios para Windows o Linux todavía no se integran.

## Issues

Los issues van a [GitHub Issues](https://github.com/landygg/marks/issues), con una plantilla para errores y otra para funciones.

Para informar de un error, incluye:

- La versión de marks y la de macOS, y si el Mac es Apple Silicon o Intel.
- Los pasos para reproducirlo, lo que esperabas y lo que pasó.
- El texto de la notificación o del error, si lo hay. El error de un bookmark se puede copiar desde su detalle.
- Si el fallo es de la extensión, el navegador y su versión.

Antes de adjuntar un log, una base de datos o un `bookmarks.html`, revísalo: contienen tus URL y el contenido de tus páginas. No pegues nunca el token de la extensión ni una clave de API.

Para un fallo de seguridad, no abras un issue público: infórmalo en privado con los [avisos de seguridad de GitHub](https://github.com/landygg/marks/security/advisories/new). Mira [`SECURITY.md`](../../../SECURITY.md).

Para proponer una función, abre antes un issue y explica el problema que resuelve. Un cambio grande sin acuerdo previo puede no integrarse.

## Pull requests

1. Crea una rama a partir de `main`.
2. Haz un cambio por pull request. Un arreglo y una reorganización no van juntos.
3. Añade o actualiza los tests. Al arreglar un error, añade un test que falle sin el arreglo.
4. Ejecuta `scripts/check.sh` y comprueba que pasa entero.
5. Describe en el pull request qué cambia, por qué y cómo lo probaste. Di también lo que no pudiste probar.

Lo que se revisa:

- La lógica está en `crates/core`, no en un comando Tauri, en el CLI ni en la extensión.
- Todo texto de usuario está en los catálogos, en español y en inglés. Mira [Idiomas y textos](i18n.md).
- No hay `unwrap()` fuera de los tests ni valores mágicos en línea.
- Un cambio de esquema es una migración nueva, al final de la lista. Mira [Migraciones](migrations.md).
- Un cambio de interfaz sigue [`DESIGN.md`](../../../DESIGN.md).
- Un permiso nuevo, de la app o de la extensión, viene explicado y es el más estrecho posible.
- La documentación de `docs/es` y `docs/en` refleja el cambio, en los dos idiomas.

## Estilo de commit

- [Conventional Commits](https://www.conventionalcommits.org/es/): `tipo(ámbito): resumen`, en inglés, en imperativo y sin punto final. Por ejemplo: `feat(desktop): add a Corners setting to Appearance`.
- Tipos: `feat`, `fix`, `docs`, `build`, `ci`, `chore`, `refactor`, `test` y `perf`.
- Ámbitos: `core`, `desktop`, `extension`, `cli`, `i18n` y `api`. Sin ámbito si el cambio toca varios.
- El mensaje describe el efecto, no los ficheros tocados.
- No menciones herramientas de IA ni asistentes en commits, pull requests o issues.

## Verificación

`scripts/check.sh` es la verificación completa: formato, lints y tests de Rust, del frontend, de la extensión y de `packages/extract`, y la comprobación de tipos del manejador Swift. Sus pasos están en [Compilar y verificar](building.md#scriptschecksh).

Lo que los tests no cubren se prueba a mano con `tauri dev` y una base de pruebas. Si tu cambio toca la extensión de Safari, pruébalo con la app construida por `scripts/build-app.sh`: `tauri dev` no la incluye.

## Licencia

marks se publica bajo la licencia MIT, y las contribuciones entran bajo esa misma licencia, sin DCO ni CLA que firmar.
