# Privacidad

**Español** · [English](../en/privacy.md)

marks guarda tus datos en tu equipo y solo se conecta a la red para lo que tú le pides. Esta página dice exactamente qué sale y qué no.

## Resumen

| Pregunta | Respuesta |
|---|---|
| ¿Tiene cuentas o servidor propio? | No |
| ¿Envía telemetría, estadísticas o informes de errores? | No |
| ¿Sincroniza tus bookmarks con algún sitio? | No |
| ¿Descarga las páginas que guardas? | Sí: es su función |
| ¿Envía el contenido a un proveedor de IA? | Solo si activas la IA. Por defecto, la IA está desactivada, y el proveedor por defecto es local |
| ¿Comprueba si hay actualizaciones? | Sí, en GitHub, una vez al día. Se desactiva en Ajustes → Actualizaciones |

## Qué se queda en tu equipo

- Los bookmarks, su contenido, notas, etiquetas, grupos, resúmenes y vectores de búsqueda: en ficheros SQLite. Ver [Datos y copia de seguridad](data-and-backups.md).
- Las claves de API: en el llavero de macOS.
- Los registros de la app: en `~/Library/Logs/dev.landygg.marks/`. No se envían a ningún sitio.
- Las búsquedas: se resuelven en tu equipo. Con la IA activada, el texto de la búsqueda se envía a tu proveedor de embeddings para convertirlo en un vector; con Ollama local, tampoco sale.

Las bases de datos **no están cifradas**. Las protege lo mismo que al resto de tu carpeta de usuario: tu sesión de macOS y, si lo tienes activado, FileVault.

## Qué sale a la red

### 1. La descarga de las páginas que guardas

Cuando guardas una URL desde la app, la barra de menús o la terminal, marks pide esa página al sitio web, como haría un navegador.

- La petición se identifica con el agente de usuario `marks/<versión> (+https://github.com/landygg/marks)`, que indica a los administradores de cada sitio qué proyecto hace la petición.
- Va **sin tus cookies ni tu sesión**: el sitio no sabe quién eres más allá de tu dirección IP.
- Si hace falta, marks carga la página en una ventana oculta y privada para ejecutar su JavaScript. Esa página puede pedir sus propios recursos (scripts, imágenes), como en un navegador. La ventana no comparte nada con tu navegador, no tiene acceso a la app y se destruye al terminar.
- marks descarga también el **favicon** del sitio, una vez, y solo de direcciones públicas.

Lo que guardas desde la **extensión** no se vuelve a descargar: llega ya extraído desde tu navegador.

Importar un `bookmarks.html` **no** descarga nada. Las páginas se piden cuando tú ejecutas «Extraer».

### 2. La IA, si la activas

Con la IA activada, marks envía a tu proveedor:

- El texto de cada página, para resumirla y etiquetarla.
- Trozos del texto, para calcular los vectores de búsqueda.
- El texto de cada búsqueda, para la parte semántica.
- Con «Extraer con IA», el HTML limpio de la página.

| Proveedor | A dónde va |
|---|---|
| Ollama en `localhost` (por defecto) | A ningún sitio: se queda en tu equipo |
| Ollama, oMLX o un servidor compatible en otra máquina | A esa máquina |
| OpenAI o Anthropic | A sus servidores, con sus condiciones |

La app avisa en dos momentos: al extraer varios importados con la IA activada, y en Ajustes cuando el endpoint de «Extraer con IA» está fuera de tu equipo.

### 3. Las actualizaciones

marks comprueba en GitHub si hay una versión nueva.

- **Qué lleva:** la URL de la petición nombra el sistema (`darwin`) y la arquitectura del Mac (`aarch64`), y la cabecera User-Agent dice `tauri-plugin-updater/<versión>`, el nombre y la versión del componente que comprueba las actualizaciones. No lleva la versión de marks que tienes. Como en cualquier conexión, GitHub ve tu dirección IP. No lleva tus bookmarks, ni tu token, ni nada de tus datos.
- **Cuándo:** 30 segundos después de abrir marks y una vez al día mientras siga abierta, solo en Mac con Apple silicon; y cada vez que pulsas «Buscar actualizaciones».
- **Cómo se desactiva:** Ajustes → Actualizaciones → «Buscar automáticamente». Ver [Actualizaciones](updates.md).

### Y nada más

Al **leer** un bookmark, marks no carga nada remoto. Las imágenes, los vídeos y el contenido incrustado se muestran como una línea con «Abrir», y los enlaces se abren en tu navegador cuando tú los pulsas. Un sitio no puede saber cuándo relees una página guardada.

## La API local

Para atender a la extensión, marks abre un pequeño servidor en `127.0.0.1:47821`.

- Solo escucha en tu propio equipo: no es accesible desde tu red ni desde internet.
- Cada petición debe traer el token del perfil. marks comprueba además la cabecera `Host` y que la petición no venga de una página web.
- El token viaja en una cabecera, nunca en la URL.

Limitaciones que conviene conocer:

- Una página web puede detectar que marks está en marcha (que el puerto está abierto). No puede leer, añadir ni detectar bookmarks.
- La conexión es HTTP dentro de tu equipo, sin cifrar, y la extensión no puede comprobar que quien responde es marks. Un programa malicioso que ya se ejecute en tu Mac y ocupe el puerto 47821 antes que marks recibiría el token y las páginas que guardes. Si marks no puede abrir el puerto, deja una notificación y lo indica en Ajustes → «Extensión del navegador»: «El puerto 47821 está ocupado…».

## La extensión del navegador

- Solo actúa cuando pulsas su botón o su atajo.
- Solo lee la pestaña activa, y sobre una copia.
- Solo se conecta a marks en tu equipo.

Sus permisos están en [Extensión del navegador](browser-extension.md#permisos-que-pide).

## Lo que marks guarda de cada página

Además del texto: el título, la URL (la normalizada y la original), la descripción, el autor, el sitio, la fecha de publicación, el idioma, el favicon y la dirección de la imagen de portada (solo la dirección; la imagen no se descarga).

## Código abierto

marks es software libre con licencia MIT. Todo lo que dice esta página se puede comprobar en el código.
