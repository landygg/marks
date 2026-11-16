# Datos y copia de seguridad

**Español** · [English](../en/data-and-backups.md)

Todo lo que guardas en marks está en ficheros de tu equipo. No hay copia en ningún servidor: **la copia de seguridad es cosa tuya**.

## Dónde está cada cosa

| Qué | Dónde |
|---|---|
| Primer perfil, lista de perfiles y ajustes de IA | `~/Library/Application Support/marks/marks.db` |
| Los demás perfiles | `~/Library/Application Support/marks/marks-profiles/<n>.db` |
| Claves de API de los proveedores de IA | Llavero de macOS, servicio `dev.landygg.marks` |
| Idioma elegido | Preferencias de macOS de la app (`dev.landygg.marks`) |
| Tema, esquinas y anchos de la ventana | Almacenamiento local de la ventana de la app |
| Registros (logs) | `~/Library/Logs/dev.landygg.marks/` |
| Token de la extensión | En la base de datos (uno por perfil) y en el almacenamiento de la extensión del navegador |

Cada base de datos es un fichero SQLite. Guarda los bookmarks con su contenido, notas, etiquetas, grupos, resúmenes, favicons y los vectores de la búsqueda.

Junto a cada `.db` puede haber dos ficheros auxiliares, `-wal` y `-shm` (por ejemplo `marks.db-wal`). Son parte de la base de datos mientras marks está abierto: no los borres.

`<n>` es el número interno del perfil. `marks profiles` lista los números y los nombres; ver [Línea de comandos](cli.md).

## Hacer una copia de seguridad

1. **Sal de marks** (icono de la barra de menús → «Salir», o ⌘Q). Cerrar la ventana no basta: la app sigue abierta.
2. Copia la carpeta entera `~/Library/Application Support/marks/` a otro disco.

Desde la terminal, con marks cerrado:

```sh
cp -R ~/Library/Application\ Support/marks ~/Desktop/marks-copia-$(date +%F)
```

Si usas Time Machine u otra copia de todo el disco, esa carpeta ya está incluida.

La copia **no incluye**:

- Las claves de API (están en el llavero). Tendrás que escribirlas de nuevo.
- El tema, las esquinas, los anchos y el idioma. Se vuelven a elegir en Ajustes.

## Restaurar una copia

1. Sal de marks.
2. Aparta la carpeta actual, por si acaso: renombra `~/Library/Application Support/marks` a `marks-anterior`.
3. Copia tu carpeta de seguridad a `~/Library/Application Support/marks`.
4. Abre marks.

Restaura siempre la carpeta completa. `marks.db` guarda la lista de perfiles, y cada perfil es un fichero de `marks-profiles/`: mezclar ficheros de copias distintas puede dejar perfiles sin fichero o ficheros sin perfil.

Después de restaurar, el token de cada perfil es el que había en la copia. Si lo habías regenerado después, pega el de ahora en la extensión.

## Copias automáticas antes de una migración

Una versión nueva de marks puede cambiar el formato de la base de datos. A eso se le llama migración.

- marks hace una **copia de cada base antes de migrarla**. La v1.0 es la primera versión, así que la primera copia la hará la primera versión que cambie el formato.
- Una versión antigua de marks **se niega a abrir** una base que no puede leer, con un mensaje claro, en vez de estropearla.
- **No hay vuelta atrás automática.** Para volver a una versión anterior de marks hay que restaurar la copia hecha antes de la migración. Lo que hayas guardado después de actualizar no estará en ella.

Las copias van a una carpeta `backups` junto a cada base:

- la base principal, en `~/Library/Application Support/marks/backups/marks-v<n>-<fecha>.db`;
- el perfil `<p>`, en `~/Library/Application Support/marks/marks-profiles/backups/<p>-v<n>-<fecha>.db`.

`<n>` es la versión del formato antes de migrar, y `<fecha>`, la hora UTC de la copia (por ejemplo `20271102T091500Z`). Se conservan las **3** más nuevas de cada base; las anteriores se borran solas. Si la copia no se puede hacer (por ejemplo, con el disco lleno), marks no migra la base y avisa al abrirse.

Para restaurar una copia:

1. Sal de marks (menú marks → Salir).
2. Instala la versión de marks que usabas antes de actualizar.
3. Copia el fichero de la copia sobre `marks.db` (o sobre `marks-profiles/<p>.db`, para un perfil), con el nombre de ese fichero. Si junto a él hay ficheros `-wal` o `-shm`, bórralos antes.
4. Abre marks.

## Exportar no es una copia completa

«Exportar bookmarks…» escribe un `bookmarks.html` con las URL, los títulos, las fechas y las etiquetas. No lleva notas, resúmenes ni contenido. Sirve para llevar tus bookmarks a un navegador, no para restaurar marks. Ver [Importar y exportar](import-export.md).

`marks export <DIR>` escribe un fichero Markdown por bookmark, con su contenido, resumen y notas. Es legible con cualquier editor, pero marks no lo vuelve a importar. Ver [Línea de comandos](cli.md).

## Mover marks a otro Mac

1. En el Mac antiguo: sal de marks y copia `~/Library/Application Support/marks/`.
2. En el nuevo: instala marks, **no lo abras todavía**, y pon la carpeta en el mismo sitio.
3. Abre marks. Vuelve a escribir las claves de API, si usabas alguna, y empareja de nuevo la extensión.

marks no sincroniza entre equipos. No pongas la carpeta en un servicio de sincronización para usarla desde dos Mac a la vez: dos copias de marks escribiendo en la misma base de datos pueden corromperla.

## Usar otra ubicación

La variable de entorno `MARKS_DB` cambia la ruta de la base principal, tanto en la app como en la línea de comandos. Los demás perfiles van en una carpeta junto a ella, con el mismo nombre sin extensión más `-profiles`: con `MARKS_DB=/tmp/prueba.db`, en `/tmp/prueba-profiles/`.

Está pensada para pruebas y desarrollo. Una app abierta desde el Finder o el Dock no ve las variables de tu terminal.

## Borrar todos los datos

Con marks cerrado, elimina:

- `~/Library/Application Support/marks/`: todos los bookmarks.
- `~/Library/Logs/dev.landygg.marks/`: los registros.
- En **Acceso a Llaveros**, las entradas del servicio `dev.landygg.marks`: las claves de API.
