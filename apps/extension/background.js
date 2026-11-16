// Runs the save so it survives the popup closing mid-request.
import {
	buildCapture,
	isSaved,
	pickTransport,
	SAVE_MESSAGE,
	saveTab,
	usableResult,
} from "./client.js";
import { ext } from "./i18n.js";

export const FAILURE_BADGE = "!"; // cleared by the popup when it opens

const EXTRACT_FILE = "page-extract.js"; // bundled by build.mjs; defines globalThis.marksExtract

/** Extracts on a clone of the page (never the live one); falls back to plain text, then to nothing. */
async function extractTab(tabId) {
	const target = { target: { tabId } };
	try {
		await ext.scripting.executeScript({ ...target, files: [EXTRACT_FILE] });
		const [injected] = await ext.scripting.executeScript({
			...target,
			func: () => globalThis.marksExtract(),
		});
		// A func that throws does not reject in every engine: treat no result or an error as a failure.
		const result = usableResult(injected);
		if (!result) throw new Error("extraction failed"); // i18n-allow: invariant
		return result;
	} catch {
		try {
			const [injected] = await ext.scripting.executeScript({
				...target,
				func: () => ({ text: document.body?.innerText ?? "" }),
			});
			return usableResult(injected) ?? {};
		} catch {
			return {}; // restricted page: save just URL and title
		}
	}
}

async function save({ token, tabId, url, title }) {
	const extracted = await extractTab(tabId);
	const result = await saveTab(
		pickTransport(location.protocol, ext.runtime),
		token,
		buildCapture({ url, title }, extracted),
	);
	// The popup may be gone: flag a failure on the toolbar icon so it is not lost silently.
	if (!isSaved(result.key))
		await ext.action.setBadgeText({ text: FAILURE_BADGE }).catch(() => {});
	return result;
}

ext.runtime.onMessage.addListener((message, sender, sendResponse) => {
	if (message?.type !== SAVE_MESSAGE || sender.id !== ext.runtime.id)
		return false;
	save(message).then(sendResponse);
	return true; // keeps the channel open for the async reply
});
