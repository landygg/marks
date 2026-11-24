// Spanish catalog. Its shape is `Messages`, which `en.ts` must satisfy.
import type { ImportSummary } from '../api';
import type { Theme } from '../features/settings/theme';
import { num, plural } from './format';

export const es = {
  appTitle: 'marks',
  addWithShortcut: 'Añadir (⌘N)',
  sidebarLabel: 'Etiquetas y categorías',
  profilesLabel: 'Perfiles',
  newProfile: 'Nuevo perfil',
  profileTitle: (name: string, shortcut: string | null) =>
    shortcut ? `${name} (${shortcut})` : name,
  /** Shortcut of the profile at `position` (0-based) in the rail. */
  profileShortcut: (position: number) => `⌥⌘${position + 1}`,
  renameProfileTitle: 'Renombrar perfil',
  renameProfile: 'Renombrar',
  profileNameLabel: 'Nombre',
  profileInitialsLabel: 'Iniciales (1 o 2 caracteres)',
  deleteProfile: 'Eliminar perfil',
  deleteProfileMenu: 'Eliminar perfil…',
  confirmDeleteProfile: (n: number) =>
    plural(n, {
      one: '¿Eliminar el perfil y su bookmark?',
      other: `¿Eliminar el perfil y sus ${num(n)} bookmarks?`,
    }),
  confirmDeleteProfileHint: 'No se puede deshacer.',
  confirmDeleteProfileYes: 'Sí, eliminar el perfil',
  defaultProfileHelp: 'El primer perfil no se puede eliminar.',
  profileSaved: 'Perfil guardado.',
  create: 'Crear',
  allBookmarks: 'Todos',
  failedBookmarks: 'Fallidos',
  noGroup: 'Sin grupo',
  bookmarkCount: (n: number) => plural(n, { one: '1 bookmark', other: `${num(n)} bookmarks` }),
  selectBookmark: 'Selecciona un bookmark',
  newGroup: 'Nuevo grupo',
  newGroupMenu: 'Nuevo grupo…',
  groupNameLabel: 'Nombre del grupo',
  moveTo: 'Mover a',
  moveToProfile: 'Mover a perfil',
  removeFromGroup: 'Quitar del grupo',
  renameGroup: 'Renombrar',
  deleteGroup: 'Eliminar grupo',
  confirmDeleteGroup: '¿Eliminar grupo?',
  confirmDeleteGroupHint: 'Sus etiquetas quedan sin grupo.',
  confirmDeleteGroupYes: 'Sí, eliminar el grupo',
  confirmDeleteGroupLabel: (group: string, hint: string) =>
    `¿Eliminar el grupo «${group}»? ${hint}`,
  dragHint: 'Arrastra para mover a un grupo',
  commandsLabel: 'Comandos',
  commandsShortcut: '⌘K',
  shortcutAdd: '⌘N',
  shortcutSearch: '⌘2',
  shortcutSettings: '⌘,',
  shortcutSidebar: '⌃⌘S',
  search: 'Buscar',
  paletteLabel: 'Paleta de comandos',
  palettePlaceholder: {
    commands: 'Escribe un comando…',
    bookmarks: 'Buscar en todos los bookmarks…',
    tags: 'Buscar una etiqueta…',
  },
  paletteModes: { commands: 'Comandos', bookmarks: 'Bookmarks', tags: 'Etiquetas' },
  paletteModeKeys: { commands: '⌘1', bookmarks: '⌘2', tags: '⌘3' },
  paletteRun: 'Ejecutar',
  paletteOpen: 'Abrir',
  paletteClose: 'Cerrar',
  paletteRunKey: '↵',
  paletteCloseKey: 'esc',
  paletteEmpty: 'Sin resultados.',
  cmdAdd: 'Añadir bookmark',
  cmdSearch: 'Buscar en todos los bookmarks',
  cmdNewGroup: 'Nuevo grupo de etiquetas',
  cmdToggleExpanded: 'Expandir o contraer el detalle',
  cmdToggleSidebar: 'Mostrar u ocultar la barra lateral',
  cmdReindex: 'Ajustes: reindexar embeddings',
  cmdProcess: 'Ajustes: procesar bookmarks sin resumen',
  cmdTestConnection: 'Ajustes: probar conexión con el modelo',
  cmdOpenLink: 'Abrir el bookmark en el navegador',
  cmdRetry: 'Reintentar el bookmark',
  cmdExtract: 'Extraer',
  cmdExtractView: (n: number) =>
    plural(n, {
      one: 'Extraer el bookmark sin extraer de esta vista',
      other: `Extraer los ${num(n)} sin extraer de esta vista`,
    }),
  cmdRetryFailed: (n: number) =>
    plural(n, {
      one: 'Reintentar el bookmark fallido',
      other: `Reintentar los ${num(n)} fallidos`,
    }),
  cmdDelete: 'Eliminar el bookmark',
  cmdRenameGroup: (group: string) => `Renombrar el grupo «${group}»`,
  cmdDeleteGroup: (group: string) => `Eliminar el grupo «${group}»`,
  cmdMoveTag: (tag: string, group: string) => `Mover la etiqueta «${tag}» a «${group}»`,
  cmdUngroupTag: (tag: string) => `Quitar la etiqueta «${tag}» de su grupo`,
  cmdSettings: 'Ajustes',
  cmdSettingsProfile: 'Ajustes: perfil',
  cmdSwitchProfile: (name: string) => `Cambiar al perfil «${name}»`,
  cmdMoveToProfile: (name: string) => `Mover el bookmark al perfil «${name}»`,
  cmdTheme: (theme: Theme) =>
    ({ system: 'Tema: como el sistema', light: 'Tema: claro', dark: 'Tema: oscuro' })[theme],
  cmdLanguage: (setting: 'system' | 'es' | 'en') =>
    ({ system: 'Idioma: como el sistema', es: 'Idioma: Español', en: 'Idioma: English' })[setting],
  cmdImport: 'Importar bookmarks…',
  cmdExport: 'Exportar bookmarks…',
  settingsNavLabel: 'Secciones de Ajustes',
  settingsApp: 'Aplicación',
  settingsProfile: (name: string) => `Perfil · ${name}`,
  settingsSections: {
    appearance: 'Apariencia',
    ai: 'Inteligencia artificial',
    profile: 'Perfil',
    maintenance: 'Mantenimiento',
    extension: 'Extensión del navegador',
    transfer: 'Importar y exportar',
    updates: 'Actualizaciones',
  },
  backToLibrary: 'Biblioteca',
  themeLabel: 'Tema',
  languageLabel: 'Idioma',
  cornersLabel: 'Esquinas',
  cornersHelp: 'Radio de los paneles y las ventanas, en píxeles. Por defecto, 4.',
  updateCurrent: (v: string) => `Versión ${v}`,
  updateCheck: 'Buscar actualizaciones',
  updateAuto: 'Buscar automáticamente',
  updateAutoHelp:
    'Una vez al día, en GitHub. Solo envía el sistema, el tipo de Mac y el User-Agent del actualizador.',
  updateAutoModes: { on: 'Sí', off: 'No' },
  updateNone: 'Tienes la última versión.',
  updateInstall: (v: string) => `Instalar ${v} y reiniciar`,
  updateManual: 'En este Mac, descarga las versiones nuevas desde GitHub.',
  updateReleases: 'Ver versiones',
  languages: { system: 'Como el sistema', es: 'Español', en: 'English' },
  confirmRelaunch: 'marks se reiniciará para cambiar el idioma.',
  confirmRelaunchYes: 'Sí, reiniciar',
  themes: { system: 'Como el sistema', light: 'Claro', dark: 'Oscuro' },
  expand: 'Expandir',
  collapse: 'Contraer',
  layout: {
    hideSidebar: 'Ocultar barra lateral',
    showSidebar: 'Mostrar barra lateral',
    hideSidebarTitle: 'Ocultar barra lateral (⌃⌘S)',
    showSidebarTitle: 'Mostrar barra lateral (⌃⌘S)',
    sidebarWidth: 'Ancho de la barra lateral',
    listWidth: 'Ancho de la lista',
  },
  settings: 'Ajustes',
  notifications: 'Notificaciones',
  notices: {
    title: 'Notificaciones',
    unread: (n: number) => `${num(n)} sin leer`,
    bell: (n: number) => (n === 0 ? 'Notificaciones' : `Notificaciones: ${num(n)} sin leer`),
    markRead: 'Marcar todo como leído',
    clear: 'Vaciar',
    close: 'Cerrar',
    dismiss: 'Descartar',
    empty: 'Sin notificaciones. Aquí quedan los errores y avisos de esta sesión.',
    severities: { error: 'Error', warning: 'Aviso', info: 'Info' },
    /** Read by screen readers on an unread row, which otherwise only shows a yellow bar. */
    unreadRow: 'sin leer',
    open: 'Abrir bookmark',
    retry: 'Reintentar',
    settings: 'Abrir Ajustes',
    more: (n: number) => `+${num(n)} más en Notificaciones`,
    reextracted: { defuddle: 'Volver a extraer terminó', ai: 'Extraer con IA terminó' },
    clearCommand: 'Vaciar notificaciones',
    extractQueued: (n: number) =>
      plural(n, {
        one: '1 bookmark en cola para extraer.',
        other: `${num(n)} bookmarks en cola para extraer.`,
      }),
    retryQueued: (n: number) =>
      plural(n, {
        one: '1 bookmark en cola para reintentar.',
        other: `${num(n)} bookmarks en cola para reintentar.`,
      }),
    importDone: (n: number) =>
      plural(n, {
        one: '1 bookmark importado, sin extraer.',
        other: `${num(n)} bookmarks importados, sin extraer.`,
      }),
    exportDone: (path: string) => `Exportado en ${path}.`,
  },
  statusBar: {
    label: 'Barra de estado',
    extracting: (title: string) => `Extrayendo «${title}»`,
    pending: (n: number) => plural(n, { one: '1 pendiente', other: `${num(n)} pendientes` }),
    failed: (n: number) => plural(n, { one: '1 fallido', other: `${num(n)} fallidos` }),
    ai: {
      off: 'IA desactivada',
      ready: (model: string) => `IA · ${model}`,
      failed: 'IA con error',
      failedTitle: (error: string) => `La última llamada al modelo falló: ${error}`,
      unavailableTitle: (error: string) => `No se pudo preparar el modelo: ${error}`,
    },
    api: {
      listening: 'API local',
      listeningTitle: (port: number) => `Escuchando en 127.0.0.1:${port}`,
      busy: (port: number) => `Puerto ${port} ocupado`,
      stopped: 'API local detenida',
    },
  },
  settingsButton: 'Ajustes (⌘,)',
  categoriesLabel: 'Categorías',
  tagsLabel: 'Etiquetas',
  bookmarksLabel: 'Bookmarks',
  emptyLibrary: 'Aún no hay bookmarks. Pulsa Añadir o usa ⌘N.',
  noFailed: 'Ningún bookmark ha fallado.',
  separator: ' · ',
  addDialogTitle: 'Añadir bookmark',
  urlLabel: 'URL',
  urlPlaceholder: 'https://…',
  tagsInputPlaceholder: 'etiqueta, otra…',
  titleLabel: 'Título',
  copyLink: 'Copiar enlace',
  linkCopied: 'Enlace copiado',
  copyMessage: 'Copiar mensaje',
  messageCopied: 'Mensaje copiado',
  showMore: 'Ver más',
  showLess: 'Ver menos',
  openLink: 'Abrir en el navegador',
  retry: 'Reintentar',
  extract: 'Extraer',
  confirmExtractView: (n: number) =>
    plural(n, { one: '¿Extraer 1 bookmark?', other: `¿Extraer ${num(n)} bookmarks?` }),
  confirmExtractRemote: (n: number) =>
    plural(n, {
      one: 'Con la IA activada, su página se enviará a tu proveedor para resumirla.',
      other: `Con la IA activada, sus ${num(n)} páginas se enviarán a tu proveedor para resumirlas.`,
    }),
  extractYes: 'Extraer',
  retryAll: 'Reintentar todos',
  confirmRetryFailed: (n: number) =>
    plural(n, { one: '¿Reintentar 1 bookmark?', other: `¿Reintentar ${num(n)} bookmarks?` }),
  retryYes: 'Reintentar',
  reextract: 'Volver a extraer',
  extractWithAi: 'Extraer con IA',
  extractWithAiUnavailable:
    'Extraer con IA necesita un modelo: actívala y configúrala en Ajustes → Inteligencia artificial.',
  confirmReextract: 'Se sustituirá el contenido guardado por el de la nueva extracción.',
  confirmReextractYes: 'Sí, sustituir',
  summaryLabel: 'Resumen',
  addTagsLabel: 'Añadir etiquetas',
  addTagsPlaceholder: 'etiqueta, otra… (Enter)',
  removeTag: (tag: string) => `Quitar la etiqueta ${tag}`,
  notesLabel: 'Notas',
  contentLabel: 'Contenido',
  noContent: 'Sin contenido extraído todavía.',
  contentUnrenderable: 'No se pudo mostrar el contenido de esta página.',
  mediaOpen: 'Abrir',
  /** Accessible name of a media line's action: several lines would otherwise all be "Abrir". */
  mediaOpenNamed: (label: string) => `Abrir ${label}`,
  mediaKinds: { image: 'Imagen', video: 'Vídeo', audio: 'Audio', frame: 'Contenido incrustado' },
  taskDone: 'Tarea hecha',
  taskOpen: 'Tarea pendiente',
  delete: 'Eliminar',
  confirmDelete: '¿Eliminar este bookmark? No se puede deshacer.',
  confirmDeleteYes: 'Sí, eliminar',
  cancel: 'Cancelar',
  aiLabel: 'IA',
  aiModes: { on: 'Activada', off: 'Desactivada' },
  aiOffNotice:
    'Sin IA: los bookmarks se extraen y quedan pendientes; sin resumen, etiquetas ni búsqueda semántica.',
  providerLabel: 'Proveedor',
  providers: {
    ollama: 'Ollama',
    omlx: 'oMLX',
    openai: 'OpenAI',
    anthropic: 'Anthropic',
    openai_compatible: 'Compatible con OpenAI',
  },
  endpointLabel: 'Endpoint',
  endpointRequired: 'Obligatorio con este proveedor.',
  chatModelLabel: 'Modelo de chat',
  embedModelLabel: 'Modelo de embeddings',
  modelRecommended: (model: string) => `Recomendado: ${model}.`,
  modelLocalNeeds: (sizeGb: number, ramGb: number) =>
    `Unos ${num(sizeGb)} GB de descarga y ${num(ramGb)} GB de memoria libre.`,
  // Memory over Ollama's 4k default: qwen3:8b measured with `ollama ps` (Ollama 0.40.14, 2026-11-13);
  // ReaderLM-v2 estimated from its KV cache (28 layers, 2 KV heads).
  ollamaContextChat: (summary: number, extract: number) =>
    `Ollama lo carga con ${num(summary)} tokens de contexto para resumir y ${num(extract)} para extraer: con qwen3:8b, unos ${num(2.2)} y ${num(4.6)} GB más de memoria (medido).`,
  ollamaContextExtraction: (extract: number) =>
    `Ollama lo carga con ${num(extract)} tokens de contexto: con ReaderLM-v2, unos ${num(0.9)} GB más de memoria (estimado).`,
  noEmbeddings: 'Anthropic no ofrece embeddings: la búsqueda necesita otro proveedor.',
  useRecommended: 'Usar',
  useRecommendedTitle: (model: string) => `Usar el recomendado: ${model}`,
  copyCommand: 'Copiar comando',
  commandCopied: 'Comando copiado',
  apiKeyLabel: 'Clave de API',
  apiKeyStored: 'Hay una clave guardada en el llavero.',
  apiKeyWillClear: 'La clave se borrará al guardar.',
  clearKey: 'Borrar clave',
  save: 'Guardar',
  saved: 'Ajustes guardados.',
  testConnection: 'Probar conexión',
  testConnectionHint: 'Prueba los ajustes guardados.',
  connectionOk: (chatMs: number, embedMs: number, dim: number) =>
    `Conexión correcta: chat ${chatMs} ms, embeddings ${embedMs} ms, ${dim} dimensiones.`,
  summaryBlock: 'Resumen y búsqueda',
  extractionBlock: 'Extracción con IA',
  extractionSettingsLabel: 'Ajustes',
  extractionModes: { same: 'Mismos ajustes', own: 'Propios' },
  extractionSameHelp:
    'Usa el proveedor, el endpoint, la clave y el modelo de chat de Resumen y búsqueda. Con Propios puedes usar un modelo hecho para HTML → Markdown, como ReaderLM-v2.',
  extractionModelLabel: 'Modelo',
  extractionRemoteWarning:
    'Extraer con IA envía el HTML de la página a un servidor fuera de este equipo.',
  extractionConnectionOk: (chatMs: number) => `Conexión correcta: extracción en ${chatMs} ms.`,
  reindex: 'Reindexar embeddings',
  reindexAction: 'Reindexar',
  confirmReindex: (n: number) =>
    plural(n, {
      one: 'Se volverán a calcular los vectores de 1 bookmark.',
      other: `Se volverán a calcular los vectores de ${num(n)} bookmarks.`,
    }),
  confirmReindexYes: 'Sí, reindexar',
  reindexQueued: (n: number) =>
    plural(n, {
      one: '1 bookmark en cola para reindexar.',
      other: `${num(n)} bookmarks en cola para reindexar.`,
    }),
  processLabel: 'Procesar bookmarks sin resumen',
  processPending: (n: number) => plural(n, { one: '1 pendiente.', other: `${num(n)} pendientes.` }),
  processAction: 'Procesar',
  importLabel: 'Importar',
  importAction: 'Importar bookmarks…',
  importHelp: 'Un fichero HTML exportado desde Chrome o Safari. Lo importado no se extrae solo.',
  importSummary: (s: ImportSummary, profile: string) => {
    const skipped = s.unsupported + s.invalid;
    const added = plural(s.new, { one: '1 nuevo', other: `${num(s.new)} nuevos` });
    const existing = plural(s.existing, {
      one: '1 ya estaba',
      other: `${num(s.existing)} ya estaban`,
    });
    const rejected = plural(skipped, {
      one: '1 no se puede importar',
      other: `${num(skipped)} no se pueden importar`,
    });
    return `${added}, ${existing}, ${rejected}. Perfil ${profile}.`;
  },
  importYes: 'Importar',
  exportLabel: 'Exportar',
  exportAction: 'Exportar bookmarks…',
  exportHelp:
    'Un fichero HTML que Chrome y Safari pueden importar, con tus grupos y etiquetas como carpetas.',
  apiListening: (port: number) => `API escuchando en 127.0.0.1:${port}.`,
  apiPortBusy: (port: number) =>
    `El puerto ${port} está ocupado (¿otra instancia de marks?): la extensión no podrá conectarse.`,
  tokenLabel: 'Token de la extensión',
  copyToken: 'Copiar',
  tokenCopied: 'Token copiado.',
  regenerateToken: 'Regenerar token',
  confirmRegenerateToken: 'La extensión dejará de funcionar hasta que pegues el nuevo token.',
  confirmRegenerateTokenYes: 'Sí, regenerar',
  tokenRegenerated: 'Token regenerado. Pégalo en las opciones de la extensión.',
  extensionChromiumTitle: 'Chrome, Brave y otros Chromium',
  extensionChromiumRecommended: 'Recomendada',
  extensionChromiumDownload: 'Abrir la página de versiones',
  extensionChromiumSteps: [
    'Descarga marks-chromium-<versión>.zip de la página de versiones de marks y descomprímelo.',
    'En Chrome o Brave, abre la página de extensiones y activa el modo de desarrollador.',
    'Pulsa «Cargar descomprimida» y elige la carpeta descomprimida.',
    'Pega el token en las opciones de la extensión.',
  ],
  extensionSafariTitle: 'Safari',
  extensionSafariExperimental: 'Experimental',
  extensionSafariSteps: [
    'Abre marks una vez.',
    'En Safari, Ajustes > Avanzado, activa «Mostrar funciones para desarrolladores web».',
    'En Ajustes > Desarrollador, activa «Permitir extensiones no firmadas». Hay que repetirlo cada vez que abras Safari.',
    'Activa marks en Ajustes > Extensiones.',
    'Pega el token en las opciones de la extensión.',
  ],
  extensionSafariNote:
    'Safari solo la carga con «Permitir extensiones no firmadas», y lo desactiva cada vez que se cierra. Para el día a día, usa la de Chromium.',
  processQueued: (n: number) =>
    plural(n, {
      one: '1 bookmark en cola para resumir.',
      other: `${num(n)} bookmarks en cola para resumir.`,
    }),
  extracting: 'Extrayendo',
  status: {
    pending: 'Sin resumir',
    pending_embed: 'Pendiente de indexar',
    ready: 'Listo',
    failed: 'Fallido',
    imported: 'Sin extraer',
  },
  readingTime: (min: number) => `${num(min)} min de lectura`,
  extractedWithAi: 'Extraído con IA',
  blocked: {
    dot: 'Bloqueada por una verificación',
    notice:
      'La página pidió una verificación antibots. Ábrela en el navegador, resuélvela y guárdala con la extensión.',
  },
  errors: {
    invalidExternalUrl: 'Solo se pueden abrir enlaces http o https',
  },
} as const;

/** `T` with its string literals widened to `string`, so another catalog can hold other text. */
type Widen<T> = T extends string
  ? string
  : T extends (...args: infer A) => infer R
    ? (...args: A) => Widen<R>
    : T extends readonly (infer U)[]
      ? readonly Widen<U>[]
      : { readonly [K in keyof T]: Widen<T[K]> };

export type Messages = Widen<typeof es>;
