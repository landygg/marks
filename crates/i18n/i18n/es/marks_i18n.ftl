## errors-store

error-chain = { $head }: { $rest }

error-no-content = sin contenido
error-bookmark-not-found = bookmark no encontrado
error-store-poisoned = el almacén quedó inutilizable tras un fallo anterior
error-stores-poisoned = el mapa de almacenes abiertos quedó inutilizable tras un fallo anterior
error-unknown-status = estado desconocido: { $value }
error-unknown-error-kind = tipo de error desconocido: { $value }
error-open-database = no se pudo abrir { $path }
error-database-too-new = Esta base de datos ({ $path }) es de una versión más nueva de marks y esta versión no sabe abrirla. Instala la versión más reciente de marks, o restaura la copia hecha antes de la migración.
error-database-unknown = marks no reconoce esta base de datos ({ $path }): no la ha creado marks 1.0 ni una versión posterior. No se ha tocado.
error-database-unreadable = marks no puede leer esta base de datos ({ $path }): no es una base de datos SQLite o está dañada. No se ha tocado. Si marks la copió antes de alguna actualización, las copias están en la carpeta backups, junto a ella.
error-backup = No se pudo copiar la base de datos a { $path } antes de actualizarla, así que no se ha tocado. Comprueba que hay espacio libre en el disco y vuelve a abrir marks.
error-invalid-url = URL no válida
error-group-name-empty = el nombre del grupo no puede estar vacío
error-group-exists = ya existe un grupo con ese nombre
error-group-not-found = grupo no encontrado
error-profile-not-found = perfil no encontrado
error-profile-name = el nombre del perfil debe tener entre 1 y { $max } caracteres
error-profile-initials = las iniciales deben tener 1 o { $max } caracteres
error-profile-exists = ya existe un perfil con ese nombre
error-default-profile = el primer perfil no se puede eliminar
error-same-profile = el bookmark ya está en ese perfil
error-url-in-profile = ese bookmark ya existe en el perfil «{ $profile }»
error-file-too-large = el fichero pasa de { $max_mb } MB
error-not-utf8 = el fichero no está en UTF-8

## errors-fetch

error-blocked = la página pidió una verificación antibots
error-blocked-bookmark = la página pidió una verificación antibots: rescátala desde la extensión del navegador
error-unsupported-scheme = solo se pueden guardar URL http o https
error-destination = destino no admitido: solo http(s) remoto
error-not-html = la respuesta no es una página HTML
error-too-many-redirects = demasiadas redirecciones
error-not-image = la respuesta no es una imagen admitida
error-favicon-too-large = el favicon supera el tamaño máximo
error-no-location = redirección sin destino
error-fetch-task = la tarea de descarga terminó de forma inesperada
error-thin-result = la nueva extracción salió escasa: el contenido no ha cambiado

## errors-ai

error-extract-timeout = la extracción con IA no terminó a tiempo
error-embed-chunk-count = el modelo devolvió un número de vectores distinto al de trozos
error-embed-dim = el modelo devolvió vectores vacíos o de dimensiones distintas
error-vectors-len-mismatch = el número de trozos y de vectores no coincide
error-vectors-dim-mismatch = la dimensión del vector no coincide con la del índice
error-vectors-no-index = el índice vectorial no está inicializado
error-vectors-bad-dim = la dimensión guardada del índice no es válida
error-no-json = la respuesta del modelo no contiene un objeto JSON
error-invalid-json = el JSON del modelo no tiene la forma esperada
error-api-key-required = este proveedor requiere una clave de API
error-anthropic-no-embeddings = Anthropic no ofrece embeddings: elige otro proveedor
error-client-build = no se pudo crear el cliente del modelo
error-chat-failed = falló la llamada al modelo de chat
error-embed-failed = falló la llamada al modelo de embeddings
error-empty-chat = el modelo no devolvió texto
error-embed-batch-count = el modelo devolvió un número de vectores distinto al de textos
error-ollama-too-long = la entrada no cabe en el contexto que marks pide a Ollama: la página es demasiado larga para este modelo
error-ollama-status = Ollama respondió con error { $status }: { $detail }
error-ollama-bad-response = la respuesta de Ollama no tiene la forma esperada
error-endpoint-required = este proveedor requiere un endpoint
error-empty-model = los nombres de modelo no pueden estar vacíos
error-invalid-settings = los ajustes de IA guardados no son válidos
error-empty-extraction-model = el modelo de extracción no puede estar vacío
error-keyring-entry = no se pudo abrir la entrada del llavero
error-keyring-read = no se pudo leer la clave de API del llavero
error-keyring-write = no se pudo guardar la clave de API en el llavero
error-keyring-delete = no se pudo borrar la clave de API del llavero
error-token-random = no se pudo generar el token de la API

## errors-app

error-webview-url = URL no admitida en la webview: solo http(s) remoto
error-webview-timeout = la webview no terminó de cargar la página a tiempo: { $stage }
error-webview-decode = la webview devolvió una respuesta no decodificable
error-webview-closed-loading = la webview se cerró antes de terminar de cargar
error-webview-closed-html = la webview se cerró antes de devolver el HTML
error-parser-page = la ventana parser no devolvió una página válida
error-parser-clean = la ventana parser no devolvió un HTML limpio válido
error-parser-timeout = la ventana parser no terminó de extraer a tiempo: { $stage }
error-parser-closed-loading = la ventana parser se cerró antes de terminar de cargar
error-parser-closed-page = la ventana parser se cerró antes de devolver la página
error-no-pending-import = no hay ninguna importación preparada
error-import-other-profile = la importación se preparó para otro perfil
error-import-read = no se pudo leer { $path }
error-no-model = no hay ningún modelo de IA configurado
error-no-extraction-model = no hay ningún modelo de extracción configurado
error-export-write = no se pudo escribir el fichero exportado
error-stage-open = abriendo la ventana
error-stage-load = esperando a que termine de cargar
error-stage-settle = esperando a que la página se asiente
error-stage-eval = esperando la respuesta del script
error-stage-ready = esperando a que el módulo de extracción esté listo

## data

data-untagged-folder = Sin etiqueta
data-reading-list-tag = lista de lectura
data-notes-heading = ## Notas
data-default-profile-name = Personal
data-default-profile-initials = PE

## app
app-window-title = marks
tray-open = Abrir marks
tray-add = Añadir URL…
tray-quit = Salir
app-import-dialog-title = Importar bookmarks
app-export-dialog-title = Exportar bookmarks
app-html-filter = Fichero HTML
app-model-unavailable = la IA está activada, pero el modelo no se pudo preparar
app-profiles-failed = no se pudo leer el registro de perfiles
app-store-failed = fallo del almacén al procesar los pendientes
app-favicons-failed = no se pudieron guardar los favicons
app-profile-open-failed = no se pudo abrir el perfil
app-api-port-busy = la API local para la extensión no pudo abrir el puerto { $port }
app-api-stopped = la API local para la extensión se detuvo
app-empty-notification = error sin mensaje
app-update-available = Hay una versión nueva de marks: { $version }.
app-update-check-failed = No se pudo comprobar si hay versión nueva. Revisa la conexión.
app-update-install-failed = No se pudo instalar la versión nueva.
app-update-needs-macos = marks { $version } necesita macOS { $minimum } o posterior, y este Mac tiene macOS { $current }. marks sigue en esta versión.
menu-about = Acerca de marks
menu-services = Servicios
menu-hide = Ocultar marks
menu-hide-others = Ocultar otros
menu-quit = Salir de marks
menu-close-window = Cerrar ventana
menu-undo = Deshacer
menu-redo = Rehacer
menu-cut = Cortar
menu-copy = Copiar
menu-paste = Pegar
menu-select-all = Seleccionar todo
menu-fullscreen = Activar pantalla completa
menu-minimize = Minimizar
menu-maximize = Zoom
menu-file = Archivo
menu-edit = Edición
menu-view = Visualización
menu-window = Ventana
menu-help = Ayuda

## cli
cli-about = Bookmarks locales con búsqueda de texto completo
cli-help-db = Ruta de la base de datos SQLite (variable { $env }; por defecto, { $default })
cli-help-add = Encola la URL; la app la descarga y extrae al abrirse
cli-help-title = Título manual (si no, la app usa el de la página)
cli-help-notes = Notas del usuario
cli-help-tag = Etiqueta (repetible)
cli-help-profile = Perfil sobre el que operar; por defecto, el primero
cli-help-profiles = Lista los perfiles
cli-help-list = Lista todos los bookmarks
cli-help-search = Búsqueda de texto completo
cli-help-export = Exporta un .md por bookmark en el directorio
cli-help-import = Importa un fichero de marcadores (Netscape HTML); quedan sin extraer hasta pedirlo en la app
cli-help-export-html = Exporta todos los bookmarks a un fichero de marcadores (Netscape HTML)
cli-help-stats = Cuenta los bookmarks por estado
cli-usage-root = marks [OPCIONES] <COMANDO>
cli-usage-add = marks add [OPCIONES] <URL>
cli-usage-profiles = marks profiles [OPCIONES]
cli-usage-list = marks list [OPCIONES]
cli-usage-search = marks search [OPCIONES] [CONSULTA]...
cli-usage-export = marks export [OPCIONES] <DIRECTORIO>
cli-usage-import = marks import [OPCIONES] <FICHERO>
cli-usage-export-html = marks export-html [OPCIONES] <FICHERO>
cli-usage-stats = marks stats [OPCIONES]
cli-value-command = COMANDO
cli-value-db = RUTA
cli-value-profile = NOMBRE
cli-value-title = TÍTULO
cli-value-notes = NOTAS
cli-value-tag = ETIQUETA
cli-value-url = URL
cli-value-query = CONSULTA
cli-value-dir = DIRECTORIO
cli-value-file = FICHERO
cli-exported =
    { $n ->
        [one] { $n } bookmark exportado en { $path }
       *[other] { $n } bookmarks exportados en { $path }
    }
cli-import-summary =
    { $new ->
        [one] { $new } nuevo
       *[other] { $new } nuevos
    }, { $existing ->
        [one] { $existing } ya estaba
       *[other] { $existing } ya estaban
    }, { $skipped ->
        [one] { $skipped } no importable
       *[other] { $skipped } no importables
    }
cli-row-error = (error: { $error })
cli-status-pending = pendiente
cli-status-pending-embed = pendiente de indexar
cli-status-ready = listo
cli-status-failed = fallido
cli-status-imported = sin extraer
cli-stats-total = total: { $n }
cli-stats-pending = pendientes: { $n }
cli-stats-failed = fallidos: { $n }
cli-stats-unsummarized = sin resumen: { $n }
cli-stats-imported = sin extraer: { $n }
cli-usage = Uso:
cli-arguments-heading = Argumentos
cli-options-heading = Opciones
cli-commands-heading = Comandos
cli-help-flag = Muestra la ayuda
cli-version-flag = Muestra la versión
cli-error-unknown-argument = argumento desconocido: { $arg }
cli-error-invalid-subcommand = subcomando desconocido: { $arg }
cli-error-missing-required-argument = faltan argumentos obligatorios: { $arg }
cli-error-invalid-value = valor no válido para { $arg }
cli-error-missing-value = falta el valor de { $arg }
cli-error-value-validation = el valor de { $arg } no es válido
cli-error-too-many-values = demasiados valores para { $arg }
cli-error-argument-conflict = { $arg } no se puede usar con los otros argumentos
cli-error-argument-repeated = { $arg } no se puede usar más de una vez
cli-error-missing-subcommand = { $arg } necesita un subcomando
cli-error-invalid-utf8 = hay un argumento que no es UTF-8 válido
cli-error-generic = los argumentos no son válidos
cli-error-hint = Para más información, prueba '--help'.
cli-error-did-you-mean = ¿Querías decir «{ $suggestion }»?

## locale

error-invalid-locale = idioma no admitido
