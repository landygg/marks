---
version: alpha
name: Marcador
description: >-
  Sistema de diseño de marks. Herramienta de teclado en grafito, con
  tipografía monoespaciada para los metadatos y amarillo de rotulador como
  único acento. Cinco paneles (perfiles, etiquetas, lista, detalle y barra
  inferior) sobre un marco.
colors:
  primary: "#ffd60a"
  primary-hover: "#ffe14d"
  on-primary: "#17170f"
  neutral: "#131416"
  surface: "#1c1e21"
  on-surface: "#e8e8ea"
  secondary: "#92959c"
  outline: "#2c2f34"
  error: "#ff6b6b"
  success: "#4ade80"
  warning: "#ff9f43"
  neutral-light: "#fafafa"
  surface-light: "#f0f0f1"
  on-surface-light: "#17181a"
  secondary-light: "#63666d"
  outline-light: "#dcdde0"
  error-light: "#c81e1e"
  success-light: "#1f7a3a"
  warning-light: "#b45309"
typography:
  headline-lg:
    fontFamily: system-ui
    fontSize: 20px
    fontWeight: 600
    lineHeight: 1.25
    letterSpacing: -0.01em
  headline-md:
    fontFamily: "ui-monospace, 'SF Mono', Menlo, monospace"
    fontSize: 15px
    fontWeight: 700
    lineHeight: 1.3
  body-lg:
    fontFamily: system-ui
    fontSize: 15px
    fontWeight: 400
    lineHeight: 1.6
  body-md:
    fontFamily: system-ui
    fontSize: 13px
    fontWeight: 400
    lineHeight: 1.4
  body-sm:
    fontFamily: system-ui
    fontSize: 12.5px
    fontWeight: 400
    lineHeight: 1.4
  label-lg:
    fontFamily: system-ui
    fontSize: 13.5px
    fontWeight: 600
    lineHeight: 1.3
  label-md:
    fontFamily: "ui-monospace, 'SF Mono', Menlo, monospace"
    fontSize: 11.5px
    fontWeight: 500
    lineHeight: 1.3
  label-sm:
    fontFamily: "ui-monospace, 'SF Mono', Menlo, monospace"
    fontSize: 10.5px
    fontWeight: 500
    lineHeight: 1.3
    letterSpacing: 0.06em
  help:
    fontFamily: system-ui
    fontSize: 12px
    fontWeight: 400
    lineHeight: 1.4
rounded:
  none: 0px
  sm: 2px
  md: 4px
spacing:
  xxs: 2px
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 24px
  rail-width: 48px
  sidebar-width: 200px
  list-width: 340px
  detail-min-width: 420px
  panel-gap: 8px
  resize-hit-width: 8px
  sidebar-min-width: 160px
  sidebar-max-width: 320px
  list-min-width: 280px
  list-max-width: 520px
  content-max-width: 720px
  control-height: 28px
  topbar-height: 38px
  window-min-width: 1032px
  palette-width: 640px
  palette-top: 96px
  palette-search-height: 46px
  row-height: 26px
  command-item-height: 36px
  command-hit-height: 52px
  setting-row-min-height: 40px
  field-width: 250px
  icon-size: 16px
  indicator: 2px
  window-controls-width: 76px
  icon-small-size: 12px
  icon-stroke-width: 1.5px
  status-dot-size: 8px
  palette-list-max-height: 320px
  textarea-min-height: 96px
  dialog-width: 512px
  tag-height: 24px
  tag-field-width: 208px
  notification-panel-width: 360px
  notification-panel-max-height: 400px
components:
  profile-rail:
    backgroundColor: "{colors.neutral}"
    width: "{spacing.rail-width}"
  profile-item:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    size: "{spacing.control-height}"
  profile-item-hover:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  profile-item-selected:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  profile-add:
    textColor: "{colors.secondary}"
    rounded: "{rounded.sm}"
    size: "{spacing.control-height}"
  profile-add-hover:
    textColor: "{colors.on-surface}"
  panel:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.md}"
  resize-handle:
    width: "{spacing.resize-hit-width}"
  resize-handle-active:
    backgroundColor: "{colors.primary}"
    width: "{spacing.indicator}"
  sidebar-toggle:
    textColor: "{colors.secondary}"
    rounded: "{rounded.sm}"
    size: "{spacing.control-height}"
  sidebar:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    width: "{spacing.sidebar-width}"
    padding: "{spacing.sm}"
  sidebar-group:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.secondary}"
    typography: "{typography.label-sm}"
    height: "{spacing.row-height}"
  sidebar-group-hover:
    textColor: "{colors.on-surface}"
  sidebar-group-deleting:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  sidebar-item:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.row-height}"
    padding: "{spacing.sm}"
  sidebar-item-hover:
    textColor: "{colors.on-surface}"
  sidebar-item-selected:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
  list-pane:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    width: "{spacing.list-width}"
  list-row:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.label-lg}"
    padding: 8px 12px
  list-row-hover:
    backgroundColor: "{colors.neutral}"
  list-row-selected:
    backgroundColor: "{colors.neutral}"
  list-row-summary:
    textColor: "{colors.secondary}"
    typography: "{typography.body-sm}"
  list-row-meta:
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
  detail-pane:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-lg}"
    padding: "{spacing.xl}"
  detail-title:
    textColor: "{colors.on-surface}"
    typography: "{typography.headline-lg}"
  wordmark:
    textColor: "{colors.on-surface}"
    typography: "{typography.headline-md}"
  sidebar-heading:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.headline-md}"
    padding: 8px 4px 8px 16px
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.control-height}"
    padding: "{spacing.md}"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
  button-secondary:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.control-height}"
    padding: "{spacing.md}"
  button-secondary-hover:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  button-danger:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.error}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.control-height}"
    padding: "{spacing.md}"
  button-danger-hover:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.error}"
  button-link:
    textColor: "{colors.secondary}"
    typography: "{typography.body-md}"
  icon-button:
    textColor: "{colors.secondary}"
    rounded: "{rounded.sm}"
    size: "{spacing.control-height}"
  icon-button-hover:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  icon-button-pressed:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  icon-button-danger:
    textColor: "{colors.error}"
  icon-button-marked:
    textColor: "{colors.on-surface}"
  icon-button-mark:
    backgroundColor: "{colors.primary}"
    rounded: "{rounded.sm}"
    size: "{spacing.status-dot-size}"
  topbar:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
    height: "{spacing.topbar-height}"
  topbar-title:
    textColor: "{colors.on-surface}"
    typography: "{typography.label-md}"
  segmented-control:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.sm}"
  segment:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
    height: "{spacing.row-height}"
    padding: "{spacing.md}"
  segment-hover:
    textColor: "{colors.on-surface}"
  segment-selected:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
  setting-rows:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.md}"
  setting-row:
    typography: "{typography.body-md}"
    height: "{spacing.setting-row-min-height}"
    padding: 8px 12px
  setting-help:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.secondary}"
    typography: "{typography.help}"
  command-line:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.on-surface}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
  inline-confirm:
    textColor: "{colors.error}"
  inline-confirm-neutral:
    textColor: "{colors.on-surface}"
  section-label:
    textColor: "{colors.secondary}"
    typography: "{typography.label-sm}"
  dialog:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.md}"
    width: "{spacing.dialog-width}"
  link:
    textColor: "{colors.on-surface}"
  error-text:
    textColor: "{colors.error}"
  copy-button:
    textColor: "{colors.secondary}"
    rounded: "{rounded.sm}"
    size: "{spacing.control-height}"
  copy-button-copied:
    textColor: "{colors.success}"
  detail-url:
    textColor: "{colors.on-surface}"
    typography: "{typography.label-md}"
  clamp-text:
    typography: "{typography.label-md}"
  empty-state:
    textColor: "{colors.secondary}"
    typography: "{typography.body-md}"
  drop-target:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  command-palette:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.md}"
    width: "{spacing.palette-width}"
  command-item:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    height: "{spacing.command-item-height}"
    padding: "{spacing.md}"
  command-item-hit:
    height: "{spacing.command-hit-height}"
  palette-search:
    textColor: "{colors.on-surface}"
    typography: "{typography.body-lg}"
    height: "{spacing.palette-search-height}"
  palette-tab:
    textColor: "{colors.secondary}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.row-height}"
    padding: "{spacing.sm}"
  palette-tab-hover:
    textColor: "{colors.on-surface}"
  palette-tab-selected:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  command-item-selected:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  kbd:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    padding: "{spacing.xs}"
  input:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.control-height}"
    padding: "{spacing.sm}"
  input-inset:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.on-surface}"
  tag:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.tag-height}"
    padding: "{spacing.sm}"
  tag-field:
    textColor: "{colors.on-surface}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.tag-height}"
    width: "{spacing.tag-field-width}"
  field-label:
    textColor: "{colors.secondary}"
    typography: "{typography.help}"
  search-highlight:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.none}"
  status-dot-pending:
    backgroundColor: "{colors.secondary}"
    rounded: "{rounded.sm}"
    size: "{spacing.status-dot-size}"
  status-dot-extracting:
    backgroundColor: "{colors.primary}"
    rounded: "{rounded.sm}"
    size: "{spacing.status-dot-size}"
  status-dot-pending-embed:
    backgroundColor: "{colors.warning}"
  status-dot-ready:
    backgroundColor: "{colors.success}"
  status-dot-failed:
    backgroundColor: "{colors.error}"
  status-dot-imported:
    textColor: "{colors.secondary}"
    rounded: "{rounded.sm}"
    size: "{spacing.status-dot-size}"
  code-block:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.on-surface}"
    typography: "{typography.label-md}"
    rounded: "{rounded.md}"
    padding: "{spacing.md}"
  profile-item-light:
    backgroundColor: "{colors.neutral-light}"
    textColor: "{colors.secondary-light}"
  profile-item-hover-light:
    backgroundColor: "{colors.outline-light}"
    textColor: "{colors.on-surface-light}"
  profile-item-selected-light:
    backgroundColor: "{colors.outline-light}"
    textColor: "{colors.on-surface-light}"
  topbar-light:
    backgroundColor: "{colors.neutral-light}"
    textColor: "{colors.secondary-light}"
  sidebar-light:
    backgroundColor: "{colors.surface-light}"
    textColor: "{colors.on-surface-light}"
  sidebar-item-light:
    backgroundColor: "{colors.surface-light}"
    textColor: "{colors.secondary-light}"
  list-pane-light:
    backgroundColor: "{colors.surface-light}"
    textColor: "{colors.on-surface-light}"
  list-row-meta-light:
    backgroundColor: "{colors.surface-light}"
    textColor: "{colors.secondary-light}"
  icon-button-hover-light:
    backgroundColor: "{colors.outline-light}"
    textColor: "{colors.on-surface-light}"
  button-danger-light:
    backgroundColor: "{colors.neutral-light}"
    textColor: "{colors.error-light}"
  button-secondary-light:
    backgroundColor: "{colors.neutral-light}"
    textColor: "{colors.on-surface-light}"
  setting-help-light:
    backgroundColor: "{colors.neutral-light}"
    textColor: "{colors.secondary-light}"
  segment-light:
    backgroundColor: "{colors.surface-light}"
    textColor: "{colors.secondary-light}"
  status-dot-extracting-light:
    backgroundColor: "{colors.on-surface-light}"
  status-dot-pending-embed-light:
    backgroundColor: "{colors.warning-light}"
  status-dot-ready-light:
    backgroundColor: "{colors.success-light}"
  detail-meta:
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
  detail-description:
    textColor: "{colors.secondary}"
    typography: "{typography.body-md}"
  detail-favicon:
    rounded: "{rounded.sm}"
    size: "{spacing.icon-size}"
  content-link:
    textColor: "{colors.on-surface}"
  content-link-hover:
    textColor: "{colors.on-surface}"
  content-media:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    padding: "{spacing.sm}"
  content-callout:
    textColor: "{colors.on-surface}"
    typography: "{typography.body-lg}"
    padding: "{spacing.md}"
  content-table:
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    padding: "{spacing.sm}"
  content-footnotes:
    textColor: "{colors.secondary}"
    typography: "{typography.body-sm}"
  content-raw:
    textColor: "{colors.on-surface}"
    typography: "{typography.body-lg}"
  detail-blocked:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    padding: "{spacing.sm}"
  status-bar:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.md}"
    height: "{spacing.control-height}"
  status-item:
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.control-height}"
    padding: "{spacing.sm}"
  status-item-hover:
    backgroundColor: "{colors.outline}"
    textColor: "{colors.on-surface}"
  notification-panel:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.md}"
    width: "{spacing.notification-panel-width}"
  notification-item:
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    padding: 8px 12px
  notification-item-meta:
    textColor: "{colors.secondary}"
    typography: "{typography.label-md}"
  toast:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.md}"
    width: "{spacing.notification-panel-width}"
---

# Marcador

## Overview

marks es una herramienta local para guardar, encontrar y leer bookmarks. Su
interfaz debe sentirse como un buscador rápido que se maneja con el teclado:
densa, plana, precisa y sin adornos. La personalidad viene de dos decisiones:
los metadatos (dominios, fechas, etiquetas) van en monoespaciada, como en una
terminal, y el único color de acento es el amarillo de un rotulador, porque
*marks* es marcar.

El tema oscuro es el de referencia y el claro es su equivalente exacto. Por
defecto la app sigue la preferencia del sistema; en Ajustes > Apariencia (o
desde la paleta) se puede fijar el tema claro o el oscuro. La interfaz está
en español o en inglés, según el idioma del sistema o el que se fije en Ajustes. La app es local-first: no se carga ninguna fuente, imagen ni
icono remoto.

## Colors

Grafito neutro y un solo acento. El color nunca decora: señala selección,
acción principal, coincidencia de búsqueda o estado.

- **Primary (#ffd60a), amarillo de rotulador:** es un relleno, no una tinta.
  Rellena el elemento seleccionado de la barra lateral, las coincidencias de
  búsqueda, el botón principal de diálogos y Ajustes, la marca de
  notificaciones sin leer y el punto de "extrayendo" de la barra inferior
  (`status-dot-extracting`). Fuera de eso solo aparece como indicador fino de 2px
  (fila seleccionada, subrayado de enlaces, barra del nombre del perfil). El texto sobre amarillo es siempre **on-primary (#17170f)**.
- **Neutral (#131416):** el marco (carril, franjas superiores y huecos entre
  paneles), la fila bajo el cursor o seleccionada, los bloques de Ajustes, de
  código y de medios, los botones secundarios y los campos que van sobre un
  panel.
- **Surface (#1c1e21):** fondo de los paneles (barra lateral, lista, detalle,
  contenido de Ajustes y barra inferior), de los campos y de los controles
  segmentados.
- **On-surface (#e8e8ea):** texto principal.
- **Secondary (#92959c):** metadatos, snippets, etiquetas y elementos no
  seleccionados de la barra lateral.
- **Outline (#2c2f34):** ya no separa zonas: queda para separar filas y como
  borde de campos, botones y superposiciones.
- **Error, success, warning:** solo para el estado de un bookmark, para
  errores y para la gravedad de una notificación. `success` = `ready`,
  `warning` = `pending_embed`, `error` = `failed`, `secondary` = `pending`. En
  una notificación, `error` = Error, `warning` = Aviso y `secondary` = Info.

Tema claro: cada token con sufijo `-light` sustituye al token del mismo nombre
(`neutral-light` por `neutral`, etc.). `primary`, `primary-hover` y
`on-primary` son los mismos en los dos temas. En claro el amarillo no alcanza
contraste como texto, así que la regla "relleno, no tinta" es obligatoria. Los
componentes con sufijo `-light` fijan los pares fondo/texto del tema claro para
que el linter compruebe también su contraste.

En `apps/desktop/src/app.css` los tokens se exponen como variables CSS:
`--accent` (`primary`), `--accent-hover` (`primary-hover`), `--on-accent`
(`on-primary`), `--bg` (`neutral`), `--surface`, `--fg` (`on-surface`),
`--muted` (`secondary`), `--border` (`outline`), `--danger` (`error`), `--ok`
(`success`), `--warn` (`warning`) y `--focus` (el anillo de foco: `primary` en
oscuro, `on-surface-light` en claro). Cada token de tipografía es
`--type-<nombre>`, cada radio `--radius-<nombre>` y cada medida de `spacing`
tiene su variable (`--space-md`, `--row`, `--topbar`…). Un test comprueba que
todos coinciden con este fichero.

## Typography

Dos familias del sistema, sin ficheros de fuente: `system-ui` (SF Pro) para
leer y `ui-monospace` (SF Mono) para datos.

- **headline-lg:** título del bookmark en el detalle.
- **headline-md:** la marca `marks`, en minúsculas, en la franja superior de la
  barra lateral, y el nombre del perfil activo en la cabecera de su panel.
- **body-lg:** contenido extraído (Markdown) y resumen. Es el único texto
  pensado para lectura larga, de ahí el interlineado de 1.6. También el campo
  de búsqueda de la paleta.
- **body-md:** texto de la interfaz: botones, campos (también las notas),
  mensajes.
- **body-sm:** snippet de cada fila.
- **label-lg:** título de cada fila de la lista.
- **label-md:** todo lo que es dato: dominio, fecha, URL, etiquetas, elementos
  de la barra lateral, iniciales y nombre de los perfiles, bloques de código.
- **label-sm:** encabezados de grupo de la barra lateral, en mayúsculas.
- **help:** ayuda, estado o resultado bajo el nombre de un ajuste, y la
  etiqueta de un campo en un diálogo.

Solo se usan los pesos 400, 500, 600 y 700 definidos en los tokens. No se usa
cursiva en la interfaz (sí en el contenido Markdown).

## Layout

La ventana usa la barra de título integrada de macOS: no hay barra de título
propia, los semáforos quedan en la esquina superior izquierda de la ventana, a
caballo entre el carril de perfiles y la barra lateral, y la franja superior de
38px de cada zona sirve para arrastrar la ventana.

La ventana se divide en el carril de perfiles, tres paneles (barra lateral,
lista y detalle) y una barra inferior que también es un panel. El marco, de
fondo `neutral`, lo forman el carril, las franjas superiores de 38px y los
huecos; los paneles (`panel`: `surface`, radio `md`, sin borde) son el cuerpo
de la barra lateral, de la lista, del detalle o del contenido de Ajustes, y la
barra inferior. Los separa un hueco de `panel-gap` (8px) del color del marco:
entre la barra lateral y la lista, entre la lista y el detalle, entre el último
panel y el borde derecho, entre los paneles y la barra inferior y bajo la barra
inferior. Entre el carril y el primer panel no hay hueco, porque el carril ya
es marco. Las franjas no llevan línea: el panel empieza debajo. La ventana no
se desplaza: cada zona tiene su propio scroll vertical. Ancho mínimo de
ventana: 1032px (carril de 48px, barra lateral de 200, lista de 340, detalle de
420 y tres huecos de 8). No hay campos de búsqueda en las zonas: toda búsqueda
pasa por la paleta de comandos.

1. **Carril de perfiles** (48px fijos, fondo `neutral`, el mismo de las
   franjas superiores). Su franja superior
   (38px) está vacía y arrastra la ventana; los semáforos empiezan sobre ella.
   Debajo, un elemento por perfil con sus iniciales, centrados y separados
   8px (`sm`), y justo después del último el elemento "Nuevo perfil" (`+`). Si
   no caben, el carril se desplaza en vertical. Al pie, el botón de icono
   "Ajustes" (⌘,), centrado y a la altura de la barra inferior (de
   `control-height`, con `panel-gap` debajo), activo (`icon-button-pressed`)
   mientras Ajustes está abierto. El carril ocupa toda la altura de la
   ventana. Clic en un perfil, o `⌥⌘1`…`⌥⌘9`
   para los nueve primeros, cambia a él:
   en la biblioteca, la vista queda en "Todos", sin bookmark seleccionado y con
   los grupos desplegados; en Ajustes se sigue en Ajustes. Antes de cambiar se
   envían las ediciones pendientes del detalle. El carril se ve también en
   Ajustes. El clic derecho sobre un perfil abre su menú contextual nativo
   ("Renombrar" y, salvo en el primer perfil, "Eliminar perfil…", que cambia a
   ese perfil y abre Ajustes > Perfil con la confirmación abierta: en el carril
   no cabe). "Nuevo perfil" y "Renombrar" abren el mismo `dialog`
   ("Nuevo perfil" o "Renombrar perfil"): Nombre e Iniciales (1 o 2
   caracteres, en monoespaciada y de `field-width`), que se proponen a partir
   del nombre (dos letras de una palabra o la inicial de las dos primeras)
   hasta que el usuario las cambia; abajo a la derecha "Cancelar" y "Crear" o
   "Guardar", deshabilitado mientras el nombre o las iniciales no son válidos.
   El error del backend va en `error-text` sobre los botones. Al crear, la app
   cambia al perfil nuevo.
2. **Barra lateral** (de `sidebar-min-width` a `sidebar-max-width`, 160 a
   320px; por defecto 200px). Su franja superior lleva el fondo `neutral` del
   carril y de las demás franjas; el resto es un `panel` `surface`. De arriba
   abajo:
   - franja superior (38px): a la izquierda, lo que los semáforos sobresalen
     del carril y, tras ellos, la marca `marks`;
   - la cabecera del panel (`sidebar-heading`, con `sm` encima y debajo):
     el nombre del perfil activo, precedido de la barra `primary`, y al final
     de la misma línea, pegado al borde derecho, el botón de icono "Nuevo
     grupo";
   - el elemento "Todos" y, justo debajo y al mismo nivel, "Fallidos" (solo los
     bookmarks `failed` del perfil activo). "Fallidos" sale solo si el perfil
     tiene alguno o si es la selección actual, y no lleva contador: el número va
     en la cabecera de la lista y en la barra inferior;
   - los grupos, plegables. Primero "Categorías" (automático: las asigna la
     IA), después los grupos de etiquetas creados por el usuario y al final
     "Sin grupo", con las etiquetas que no están en ninguno. Cada grupo tiene
     un encabezado con chevrón y nombre que lo pliega o despliega. Una etiqueta
     pertenece como mucho a un grupo. Hay un único elemento seleccionado a la
     vez y determina qué muestra la lista.
3. **Lista** (de `list-min-width` a `list-max-width`, 280 a 520px; por defecto
   340px; panel `surface`). Franja superior (38px): a la
   izquierda, en `label-md` y color `secondary`, el nombre de la selección
   actual y el número de bookmarks; a la derecha los botones de icono "Buscar"
   (abre la paleta en modo Bookmarks) y "Añadir" (`+`, ⌘N). Con "Fallidos"
   elegido y algún bookmark en la lista, delante de ellos va "Reintentar todos"
   (icono `retry`), que pregunta antes con un `inline-confirm-neutral` bajo la
   franja ("¿Reintentar N bookmarks?" y, con la IA activada y un proveedor
   remoto, el aviso de que sus páginas irán a él), como "Extraer los N de esta
   vista". Un filtro sin bookmarks muestra su `empty-state` ("Ningún bookmark ha
   fallado." en "Fallidos"). Debajo, las filas. Cada fila: punto de estado, título (una línea,
   con elipsis), primera línea del resumen (una línea, con elipsis; se omite si
   aún no hay resumen) y, en monoespaciada, dominio y fecha.
4. **Detalle** (el resto, mínimo 420px, panel `surface`). Franja superior
   (38px) solo con botones de icono, a la derecha: "Abrir en el navegador", "Reintentar" (solo si el estado es `failed`),
   "Volver a extraer" y "Extraer con IA" (ninguno de los dos en una página
   bloqueada; "Extraer con IA", deshabilitado y con un `title` que lo explica
   si no hay modelo de extracción) y "Eliminar", y al final, separados `sm` de los demás, los dos botones de la
   vista, juntos: "Expandir" (o "Contraer") y, el último, el botón de plegar la
   barra lateral (`sidebar-toggle`). Los dos se ven también sin bookmark
   seleccionado y no se mueven al plegar ni al expandir. Sustituir un contenido que
   ya existe pide antes un `inline-confirm` bajo la franja. Debajo, en una columna de 720px como máximo: título editable,
   URL (`detail-url`, en una línea, con su `copy-button`), la línea de metadatos con el favicon (`detail-favicon`, si lo hay),
   estado (en un fallo, el mensaje va en su propia línea debajo, en `error-text` y como `clamp-text`
   de dos líneas, con su `copy-button`), resumen, categoría, etiquetas, notas y contenido. Sin
   selección, muestra un mensaje centrado en `secondary`.
5. **Barra inferior** (`status-bar`, un panel `surface` de radio `md` y
   `control-height`, sin línea, con `panel-gap` debajo y a la derecha). Empieza
   donde empieza la barra lateral, no bajo el carril; con la barra lateral
   oculta, empieza donde empieza la lista (a la derecha del carril). En la
   columna de la barra lateral, "Comandos" con su atajo `⌘K`, que abre la
   paleta. En medio,
   el trabajo de fondo del perfil activo: "Extrayendo «título»" con el punto
   `status-dot-extracting` (clic: vuelve a "Todos" y lo selecciona), "N
   pendientes" (solo texto: los que aún esperan a extraerse; sin IA, los ya
   extraídos se quedan en `pending` esperando un modelo y no cuentan) y "N fallidos" en `on-surface` (clic: el filtro
   "Fallidos" de la barra lateral). Sin nada en curso, el centro queda vacío. Lo de en medio se
   recorta con elipsis y nunca empuja a la derecha, donde va la campana de
   notificaciones, marcada cuando hay alguna sin leer. La barra se ve también
   en Ajustes.

**Plegar la barra lateral.** Un botón de icono (`sidebar-toggle`: un rectángulo
con una columna a la izquierda) va al final de la franja del detalle, junto a
"Expandir", y al final de la de Ajustes. Está fuera de la barra lateral, así que
plegarla no lo oculta ni lo mueve. Su nombre es "Ocultar barra lateral" o "Mostrar barra lateral", con
el atajo `⌃⌘S` en el `title`; lleva `aria-expanded` (abierta = verdadero) y
`aria-controls`, y no `aria-pressed`. `⌃⌘S` y el comando de la paleta hacen lo
mismo, también con el foco en un campo. Solo se pliega la barra lateral (también
en Ajustes, donde pliega su navegación); el carril sigue, y con ella se va el
hueco de su derecha. La columna va de su ancho a 0 en 180ms con `ease-out`; el
contenido conserva su ancho y se recorta, y la barra inferior sigue con la misma
transición. Con "reducir movimiento", y mientras se arrastra un asa, no hay
transición. Plegada, la barra queda inerte (ni foco ni lector de pantalla), y si
el foco estaba dentro pasa al botón. Los semáforos dejan sitio en el primer
panel tras el carril: la barra lateral si está abierta; si no, la lista; o el
detalle si "Expandir" oculta la lista; o Ajustes. Expandir y plegar son
independientes.

**Redimensionar.** El ancho de la barra lateral y el de la lista se cambian
arrastrando el hueco de su derecha. El asa (`resize-handle`) es una zona
invisible de `resize-hit-width` centrada sobre el hueco, del borde superior al
inferior del panel (no sobre la franja, que arrastra la ventana), con cursor
`col-resize`. Bajo el cursor (tras 120ms), al arrastrar o con el foco del
teclado muestra `resize-handle-active`: una línea `primary` de `indicator`
centrada en el hueco; con foco lleva además el anillo normal. Los límites son
los de `sidebar-min-width`, `sidebar-max-width`, `list-min-width` y
`list-max-width`, y el detalle nunca baja de `detail-min-width`: el máximo real
de cada asa es el menor entre su máximo y lo que deja libre el detalle. Si la
ventana es más estrecha que los anchos guardados, encoge primero la lista y
después la barra lateral, sin plegar nada, y lo guardado no cambia. No hay asa
en la barra plegada ni en la lista oculta. Es un separador accesible
(`role="separator"`, enfocable, con su valor, mínimo y máximo en px): `←` y `→`
mueven 8px (`sm`), con `⇧` 24px (`xl`), `Inicio` y `Fin` llevan al mínimo y al
máximo, y `↵` y el doble clic restablecen el ancho por defecto. Anchos y estado
plegado son globales, no por perfil.

**Organizar etiquetas.** Una etiqueta se mueve a un grupo de dos maneras
equivalentes:

- *Arrastrando:* se arrastra la etiqueta y se suelta sobre el encabezado de un
  grupo o sobre cualquiera de sus etiquetas. Soltarla en "Sin grupo" la saca de
  su grupo. Durante el arrastre la etiqueta se queda en su sitio al 50% de
  opacidad, una copia sigue al cursor y el grupo de destino se marca con fondo
  `outline` y una barra `primary` de 2px a la izquierda. "Categorías" no acepta
  etiquetas.
- *Menú contextual* (clic derecho), que es el menú nativo de macOS. Sobre una
  etiqueta: "Mover a" (submenú con los grupos y "Nuevo grupo…") y "Quitar del
  grupo". Sobre un grupo: "Renombrar" y "Eliminar grupo". Eliminar pide
  confirmación en línea, justo debajo del encabezado del grupo: una fila con la
  pregunta "¿Eliminar grupo?" en `error` y dos botones de icono, confirmar
  (marca de verificación, en `error`) y cancelar (aspa). La fila no repite el
  nombre: el encabezado del grupo se resalta (fondo `outline`, texto
  `on-surface`) y comparte con la pregunta una barra `error` de 2px a la
  izquierda. Debajo, en `secondary`, la consecuencia: "Sus etiquetas quedan
  sin grupo". El foco cae en cancelar, y `Esc` cancela mientras el foco esté
  en la pregunta.

"Nuevo grupo" añade un grupo al final con su nombre en edición en línea.

**Mover un bookmark a otro perfil.** De dos maneras equivalentes, más el
comando de la paleta para el bookmark seleccionado:

- *Arrastrando:* se arrastra una fila de la lista y se suelta sobre otro perfil
  del carril. Durante el arrastre el perfil de destino tiene el aspecto de
  `profile-item` seleccionado; el perfil activo y "Nuevo perfil" no aceptan
  bookmarks. Soltarla en otro sitio no hace nada. Con un
  solo perfil las filas no se arrastran.
- *Menú contextual* (clic derecho sobre la fila), el nativo de macOS: "Mover a
  perfil", un submenú con los demás perfiles. Con un solo perfil el menú no se
  abre.

El bookmark sale de la lista, la app sigue en el perfil activo y, si estaba
seleccionado, el detalle queda sin selección; en el destino se le vuelven a
calcular los embeddings. Si el destino ya tiene esa URL, el error llega como
notificación.

**Detalle expandido.** "Expandir" oculta la lista y el detalle ocupa también su
espacio; la barra lateral sigue visible y la columna de contenido se centra. El
icono pasa a ser "Contraer", que devuelve la lista. Elegir otro elemento en la
barra lateral también contrae.

**Paleta de comandos.** Superposición de 640px centrada horizontalmente, a 96px
del borde superior. Se abre con `⌘K`, con el botón "Comandos" o, directamente
en modo Bookmarks, con el icono "Buscar". De arriba abajo: campo de búsqueda
con icono de lupa; lista de resultados (el resto con scroll); y un pie con los
modos como pestañas a la izquierda (Comandos `⌘1`, Bookmarks `⌘2`, Etiquetas
`⌘3`) y la acción de la fila seleccionada con su tecla (`↵`) a la derecha. Las
flechas mueven la selección, `↵` ejecuta y `Esc` cierra.

- *Comandos:* todas las acciones de la app, cada una con icono, nombre y atajo
  si lo tiene: las generales, las del bookmark seleccionado (abrir, reintentar,
  eliminar, mover a cada uno de los demás perfiles), renombrar y eliminar cada grupo, mover la etiqueta seleccionada,
  mostrar u ocultar la barra lateral y elegir el tema. Elegir las esquinas solo
  existe en Ajustes.
  Reindexar, procesar sin resumen y probar la conexión se llaman "Ajustes: …" y
  abren Ajustes, donde están su confirmación y su resultado. No hay ninguna
  acción que solo exista en la paleta.
- *Bookmarks:* es la búsqueda de la app, sobre todos los bookmarks. Cada
  resultado ocupa dos líneas (52px): título y fragmento, con las coincidencias
  resaltadas, y el dominio a la derecha. `↵` lo abre: la barra lateral pasa a
  "Todos", la lista lo selecciona y el detalle lo muestra.
- *Etiquetas:* filtra etiquetas por nombre. `↵` selecciona la etiqueta en la
  barra lateral.

### Ajustes

Ajustes ocupa el espacio de la lista y el detalle (el carril sigue a la
izquierda), y la barra lateral pasa a
navegarlo: conserva la marca, pero en lugar de las etiquetas muestra

- "Biblioteca", con un chevrón hacia la izquierda, que vuelve a la biblioteca
  tal como estaba (misma selección y mismo bookmark);
- dos encabezados con sus secciones como elementos de la barra debajo:
  "Aplicación" (Apariencia e Inteligencia artificial, comunes a todos los
  perfiles) y "Perfil · nombre del perfil activo" (Perfil, Extensión del
  navegador, Importar y exportar y Mantenimiento, que se refieren a ese
  perfil y cambian con él). La sección abierta lleva el relleno `primary` de
  un elemento seleccionado.

Apariencia lleva tres filas de `setting-row`: "Tema" (`segmented-control`),
"Idioma" (`input` de selección, `SelectField`), con los nombres de idioma en su
propio idioma, y "Esquinas" (`segmented-control` con 0, 2, 4, 6 y 8 en
`label-md`, y la ayuda "Radio de los paneles y las ventanas, en píxeles. Por
defecto, 4."; se aplica al momento). Elegir un idioma muestra debajo un
`inline-confirm` neutral ("marks se reiniciará para cambiar el idioma"); al
aceptar, la app se reinicia.

La sección Perfil edita el perfil activo: un formulario con Nombre, Iniciales
(monoespaciada, de `field-width`) y la fila de "Guardar" con su resultado, y
debajo, en un bloque aparte, "Eliminar perfil" con `inline-confirm` de texto
que dice cuántos bookmarks se borran y que no se puede deshacer. En el primer
perfil el botón está deshabilitado y su ayuda lo explica.

Mientras Ajustes está abierto, el icono "Nuevo grupo" de la cabecera del panel
no se muestra y el botón "Ajustes" del pie del carril aparece activo
(fondo `outline`, icono `on-surface`); pulsarlo otra vez, o `⌘,`, también
vuelve a la biblioteca. El botón de plegar la barra lateral va al final de la
franja.

El contenido muestra una sola sección: franja superior (38px) con "Ajustes ·
nombre de la sección" en `label-md`, y debajo, con 24px de relleno, una
columna centrada de 720px como máximo (la misma del detalle expandido) con el
título en `headline-lg` y los controles. Ajustes se abre en la última sección visitada de la sesión, y
la primera vez en Apariencia. Los comandos "Ajustes: …" de la paleta abren la
sección que les corresponde.

Escala de espaciado: `xxs` 2, `xs` 4, `sm` 8, `md` 12, `lg` 16, `xl` 24. Toda
separación, relleno o margen sale de ella. Las
filas de la lista tienen 8px de relleno vertical y 12px horizontal (la barra
de selección cuenta dentro de los 12px de la izquierda); el detalle, 24px. Los controles
(botones, campos, botones de icono) miden 28px de alto.

### Notificaciones

Todo error, aviso o información de la sesión queda en el panel de
notificaciones del perfil activo. Las globales (la IA y la API de la extensión)
se ven en todos los perfiles. No sobreviven a salir de la app.

- **Panel** (`notification-panel`): se abre con la campana de la barra
  inferior o con el comando "Notificaciones" de la paleta. Sale abajo a la
  derecha, a `panel-gap` del borde de la ventana y a `panel-gap` por encima de
  la barra inferior, sobre la lista o el detalle. Cabecera: "Notificaciones", con "· N sin leer" si hay
  alguna, y los botones de icono "Marcar todo como leído", "Vaciar" y
  "Cerrar". Debajo, la lista, la más reciente arriba, con scroll propio; sin
  ninguna, el mensaje vacío en `secondary`.
- **Fila** (`notification-item`): arriba, la gravedad escrita (Error, Aviso,
  Info) en su color, el contador ×N si se repitió y la hora (`HH:MM`) a la
  derecha; en medio, el mensaje; abajo, el detalle (título, dominio u
  operación) y las acciones. "Abrir bookmark" y "Reintentar" van en las que
  nombran un bookmark ("Reintentar" solo si sigue fallido), "Abrir Ajustes" en
  las de la IA y la API, y "Descartar" en todas. Las no leídas llevan una barra
  `primary` de `indicator` a la izquierda y dicen "sin leer" a los lectores de
  pantalla. Una notificación de un bookmark de otro perfil (un toast que
  sobrevivió a un cambio de perfil) no ofrece "Abrir" ni "Reintentar": los `id`
  se repiten entre perfiles.
- **Agrupación:** un mensaje repetido (mismo perfil, gravedad y texto) no añade
  fila: suma ×N, sube arriba y vuelve a "sin leer". Un fallo de fondo que se
  repite igual en cada vuelta del worker suma ×N sin volver a marcar la
  campana. La lista guarda 200 como mucho y pierde las más antiguas de un
  perfil antes que las globales.
- **Toasts** (`toast`): solo para lo que el usuario acaba de hacer (un fallo
  suyo, o el final de "Volver a extraer" o "Extraer con IA"); lo de fondo solo
  marca la campana. Se apilan en la misma esquina que el panel, la más nueva
  abajo: tres como mucho, con "+N más en Notificaciones" encima, un
  `button-link` que abre el panel. El aspa de un toast es "Cerrar": oculta el
  toast y la notificación sigue en el panel. Duran 6 s y no vencen mientras el
  cursor o el foco están encima (cada uno cuenta por su lado); el que sale de la
  vista empujado por uno nuevo vuelve a correr. Un
  error repetido sustituye a su toast y su reloj vuelve a empezar. Con el
  panel abierto no hay toasts: lo nuevo entra en la lista a la vista.
- **Cierre y lectura:** el panel se cierra con la campana, con "Cerrar", con
  `Esc` (si no hay paleta ni diálogo abiertos) o con un clic fuera del panel y
  de la campana. Al cerrarlo se marcan como leídas las que se veían; abrirlo no
  marca nada, y "Marcar todo como leído" lo hace sin cerrarlo. "Vaciar" quita
  todas, también las globales, en todos los perfiles. Al abrirse, el foco entra
  en el panel; al cerrarse con "Cerrar", `Esc` o una acción, vuelve a la
  campana (con un clic fuera se queda donde se hizo clic). La campana dice si
  está abierto (`aria-expanded`). Un `Esc` que ya atendió una pregunta en línea
  o un campo no cierra el panel.
- **Formularios:** un error de formulario sigue en su sitio, en `error-text`,
  y además se copia al panel, sin toast. Las confirmaciones ("Ajustes
  guardados", "Token copiado") se quedan solo en su sitio.

## Elevation & Depth

Diseño plano, sin sombras. La jerarquía se expresa con dos niveles tonales
(`neutral` y `surface`) y con los huecos entre paneles. Las líneas `outline`
de 1px quedan para filas, campos y superposiciones. Las únicas superposiciones
propias son el diálogo "Añadir", la paleta de comandos, el panel de
notificaciones y los toasts: fondo `surface` y borde `outline`. El diálogo y
la paleta son modales y llevan un velo `neutral` al 60% sobre el resto de la
ventana; el panel y los toasts no lo son y no llevan velo. El menú contextual
es el del sistema y trae su propia sombra.

## Shapes

Esquinas de 4px (`md`) por defecto en paneles y superposiciones, y 2px (`sm`)
en controles. En Ajustes › Apariencia, "Esquinas" fija `md` en 0, 2, 4, 6 u
8px; `sm` es el menor de 2px y `md`. Nada más se redondea por su cuenta. No hay
píldoras ni círculos: el punto de estado es un cuadrado de 8px. Las coincidencias de búsqueda no llevan radio, como un trazo
de rotulador.

## Components

**Regla.** Cada entrada de `components` del encabezado es un componente de
`apps/desktop/src/lib/ui/` (o un estilo de elemento en `app.css`), y las
pantallas solo los componen: fuera de `lib/ui/` no hay `<button>`, `<input>`,
`<select>` ni `<textarea>`, ni colores o tipografías escritos a mano. Un
cambio de aspecto se hace en un único sitio. La tabla dice qué fichero
implementa cada entrada; un test comprueba que la tabla y la carpeta coinciden.

**Estados.** Una variante o un estado es una entrada aparte con sufijo
(`-hover`, `-selected`, `-pressed`, `-danger`, `-deleting`, `-inset`,
`-hit`). Tres
estados son iguales en todos los componentes y por eso no tienen entrada:

- *Foco de teclado:* contorno de 2px con 1px de separación. En el tema oscuro
  es `primary`; en el claro es `on-surface-light`, porque el amarillo no tiene
  contraste sobre fondo claro. El "cancelar" de una confirmación lo muestra
  siempre que tiene el foco, también si la pregunta se abrió con el ratón.
- *Deshabilitado* (botones y campos): 50% de opacidad y sin reacción al
  cursor.
- *Pulsado:* no hay estado propio; es el mismo aspecto que bajo el cursor.

Todo componente con el que se interactúa reacciona al cursor: tiene su entrada
`-hover` o comparte la de su variante. La única excepción es `command-item`,
donde el cursor mueve la selección.

| Entrada | Fichero | Variantes y estados |
|---|---|---|
| `button-primary`, `button-secondary`, `button-danger`, `button-link` | `ui/Button.svelte` | `-hover` en las tres primeras; `button-link`, subrayado bajo el cursor |
| `icon-button` | `ui/IconButton.svelte`, `ui/Icon.svelte` | `-hover`, `-pressed`, `-danger`, `-marked` (con su `icon-button-mark`) |
| `input`, `input-inset` | `app.css`, `ui/TextField.svelte`, `ui/SelectField.svelte`, `ui/TextArea.svelte` | foco: borde del color del foco |
| `field-label` | `ui/Field.svelte` | |
| `detail-title` | `ui/TitleField.svelte` | borde `outline` bajo el cursor |
| `segmented-control`, `segment` | `ui/Segmented.svelte` | `-hover`, `-selected` |
| `topbar`, `topbar-title` | `ui/Topbar.svelte` | |
| `wordmark` | `ui/Wordmark.svelte` | |
| `sidebar-heading` | `ui/SidebarHeading.svelte` | |
| `profile-rail` | `ui/ProfileRail.svelte` | con pie |
| `panel` | `ui/Panel.svelte` | |
| `sidebar-toggle` | `features/layout/SidebarToggle.svelte` | un `icon-button` (`-hover`, `-pressed`) |
| `profile-item`, `profile-add` | `ui/ProfileItem.svelte` | `-hover`, `-selected`; destino de arrastre igual que `-selected` |
| `sidebar-item` | `ui/SidebarItem.svelte` | `-hover`, `-selected`; anidado, con asa, arrastrándose |
| `sidebar-group`, `drop-target` | `ui/SidebarGroup.svelte` | `-hover`, `-deleting`, destino de arrastre |
| `list-row`, `list-row-summary`, `list-row-meta` | `ui/ListRow.svelte` | `-hover`, `-selected`; arrastrable a un perfil |
| `status-dot-*` | `ui/StatusDot.svelte` | un color por estado; `status-dot-imported` hueco (borde de 1px `secondary`); `status-dot-extracting` (`primary`; en claro `on-surface-light`; quieto) mientras se extrae |
| `tag`, `tag-field` | `ui/Tag.svelte`, `ui/TagField.svelte` | quitar: `on-surface` bajo el cursor; campo: borde `secondary` bajo el cursor |
| `inline-confirm` | `ui/InlineConfirm.svelte` | compacta (iconos) o con texto; `-neutral` para lo que no reemplaza ni elimina |
| `setting-rows`, `setting-row`, `setting-help` | `ui/SettingRows.svelte`, `ui/SettingRow.svelte` | |
| `command-line` | `ui/CommandLine.svelte` | lleva un `copy-button` |
| `copy-button` | `ui/CopyButton.svelte` | `icon-button` (`-hover`); `-copied` 1,5 s tras copiar, con aviso al lector de pantalla |
| `detail-url` | `ui/DetailUrl.svelte` | una línea con elipsis en medio |
| `clamp-text` | `ui/ClampText.svelte` | recortado o entero; el conmutador es un `button-link` |
| `command-palette` | `app.css`, `CommandPalette.svelte` | es un `dialog` |
| `command-item` | `ui/CommandItem.svelte` | `-hit`, `-selected` (el cursor selecciona la fila) |
| `palette-search` | `ui/PaletteSearch.svelte` | |
| `palette-tab` | `ui/PaletteTab.svelte` | `-hover`, `-selected`; las acciones del pie, sin `-selected` |
| `empty-state` | `ui/EmptyState.svelte` | |
| `kbd` | `app.css`, `ui/Keys.svelte` | combinación: teclas separadas por `·` |
| `search-highlight`, `section-label`, `dialog`, `link`, `error-text` | `app.css` | |
| `code-block` | `ContentView.svelte` | solo dentro del Markdown renderizado |
| `detail-meta`, `detail-description` | `Detail.svelte` | |
| `detail-favicon` | `ui/Favicon.svelte` | sin estados: no se interactúa con él |
| `content-link` | `ui/ContentLink.svelte` | `-hover` |
| `content-media` | `ui/ContentMedia.svelte` | sin `-hover` propio: la acción "Abrir" es un `button-secondary` |
| `content-callout`, `content-table`, `content-footnotes` | `ContentView.svelte` | solo dentro del Markdown renderizado |
| `content-raw` | `ContentView.svelte` | solo si el Markdown no se puede dibujar |
| `detail-blocked` | `ui/DetailNotice.svelte` | la acción es un `button-secondary` |
| `status-bar` | `ui/StatusBar.svelte` | un panel con tres tramos |
| `status-item` | `ui/StatusItem.svelte` | `-hover` si es un botón; con punto, icono o atajo |
| `resize-handle` | `ui/Resizer.svelte` | `-hover` (tras 120ms), `-active` al arrastrar o con el foco; teclado y doble clic |
| `notification-panel` | `ui/NotificationPanel.svelte` | vacío: el mensaje en `secondary` |
| `notification-item`, `notification-item-meta` | `ui/NotificationItem.svelte` | `-unread`: barra `primary` de `indicator` |
| `toast` | `ui/Toast.svelte` | barra del color de la gravedad |

### Botones

- **`button-primary`**, **`button-secondary`**, **`button-danger`**: botones
  con texto. Solo en diálogos, Ajustes y confirmaciones; uno principal
  (amarillo) como mucho. El secundario lleva fondo `neutral` y borde `outline`;
  el de peligro, texto `error` sin relleno ni borde. Bajo el cursor el
  principal pasa a `primary-hover` y los otros dos a fondo `outline`.
- **`button-link`**: solo el texto, en `body-md` y `secondary`, sin borde ni
  relleno; subrayado bajo el cursor. Para una acción discreta sobre una
  superposición, donde un botón con caja pesaría demasiado: "+N más en
  Notificaciones" sobre los toasts.
- **`icon-button`**: el control de acción de las zonas, sin relleno ni
  borde: se ve como un icono, no como un botón. 28×28px de área, icono de
  16px dibujado como SVG en línea con `currentColor` y trazo de
  `icon-stroke-width` (en la rejilla de 24 unidades del icono), sin
  librería de iconos. Color `secondary`; bajo el cursor o activo
  (`-pressed`), fondo `outline` y color `on-surface`. Siempre con
  `aria-label` y `title` (el nombre de la acción y su atajo), porque no lleva
  texto visible. "Eliminar" usa `-danger`.
- **Iconos:** nuevo grupo (carpeta con `+`), buscar (lupa), comandos (`⌘`),
  barra lateral (recuadro con una columna a la izquierda), ajustes (engranaje), notificaciones (campana), añadir (`+`), expandir y contraer (flechas diagonales
  hacia fuera y hacia dentro), abrir en el navegador (flecha que sale de un
  recuadro), reintentar (flechas en círculo), eliminar (papelera), grupo
  plegado y desplegado (chevrón a la derecha y hacia abajo), volver (chevrón a
  la izquierda), asa de arrastre (seis puntos), confirmar (marca de
  verificación) y cancelar (aspa).

### Campos

- **`input`**: campo de texto, desplegable o área de texto. Fondo `surface`,
  borde `outline`; con foco, el borde toma el color del foco de teclado. Los
  tres comparten un único estilo (la clase `input` de `app.css`). Un campo
  dentro de un contenedor que ya dibuja el borde (el nombre de un grupo en
  la barra lateral) va sin borde ni fondo.
  **`input-inset`** es el mismo campo sobre un panel `surface` (las notas del
  detalle, el diálogo): fondo `neutral`. Dentro de un bloque de Ajustes,
  que ya es `neutral`, el campo es un `input` (`surface`). Los datos (URL, nombre de modelo, token)
  van en `label-md`. En un diálogo la etiqueta va encima, en `help`.
- **`detail-title`**: el título del bookmark, editable en su sitio; el borde
  solo aparece bajo el cursor (`outline`) o con foco.
- **`segmented-control`** y **`segment`**: para elegir una entre pocas
  opciones fijas (el tema, las esquinas). Fondo `surface`, porque va sobre un
  bloque de Ajustes. Segmentos de `row-height` en `label-md` dentro de
  un borde `outline`, separados por una línea `outline`; bajo el cursor el
  texto pasa a `on-surface` y el elegido lleva relleno `primary` con texto
  `on-primary`.
- **`tag`**: texto `secondary` en `label-md` con borde `outline`, 24px de
  alto; el botón de quitar es un aspa dentro de la propia etiqueta. El campo
  para añadir lleva borde discontinuo.

### Navegación

- **`topbar`** y **`topbar-title`**: franja superior de cada zona
  (`topbar-height`), que también arrastra la ventana. El texto va en
  `label-md` y `secondary`; la parte que nombra lo mostrado, en `on-surface`.
  Fondo `neutral` y sin línea: el panel empieza debajo.
- **`wordmark`**: la marca `marks` en `headline-md`, sin adorno. Va en la
  franja superior de la barra lateral, tras los semáforos.
- **`sidebar-heading`**: cabecera del panel de la barra lateral: una barra
  `primary` de `indicator` de ancho y el nombre de lo que muestra (el perfil
  activo) en `headline-md` y `on-surface`, en una línea con elipsis, con `sm`
  encima y debajo para que se lea como cabecera. Al final de la línea,
  pegados al borde derecho, sus botones de icono ("Nuevo grupo").
- **`icon-button-marked`**: un botón de icono con algo nuevo detrás (las
  notificaciones sin leer): el icono pasa a `on-surface` y lleva en su esquina
  superior derecha una marca (**`icon-button-mark`**): un cuadrado `primary`
  de `status-dot-size` y radio `sm`. Al aparecer, la marca late tres veces
  (crece y vuelve, 1 s cada vez) y se queda quieta; con "reducir movimiento"
  activado en el sistema no late. Es el único movimiento de la interfaz. No
  lleva número: cuántas son lo dicen el `title` y el `aria-label` del botón.
- **`profile-rail`**: el carril de perfiles, de `rail-width`, con el fondo
  `neutral` del marco y sin línea a su lado. Franja superior de
  `topbar-height` vacía, que arrastra la ventana; debajo, los elementos
  centrados, separados `sm`. Al pie lleva el botón "Ajustes" (`⌘,`), un
  `icon-button` centrado, de `control-height` de alto y con `panel-gap`
  debajo, a la altura de la barra inferior; se ve pulsado
  (`icon-button-pressed`) mientras Ajustes está abierto.
- **`profile-item`**: caja de `control-height` de lado, radio `sm`, iniciales
  en `label-md` y texto `secondary`, sin relleno propio (el fondo del
  carril); bajo el cursor, fondo `outline` y texto `on-surface`, como un
  `icon-button`. Seleccionado (el perfil activo), fondo `outline`, texto
  `on-surface` y una barra `primary` de `indicator` en el borde izquierdo del
  carril, no de la caja. Como destino de arrastre (un bookmark encima), el
  mismo aspecto que seleccionado. `title` y `aria-label`: el nombre del perfil
  y su atajo.
- **`profile-add`**: la misma caja, sin relleno, con el icono `+` y borde
  discontinuo `outline`, como el campo de añadir etiqueta; bajo el cursor,
  borde `secondary` e icono `on-surface`.
- **`sidebar-item`**: texto `secondary` en `label-md`, `row-height` de alto;
  `on-surface` bajo el cursor; relleno `primary` con texto `on-primary` cuando
  está seleccionado. Las etiquetas llevan a la derecha un asa de arrastre de
  12px, siempre visible y atenuada, y el cursor de agarrar; mientras se
  arrastra, la etiqueta se queda en su sitio al 50% de opacidad. "Biblioteca"
  lleva delante un chevrón de 12px.
- **`sidebar-group`**: chevrón de 12px y nombre en `label-sm`, en mayúsculas
  y color `secondary`; `on-surface` bajo el cursor. Como **`drop-target`**
  (el grupo sobre el que se va a soltar una etiqueta) usa fondo `outline`,
  texto `on-surface` y barra `primary` de `indicator`. Como
  **`sidebar-group-deleting`**, lo mismo con barra `error`, compartida con la
  pregunta que aparece debajo.
- **Menú contextual:** el nativo de macOS. No se dibuja ni se le aplican
  tokens.

### Lista y estado

- **`list-row`**: título en `label-lg`, resumen (`list-row-summary`) y
  dominio y fecha (`list-row-meta`). Bajo el cursor o seleccionada usa fondo
  `neutral`; la seleccionada añade una barra `primary` de `indicator` en el
  borde izquierdo.
- **`status-dot-*`**: cuadrado de 8px con el color del estado y siempre con
  `title` y `aria-label`, porque el color solo no basta. Donde el estado va
  escrito al lado (el detalle, Ajustes), el punto es decorativo.
  `status-dot-extracting` (`primary`, sin animación) sustituye al del estado
  mientras el bookmark se está extrayendo; no es un estado, y no tiene
  variante clara (`status-dot-extracting-light`) usa `on-surface-light`, como
  el anillo de foco: `primary` sobre `neutral-light` da 1,35:1 y sobre
  `surface-light` 1,24:1, y `on-surface-light` da 17,0:1 y 15,6:1 (mínimo 3:1).
  `status-dot-imported` ("Sin extraer": importado y aún sin poner en cola) es
  el único hueco: sin relleno y con un borde de 1px `secondary`, para que no se
  confunda con `status-dot-pending`, que ya está en cola.
- **`empty-state`**: mensaje centrado en `secondary` cuando no hay nada que
  mostrar.

- **`detail-meta`** y **`detail-description`**: una línea bajo la URL (sitio ·
  autor · fecha · minutos de lectura) en `label-md` y `secondary`, y la
  descripción de la página bajo el título en `body-md` y `secondary`.
- **`detail-favicon`**: el favicon de la página, descargado y guardado por
  core al guardar o volver a extraer el bookmark, y mostrado como `data:` URL:
  abrir un bookmark no pide nada a la red. De `icon-size` y con 4px (`xs`) de
  separación, abre la línea de `detail-meta`, delante del sitio. Es decorativo
  (`alt` vacío) y, si falta o no se puede decodificar, no queda nada: ni hueco
  ni icono de imagen rota. Puede ser PNG, JPEG, GIF, WebP, ICO, AVIF o SVG; el
  SVG solo como `<img>`, nunca en línea. Un fallo de red se reintenta más
  tarde, y entretanto no se muestra nada. Los bookmarks que ya había antes de
  la migración 6 no lo descargan solos; al volver a extraerlos, sí. La imagen
  de la página (`og:image`) se guarda como URL y no se muestra.
- **`content-link`**: el `link` de siempre dentro del contenido. Bajo el cursor
  el subrayado pasa a `primary-hover`. No navega la webview: abre la URL en el
  navegador.
- **`content-media`**: sustituye cada imagen, `iframe`, `video` o `audio` por
  una línea sobre `neutral` con icono, el texto alternativo o el dominio (en
  `label-md`, `secondary`) y la acción "Abrir" como `button-secondary`. No se
  carga nada; sin URL no hay acción.
- **`content-callout`**: cita con el tipo del aviso como `section-label` y una
  línea `outline` de 1px a la izquierda.
- **`content-table`**: bordes `outline` y desplazamiento horizontal propio.
- **`content-footnotes`**: `body-sm` tras una línea `outline`.
- **`content-raw`**: si el Markdown no se puede dibujar (un anidamiento hostil),
  el aviso en `secondary` y el texto tal cual en `body-lg`, con sus saltos de
  línea.
- **`detail-blocked`**: aviso de página bloqueada, con borde `outline`; el
  mensaje a la izquierda y un `button-secondary` a la derecha.
- El `==resaltado==` no usa `mark`, que significa "coincidencia de búsqueda".

### Confirmaciones y ajustes

- **`inline-confirm`**: toda acción destructiva pregunta en su sitio, sin
  diálogo. Dos formas: con texto (pregunta en `error`, botón de peligro y
  "Cancelar") y compacta para sitios estrechos (dos botones de icono:
  confirmar en `error` y cancelar). El foco cae en cancelar y `Esc` cancela
  mientras el foco esté en la pregunta.
  **`inline-confirm-neutral`**: la variante para una acción que no reemplaza ni
  elimina contenido (extraer los pendientes de una vista, importar, reindexar): pregunta en
  `on-surface` y el sí como botón secundario (o icono sin `-danger` en la
  compacta). El peligro queda para lo que reemplaza o elimina.
- **`setting-rows`** y **`setting-row`**: el contenido de cada sección de
  Ajustes es un bloque con fondo `neutral`, borde `outline` y radio `md`,
  dividido en filas de `setting-row-min-height` como mínimo separadas por una
  línea `outline`. Cada fila lleva el nombre a la izquierda y debajo, como
  **`setting-help`**, su ayuda, estado o resultado (en `error` si es un
  fallo); a la derecha el control: campo de `field-width`, control segmentado
  o botones. La última fila de un formulario lleva el resultado a la
  izquierda y los botones a la derecha, con el principal al final. Las
  confirmaciones ocurren en la misma fila.
- **`copy-button`**: un `icon-button` (dos hojas) que copia al portapapeles
  un texto entero, aunque en pantalla se vea abreviado. Al copiar, durante
  1,5 s el icono pasa a una marca de confirmación en `success`
  (**`copy-button-copied`**), su nombre pasa a "… copiado" y una región
  `status` educada lo anuncia al lector de pantalla. Si el portapapeles falla,
  no cambia nada y el texto sigue seleccionable.
- **`detail-url`**: la URL del detalle en una sola línea, `label-md` y `link`.
  Si no cabe, se corta en medio: se ve el principio (esquema, host y ruta) con
  "…" y siempre el final de la URL (sus últimos 32 caracteres). El `title` y
  el lector de pantalla tienen la URL entera, y el clic abre la original.
- **`clamp-text`**: un texto largo recortado a dos líneas con "…". Solo si no
  cabe aparece debajo un `button-link` "Ver más" (con `aria-expanded`), que lo
  muestra entero y pasa a "Ver menos". Las palabras sin espacios (URL, rutas)
  se parten donde haga falta. El lector de pantalla lee siempre el texto
  entero.
- **`command-line`**: bajo la ayuda de un ajuste, una orden para ejecutar en
  otro programa (`ollama pull qwen3:8b`) en `label-md` sobre `neutral`, con
  borde `outline` y radio `sm`, seguida de un `copy-button`; el
  texto también se puede seleccionar. Los campos de modelo de Ajustes → IA
  dicen en `setting-help` el modelo recomendado para el proveedor elegido
  (y, si es local, su descarga y la memoria que pide), y un
  `button-secondary` "Usar" a la izquierda del campo lo rellena cuando el
  campo tiene otro.

### Paleta y superposiciones

- **`command-palette`**: superposición de `palette-width` a `palette-top`
  del borde superior. Campo de búsqueda de `palette-search-height` en
  `body-lg` con una línea `outline` debajo; pie con los modos como pestañas
  (fondo `outline` y texto `on-surface` en la elegida) y, a la derecha, las
  acciones "Ejecutar" (o "Abrir") y "Cerrar" con el mismo aspecto de pestaña,
  sin estado elegido: se pulsan igual que `↵` y `esc`.
- **`command-item`**: fila de `command-item-height`; un resultado de búsqueda
  (`command-item-hit`) ocupa dos líneas. La seleccionada (`-selected`) usa
  fondo `outline` y la barra `primary` de `indicator`. No hay aspecto propio
  bajo el cursor: mover el cursor sobre una fila la selecciona, así la fila
  resaltada es siempre la que ejecuta `↵`. Al salir el cursor de la lista
  vuelve la fila elegida antes de que entrara, para que "Ejecutar" en el pie
  no lance la última fila que el cursor cruzó al bajar.
- **`dialog`**: fondo `surface`, borde `outline`, radio `md` y un velo
  `neutral` al 60% sobre el resto de la ventana.
- **`kbd`**: atajo en `label-md`, color `secondary`, fondo `neutral` y borde
  `outline`. Una combinación es una sola caja con sus teclas separadas por un
  punto centrado (`⌥·⌘·2`), sin más hueco que el del propio carácter; el punto
  es decorativo y no lo leen los lectores de pantalla. Los atajos dentro de una
  frase o de un `title` ("Añadir (⌘N)") van sin punto.
- **`search-highlight`**: en los resultados de la paleta, el término
  encontrado se rellena con `primary` y texto `on-primary`, sin radio.
- **`section-label`**: encabezado de un apartado del detalle, de la paleta o
  de un bloque de una sección de Ajustes, en `label-sm`, mayúsculas y
  `secondary`.
- **`error-text`**: todo mensaje que informa de un fallo (en un diálogo, en
  la paleta, en el detalle o bajo un ajuste) va en `error`.
- **`link`**: texto `on-surface` con subrayado `primary` de `indicator`.
  Nunca texto amarillo.
- **`code-block`**, dentro del contenido: código en `label-md` sobre
  `neutral`.

### Barra inferior y notificaciones

- **`status-bar`**: la barra inferior, un panel de `control-height`, fondo
  `surface`, radio `md`, sin línea y texto en `label-md` y `secondary`. Va bajo
  la barra lateral, la lista y el detalle, con `panel-gap` debajo y a la
  derecha. Tres tramos: la columna de la barra lateral (del ancho actual de la
  barra lateral; con ella oculta se ajusta a "Comandos", con la misma
  transición), el centro, que se recorta con elipsis y nunca empuja a la
  derecha, y la derecha (la campana). "Ajustes" no está en la barra: vive al
  pie del carril.
- **`status-item`**: un elemento de la barra, en una línea con elipsis, con un
  `status-dot-*`, un icono o un atajo (`kbd`). Si lleva acción es un botón y,
  bajo el cursor (`status-item-hover`), toma fondo `outline` y texto
  `on-surface`; si no, es solo texto. Lo que pide atención (los fallidos) va en
  `on-surface`.
- **`notification-panel`**: fondo `surface`, borde `outline`, radio `md`, de
  `notification-panel-width` y como mucho `notification-panel-max-height`, con
  scroll solo en la lista. La cabecera, con una línea `outline` debajo, lleva
  la etiqueta en `label-md` y `secondary` y los botones de icono a la derecha.
  Es una región con nombre ("Notificaciones") para los lectores de pantalla.
- **`notification-item`**: una fila de la lista, con `body-md` y `on-surface`
  para el mensaje y `notification-item-meta` (`label-md`, `secondary`) para la
  hora y el detalle. La gravedad va en `label-sm`, en mayúsculas y en su
  color, con un cuadrado de `status-dot-size` del mismo color; el contador ×N
  es una caja como `kbd`. Las acciones son botones de icono. Filas separadas
  por una línea `outline`. `-unread`: barra `primary` de `indicator` a la
  izquierda.
- **`toast`**: un `notification-item` en su caja (`surface`, borde `outline`,
  radio `md`, de `notification-panel-width`), con una barra de `indicator` a la
  izquierda del color de la gravedad (`error`, `warning` o `secondary`) en vez
  de la de "sin leer". Ningún toast lleva rol de aviso propio: los anuncia una
  vez el contenedor de los toasts (`role="status"`).

## Do's and Don'ts

- Do: usa el amarillo como relleno con texto `on-primary`, o como indicador de
  2px.
- Don't: no escribas texto en amarillo ni lo uses en bordes de más de 2px,
  iconos o ilustraciones.
- Don't: no pongas botones con relleno en las zonas; ahí las acciones son
  iconos. El relleno amarillo de acción es solo para el botón principal de
  diálogos y Ajustes.
- Don't: no añadas campos de búsqueda a las zonas; la búsqueda vive en la
  paleta.
- Do: todo botón de icono lleva `aria-label` y `title`.
- Do: cada tipo de acción de la interfaz (añadir, buscar, abrir, reintentar,
  eliminar; crear, renombrar y eliminar grupos; mover etiquetas; mover un
  bookmark a otro perfil) existe
  también como comando en la paleta.
- Do: todo lo que se hace arrastrando se puede hacer también desde el menú
  contextual, salvo cambiar un ancho, que se hace también con el teclado sobre
  el asa.
- Do: pon en monoespaciada los datos (dominios, fechas, URLs, etiquetas,
  atajos) y en `system-ui` todo lo que se lee como frase.
- Do: construye cada pantalla con los componentes de `lib/ui/`; si falta
  uno, añade primero su entrada aquí (con sus estados) y luego el componente.
- Don't: no escribas un `<button>` o un `<input>` suelto en una pantalla ni
  ajustes localmente el aspecto de un componente.
- Don't: no añadas sombras, degradados ni píldoras, y no redondees nada fuera de `sm` y `md`: el radio lo elige el usuario.
- Don't: no cargues fuentes, iconos ni imágenes remotas. El favicon de la
  página lo descarga core una vez y se muestra desde la base.
- Do: mantén un contraste mínimo de 4,5:1 en todo el texto, en los dos temas.
- Don't: no transmitas un estado solo con color; acompáñalo de texto accesible.
- Do: todas las cadenas visibles van en `apps/desktop/src/lib/i18n/{es,en}.ts` (se leen con `t`).
