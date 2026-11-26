import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path) => readFileSync(new URL(path, import.meta.url), "utf8");
const options = read("./options.html");
const tag = (html, id) => html.match(new RegExp(`<[a-z]+\\b[^>]*\\bid="${id}"[^>]*>`))?.[0] ?? "";

test("the token field and its two buttons sit in a form: Enter saves", () => {
	const form = options.match(/<form\b[\s\S]*?<\/form>/)?.[0] ?? "";
	for (const id of ["token", "save", "test"]) assert.match(form, new RegExp(`id="${id}"`), id);
	assert.match(tag(options, "save"), /type="submit"/);
	assert.match(tag(options, "test"), /type="button"/);
});

test("the status region is always in the page, so every result is announced", () => {
	const status = tag(options, "status");
	assert.match(status, /role="status"/);
	assert.doesNotMatch(status, /\bhidden\b/);
});

test("both pages have a translated title", () => {
	for (const locale of ["en", "es"]) {
		const messages = JSON.parse(read(`./_locales/${locale}/messages.json`));
		for (const page of ["options.html", "popup.html"]) {
			const key = read(`./${page}`).match(/<title data-i18n="([^"]+)"><\/title>/)?.[1];
			assert.ok(key, `${page} has no <title data-i18n>`);
			assert.ok(messages[key], `${locale}: ${key}`);
		}
	}
});

test("options.js saves on the form's submit, not on a click on Save", () => {
	const script = read("./options.js");
	assert.match(script, /addEventListener\("submit"/);
	assert.doesNotMatch(script, /getElementById\("save"\)\.addEventListener\("click"/);
});
