// Shared client for the marks local API. No DOM or extension APIs here: it must run under Node.
export const API_PORT = 47821;
export const API_BASE = `http://127.0.0.1:${API_PORT}`;
export const ROUTES = { ping: "/v1/ping", bookmarks: "/v1/bookmarks" };
export const TOKEN_KEY = "token"; // key in storage.local
export const NATIVE_APP_ID = "dev.landygg.marks"; // Safari ignores it and routes to the app's appex
export const REQUEST_TIMEOUT_MS = 30000;
export const SAVE_MESSAGE = "save"; // popup → background: { type, token, tabId, url, title }
/** Lowest API level (`api` in /v1/ping, `API_LEVEL` in the app) this extension works with; a missing `api` counts as 0. */
export const MIN_API = 1;

const SAVED_KEYS = ["saved", "savedIn", "alreadySaved", "alreadySavedIn"];
const BLOCKED_KEY = "blocked";
const OK_KEYS = ["pingOk", "pingOkIn", "tokenSaved"];
const PENDING_KEYS = ["saving"];

const SAFARI_PROTOCOL = "safari-web-extension:";
const SAVABLE_PROTOCOLS = ["http:", "https:"];
const NATIVE_UNREACHABLE = "unreachable"; // the Swift handler's error when marks does not answer
const STATUS_OK = 200;
const STATUS_BAD_REQUEST = 400;
const STATUS_UNAUTHORIZED = 401;
const STATUS_NOT_FOUND = 404; // a route this app no longer has: the extension is behind
const STATUS_TOO_LARGE = 413;

/** `true` only for http: and https: URLs. */
export function isSavable(url) {
	try {
		return SAVABLE_PROTOCOLS.includes(new URL(url).protocol);
	} catch {
		return false;
	}
}

/** A transport takes { method, path, token, body? } and resolves to { status, body }; it throws when marks is unreachable. */
export async function fetchTransport(
	{ method, path, token, body },
	fetchImpl = fetch,
) {
	const headers = { Authorization: `Bearer ${token}` }; // i18n-allow: format
	if (body !== undefined) headers["Content-Type"] = "application/json";
	const response = await fetchImpl(API_BASE + path, {
		method,
		headers,
		body: body === undefined ? undefined : JSON.stringify(body),
		signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
	});
	return {
		status: response.status,
		body: await response.json().catch(() => ({})),
	};
}

/** Throws only when the Swift handler could not reach marks; any other handler error is a 400. */
export async function nativeTransport(request, runtime) {
	const reply = await runtime.sendNativeMessage(NATIVE_APP_ID, request);
	if (reply?.error === NATIVE_UNREACHABLE) throw new Error(reply.error);
	if (reply?.error) return { status: STATUS_BAD_REQUEST, body: {} };
	return reply;
}

/** Returns a one-argument transport: native messaging on Safari, fetch elsewhere. */
export function pickTransport(protocol = location.protocol, runtime) {
	return protocol === SAFARI_PROTOCOL
		? (request) => nativeTransport(request, runtime)
		: fetchTransport;
}

/** The injected script's { page?, text? }, or null when it threw (no result or an `error`) or came back empty. */
export function usableResult(injected) {
	const result = injected?.result;
	return !injected?.error && (result?.page || result?.text) ? result : null;
}

/** Body for POST /v1/bookmarks: { url, title } plus the extracted page and text, leaving out what is empty. */
export function buildCapture(tab, extracted) {
	const capture = { url: tab.url };
	if (tab.title) capture.title = tab.title;
	if (extracted?.page) capture.page = extracted.page;
	if (extracted?.text) capture.text = extracted.text;
	return capture;
}

/** Message key for a response: "saved(In)" | "alreadySaved(In)" | "blocked(In)" | "extensionTooOld" | "invalidToken" | "serverError" | "error_<code>" (the API's error code); the "In" keys when the app names its profile. */
export function outcomeKey({ status, body }) {
	if (status === STATUS_NOT_FOUND) return "extensionTooOld";
	if (status !== STATUS_OK && body?.error)
		return `error_${String(body.error).replaceAll("-", "_")}`; // the API's code; popup/options translate it
	if (status === STATUS_UNAUTHORIZED) return "invalidToken";
	if (status !== STATUS_OK) return "serverError";
	const key = body?.blocked
		? BLOCKED_KEY
		: body?.created
			? "saved"
			: "alreadySaved";
	return body?.profile ? `${key}In` : key; // i18n-allow: identifier
}

/** `true` when the key means the page is in marks (new or already there). */
export const isSaved = (key) => SAVED_KEYS.includes(key);

// Manifest key names → macOS symbols, in Apple's order (⌃⌥⇧⌘). "Ctrl" is ⌘ on macOS, as in the manifest.
const MAC_MODIFIERS = [ // i18n-allow: identifier
	["MacCtrl", "⌃"],
	["Alt", "⌥"],
	["Shift", "⇧"],
	["Ctrl", "⌘"],
	["Command", "⌘"],
];
const MAC_SYMBOLS = MAC_MODIFIERS.map(([, symbol]) => symbol);
const SHORTCUT_SEPARATOR = "+";
const MAC_OS = "mac"; // runtime.getPlatformInfo().os

/**
 * The shortcut's keys, as the platform writes them (the popup joins them with "·"). Safari returns the manifest's
 * "Alt+Shift+M", which on macOS becomes ["⌥", "⇧", "M"]; Chromium on macOS already returns "⌥⇧M".
 * Elsewhere the names stay: ["Alt", "Shift", "M"].
 */
export function shortcutKeys(shortcut, os) {
	if (shortcut.includes(SHORTCUT_SEPARATOR)) {
		const parts = shortcut.split(SHORTCUT_SEPARATOR);
		const key = parts.pop();
		if (os !== MAC_OS) return [...parts, key];
		const symbols = MAC_MODIFIERS.filter(([name]) => parts.includes(name)).map(([, symbol]) => symbol);
		return [...symbols, key];
	}
	// Already in symbols: each leading modifier is a key, the rest ("M", "Space") is the last one.
	const chars = [...shortcut];
	const keyStart = chars.findIndex((char) => !MAC_SYMBOLS.includes(char));
	if (keyStart < 0) return chars;
	return [...chars.slice(0, keyStart), chars.slice(keyStart).join("")];
}

/** Status-dot state for a message key: "pending" | "ok" | "error". */
export function stateOf(key) {
	if (PENDING_KEYS.includes(key)) return "pending";
	return isSaved(key) || OK_KEYS.includes(key) ? "ok" : "error";
}

/** The message for a key; an error code this version doesn't know (no message) falls back to "serverError", without detail. */
export function messageFor(key, t, profile) {
	return t(key, profile) || t("serverError");
}

function withProfile(response) {
	const key = outcomeKey(response);
	return response.body?.profile
		? { key, profile: response.body.profile }
		: { key };
}

/** Posts the capture; on 413 retries once without `text`. Resolves to { key, profile? }; key "appClosed" when the transport throws. */
export async function saveTab(transport, token, capture) {
	const post = (body) =>
		transport({ method: "POST", path: ROUTES.bookmarks, token, body });
	try {
		let response = await post(capture);
		if (response.status === STATUS_TOO_LARGE) {
			const { text, ...withoutText } = capture;
			response = await post(withoutText);
		}
		return withProfile(response);
	} catch {
		return { key: "appClosed" };
	}
}

/** Resolves to { key, profile? }: "pingOk" | "pingOkIn" | "appTooOld" | "extensionTooOld" | "invalidToken" | "appClosed" | "serverError". */
export async function ping(transport, token) {
	try {
		const response = await transport({
			method: "GET",
			path: ROUTES.ping,
			token,
		});
		if (response.status !== STATUS_OK) return { key: outcomeKey(response) };
		if ((response.body?.api ?? 0) < MIN_API) return { key: "appTooOld" };
		const profile = response.body?.profile;
		return profile ? { key: "pingOkIn", profile } : { key: "pingOk" };
	} catch {
		return { key: "appClosed" };
	}
}
