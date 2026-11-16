# Actualizaciones

**Español** · [English](../en/updates.md)

## La app

marks se actualiza desde las versiones publicadas en GitHub, en <https://github.com/landygg/marks/releases>. Hay un solo canal: las versiones estables.

- **Cuándo comprueba:** 30 segundos después de abrirse y, mientras siga abierta, una vez al día. También a mano, con «Buscar actualizaciones» en Ajustes → Actualizaciones.
- **Cómo avisa:** con una notificación «Hay una versión nueva de marks: …». Su botón de Ajustes lleva a Ajustes → Actualizaciones.
- **Pide confirmación:** no descarga ni instala nada hasta que pulsas «Instalar … y reiniciar». Entonces descarga la versión nueva, comprueba su firma, la instala y reinicia marks.
- **Se puede desactivar:** el interruptor «Buscar automáticamente», en Ajustes → Actualizaciones. Con él apagado, marks no se conecta a GitHub salvo cuando pulsas «Buscar actualizaciones».
- **Solo en Mac con Apple silicon.** En un Mac con Intel, la sección dice que descargues las versiones nuevas desde GitHub, y no comprueba sola. Si una comprobación automática falla (sin conexión, por ejemplo), no avisa: lo intenta otra vez al día siguiente.

Para actualizar a mano, descarga la versión nueva y sustituye marks en la carpeta Aplicaciones. Tus datos no están dentro de la app: siguen en `~/Library/Application Support/marks/`. Cada versión descargada a mano trae la marca de descarga, así que macOS vuelve a bloquearla la primera vez que la abres: ábrela otra vez con «Abrir igualmente» (ver [Instalación](installation.md#la-primera-vez-abrir-igualmente)). En un Mac con Intel, que no se actualiza solo, pasa en cada actualización.

### Qué pasa con tus datos al actualizar

Una versión nueva puede cambiar el formato de la base de datos (una migración). marks lo hace al abrirse, sin preguntar, y con dos protecciones:

- Hace una **copia de cada base antes de migrarla**.
- Una versión **antigua** de marks se niega a abrir una base más nueva que no puede leer, y lo dice con un mensaje claro.

**No hay vuelta atrás automática.** Si después de actualizar quieres volver a la versión anterior, tienes que restaurar la copia hecha antes de la migración; lo guardado después no estará en ella. Ver [Datos y copia de seguridad](data-and-backups.md#copias-automáticas-antes-de-una-migración).

Antes de una actualización grande, una copia manual de la carpeta de datos no está de más.

### Qué se conecta a la red

La comprobación de actualizaciones contacta con GitHub. Es la única conexión que marks hace sin que tú guardes nada. Ver [Privacidad](privacy.md).

## La extensión de Safari

Es experimental: Safari solo la carga con «Permitir extensiones no firmadas». Mira [Extensión del navegador](browser-extension.md#safari-experimental).

En la v1.0 va dentro de marks.app, así que **se actualiza con la app**. Tiene su propia versión, y la app le dice si es compatible con su versión de la API.

Después de actualizar marks, si Safari sigue mostrando la extensión antigua, cierra Safari y vuelve a abrirlo.

Si no son compatibles, la extensión lo dice al probar la conexión o al guardar:

- «Esta versión de marks es demasiado antigua para la extensión. Actualiza marks.»: actualiza la app (Ajustes → Actualizaciones, o descárgala desde [GitHub](https://github.com/landygg/marks/releases)).
- «La extensión es demasiado antigua para esta versión de marks. Actualízala.»: en Chromium, descomprime la versión nueva en la misma carpeta, sustituyendo la vieja, y pulsa «Recargar» en `chrome://extensions`; en Safari, cierra Safari y vuelve a abrirlo, porque la extensión llega con la app.

Más adelante, si se paga el Apple Developer Program, que lo exige, la extensión pasará a una app propia en el Mac App Store y se actualizará desde allí, con su propio ciclo.

## La extensión de Chromium

- Cargada sin empaquetar: no se actualiza sola. Descomprime la versión nueva en la misma carpeta, sustituyendo la vieja, y pulsa «Recargar» en `chrome://extensions`. Así el token se conserva: el navegador identifica una extensión sin empaquetar por su carpeta, y si la cargas desde otra, es una extensión nueva sin token.

La v1.0 no está en la Chrome Web Store.

## Saber qué versión tienes

- **La app:** menú marks → «Acerca de marks».
- **La línea de comandos:** `marks --version`.

- **La extensión** (Safari o Chromium): al pie de sus opciones, «Versión 1.0.0».

Las novedades de cada versión están en [CHANGELOG.md](../../CHANGELOG.md), en inglés, y en las notas de cada versión en [GitHub Releases](https://github.com/landygg/marks/releases).
