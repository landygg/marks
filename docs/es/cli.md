# Línea de comandos

**Español** · [English](../en/cli.md)

`marks` es un binario de terminal que trabaja sobre la misma base de datos que la app. Sirve para añadir, listar, buscar, importar y exportar desde scripts.

El binario `marks` no se distribuye: se compila desde el código con `cargo build --release -p marks-cli`, que deja `target/release/marks`. Ver [Compilar](developers/building.md).

## Qué hace y qué no

- **Solo encola.** `marks add` guarda la URL y nada más: no descarga la página ni llama a ningún modelo. La app la extrae la próxima vez que esté abierta.
- **Busca solo por texto completo.** La búsqueda semántica está en la app.
- **No necesita la app abierta**, y puede usarse con ella abierta.

## Uso

```
marks [OPCIONES] <COMANDO>
```

| Comando | Qué hace |
|---|---|
| `add <URL>` | Encola la URL |
| `list` | Lista todos los bookmarks |
| `search [CONSULTA]...` | Búsqueda de texto completo (hasta 50 resultados) |
| `export <DIRECTORIO>` | Escribe un `.md` por bookmark |
| `import <FICHERO>` | Importa un `bookmarks.html` |
| `export-html <FICHERO>` | Exporta el perfil a un `bookmarks.html` |
| `stats` | Cuenta los bookmarks por estado |
| `profiles` | Lista los perfiles |

| Opción | Qué hace |
|---|---|
| `--profile <NOMBRE>` | Perfil sobre el que operar. Sin ella, **el primero**, no el activo de la app |
| `--db <RUTA>` | Ruta de la base de datos. También con la variable `MARKS_DB` |
| `-h`, `--help` | Ayuda, general o de un comando (`marks add --help`) |
| `-V`, `--version` | Versión |

No hay subcomando `help`: la ayuda es `--help`.

## Ejemplos

```sh
# Añadir, con etiquetas y una nota
marks add https://example.com --tag lectura --tag rust --notes "para el fin de semana"

# Con un título propio
marks add https://example.com/post --title "Mi título"

# Listar y buscar
marks list
marks search rust ownership

# En otro perfil
marks profiles
marks --profile Trabajo list

# Contar por estado
marks stats
```

Opciones de `add`: `--title <TÍTULO>` (si no, la app usa el de la página), `--notes <NOTAS>` y `--tag <ETIQUETA>`, que se puede repetir.

### Salida

`list`, `search` y `add` escriben una línea por bookmark:

```
#12  [listo]  Título de la página  https://example.com/post
```

Un bookmark fallido añade `(error: …)` al final. Los estados son `pendiente`, `pendiente de indexar`, `listo`, `fallido` y `sin extraer`.

`profiles` escribe `#id  [iniciales]  nombre`.

`stats` escribe cinco líneas: `total`, `pendientes`, `fallidos`, `sin resumen` y `sin extraer`.

## Exportar a Markdown

```sh
marks export ./mis-bookmarks
```

Crea el directorio si no existe y escribe un fichero `<id>.md` por bookmark del perfil. Cada uno empieza con un bloque de metadatos (título, URL, categoría, etiquetas, autor, sitio, fecha de publicación, idioma, descripción y fecha de creación) y sigue con las notas, el resumen y el contenido.

Es la forma de llevarte el **contenido** en un formato legible. marks no vuelve a importar estos ficheros.

## Importar y exportar `bookmarks.html`

```sh
marks import bookmarks.html
marks export-html marks.html
```

- `import` importa **sin preguntar** (la app muestra antes un resumen) y escribe el resultado: `12 nuevos, 3 ya estaban, 1 no importable`. Lo importado queda «sin extraer» hasta que lo pidas en la app.
- `export-html` escribe el perfil entero.

Las reglas de carpetas, etiquetas y grupos son las mismas que en la app. Ver [Importar y exportar](import-export.md).

## Idioma

La ayuda, los errores y la salida están en español o en inglés. El idioma sale, por este orden, de las variables `MARKS_LOCALE` (`es` o `en`), `LC_ALL`, `LC_MESSAGES` y `LANG`, y después del idioma del sistema. Si ninguno es español o inglés, se usa el inglés.

```sh
MARKS_LOCALE=en marks --help
```

En un script, fija `MARKS_LOCALE` si vas a leer la salida.

## Base de datos de pruebas

Para probar sin tocar tus datos:

```sh
MARKS_DB=/tmp/marks-prueba.db marks add https://example.com
MARKS_DB=/tmp/marks-prueba.db marks list
```

## Código de salida

`0` si todo fue bien; distinto de cero si hubo un error, que se escribe en la salida de error.
