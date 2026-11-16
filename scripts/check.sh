#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

scripts/check-version.sh
# Install first: a clean clone (CI) has no node_modules.
pnpm install --frozen-lockfile

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir apps/extension build
node --test "apps/extension/*.test.js"
node --test "scripts/*.test.mjs"
pnpm --dir packages/extract check
pnpm --dir packages/extract test
pnpm --dir apps/desktop format:check
pnpm --dir apps/desktop lint
pnpm --dir apps/desktop check
pnpm --dir apps/desktop test

if [[ "$(uname)" == "Darwin" ]]; then
  xcrun swiftc -typecheck -parse-as-library -application-extension apps/extension/safari/SafariWebExtensionHandler.swift
fi
