# Problemas frecuentes y preguntas

**Español** · [English](../en/troubleshooting.md)

Antes de nada, abre el panel de notificaciones (la campana de la barra de estado): casi todos los fallos dejan allí un aviso con su causa.

## Guardar y extraer

### Un bookmark está «Fallido»

Abre su detalle: muestra el error. Usa «Copiar mensaje» si vas a informar del problema.

| Mensaje | Qué hacer |
|---|---|
| «la página pidió una verificación antibots» | Ábrela en el navegador, resuelve la verificación y guárdala con la [extensión](browser-extension.md) |
| «la respuesta no es una página HTML» | Es un PDF u otro fichero. marks solo guarda páginas HTML |
| «demasiadas redirecciones» | El sitio redirige en bucle. Prueba con la URL final |
| «la webview no terminó de cargar la página a tiempo» | El sitio es lento o no responde. «Reintentar» más tarde |
| «destino no admitido: solo http(s) remoto» | La URL apunta a tu propio equipo o a una red privada. marks no descarga de ahí |
| Un error de estado HTTP (404, 500…) | La página no existe o el servidor falla. «Reintentar» más tarde |

### El contenido sale incompleto o vacío

- Prueba «Volver a extraer».
- Si la página necesita tu sesión (un muro de pago, una red social), guárdala con la extensión desde el navegador.
- Con la IA activada, prueba «Extraer con IA».

### «la nueva extracción salió escasa: el contenido no ha cambiado»

«Volver a extraer» o «Extraer con IA» obtuvo menos texto del mínimo. marks conserva el contenido que ya tenías.

### Un bookmark se queda en «Sin resumir»

Es lo normal con la IA desactivada. Con la IA activada, mira las notificaciones: el modelo puede no estar disponible. Para volver a intentarlo: Ajustes → «Mantenimiento» → «Procesar bookmarks sin resumen».

### Importé mis bookmarks y están «Sin extraer»

Es intencionado: importar no descarga nada. Ver [Importar y exportar](import-export.md#lo-importado-no-se-extrae-solo).

### Guardé una URL y no aparece un bookmark nuevo

Ya estaba guardada. marks compara las URL sin el fragmento `#…` ni los parámetros `utm_…`, y no duplica.

### Las imágenes no se ven

Es intencionado: al leer, marks no carga nada remoto. Cada imagen es una línea con «Abrir». Ver [Privacidad](privacy.md).

## Buscar

### La búsqueda no encuentra algo que sé que guardé

- La búsqueda es por perfil. Comprueba que estás en el perfil correcto.
- Tienen que aparecer **todas** las palabras que escribes. Prueba con menos.
- Si el bookmark está «Sin extraer» o «Fallido», no tiene contenido en el que buscar.
- La URL no entra en la búsqueda.

### La búsqueda no encuentra por significado

La búsqueda semántica necesita la IA activada, un modelo de embeddings y bookmarks en estado «Listo». Con Anthropic no hay embeddings. Si acabas de cambiar de modelo de embeddings, espera a que marks termine de reindexar.

## Inteligencia artificial

### «Probar conexión» falla

- **Ollama:** comprueba que está en marcha (`ollama list` en la terminal) y que has descargado los dos modelos. El endpoint por defecto es `http://localhost:11434`.
- **«este proveedor requiere un endpoint»:** oMLX y «Compatible con OpenAI» lo necesitan.
- **«este proveedor requiere una clave de API»:** escríbela y pulsa «Guardar».
- «Probar conexión» prueba lo **guardado**: guarda antes de probar.

### «IA con error» en la barra de estado

El modelo de resumen no se pudo preparar o su última llamada falló. Pasa el puntero por «IA con error» para ver el error, y haz clic para abrir Ajustes → «Inteligencia artificial». Comprueba que el proveedor está en marcha (con Ollama, que la app está abierta y el modelo descargado) y pulsa «Probar conexión»: si sale bien, el error se borra. También se borra con la siguiente búsqueda o el siguiente bookmark que salgan bien, o al guardar los ajustes.

### «la IA está activada, pero el modelo no se pudo preparar»

Los ajustes guardados no permiten crear el modelo (falta la clave, el endpoint o un nombre de modelo). Revisa Ajustes → «Inteligencia artificial».

### «la entrada no cabe en el contexto que marks pide a Ollama»

La página es demasiado larga para ese modelo. Prueba con otro modelo.

### «Extraer con IA» está deshabilitado

Necesita la IA activada y un modelo de extracción válido. Ver [Inteligencia artificial](ai.md#extraer-con-ia).

### El equipo va lento mientras marks resume

Un modelo local usa mucha memoria y procesador. marks procesa los bookmarks de uno en uno. Puedes usar un modelo más pequeño, o desactivar la IA y volver a activarla cuando te venga bien: los bookmarks esperan en «Sin resumir».

## Extensión del navegador

### «marks no responde. Abre la app e inténtalo de nuevo.»

marks está cerrado, o no pudo abrir su puerto. Abre marks y mira Ajustes → «Extensión del navegador»: debe decir «API escuchando en 127.0.0.1:47821.»

### «El puerto 47821 está ocupado»

Otro programa, u otra copia de marks, usa ese puerto; la barra de estado lo dice con «Puerto 47821 ocupado». Cierra la otra copia y reinicia marks. El puerto no se puede cambiar. Para ver qué programa lo usa: `lsof -nP -iTCP:47821 -sTCP:LISTEN` en Terminal.

### «El token no es válido.»

El token se regeneró o su perfil se eliminó. Copia el actual de Ajustes → «Extensión del navegador» y pégalo en las opciones de la extensión.

### La extensión guarda en el perfil equivocado

Guarda en el perfil dueño del token, no en el perfil activo de la app. Pega el token del perfil que quieras.

### La extensión no aparece en Safari

La extensión de Safari es experimental: Safari solo la carga con «Permitir extensiones no firmadas» (Ajustes → Desarrollador), y lo desactiva cada vez que se cierra.

- Cada vez que abras Safari, vuelve a activar «Permitir extensiones no firmadas». Si no ves la pestaña Desarrollador, activa antes «Mostrar funciones para desarrolladores web» en Ajustes → Avanzado.
- Comprueba que marks está en `/Applications` y que lo has abierto al menos una vez.
- En Terminal, `pluginkit -m -i dev.landygg.marks.safari-extension` debe listarla.
- Si hay dos copias de marks.app (por ejemplo, una vieja en Descargas), borra la que sobra y vuelve a abrir Safari.
- Para no repetir estos pasos, usa la extensión de Chromium: ver [Extensión del navegador](browser-extension.md).

### El icono de la extensión muestra `!`

El último guardado falló después de cerrar la ventanita. Ábrela para ver el motivo.

## La app

### macOS dice que no puede comprobar marks

«Apple no ha podido verificar que marks no contenga software malicioso…»: marks está firmada, pero no notarizada por Apple. Pasa la primera vez y con cada versión que descargues a mano (en Intel, en cada actualización).

1. Pulsa «Aceptar» en el aviso.
2. Ajustes del Sistema → Privacidad y seguridad → «Abrir igualmente», y confirma.

O, en Terminal: `xattr -dr com.apple.quarantine /Applications/marks.app`. Más detalle en [Instalación](installation.md#la-primera-vez-abrir-igualmente).

### marks pide la contraseña del llavero tras actualizar

Para leer la clave de API del proveedor de IA. Escribe la contraseña de tu sesión y pulsa «Permitir siempre»: no vuelve a pedirla hasta la siguiente actualización.

### Cerré la ventana y marks sigue en marcha

Es intencionado: marks sigue en la barra de menús para extraer lo pendiente y atender a la extensión. Para cerrarla: icono de la barra de menús → «Salir», o ⌘Q.

### Cambié el idioma y la app se reinició

Es necesario: el idioma se aplica al arrancar.

### marks no abre y habla de la base de datos

marks avisa con una alerta y se cierra. Hay tres mensajes:

- «Esta base de datos (<ruta>) es de una versión más nueva de marks y esta versión no sabe abrirla. Instala la versión más reciente de marks, o restaura la copia hecha antes de la migración.»

  Has vuelto a una versión anterior de marks, y esa versión no puede leer una base ya migrada por una más nueva. Vuelve a instalar la versión nueva, o restaura la copia hecha antes de la migración.
- «marks no reconoce esta base de datos (<ruta>): no la ha creado marks 1.0 ni una versión posterior. No se ha tocado.»

  La base es de una versión de prueba anterior a la 1.0, o el fichero no es de marks. marks no la abre ni la cambia.
- «No se pudo copiar la base de datos a <ruta> antes de actualizarla, así que no se ha tocado. Comprueba que hay espacio libre en el disco y vuelve a abrir marks.»

  marks tenía que actualizar el formato de la base y no pudo hacer antes la copia de seguridad, normalmente por falta de espacio. Libera espacio en el disco y vuelve a abrir marks.

Ver [Datos y copia de seguridad](data-and-backups.md).

### Dónde están los registros

En `~/Library/Logs/dev.landygg.marks/`. Antes de adjuntarlos a un informe, revísalos: pueden contener direcciones de páginas que has guardado.

## Preguntas

**¿Necesito una cuenta?** No.

**¿Funciona sin conexión?** Sí, para leer, buscar y ordenar. Guardar una página nueva necesita red para descargarla.

**¿Puedo usar marks sin IA?** Sí. Viene desactivada.

**¿Mis páginas salen de mi equipo?** Solo si activas la IA con un proveedor remoto. Ver [Privacidad](privacy.md).

**¿Puedo sincronizar entre dos Mac?** No. Puedes mover la carpeta de datos de un Mac a otro, pero no usarla desde dos a la vez.

**¿Hay versión para Windows, Linux o móvil?** Todavía no. Windows y Linux llegarán más adelante.

**¿Funciona en un Mac con Intel?** Se publica, pero no está probado.

**¿Qué navegadores tienen extensión?** Safari y los Chromium (Chrome, Brave). Firefox, no.

**¿Puedo buscar en todos los perfiles a la vez?** No.

**¿Puedo recuperar un bookmark o un perfil eliminado?** No, salvo desde una copia de seguridad.

**¿marks guarda las imágenes de las páginas?** No. Guarda el texto y el favicon.

**¿Puedo leer mis datos sin marks?** Sí. Son ficheros SQLite estándar, y `marks export` los escribe como Markdown. Ver [Línea de comandos](cli.md).

**¿Puedo cambiar los atajos?** Los de la app, no. El de la extensión, sí, desde el navegador.

**¿Cómo informo de un problema?** Abre un issue en [GitHub](https://github.com/landygg/marks/issues) con la plantilla de error. Revisa antes los registros que adjuntes (ver [Dónde están los registros](#dónde-están-los-registros)). Un fallo de seguridad, en privado: ver [`SECURITY.md`](../../SECURITY.md).
