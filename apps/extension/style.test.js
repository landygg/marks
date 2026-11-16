import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path) => readFileSync(new URL(path, import.meta.url), "utf8");
const TOKEN = /^\s*(--[\w-]+):\s*([^;]+);/gm;
const tokens = (css) => new Map([...css.matchAll(TOKEN)].map(([, name, value]) => [name, value.trim()]));

test("extension.css tokens are the app's (app.css is checked against DESIGN.md)", () => {
	const app = tokens(read("../desktop/src/app.css"));
	const extension = tokens(read("./extension.css"));
	assert.ok(extension.size > 0);
	for (const [name, value] of extension) assert.equal(value, app.get(name), name);
});

test("every data-i18n key and the lang message exist in the default locale", () => {
	const messages = JSON.parse(read("./_locales/es/messages.json"));
	assert.equal(messages.lang.message, "es");
	for (const page of ["popup.html", "options.html"]) {
		for (const [, key] of read(`./${page}`).matchAll(/data-i18n="([^"]+)"/g))
			assert.ok(messages[key], `${page}: ${key}`);
	}
});
