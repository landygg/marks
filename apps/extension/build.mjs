// Bundles the extension into dist/: ESM entry points, the injected page-extract IIFE and the static files.
import { build, context } from "esbuild";
import { cp, rm } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const dist = join(root, "dist");
const STATIC = ["manifest.json", "popup.html", "options.html", "extension.css", "_locales", "icons"];

const esm = {
  entryPoints: ["popup.js", "options.js", "background.js"].map((f) => join(root, f)),
  outdir: dist,
  bundle: true,
  format: "esm",
  target: "es2022",
};
// Minified because unminified esbuild leaves 1.1 MB (Defuddle and its deps).
const page = {
  entryPoints: [join(root, "page-extract.ts")],
  outfile: join(dist, "page-extract.js"),
  bundle: true,
  format: "iife",
  target: "es2022",
  minifyWhitespace: true,
};

async function copyStatic() {
  for (const f of STATIC) await cp(join(root, f), join(dist, f), { recursive: true });
}

await rm(dist, { recursive: true, force: true });
await copyStatic();
if (process.argv.includes("--watch")) {
  for (const o of [esm, page]) await (await context(o)).watch();
} else {
  await Promise.all([build(esm), build(page)]);
}
