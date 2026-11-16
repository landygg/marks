// Runs build-app.sh in a scratch copy with pnpm, codesign and build-appex.sh stubbed, and checks the
// tauri build arguments and the bundle path it verifies.
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, copyFileSync, mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT = fileURLToPath(new URL('./build-app.sh', import.meta.url));
const EXEC = 0o755;

function run(...args) {
  const root = mkdtempSync(join(tmpdir(), 'build-app-'));
  const scripts = join(root, 'scripts');
  const bin = join(root, 'bin');
  mkdirSync(scripts);
  mkdirSync(bin);
  copyFileSync(SCRIPT, join(scripts, 'build-app.sh'));
  const stubs = {
    [join(scripts, 'build-appex.sh')]: '#!/bin/sh\nexit 0\n',
    [join(bin, 'pnpm')]: '#!/bin/sh\necho "PNPM $*"\n',
    [join(bin, 'codesign')]: '#!/bin/sh\necho "CODESIGN $*"\n',
  };
  for (const [path, body] of Object.entries(stubs)) writeFileSync(path, body);
  for (const path of [join(scripts, 'build-app.sh'), ...Object.keys(stubs)]) chmodSync(path, EXEC);
  const out = spawnSync(join(scripts, 'build-app.sh'), args, {
    env: { ...process.env, PATH: `${bin}:${process.env.PATH}`, APPLE_SIGNING_IDENTITY: 'test' },
    encoding: 'utf8',
  });
  const line = (prefix) => out.stdout.split('\n').find((l) => l.startsWith(prefix));
  return { status: out.status, pnpm: line('PNPM '), codesign: line('CODESIGN ') };
}

const CONFIG = '{"bundle":{"createUpdaterArtifacts":true}}';

test('flags before the passthrough', () => {
  const r = run('--target', 'aarch64-apple-darwin', '--dmg', '--config', CONFIG);
  assert.equal(r.status, 0);
  assert.equal(
    r.pnpm,
    `PNPM --dir apps/desktop tauri build --bundles app,dmg --config src-tauri/tauri.release.conf.json --target aarch64-apple-darwin --config ${CONFIG}`,
  );
  assert.match(r.codesign, / target\/aarch64-apple-darwin\/release\/bundle\/macos\/marks\.app$/);
});

test('flags after the passthrough are still ours', () => {
  const r = run('--config', CONFIG, '--dmg', '--target', 'x86_64-apple-darwin');
  assert.equal(r.status, 0);
  assert.equal(
    r.pnpm,
    `PNPM --dir apps/desktop tauri build --bundles app,dmg --config src-tauri/tauri.release.conf.json --target x86_64-apple-darwin --config ${CONFIG}`,
  );
  assert.match(r.codesign, / target\/x86_64-apple-darwin\/release\/bundle\/macos\/marks\.app$/);
});

test('no arguments builds the app for this machine', () => {
  const r = run();
  assert.equal(r.status, 0);
  assert.equal(r.pnpm, 'PNPM --dir apps/desktop tauri build --bundles app --config src-tauri/tauri.release.conf.json');
  assert.match(r.codesign, / target\/release\/bundle\/macos\/marks\.app$/);
});

test('--target without a triple fails before building', () => {
  const r = run('--target');
  assert.notEqual(r.status, 0);
  assert.equal(r.pnpm, undefined);
});
