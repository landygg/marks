// Bundles the extension into dist/chromium and dist/safari: ESM entry points, the injected
// page-extract IIFE, the static files and a manifest written for each browser.
import { build, context } from "esbuild";
import { realpathSync } from "node:fs";
import { cp, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const dist = join(root, "dist");
const CHROMIUM_DIR = join(dist, "chromium");
const SAFARI_DIR = join(dist, "safari");
// The engine floor of the extension (AGENTS.md): `light-dark()` in extension.css needs it.
const CHROME_MIN = "123";
// Only the Safari appex relays to the app natively; Chromium talks to it over fetch.
const NATIVE_MESSAGING = "nativeMessaging";
const STATIC = ["popup.html", "options.html", "extension.css", "_locales", "icons"];

const esm = {
  entryPoints: ["popup.js", "options.js", "background.js"].map((f) => join(root, f)),
  outdir: CHROMIUM_DIR,
  bundle: true,
  format: "esm",
  target: "es2022",
};
// Minified because unminified esbuild leaves 1.1 MB (Defuddle and its deps).
const page = {
  entryPoints: [join(root, "page-extract.ts")],
  outfile: join(CHROMIUM_DIR, "page-extract.js"),
  bundle: true,
  format: "iife",
  target: "es2022",
  minifyWhitespace: true,
};

/** `manifest.json` as each browser gets it; `base` is not changed. */
export function manifestFor(base, browser) {
  if (browser === "safari") return structuredClone(base);
  return {
    ...structuredClone(base),
    permissions: base.permissions.filter((p) => p !== NATIVE_MESSAGING),
    minimum_chrome_version: CHROME_MIN,
  };
}

async function writeManifest(base, browser, dir) {
  await writeFile(join(dir, "manifest.json"), `${JSON.stringify(manifestFor(base, browser), null, 2)}\n`);
}

async function main() {
  const base = JSON.parse(await readFile(join(root, "manifest.json"), "utf8"));
  await rm(dist, { recursive: true, force: true });
  for (const f of STATIC) await cp(join(root, f), join(CHROMIUM_DIR, f), { recursive: true });
  await writeManifest(base, "chromium", CHROMIUM_DIR);
  if (process.argv.includes("--watch")) {
    // Only dist/chromium: it is the one loaded unpacked; Safari takes its copy from a full build.
    for (const o of [esm, page]) await (await context(o)).watch();
    return;
  }
  await Promise.all([build(esm), build(page)]);
  await cp(CHROMIUM_DIR, SAFARI_DIR, { recursive: true });
  await writeManifest(base, "safari", SAFARI_DIR);
}

// Real paths on both sides: run through a symlink (/tmp on macOS), argv[1] keeps the link.
if (realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
