import test from 'node:test';
import assert from 'node:assert/strict';
import { changelogSection, manifest } from './updater-manifest.mjs';

const NOW = new Date('2026-12-01T10:00:00Z');
const URL = 'https://github.com/landygg/marks/releases/download/v1.0.0/marks_1.0.0_aarch64.app.tar.gz';

const CHANGELOG = `# Changelog

## [Unreleased]

## [1.1.0] - 2027-01-10

### Added

- Something newer.

## [1.0.0] - 2026-12-01

### Added

- First public release.

[1.1.0]: https://github.com/landygg/marks/releases/tag/v1.1.0
[1.0.0]: https://github.com/landygg/marks/releases/tag/v1.0.0
`;

test('manifest has the tauri v2 static shape', () => {
  const m = manifest({ version: '1.0.0', signature: 'sig', url: URL, notes: 'n', now: NOW });
  assert.deepEqual(Object.keys(m), ['version', 'notes', 'pub_date', 'platforms']);
  assert.deepEqual(m.platforms, { 'darwin-aarch64': { signature: 'sig', url: URL } });
  assert.equal(m.version, '1.0.0');
  assert.equal(m.notes, 'n');
});

test('signature is the sig file content, trimmed', () => {
  const m = manifest({ version: '1.0.0', signature: '  dW50cnVzdGVk\n', url: URL, notes: 'n', now: NOW });
  assert.equal(m.platforms['darwin-aarch64'].signature, 'dW50cnVzdGVk');
});

test('pub_date is RFC 3339', () => {
  const m = manifest({ version: '1.0.0', signature: 's', url: URL, notes: 'n', now: NOW });
  assert.equal(m.pub_date, '2026-12-01T10:00:00.000Z');
});

test('changelogSection takes only that version', () => {
  assert.equal(changelogSection(CHANGELOG, '1.0.0'), '### Added\n\n- First public release.');
  assert.equal(changelogSection(CHANGELOG, '1.1.0'), '### Added\n\n- Something newer.');
});

test('missing changelog section throws', () => {
  assert.throws(() => changelogSection(CHANGELOG, '2.0.0'), /2\.0\.0/);
  assert.throws(() => changelogSection('## [3.0.0] - 2027-05-01\n\n', '3.0.0'), /empty/);
});
