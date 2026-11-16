# Compilar y verificar

**Español** · [English](../../en/developers/building.md)

## Requisitos

- macOS 14.5 o posterior. Es el mínimo que declara el bundle (`minimumSystemVersion` en `tauri.conf.json`).
- Las herramientas de línea de comandos de Xcode. `scripts/check.sh` y `scripts/build-app.sh` usan `xcrun swiftc`, `codesign` y `PlistBuddy`.
- Rust (edición 2024), con `rustfmt` y `clippy`.
- Node.js y pnpm.
- Opcional, para la IA local: [Ollama](https://ollama.com) con `qwen3:8b` y `nomic-embed-text`.

> Las versiones de las herramientas están fijadas en el repositorio: Rust en `rust-toolchain.toml`, Node.js en `.node-version` y pnpm en el campo `packageManager` del `package.json` de la raíz.

## Preparar el repositorio

```sh
git clone https://github.com/landygg/marks.git
cd marks
pnpm install
```

`pnpm install` en la raíz instala los tres paquetes del workspace: `apps/desktop`, `apps/extension` y `packages/extract`.

## App de escritorio en desarrollo

```sh
MARKS_DB=/tmp/marks-scratch.db pnpm --dir apps/desktop tauri dev
```

Usa siempre `MARKS_DB` para desarrollar. Sin esa variable, la app abre tu base real.

- `MARKS_DB` vale para la app y para el CLI. Los perfiles adicionales se guardan junto a la base, en una carpeta con su nombre más `-profiles`: con el ejemplo anterior, `/tmp/marks-scratch-profiles/`.
- `MARKS_LOCALE=es|en|pseudo` fuerza el idioma de la app, del CLI y de los tests. `pseudo` acentúa y alarga cada texto entre `⟦ ⟧`, para ver lo que no pasó por un catálogo.
- Cambiar el idioma en Ajustes reinicia la app y termina `tauri dev`. Hay que lanzarlo otra vez.
- `tauri dev` no incluye la extensión de Safari.
- Si usas un directorio `target` propio para pruebas, ponlo en `/private/tmp` y no en `/tmp`: Tauri no relanza un binario cuya ruta pasa por un enlace simbólico.

## CLI

```sh
MARKS_DB=/tmp/marks-scratch.db cargo run -p marks-cli -- add https://example.com --tag demo
MARKS_DB=/tmp/marks-scratch.db cargo run -p marks-cli -- list
cargo run -p marks-cli -- --help
```

El CLI solo encola. La app descarga y extrae al abrirse sobre la misma base.

## Build de release

```sh
pnpm --dir apps/desktop tauri build --bundles app
```

Deja `target/release/bundle/macos/marks.app`, con firma ad-hoc. Sin `--bundles app` construye también el DMG, en `target/release/bundle/dmg/`. El DMG automatiza Finder por AppleScript y falla en una sesión sin interfaz gráfica o sin permiso de Automatización.

## App con la extensión de Safari

```sh
scripts/build-app.sh
```

El script:

1. Elige la identidad de firma.
2. Llama a `scripts/build-appex.sh`, que construye la extensión, compila el manejador Swift para `arm64` y `x86_64`, los une con `lipo` y firma el appex con hardened runtime en `target/appex/marks Safari Extension.appex`. Toma la versión de `apps/extension/manifest.json` y el macOS mínimo de `tauri.conf.json`.
3. Construye `marks.app` con `tauri build --bundles app --config src-tauri/tauri.release.conf.json`, que copia el appex a `Contents/PlugIns/`. Tauri firma la app con la misma identidad y hardened runtime.
4. Verifica la firma.

Con `--target <triple>` compila para esa arquitectura (`aarch64-apple-darwin` o `x86_64-apple-darwin`) y deja la app en `target/<triple>/release/bundle/macos/`. Con `--dmg` construye también el DMG, en `target/<triple>/release/bundle/dmg/` (con `CI=true`, sin automatizar Finder). Los demás argumentos (por ejemplo, `--config '<json>'`) pasan a `tauri build`; `--target` y `--dmg` se reconocen en cualquier posición.

La identidad de firma sale, por este orden, de:

1. La variable `APPLE_SIGNING_IDENTITY`, la que usa la release (`marks Code Signing`, el certificado autofirmado).
2. La variable `CODESIGN_IDENTITY`.
3. La primera identidad "Apple Development" válida del llavero.
4. Firma ad-hoc, con un aviso.

Safari solo carga la extensión con «Permitir extensiones no firmadas», con cualquiera de ellas salvo un Developer ID.

Para comprobar el resultado:

```sh
scripts/verify-bundle.sh target/release/bundle/macos/marks.app
```

Sale con 1 si el appex no está en `Contents/PlugIns/`, si le falta el sandbox, si no es universal, si la app o el appex no llevan hardened runtime o si falla `codesign --verify --strict --deep`.

Registra en Safari una sola copia de `marks.app`. Si conviven la de `target/` y otra instalada, Safari puede cargar la extensión equivocada.

## Extensión en Chromium

```sh
pnpm --dir apps/extension build
```

Después, en `chrome://extensions`, activa el modo de desarrollador, pulsa "Cargar descomprimida" y elige `apps/extension/dist`. La app tiene que estar en marcha (vale `tauri dev`). El emparejamiento con el token está en [Extensión del navegador](../browser-extension.md).

`pnpm --dir apps/extension watch` reconstruye al guardar.

## `scripts/check.sh`

Es la verificación completa. Tiene que pasar antes de cada commit. Ejecuta, en este orden:

| Paso | Comando |
|---|---|
| Formato de Rust | `cargo fmt --all --check` |
| Lints de Rust | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests de Rust | `cargo test --workspace` |
| Dependencias de JS | `pnpm --dir apps/desktop install --frozen-lockfile` |
| Build de la extensión | `pnpm --dir apps/extension build` |
| Tests de la extensión | `node --test "apps/extension/*.test.js"` |
| Tipos de `packages/extract` | `pnpm --dir packages/extract check` |
| Tests de `packages/extract` | `pnpm --dir packages/extract test` |
| Formato del frontend | `pnpm --dir apps/desktop format:check` (Prettier) |
| Lints del frontend | `pnpm --dir apps/desktop lint` (ESLint) |
| Tipos del frontend | `pnpm --dir apps/desktop check` (svelte-check) |
| Tests del frontend | `pnpm --dir apps/desktop test` (vitest) |
| Tipos del manejador Swift | `xcrun swiftc -typecheck …`, solo en macOS |

Para iterar sobre una sola parte:

```sh
cargo test -p marks-core
cargo test -p marks-desktop
pnpm --dir apps/desktop test
node --test "apps/extension/*.test.js"
pnpm --dir apps/desktop format      # aplica el formato
```

## Tests con Ollama

Dos tests de integración están ignorados por defecto, porque necesitan Ollama en marcha:

```sh
cargo test -p marks-core ollama_roundtrip -- --ignored
cargo test -p marks-core ollama_long_input_keeps_its_start -- --ignored
```

El primero necesita `qwen3:8b` y `nomic-embed-text`. El segundo, que tarda más de medio minuto, comprueba que una entrada de 60.000 caracteres llega entera al modelo.

## Probar la API local

Con la app en marcha sobre una base de pruebas, copia el token de Ajustes y ejecuta:

```sh
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:47821/v1/ping
```

El resto está en [API local](local-api.md).

## Logs

La app escribe sus logs en stdout y en `~/Library/Logs/dev.landygg.marks/`. El nombre de la carpeta sale del identificador del bundle.
