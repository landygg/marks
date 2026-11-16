#!/usr/bin/env bash
# Checks a built marks.app: the Safari appex is inside, sandboxed, universal, and both carry
# the hardened runtime and a valid signature. Exits 1 on the first problem.
set -euo pipefail

app="${1:?usage: scripts/verify-bundle.sh <path to marks.app>}"
appex="$app/Contents/PlugIns/marks Safari Extension.appex"
appex_bin="$appex/Contents/MacOS/MarksSafariExtension"
# Any flags group naming runtime: flags=0x10000(runtime), or flags=0x10002(adhoc,runtime) when ad-hoc.
RUNTIME_FLAGS='flags=0x[0-9a-f]+\([^)]*runtime'
SANDBOX="com.apple.security.app-sandbox"
ARCHS="x86_64 arm64"

fail() { echo "verify-bundle: $*" >&2; exit 1; }

[[ -d "$appex" ]] || fail "appex missing: $appex"
# Capture first: grep -q exits early, and the SIGPIPE to codesign fails the pipe under pipefail.
entitlements=$(codesign -d --entitlements - "$appex" 2>/dev/null)
[[ "$entitlements" == *"$SANDBOX"* ]] || fail "appex lacks $SANDBOX"
for target in "$app" "$appex"; do
  details=$(codesign -dv "$target" 2>&1)
  [[ "$details" =~ $RUNTIME_FLAGS ]] || fail "no hardened runtime: $target"
done
archs=$(lipo -archs "$appex_bin")
[[ "$archs" == "$ARCHS" ]] || fail "appex archs are '$archs', want '$ARCHS'"
codesign --verify --strict --deep "$app" || fail "codesign --verify --strict --deep failed"
echo "verify-bundle: ok ($app)"
