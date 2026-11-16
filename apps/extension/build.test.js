import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const dist = join(root, "dist");
const MAX_PAGE_EXTRACT_BYTES = 900 * 1024;

execFileSync("node", [join(root, "build.mjs")], { cwd: root });

test("dist holds the static files and the four bundles", () => {
  const expected = [
    "_locales", "background.js", "extension.css", "icons", "manifest.json",
    "options.html", "options.js", "page-extract.js", "popup.html", "popup.js",
  ];
  assert.deepEqual(readdirSync(dist).sort(), expected);
});

test("page-extract.js has no dynamic code and stays small", () => {
  const file = join(dist, "page-extract.js");
  const src = readFileSync(file, "utf8");
  for (const banned of ["eval(", "new Function", "import("]) assert.ok(!src.includes(banned), banned);
  assert.ok(statSync(file).size < MAX_PAGE_EXTRACT_BYTES);
});
