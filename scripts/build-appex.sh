#!/usr/bin/env bash
# Builds the Safari extension appex, universal (arm64 + x86_64) and signed, into target/appex.
# tauri.release.conf.json copies it into marks.app (bundle.macOS.files), so the updater archive keeps it.
# Signs with $APPLE_SIGNING_IDENTITY, else $CODESIGN_IDENTITY, else ad-hoc.
set -euo pipefail
cd "$(dirname "$0")/.."

APPEX="target/appex/marks Safari Extension.appex"
SAFARI_SRC="apps/extension/safari"
EXECUTABLE="MarksSafariExtension"
PLIST_BUDDY=/usr/libexec/PlistBuddy
ARCHS=(arm64 x86_64)

# The appex carries the extension's own version, not the app's.
version=$(node -p "require('./apps/extension/manifest.json').version")
[[ -n "$version" ]] || { echo "version not found in apps/extension/manifest.json" >&2; exit 1; }
# The appex must run wherever the app does: same minimum macOS as the bundle (tauri.conf.json).
min_macos=$(sed -n 's/.*"minimumSystemVersion": *"\([0-9.]*\)".*/\1/p' apps/desktop/src-tauri/tauri.conf.json)
[[ -n "$min_macos" ]] || { echo "minimumSystemVersion not found in tauri.conf.json" >&2; exit 1; }

identity="${APPLE_SIGNING_IDENTITY:-${CODESIGN_IDENTITY:--}}"
[[ "$identity" != "-" ]] || echo "warning: no signing identity; the appex is signed ad-hoc." >&2

# Start from scratch so stale files cannot survive.
rm -rf "$APPEX"
mkdir -p "$APPEX/Contents/MacOS" "$APPEX/Contents/Resources"
cp "$SAFARI_SRC/Info.plist" "$APPEX/Contents/Info.plist"
"$PLIST_BUDDY" -c "Add :CFBundleShortVersionString string $version" "$APPEX/Contents/Info.plist"
"$PLIST_BUDDY" -c "Add :CFBundleVersion string $version" "$APPEX/Contents/Info.plist"
"$PLIST_BUDDY" -c "Add :LSMinimumSystemVersion string $min_macos" "$APPEX/Contents/Info.plist"

pnpm --dir apps/extension build
cp -R apps/extension/dist/safari/ "$APPEX/Contents/Resources/"
cp -R "$SAFARI_SRC"/*.lproj "$APPEX/Contents/Resources/"

slices=()
for arch in "${ARCHS[@]}"; do
  slice="target/appex/$EXECUTABLE-$arch"
  xcrun swiftc -O -parse-as-library -application-extension -module-name "$EXECUTABLE" \
    -target "$arch-apple-macos$min_macos" \
    -Xlinker -e -Xlinker _NSExtensionMain -framework Foundation -framework SafariServices \
    -o "$slice" "$SAFARI_SRC/SafariWebExtensionHandler.swift"
  slices+=("$slice")
done
lipo -create "${slices[@]}" -output "$APPEX/Contents/MacOS/$EXECUTABLE"
rm -f "${slices[@]}"

# No --timestamp: only notarization needs it, and marks is not notarized.
codesign --force --sign "$identity" --options runtime \
  --entitlements "$SAFARI_SRC/extension.entitlements" "$APPEX"
echo "$PWD/$APPEX"
