import {
	isSavable,
	isSaved,
	messageFor,
	SAVE_MESSAGE,
	shortcutKeys,
	stateOf,
	TOKEN_KEY,
} from "./client.js";
import { ext, t, translatePage } from "./i18n.js";

const OPTIONS_KEYS = ["noToken", "invalidToken", "error_token"];
const ACTION_COMMAND = "_execute_action";
const KEY_SEPARATOR = "·";
const status = document.getElementById("status");
const statusText = document.getElementById("status-text");
const optionsLink = document.getElementById("options-link");
const page = document.getElementById("page");

function show({ key, profile }) {
	statusText.textContent = messageFor(key, t, profile);
	status.dataset.state = stateOf(key);
	page.classList.toggle("marked", isSaved(key));
	optionsLink.hidden = !OPTIONS_KEYS.includes(key);
}

function showPage(tab) {
	document.getElementById("page-title").textContent = tab.title || tab.url;
	document.getElementById("page-domain").textContent = new URL(tab.url).hostname;
	page.hidden = false;
}

/** Shows the action's real shortcut, written as the platform does; hidden when there is none. */
async function showShortcut() {
	const commands = await ext.commands?.getAll?.().catch(() => []);
	const shortcut = commands?.find((c) => c.name === ACTION_COMMAND)?.shortcut;
	if (!shortcut) return;
	const platform = await ext.runtime.getPlatformInfo?.().catch(() => undefined);
	const kbd = document.getElementById("shortcut");
	// One box, keys split by a centered dot that screen readers skip (DESIGN.md, `kbd`).
	kbd.replaceChildren(
		...shortcutKeys(shortcut, platform?.os).flatMap((key, i) => {
			if (i === 0) return [key];
			const dot = document.createElement("span");
			dot.setAttribute("aria-hidden", "true");
			dot.textContent = KEY_SEPARATOR;
			return [dot, key];
		}),
	);
	kbd.title = t("shortcutTitle");
	kbd.hidden = false;
}

async function run() {
	const stored = await ext.storage.local.get(TOKEN_KEY);
	const token = stored[TOKEN_KEY];
	if (!token) return show({ key: "noToken" });
	const [tab] = await ext.tabs.query({ active: true, currentWindow: true });
	if (!isSavable(tab?.url)) return show({ key: "notSavable" });
	showPage(tab);
	// The background extracts and saves, so closing the popup does not cancel the request.
	show(
		await ext.runtime.sendMessage({
			type: SAVE_MESSAGE,
			token,
			tabId: tab.id,
			url: tab.url,
			title: tab.title,
		}),
	);
}

translatePage();
ext.action.setBadgeText({ text: "" }).catch(() => {}); // the user is looking now
showShortcut();
optionsLink.addEventListener("click", (event) => {
	event.preventDefault();
	ext.runtime.openOptionsPage();
});
run().catch(() => show({ key: "appClosed" }));
