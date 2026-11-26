# Publicar una versión

**Español** · [English](../../en/developers/release.md)

## 1. Versión

La versión de la app está en tres ficheros, que dicen lo mismo:

- `Cargo.toml` (`[workspace.package]`, que heredan todos los crates)
- `apps/desktop/package.json`
- `packages/extract/package.json`

`Cargo.lock` se actualiza solo al compilar. `tauri.conf.json` no lleva versión: Tauri toma la de Cargo. `scripts/check-version.sh`, que corre al principio de `scripts/check.sh`, comprueba que los dos `package.json` repiten la versión del workspace; con un tag (`scripts/check-version.sh v1.0.0`), también que el tag la nombra y que `tauri.conf.json` ya no lleva la clave pública provisional del updater.

La extensión lleva su propia versión en `apps/extension/manifest.json`, separada de la de la app, y `scripts/build-appex.sh` la lee de ahí para el appex de Safari. Se sube cuando cambia algo de `apps/extension` o de `packages/extract`, porque el appex y el zip de Chromium llevan dentro el extractor (Defuddle); si no sube, Safari se queda con la extensión vieja. `scripts/check-version.sh` no la compara.

## 2. Verificación

```sh
scripts/check.sh
```

Después, las pruebas manuales de la versión.

Pruebas a mano antes de publicar. El updater solo se puede probar con dos versiones publicadas, porque `releases/latest` ignora borradores y prereleases, y los ficheros de un borrador no se descargan sin sesión. Se hace con dos prereleases (`1.0.0-rc.1` y `1.0.0-rc.2`):

1. Para cada una, la versión `1.0.0-rc.N` en los tres ficheros de la versión y una sección `## [1.0.0-rc.N]` en `CHANGELOG.md` (sin ella, la release falla). Etiqueta `v1.0.0-rc.N`.
2. Publica el borrador como prerelease (no lo dejes en borrador).
3. Instala la rc.1 y ábrela con `MARKS_UPDATE_ENDPOINT=https://github.com/landygg/marks/releases/download/v1.0.0-rc.2/latest-darwin-aarch64.json` y una base de pruebas en `MARKS_DB`.

Con esas dos versiones, comprueba:

- Una base creada por la versión anterior, con datos, se abre con la nueva. Si la nueva trae migraciones, se hace la copia en `backups/`; si no, ninguna. `user_version` y `meta.min_reader` quedan como dice [Migraciones](migrations.md) (en la v1.0, `1` y `"1"`).
- Guardar una página desde Chromium (la recomendada) y desde Safari con «Permitir extensiones no firmadas» (experimental).
- Primer arranque desde el DMG descargado con un navegador, con «Abrir igualmente» ([Instalación](../installation.md)).
- Importar y exportar un `bookmarks.html`.
- La IA con Ollama: «Probar conexión» y un resumen.
- Actualizar desde la versión anterior con el updater: la app nueva abre sin pasar otra vez por «Abrir igualmente» y `xattr -l /Applications/marks.app` no muestra `com.apple.quarantine`.

## 3. Build

```sh
scripts/build-app.sh
```

Deja `target/release/bundle/macos/marks.app`, con la extensión de Safari dentro. El detalle está en [Compilar y verificar](building.md#app-con-la-extensión-de-safari).

Con `--target aarch64-apple-darwin` o `--target x86_64-apple-darwin`, la app sale en `target/<triple>/release/bundle/macos/`. La de Intel se compila, pero no se prueba en un Intel.

El appex de Safari lo construye antes `scripts/build-appex.sh`: universal (`arm64` y `x86_64`, unidos con `lipo`), con la versión de `apps/extension/manifest.json` y firmado con hardened runtime y sus entitlements, sin `--timestamp`. `tauri.release.conf.json` lo mete en el bundle con `bundle.macOS.files`, así que el `.app.tar.gz` del updater lo lleva dentro. Después, Tauri firma la app con la misma identidad.

Comprobado con Tauri 2.12 y el certificado autofirmado: Tauri **no** vuelve a firmar el appex (el CDHash de dentro de `marks.app` es el de `target/appex`, y los entitlements siguen) y tampoco añade `--timestamp`. La app con hardened runtime extrae (ventanas `extractor` y `parser`), usa Ollama, guarda y lee la clave del llavero tras reiniciar y contesta en la API local, sin entitlements de excepción.

`scripts/verify-bundle.sh <marks.app>` comprueba el resultado: el appex dentro de `Contents/PlugIns/`, con sandbox, universal, y la app y el appex con hardened runtime y una firma válida (`codesign --verify --strict --deep`).

Con `--dmg`, `scripts/build-app.sh` construye también el DMG: `target/<triple>/release/bundle/dmg/marks_<versión>_aarch64.dmg` (o `_x64.dmg`). Con `CI=true`, Tauri no automatiza Finder por AppleScript, así que el DMG se construye sin sesión gráfica, como en CI.

## 4. Firma, sin notarización

La release firma la app, el appex y el DMG con `marks Code Signing`, un certificado de firma de código autofirmado (`APPLE_SIGNING_IDENTITY="marks Code Signing" scripts/build-app.sh --target … --dmg`). Lleva hardened runtime en la app y en el appex, y entitlements solo en el appex (sandbox y red). Sin `APPLE_API_*` ni `APPLE_ID` en el entorno, Tauri no notariza.

Sin Developer ID ni notarización, Gatekeeper rechaza la app, y es lo esperado:

```sh
spctl -a -vv target/aarch64-apple-darwin/release/bundle/macos/marks.app
# …: rejected
# origin=marks Code Signing
```

El usuario la abre la primera vez con «Abrir igualmente» ([Instalación](../installation.md)).

Con el certificado autofirmado, macOS no reconoce una build nueva como la misma app para el llavero: tras actualizar, la app pide una vez la contraseña del llavero para leer la clave del proveedor de IA (comprobado con dos builds firmadas con `marks Code Signing`).

## 5. Actualización automática

La app se actualiza con `tauri-plugin-updater` desde GitHub Releases:

- **Endpoint:** `https://github.com/landygg/marks/releases/latest/download/latest-{{target}}-{{arch}}.json`. `releases/latest` ignora borradores y prereleases.
- **Canal:** uno, el estable, y solo Apple Silicon: cada release publica `latest-darwin-aarch64.json`. La de Intel no se actualiza sola.
- **Formato:** el manifiesto estático de Tauri v2, que escribe `scripts/updater-manifest.mjs` con las notas de la sección de `CHANGELOG.md`, y el paquete `marks_<versión>_aarch64.app.tar.gz` con su firma `.sig`.
- **Claves:** la pública va en `tauri.conf.json` (`plugins.updater.pubkey`). La privada y su contraseña las guarda el dueño en su gestor de contraseñas y en el Environment `release` de GitHub; nunca entran en el repo.
- `MARKS_UPDATE_ENDPOINT` sustituye el endpoint, solo para probar; la firma se sigue exigiendo.
- **macOS mínimo:** el manifiesto declara `minimum_system_version`, que sale de `bundle.macOS.minimumSystemVersion` en `tauri.conf.json`. Desde la 1.2.0, la app retiene una actualización que necesita un macOS más nuevo que el que corre y lo dice; un valor ilegible también la retiene. El plugin ignora el campo, así que las versiones anteriores a la 1.2.0 instalan cualquiera.
- **Subir el macOS mínimo:** solo cuando la versión publicada anterior ya tenga esa comprobación (1.2.0 o posterior), y tras probar el aviso con `MARKS_UPDATE_ENDPOINT` y un manifiesto con `"minimum_system_version": "99.0"`. Si no, quien esté en el macOS anterior instala una versión que ya no abre, como le pasó a macOS 14 con la 1.2.0.
- **La release v1.1.0 no se borra nunca** y conserva su DMG: es el camino de vuelta para macOS 14.

## 6. Extensión de Safari

En la v1.0 va dentro de `marks.app`.

La app declara `API_LEVEL` y lo responde en `ping` (`api`); la extensión declara `MIN_API`. Si la app responde un `api` menor, o no lo manda, la extensión muestra `appTooOld`; si una ruta da `404`, `extensionTooOld`. En cada release: si la API ganó algo que la extensión use, subir `API_LEVEL` y, en la extensión que lo use, `MIN_API`. Mira [API local](local-api.md).

Más adelante, si se paga el Apple Developer Program, que lo exige, una app propia en el Mac App Store que contenga la extensión.

## 7. Extensión de Chromium

Cada release adjunta `marks-chromium-<versión>.zip`, con el contenido de `apps/extension/dist/chromium` y la versión de `apps/extension/manifest.json`. Se carga sin empaquetar. `release.yml` lo crea con:

```sh
cd apps/extension/dist/chromium && zip -qr "$OUT/marks-chromium-$EXT_VERSION.zip" .
```

## 8. Migraciones y datos

Desde la v1.1, cada release que traiga una migración prueba, con una copia de una base de la release anterior y con datos, que se hace la copia previa en `backups/` y que `meta.min_reader` queda bien; y que la release anterior abre o rechaza la base migrada según su `Compat`. Mira [Migraciones](migrations.md).

## 9. Notas de la versión

`CHANGELOG.md`, en inglés y con el formato [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/). Lo nuevo se apunta en `## [Unreleased]`; al preparar una versión, esa sección pasa a `## [<versión>] - <fecha>`. Esa sección es a la vez las notas de la release en GitHub y las `notes` de `latest-darwin-aarch64.json` (`scripts/updater-manifest.mjs`), y sin ella la release falla.

## 10. Publicar

`.github/workflows/release.yml` hace la release al subir una etiqueta `v*`:

1. Con la versión ya subida en los ficheros y `## [<versión>]` en `CHANGELOG.md`, crea y sube la etiqueta: `git tag v1.0.0 && git push origin v1.0.0`.
2. Aprueba el despliegue: el Environment `release` exige la revisión del dueño, así que el job `build` espera, sin los secrets, hasta que abras la ejecución y elijas **Review deployments → Approve and deploy**.
3. El job `build` comprueba la etiqueta con `scripts/check-version.sh`, importa el certificado en un llavero temporal y, para `aarch64` y `x86_64`, ejecuta `scripts/build-app.sh --target … --dmg` con los ficheros del updater y `scripts/verify-bundle.sh` sobre la app y sobre la del `.app.tar.gz`. En `aarch64` prepara además `latest-darwin-aarch64.json` y el zip de Chromium. Registra la procedencia de cada fichero y, al final, borra el llavero temporal. No restaura cachés, así que compila desde cero.
4. El job `publish` crea una release **en borrador** con todos los ficheros. Si la etiqueta lleva `-` (`v1.0.0-rc.1`), la marca como prerelease.
5. Revisa el borrador: los ficheros, las notas, y una instalación desde el DMG.
6. Publícalo a mano. Desde ese momento, `releases/latest` sirve el `latest-darwin-aarch64.json` nuevo y las instalaciones anteriores se actualizan.

Las releases publicadas son inmutables, y un ruleset impide mover o borrar las etiquetas `v*`: una release que sale mal se arregla con una versión nueva, no editándola. Antes de publicar, el borrador y su etiqueta aún se pueden quitar desactivando un momento el ruleset `tags v*` (Settings → Rules → Rulesets). Cualquiera puede comprobar de dónde sale un fichero con `gh attestation verify <fichero> -R landygg/marks`.

Los secrets están en el Environment `release` de GitHub, no en los del repo, y ese Environment solo admite etiquetas `v*`:

| Secret | Qué es |
|---|---|
| `APPLE_CERTIFICATE` | el `.p12` del certificado de firma, en base64 (`base64 -i cert.p12 \| pbcopy`) |
| `APPLE_CERTIFICATE_PASSWORD` | la contraseña del `.p12` |
| `APPLE_SIGNING_IDENTITY` | `marks Code Signing` |
| `TAURI_SIGNING_PRIVATE_KEY` | el contenido de la clave privada del updater |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | su contraseña |

La contraseña del llavero temporal no es un secret: el job la genera con `openssl rand -base64 24`.

## Más adelante

Windows y Linux no forman parte de la v1.0. Cuando lleguen, esta página tendrá sus pasos.
