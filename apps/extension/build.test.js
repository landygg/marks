import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readdirSync, readFileSync, rmSync, statSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { manifestFor } from "./build.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const dist = join(root, "dist");
const chromium = join(dist, "chromium");
const safari = join(dist, "safari");
const MAX_PAGE_EXTRACT_BYTES = 900 * 1024;
const base = JSON.parse(readFileSync(join(root, "manifest.json"), "utf8"));
const readJson = (path) => JSON.parse(readFileSync(path, "utf8"));

// Through a symlink to this folder, as from a clone under /tmp on macOS: the build must still run.
const link = join(mkdtempSync(join(tmpdir(), "marks-ext-")), "extension");
symlinkSync(root, link);
rmSync(dist, { recursive: true, force: true });
execFileSync("node", [join(link, "build.mjs")], { cwd: link });

// Every file under `dir`, as paths relative to it.
function files(dir) {
  return readdirSync(dir, { recursive: true })
    .map((f) => join(dir, f))
    .filter((f) => statSync(f).isFile())
    .map((f) => relative(dir, f))
    .sort();
}

test("dist holds one folder per browser", () => {
  assert.deepEqual(readdirSync(dist).sort(), ["chromium", "safari"]);
});

test("dist/chromium holds the static files and the four bundles", () => {
  const expected = [
    "_locales", "background.js", "extension.css", "icons", "manifest.json",
    "options.html", "options.js", "page-extract.js", "popup.html", "popup.js",
  ];
  assert.deepEqual(readdirSync(chromium).sort(), expected);
});

test("Chromium's manifest asks for no native messaging and needs Chrome 123", () => {
  const m = readJson(join(chromium, "manifest.json"));
  assert.ok(!m.permissions.includes("nativeMessaging"));
  assert.equal(m.minimum_chrome_version, "123");
});

test("Safari's manifest keeps native messaging and names no Chrome version", () => {
  const m = readJson(join(safari, "manifest.json"));
  assert.ok(m.permissions.includes("nativeMessaging"));
  assert.ok(!("minimum_chrome_version" in m));
});

test("both manifests match manifest.json in everything else", () => {
  for (const dir of [chromium, safari]) {
    const { permissions, minimum_chrome_version: _, ...rest } = readJson(join(dir, "manifest.json"));
    const { permissions: basePermissions, ...baseRest } = base;
    assert.deepEqual(rest, baseRest, dir);
    assert.deepEqual(
      permissions,
      basePermissions.filter((p) => dir === safari || p !== "nativeMessaging"),
      dir,
    );
  }
});

test("apart from the manifest, both folders hold the same files with the same bytes", () => {
  const list = files(chromium);
  assert.deepEqual(files(safari), list);
  for (const f of list.filter((f) => f !== "manifest.json"))
    assert.ok(readFileSync(join(chromium, f)).equals(readFileSync(join(safari, f))), f);
});

test("manifestFor leaves the base manifest untouched", () => {
  const before = JSON.stringify(base);
  manifestFor(base, "chromium");
  manifestFor(base, "safari");
  assert.equal(JSON.stringify(base), before);
});

test("page-extract.js has no dynamic code and stays small", () => {
  const file = join(chromium, "page-extract.js");
  const src = readFileSync(file, "utf8");
  for (const banned of ["eval(", "new Function", "import("]) assert.ok(!src.includes(banned), banned);
  assert.ok(statSync(file).size < MAX_PAGE_EXTRACT_BYTES);
});
