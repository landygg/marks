import { messageFor, pickTransport, ping, stateOf, TOKEN_KEY } from "./client.js";
import { ext, t, translatePage } from "./i18n.js";

const input = document.getElementById("token");
const status = document.getElementById("status");
const statusText = document.getElementById("status-text");

// Emptied first and written a frame later, so the same result twice is announced twice.
function show(key, profile) {
	statusText.textContent = "";
	delete status.dataset.state;
	requestAnimationFrame(() => {
		statusText.textContent = messageFor(key, t, profile);
		status.dataset.state = stateOf(key);
	});
}

translatePage();
document.getElementById("version").textContent = t("extensionVersion", ext.runtime.getManifest().version);
ext.storage.local.get(TOKEN_KEY).then((stored) => {
	input.value = stored[TOKEN_KEY] ?? "";
});

document.getElementById("token-form").addEventListener("submit", async (event) => {
	event.preventDefault();
	const token = input.value.trim();
	try {
		await ext.storage.local.set({ [TOKEN_KEY]: token });
		input.value = token;
		show("tokenSaved");
	} catch {
		show("tokenSaveFailed");
	}
});

document.getElementById("test").addEventListener("click", async () => {
	const { key, profile } = await ping(
		pickTransport(location.protocol, ext.runtime),
		input.value.trim(),
	);
	show(key, profile);
});
