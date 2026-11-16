#!/usr/bin/env bash
# Builds marks.app with the Safari extension appex inside, both signed with the same identity.
# Usage: scripts/build-app.sh [--target <triple>] [--dmg] [extra tauri build args, e.g. --config <json>]
# Identity: $APPLE_SIGNING_IDENTITY, else $CODESIGN_IDENTITY, else the first "Apple Development", else ad-hoc.
set -euo pipefail
cd "$(dirname "$0")/.."

RELEASE_CONF="src-tauri/tauri.release.conf.json"

target=""
bundles="app"
passthrough=()
# --target and --dmg are ours wherever they appear; everything else goes to tauri build in order.
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target) target="${2:?--target needs a triple}"; shift 2 ;;
    # Tauri signs the DMG too, with the same identity; it is not notarized.
    --dmg) bundles="app,dmg"; shift ;;
    *) passthrough+=("$1"); shift ;;
  esac
done

if [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  identity="$APPLE_SIGNING_IDENTITY"; kind="APPLE_SIGNING_IDENTITY"
elif [[ -n "${CODESIGN_IDENTITY:-}" ]]; then
  identity="$CODESIGN_IDENTITY"; kind="CODESIGN_IDENTITY"
elif hash=$(security find-identity -v -p codesigning | awk '/"Apple Development/ {print $2; exit}') && [[ -n "$hash" ]]; then
  identity="$hash"; kind="Apple Development"
else
  identity="-"; kind="ad-hoc"
  echo "warning: no signing identity found; signed ad-hoc. Safari will ask for \"Allow unsigned extensions\"." >&2
fi
# Tauri signs the app (hardened runtime included) with this identity; "-" is ad-hoc.
export APPLE_SIGNING_IDENTITY="$identity"

scripts/build-appex.sh

args=(build --bundles "$bundles" --config "$RELEASE_CONF")
[[ -z "$target" ]] || args+=(--target "$target")
pnpm --dir apps/desktop tauri "${args[@]}" ${passthrough[@]+"${passthrough[@]}"}

app="target/${target:+$target/}release/bundle/macos/marks.app"
codesign --verify --strict --deep "$app"
echo "signed with: $kind"
echo "$PWD/$app"
