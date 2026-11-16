// Writes latest-darwin-aarch64.json, the static manifest the Tauri updater reads from GitHub Releases.
// Usage: node scripts/updater-manifest.mjs --version 1.0.0 --sig <.sig> --url <.app.tar.gz url> \
//          --notes CHANGELOG.md --out <dir>
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

// The stable channel only publishes Apple silicon.
export const PLATFORM = 'darwin-aarch64';
const OUT_NAME = `latest-${PLATFORM}.json`;

export function manifest({ version, signature, url, notes, now }) {
  return {
    version,
    notes,
    pub_date: now.toISOString(),
    platforms: { [PLATFORM]: { signature: signature.trim(), url } },
  };
}

// The body of `## [<version>] …` in a Keep a Changelog file, up to the next `## ` heading or link list.
export function changelogSection(text, version) {
  const lines = text.split('\n');
  const start = lines.findIndex((line) => line.startsWith(`## [${version}]`));
  if (start < 0) throw new Error(`CHANGELOG has no section for ${version}`);
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((line) => line.startsWith('## ') || /^\[[^\]]+\]: /.test(line));
  const body = (end < 0 ? rest : rest.slice(0, end)).join('\n').trim();
  if (!body) throw new Error(`CHANGELOG section for ${version} is empty`);
  return body;
}

function main() {
  const { values } = parseArgs({
    options: {
      version: { type: 'string' },
      sig: { type: 'string' },
      url: { type: 'string' },
      notes: { type: 'string' },
      out: { type: 'string' },
    },
  });
  for (const key of ['version', 'sig', 'url', 'notes', 'out']) {
    if (!values[key]) throw new Error(`missing --${key}`);
  }
  const m = manifest({
    version: values.version,
    signature: readFileSync(values.sig, 'utf8'),
    url: values.url,
    notes: changelogSection(readFileSync(values.notes, 'utf8'), values.version),
    now: new Date(),
  });
  const out = join(values.out, OUT_NAME);
  writeFileSync(out, `${JSON.stringify(m, null, 2)}\n`);
  console.log(out);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
