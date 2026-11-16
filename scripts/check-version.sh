#!/usr/bin/env bash
# Checks that the package.json files repeat the workspace version and, given a tag, that it names that version.
# Usage: scripts/check-version.sh [TAG]
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(sed -n '/^\[workspace.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p;}' Cargo.toml)
[[ -n "$version" ]] || { echo "workspace version not found in Cargo.toml" >&2; exit 1; }

status=0
for manifest in apps/desktop/package.json packages/extract/package.json; do
  found=$(node -p "require('./$manifest').version")
  if [[ "$found" != "$version" ]]; then
    echo "$manifest has version $found, but Cargo.toml has $version" >&2
    status=1
  fi
done

if [[ $# -gt 0 && "$1" != "v$version" ]]; then
  echo "tag $1 does not match version $version (expected v$version)" >&2
  status=1
fi
# A release must not ship the placeholder updater key: no installed copy could ever verify an update.
if [[ $# -gt 0 ]]; then
  pubkey=$(node -p "require('./apps/desktop/src-tauri/tauri.conf.json').plugins.updater.pubkey")
  if [[ "$pubkey" == PROVISIONAL* ]]; then
    echo "tauri.conf.json still has the provisional updater pubkey" >&2
    status=1
  fi
fi
exit $status
