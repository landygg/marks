# Troubleshooting and FAQ

[Español](../es/troubleshooting.md) · **English**

First of all, open the notification panel (the bell in the status bar): almost every failure leaves a notice there with its cause.

## Saving and extracting

### A bookmark is “Failed”

Open its detail: it shows the error. Use “Copy message” if you're going to report the problem.

| Message | What to do |
|---|---|
| “the page asked for an anti-bot check” | Open it in the browser, complete the check and save it with the [extension](browser-extension.md) |
| “the response is not an HTML page” | It's a PDF or some other file. marks only saves HTML pages |
| “too many redirects” | The site redirects in a loop. Try the final URL |
| “the webview did not finish loading the page in time” | The site is slow or not answering. “Retry” later |
| “destination not allowed: only remote http(s)” | The URL points at your own computer or a private network. marks doesn't download from there |
| An HTTP status error (404, 500…) | The page doesn't exist or the server is failing. “Retry” later |

### The content is incomplete or empty

- Try “Extract again”.
- If the page needs your session (a paywall, a social network), save it with the extension from the browser.
- With AI on, try “Extract with AI”.

### “the new extraction had too little content: nothing was changed”

“Extract again” or “Extract with AI” got less text than the minimum. marks keeps the content you already had.

### A bookmark stays at “Not summarized”

That's normal with AI off. With AI on, check the notifications: the model may be unavailable. To try again: Settings → “Maintenance” → “Process bookmarks without a summary”.

### I imported my bookmarks and they're “Not extracted”

That's on purpose: importing downloads nothing. See [Import and export](import-export.md#imported-bookmarks-are-not-extracted-on-their-own).

### I saved a URL and no new bookmark appeared

It was already saved. marks compares URLs without the `#…` fragment and the `utm_…` parameters, and doesn't duplicate.

### Images don't show

That's on purpose: when you read, marks loads nothing remote. Each image is a line with “Open”. See [Privacy](privacy.md).

## Searching

### Search doesn't find something I know I saved

- Search is per profile. Check you're in the right one.
- **Every** word you type has to appear. Try fewer.
- If the bookmark is “Not extracted” or “Failed”, it has no content to search.
- The URL isn't searched.

### Search doesn't find by meaning

Semantic search needs AI on, an embedding model and bookmarks in the “Ready” status. Anthropic has no embeddings. If you just changed the embedding model, wait for marks to finish reindexing.

## Artificial intelligence

### “Test connection” fails

- **Ollama:** check that it's running (`ollama list` in the terminal) and that you've downloaded both models. The default endpoint is `http://localhost:11434`.
- **“this provider requires an endpoint”:** oMLX and “OpenAI-compatible” need one.
- **“this provider requires an API key”:** type it and click “Save”.
- “Test connection” tests what is **saved**: save before testing.

### “AI is on, but the model could not be prepared”

The saved settings don't allow the model to be created (a missing key, endpoint or model name). Review Settings → “Artificial intelligence”.

### “the input does not fit in the context marks asks Ollama for”

The page is too long for that model. Try another model.

### “Extract with AI” is disabled

It needs AI on and a valid extraction model. See [Artificial intelligence](ai.md#extract-with-ai).

### The computer is slow while marks summarizes

A local model uses a lot of memory and CPU. marks processes bookmarks one at a time. You can use a smaller model, or turn AI off and back on when it suits you: the bookmarks wait at “Not summarized”.

## Browser extension

### “marks is not responding. Open the app and try again.”

marks is closed, or couldn't open its port. Open marks and look at Settings → “Browser extension”: it should say “API listening on 127.0.0.1:47821.”

### “Port 47821 is in use”

Another program, or another copy of marks, is using that port. Close the other copy and restart marks. The port can't be changed.

### “The token is not valid.”

The token was regenerated or its profile was deleted. Copy the current one from Settings → “Browser extension” and paste it into the extension's options.

### The extension saves to the wrong profile

It saves to the profile that owns the token, not to the app's active profile. Paste the token of the profile you want.

### The extension doesn't show up in Safari

The Safari extension is experimental: Safari only loads it with “Allow unsigned extensions” (Settings → Developer), and turns that off every time it quits.

- Every time you open Safari, turn “Allow unsigned extensions” back on. If you don't see the Developer tab, first turn on “Show features for web developers” in Settings → Advanced.
- Check that marks is in `/Applications` and that you've opened it at least once.
- In Terminal, `pluginkit -m -i dev.landygg.marks.safari-extension` should list it.
- If there are two copies of marks.app (say, an old one in Downloads), delete the extra one and open Safari again.
- To skip these steps, use the Chromium extension: see [Browser extension](browser-extension.md).

### The extension's icon shows `!`

The last save failed after you closed the popup. Open it to see why.

## The app

### macOS says it can't check marks

“Apple could not verify “marks” is free of malware…”: marks is signed, but not notarized by Apple. It happens the first time and with every version you download by hand (on Intel, on every update).

1. Click “Done” in the warning.
2. System Settings → Privacy & Security → “Open Anyway”, and confirm.

Or, in Terminal: `xattr -dr com.apple.quarantine /Applications/marks.app`. More in [Installation](installation.md#the-first-time-open-anyway).

### marks asks for the keychain password after an update

It needs it to read the AI provider's API key. Type your login password and click “Always Allow”: it won't ask again until the next update.

### I closed the window and marks is still running

That's on purpose: marks stays in the menu bar to extract what's pending and answer the extension. To quit: menu bar icon → “Quit”, or ⌘Q.

### I changed the language and the app relaunched

That's required: the language is applied at launch.

### marks won't open and mentions the database

marks shows an alert and quits. There are three messages:

- “This database (<path>) comes from a newer version of marks, and this version can't open it. Install the latest marks, or restore the copy made before the migration.”

  You went back to an earlier version of marks, and that version can't read a database already migrated by a newer one. Install the newer version again, or restore the copy made before the migration.
- “marks doesn't recognize this database (<path>): it wasn't created by marks 1.0 or later. It was left untouched.”

  The database comes from a test version before 1.0, or the file isn't from marks. marks doesn't open it or change it.
- “Couldn't copy the database to <path> before updating it, so it was left untouched. Check that the disk has free space and open marks again.”

  marks had to update the database format and couldn't make the backup first, usually for lack of space. Free some disk space and open marks again.

See [Data and backups](data-and-backups.md).

### Where the logs are

In `~/Library/Logs/dev.landygg.marks/`. Review them before attaching them to a report: they may contain addresses of pages you've saved.

## Questions

**Do I need an account?** No.

**Does it work offline?** Yes, for reading, searching and organizing. Saving a new page needs the network to download it.

**Can I use marks without AI?** Yes. It's off by default.

**Do my pages leave my computer?** Only if you turn AI on with a remote provider. See [Privacy](privacy.md).

**Can I sync between two Macs?** No. You can move the data folder from one Mac to another, but not use it from both at once.

**Is there a version for Windows, Linux or mobile?** Not yet. Windows and Linux will come later.

**Does it work on an Intel Mac?** It's published, but untested.

**Which browsers have an extension?** Safari and Chromium browsers (Chrome, Brave). Not Firefox.

**Can I search all profiles at once?** No.

**Can I recover a deleted bookmark or profile?** No, except from a backup.

**Does marks save the images of a page?** No. It saves the text and the favicon.

**Can I read my data without marks?** Yes. They're standard SQLite files, and `marks export` writes them out as Markdown. See [Command line](cli.md).

**Can I change the shortcuts?** Not the app's. The extension's, yes, from the browser.

**How do I report a problem?** Open an issue on [GitHub](https://github.com/landygg/marks/issues) with the bug template. Review any logs before you attach them (see [Where the logs are](#where-the-logs-are)). A security problem goes in private: see [`SECURITY.md`](../../SECURITY.md).
