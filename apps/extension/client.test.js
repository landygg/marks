import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import {
	API_BASE,
	fetchTransport,
	isSavable,
	isSaved,
	MIN_API,
	NATIVE_APP_ID,
	stateOf,
	nativeTransport,
	messageFor,
	outcomeKey,
	pickTransport,
	ping,
	ROUTES,
	buildCapture,
	usableResult,
	saveTab,
	shortcutKeys,
} from "./client.js";

const TOKEN = "t0k3n";
const CAPTURE = {
	url: "https://a.b/",
	title: "T",
	page: { content: "# x" },
	text: "x",
};

/** Fake transport: records requests and replays the given responses in order. */
function fakeTransport(...responses) {
	const requests = [];
	const transport = async (request) => {
		requests.push(request);
		return responses[Math.min(requests.length, responses.length) - 1];
	};
	return { transport, requests };
}
const throwing = async () => {
	throw new Error("unreachable");
};

test("isSavable accepts only http and https", () => {
	for (const url of ["https://a.b", "http://a.b"])
		assert.equal(isSavable(url), true);
	for (const url of [
		"chrome://extensions",
		"about:blank",
		"file:///x",
		"javascript:1",
		"safari-web-extension://x/popup.html",
		"",
		undefined,
	]) {
		assert.equal(isSavable(url), false, String(url));
	}
});

test("isSaved is true only for keys that mean the page is in marks", () => {
	for (const key of ["saved", "savedIn", "alreadySaved", "alreadySavedIn"])
		assert.equal(isSaved(key), true, key);
	for (const key of ["invalidToken", "serverError", "appClosed", undefined])
		assert.equal(isSaved(key), false, String(key));
});

test("stateOf colors the status dot: pending while saving, ok on success, error otherwise", () => {
	assert.equal(stateOf("saving"), "pending");
	for (const key of ["saved", "alreadySavedIn", "pingOk", "pingOkIn", "tokenSaved"])
		assert.equal(stateOf(key), "ok", key);
	for (const key of ["noToken", "notSavable", "invalidToken", "appClosed", "serverError", "tokenSaveFailed"])
		assert.equal(stateOf(key), "error", key);
});

test("shortcutKeys splits the keys, in macOS symbols on mac and in names elsewhere", () => {
	assert.deepEqual(shortcutKeys("Alt+Shift+M", "mac"), ["⌥", "⇧", "M"]); // Safari
	assert.deepEqual(shortcutKeys("⌥⇧M", "mac"), ["⌥", "⇧", "M"]); // Chromium on macOS
	assert.deepEqual(shortcutKeys("⌥⇧Space", "mac"), ["⌥", "⇧", "Space"]);
	assert.deepEqual(shortcutKeys("Shift+Ctrl+MacCtrl+K", "mac"), ["⌃", "⇧", "⌘", "K"]); // Apple's order
	assert.deepEqual(shortcutKeys("Alt+Shift+M", "win"), ["Alt", "Shift", "M"]);
	assert.deepEqual(shortcutKeys("Alt+Shift+M", "linux"), ["Alt", "Shift", "M"]);
	assert.deepEqual(shortcutKeys("Alt+Shift+M", undefined), ["Alt", "Shift", "M"]);
});

test("outcomeKey maps responses", () => {
	assert.equal(outcomeKey({ status: 200, body: { created: true } }), "saved");
	assert.equal(
		outcomeKey({ status: 200, body: { created: false } }),
		"alreadySaved",
	);
});

test("outcomeKey maps 401 to invalidToken", () => {
	assert.equal(outcomeKey({ status: 401, body: {} }), "invalidToken");
	for (const status of [400, 403, 500])
		assert.equal(outcomeKey({ status, body: {} }), "serverError");
});

test("saveTab posts the capture with the token", async () => {
	const { transport, requests } = fakeTransport({
		status: 200,
		body: { created: true },
	});
	const result = await saveTab(transport, TOKEN, CAPTURE);
	assert.deepEqual(requests, [
		{ method: "POST", path: ROUTES.bookmarks, token: TOKEN, body: CAPTURE },
	]);
	assert.equal(result.key, "saved");
});

test("saveTab names the profile when the app reports it", async () => {
	const run = async (body) =>
		saveTab(fakeTransport({ status: 200, body }).transport, TOKEN, CAPTURE);
	assert.deepEqual(await run({ created: true, profile: "Trabajo" }), {
		key: "savedIn",
		profile: "Trabajo",
	});
	assert.deepEqual(await run({ created: false, profile: "Trabajo" }), {
		key: "alreadySavedIn",
		profile: "Trabajo",
	});
	assert.deepEqual(await run({ created: true }), { key: "saved" });
});

test("ping names the profile when the app reports it", async () => {
	const run = async (body) =>
		ping(fakeTransport({ status: 200, body }).transport, TOKEN);
	assert.deepEqual(await run({ api: MIN_API, profile: "Personal" }), {
		key: "pingOkIn",
		profile: "Personal",
	});
	assert.deepEqual(await run({ api: MIN_API }), { key: "pingOk" });
});

test("buildCapture keeps page and text and drops empty ones", () => {
	const tab = { url: "https://a.b/", title: "T", id: 3 };
	assert.deepEqual(buildCapture(tab, { page: { content: "x" }, text: "t" }), {
		url: "https://a.b/",
		title: "T",
		page: { content: "x" },
		text: "t",
	});
	assert.deepEqual(buildCapture(tab, { text: "" }), { url: "https://a.b/", title: "T" });
	assert.deepEqual(buildCapture({ url: "https://a.b/" }, {}), { url: "https://a.b/" });
});

test("outcomeKey maps blocked", () => {
	assert.equal(outcomeKey({ status: 200, body: { blocked: true } }), "blocked");
	assert.equal(
		outcomeKey({ status: 200, body: { blocked: true, profile: "Trabajo" } }),
		"blockedIn",
	);
});

test("stateOf marks blocked as error", () => {
	for (const key of ["blocked", "blockedIn"]) assert.equal(stateOf(key), "error", key);
});

test("saveTab retries without text on 413", async () => {
	const { transport, requests } = fakeTransport(
		{ status: 413, body: {} },
		{ status: 200, body: { created: true } },
	);
	const result = await saveTab(transport, TOKEN, CAPTURE);
	assert.equal(requests.length, 2);
	assert.deepEqual(requests[1].body, {
		url: CAPTURE.url,
		title: CAPTURE.title,
		page: CAPTURE.page,
	});
	assert.equal(result.key, "saved");

	const again = fakeTransport({ status: 413, body: {} });
	assert.equal(
		(await saveTab(again.transport, TOKEN, CAPTURE)).key,
		"serverError",
	);
	assert.equal(again.requests.length, 2);
});

test("outcomeKey turns the API's error code into its message key", () => {
	assert.equal(outcomeKey({ status: 401, body: { error: "token" } }), "error_token");
	assert.equal(outcomeKey({ status: 400, body: { error: "unsupported-scheme" } }), "error_unsupported_scheme");
});

test("saveTab gives the error key and no detail", async () => {
	const { transport } = fakeTransport({ status: 400, body: { error: "invalid-url" } });
	assert.deepEqual(await saveTab(transport, TOKEN, CAPTURE), { key: "error_invalid_url" });
});

test("messageFor falls back to serverError for a code without message", () => {
	const t = (key) => ({ serverError: "fail", error_token: "tok" })[key] ?? "";
	assert.equal(messageFor("error_token", t), "tok");
	assert.equal(messageFor("error_nope", t), "fail");
});

test("saveTab reports appClosed when the transport throws", async () => {
	assert.deepEqual(await saveTab(throwing, TOKEN, CAPTURE), {
		key: "appClosed",
	});
});

test("ping reports appClosed when the transport throws", async () => {
	assert.deepEqual(await ping(throwing, TOKEN), { key: "appClosed" });
});

test("ping maps responses", async () => {
	const ok = fakeTransport({ status: 200, body: { app: "marks", api: MIN_API } });
	assert.deepEqual(await ping(ok.transport, TOKEN), { key: "pingOk" });
	assert.deepEqual(ok.requests, [
		{ method: "GET", path: ROUTES.ping, token: TOKEN },
	]);
	assert.deepEqual(
		await ping(fakeTransport({ status: 401, body: {} }).transport, TOKEN),
		{ key: "invalidToken" },
	);
	assert.deepEqual(
		await ping(fakeTransport({ status: 500, body: {} }).transport, TOKEN),
		{ key: "serverError" },
	);
});

test("fetchTransport sends the bearer token", async () => {
	const calls = [];
	const fetchImpl = async (url, init) => {
		calls.push({ url, init });
		return { status: 200, json: async () => ({ ok: true }) };
	};
	const response = await fetchTransport(
		{
			method: "POST",
			path: ROUTES.bookmarks,
			token: TOKEN,
			body: { url: "u" },
		},
		fetchImpl,
	);
	assert.deepEqual(response, { status: 200, body: { ok: true } });
	assert.equal(calls[0].url, API_BASE + ROUTES.bookmarks);
	assert.equal(calls[0].init.headers.Authorization, `Bearer ${TOKEN}`);
	assert.equal(calls[0].init.headers["Content-Type"], "application/json");
	assert.equal(calls[0].init.body, JSON.stringify({ url: "u" }));

	await fetchTransport(
		{ method: "GET", path: ROUTES.ping, token: TOKEN },
		fetchImpl,
	);
	assert.equal(calls[1].init.headers["Content-Type"], undefined);
	assert.equal(calls[1].init.body, undefined);
});

test("nativeTransport throws only when the handler cannot reach marks", async () => {
	const runtime = { sendNativeMessage: async () => ({ error: "unreachable" }) };
	await assert.rejects(
		nativeTransport(
			{ method: "GET", path: ROUTES.ping, token: TOKEN },
			runtime,
		),
	);
	const good = {
		sendNativeMessage: async (id, request) => ({
			status: 200,
			body: { id, request },
		}),
	};
	const response = await nativeTransport(
		{ method: "GET", path: ROUTES.ping, token: TOKEN },
		good,
	);
	assert.equal(response.body.id, NATIVE_APP_ID);
});

test("nativeTransport maps other handler errors to a server error", async () => {
	const runtime = { sendNativeMessage: async () => ({ error: "bad_request" }) };
	const response = await nativeTransport(
		{ method: "GET", path: ROUTES.ping, token: TOKEN },
		runtime,
	);
	assert.equal(outcomeKey(response), "serverError");
	assert.equal(
		(await saveTab((r) => nativeTransport(r, runtime), TOKEN, CAPTURE)).key,
		"serverError",
	);
});

test("pickTransport chooses by protocol", async () => {
	const request = { method: "GET", path: ROUTES.ping, token: TOKEN };
	const sent = [];
	const runtime = {
		sendNativeMessage: async (_id, r) => (
			sent.push(r), { status: 200, body: {} }
		),
	};
	await pickTransport("safari-web-extension:", runtime)(request);
	assert.deepEqual(sent, [request]);
	assert.equal(pickTransport("chrome-extension:", runtime), fetchTransport);
});

const here = (file) => new URL(file, import.meta.url);
const locale = JSON.parse(
	readFileSync(here("_locales/es/messages.json"), "utf8"),
);

test("every message key exists in the locale file", async () => {
	const keys = new Set([
		"noToken",
		"notSavable",
		"saving",
		"extName",
		"extDescription",
		"actionTitle",
		"tokenSaveFailed",
	]);
	for (const status of [200, 400, 401, 403, 413, 500]) {
		for (const created of [true, false]) {
			for (const profile of [undefined, "Trabajo"]) {
				for (const blocked of [true, false]) {
				const response = { status, body: { created, profile, blocked } };
				keys.add(outcomeKey(response));
				keys.add(
					(await saveTab(fakeTransport(response).transport, TOKEN, CAPTURE))
						.key,
				);
				keys.add((await ping(fakeTransport(response).transport, TOKEN)).key);
				}
			}
		}
	}
	keys.add((await saveTab(throwing, TOKEN, CAPTURE)).key);
	keys.add((await ping(throwing, TOKEN)).key);
	for (const page of ["popup.html", "options.html"]) {
		for (const [, key] of readFileSync(here(page), "utf8").matchAll(
			/data-i18n="([^"]+)"/g,
		))
			keys.add(key);
	}
	for (const key of keys)
		assert.ok(locale[key]?.message, `missing locale key: ${key}`);
});

test("the blocked messages say what to do", () => {
	const text = "La página pide una verificación: resuélvela y vuelve a guardar";
	assert.equal(locale.blocked.message, `${text}.`);
	assert.equal(locale.blockedIn.message, `${text} (perfil $PROFILE$).`);
});

test("usableResult rejects a missing, errored or empty injection result", () => {
	const ok = { page: { content: "x" }, text: "t" };
	assert.equal(usableResult({ result: ok }), ok);
	assert.equal(usableResult({ result: { text: "t" } }).text, "t");
	for (const injected of [undefined, {}, { result: null }, { result: undefined }, { result: ok, error: {} }, { result: {} }, { result: { text: "" } }])
		assert.equal(usableResult(injected), null, JSON.stringify(injected));
});

test("ping without api is appTooOld", async () => {
	const { transport } = fakeTransport({ status: 200, body: { profile: "a" } });
	assert.deepEqual(await ping(transport, TOKEN), { key: "appTooOld" });
});

test("ping with api below MIN_API is appTooOld", async () => {
	const { transport } = fakeTransport({ status: 200, body: { api: 0, profile: "a" } });
	assert.deepEqual(await ping(transport, TOKEN), { key: "appTooOld" });
});

test("ping with api at MIN_API is pingOkIn", async () => {
	const { transport } = fakeTransport({ status: 200, body: { api: 1, profile: "a" } });
	assert.deepEqual(await ping(transport, TOKEN), { key: "pingOkIn", profile: "a" });
});

test("404 is extensionTooOld", async () => {
	assert.equal(outcomeKey({ status: 404, body: {} }), "extensionTooOld");
	assert.equal(outcomeKey({ status: 404, body: { error: "x" } }), "extensionTooOld");
	const { transport } = fakeTransport({ status: 404, body: {} });
	assert.deepEqual(await saveTab(transport, TOKEN, CAPTURE), { key: "extensionTooOld" });
	assert.deepEqual(await ping(transport, TOKEN), { key: "extensionTooOld" });
});

test("401 is still invalidToken", async () => {
	assert.equal(outcomeKey({ status: 401, body: {} }), "invalidToken");
	const { transport } = fakeTransport({ status: 401, body: {} });
	assert.deepEqual(await ping(transport, TOKEN), { key: "invalidToken" });
});

test("appTooOld and extensionTooOld are errors", () => {
	assert.equal(stateOf("appTooOld"), "error");
	assert.equal(stateOf("extensionTooOld"), "error");
});
