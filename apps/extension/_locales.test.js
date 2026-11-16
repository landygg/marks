import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const load = (code) =>
  JSON.parse(readFileSync(new URL(`./_locales/${code}/messages.json`, import.meta.url), "utf8"));

test("es and en have the same keys, each with a message and the same placeholders", () => {
  const es = load("es");
  const en = load("en");
  assert.deepEqual(Object.keys(en).sort(), Object.keys(es).sort());
  for (const [key, entry] of Object.entries(en)) {
    assert.ok(entry.message.trim(), `${key} is empty in en`);
    assert.deepEqual(entry.placeholders, es[key].placeholders, `${key} placeholders differ`);
  }
});

test("each locale names its own language", () => {
  assert.equal(load("es").lang.message, "es");
  assert.equal(load("en").lang.message, "en");
});

test("the default locale is en and exists", () => {
  const manifest = JSON.parse(readFileSync(new URL("./manifest.json", import.meta.url), "utf8"));
  assert.equal(manifest.default_locale, "en");
  assert.ok(load(manifest.default_locale));
});
