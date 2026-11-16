# Privacy

[Español](../es/privacy.md) · **English**

marks keeps your data on your computer and only connects to the network for what you ask it to do. This page says exactly what goes out and what doesn't.

## Summary

| Question | Answer |
|---|---|
| Does it have accounts or a server of its own? | No |
| Does it send telemetry, statistics or crash reports? | No |
| Does it sync your bookmarks anywhere? | No |
| Does it download the pages you save? | Yes: that's its job |
| Does it send content to an AI provider? | Only if you turn AI on. AI is off by default, and the default provider is local |
| Does it check for updates? | Yes, on GitHub, once a day. Turn it off in Settings → Updates |

## What stays on your computer

- Bookmarks, their content, notes, tags, groups, summaries and search vectors: in SQLite files. See [Data and backups](data-and-backups.md).
- API keys: in the macOS Keychain.
- The app's logs: in `~/Library/Logs/dev.landygg.marks/`. They aren't sent anywhere.
- Searches: they are resolved on your computer. With AI on, the search text is sent to your embeddings provider to be turned into a vector; with a local Ollama, that doesn't leave either.

The databases are **not encrypted**. They're protected by the same things as the rest of your home folder: your macOS login and, if you have it on, FileVault.

## What goes out to the network

### 1. Downloading the pages you save

When you save a URL from the app, the menu bar or the terminal, marks requests that page from the website, as a browser would.

- The request identifies itself with the user agent `marks/<version> (+https://github.com/landygg/marks)`, which points site owners to the project.
- It goes **without your cookies or session**: the site doesn't know who you are beyond your IP address.
- When needed, marks loads the page in a hidden, private window to run its JavaScript. That page may request its own resources (scripts, images), as in a browser. The window shares nothing with your browser, has no access to the app and is destroyed when it's done.
- marks also downloads the site's **favicon**, once, and only from public addresses.

What you save from the **extension** isn't downloaded again: it arrives already extracted from your browser.

Importing a `bookmarks.html` downloads **nothing**. Pages are requested when you run “Extract”.

### 2. AI, if you turn it on

With AI on, marks sends your provider:

- The text of each page, to summarize and tag it.
- Chunks of the text, to compute the search vectors.
- The text of each search, for the semantic part.
- With “Extract with AI”, the page's cleaned HTML.

| Provider | Where it goes |
|---|---|
| Ollama on `localhost` (default) | Nowhere: it stays on your computer |
| Ollama, oMLX or a compatible server on another machine | To that machine |
| OpenAI or Anthropic | To their servers, under their terms |

The app warns you at two points: when you extract several imported bookmarks with AI on, and in Settings when the “Extract with AI” endpoint is outside your computer.

### 3. Updates

marks checks GitHub for a new version.

- **What it carries:** the request URL names the system (`darwin`) and the Mac's architecture (`aarch64`), and the User-Agent header says `tauri-plugin-updater/<version>`, the name and version of the component that checks for updates. It doesn't carry your marks version. As with any connection, GitHub sees your IP address. It carries none of your bookmarks, your token or any of your data.
- **When:** 30 seconds after marks opens and once a day while it stays open, only on Apple silicon Macs; and whenever you press “Check for updates”.
- **How to turn it off:** Settings → Updates → “Check automatically”. See [Updates](updates.md).

### And nothing else

When you **read** a bookmark, marks loads nothing remote. Images, videos and embedded content are shown as a line with “Open”, and links open in your browser when you click them. A site can't tell when you reread a saved page.

## The local API

To serve the extension, marks opens a small server at `127.0.0.1:47821`.

- It only listens on your own computer: it can't be reached from your network or from the internet.
- Every request must carry the profile's token. marks also checks the `Host` header and that the request doesn't come from a web page.
- The token travels in a header, never in the URL.

Limitations worth knowing about:

- A web page can detect that marks is running (that the port is open). It can't read, add or detect bookmarks.
- The connection is HTTP inside your computer, unencrypted, and the extension can't verify that marks is the one answering. A malicious program already running on your Mac that takes port 47821 before marks does would receive the token and the pages you save. If marks can't open the port, it notifies you (“the local API for the extension could not open port 47821”), and Settings → “Browser extension” shows “Port 47821 is in use…”.

## The browser extension

- It only acts when you click its button or press its shortcut.
- It only reads the active tab, and from a copy.
- It only connects to marks on your computer.

Its permissions are listed in [Browser extension](browser-extension.md#permissions-it-asks-for).

## What marks stores about each page

Besides the text: the title, the URL (normalized and original), the description, the author, the site, the publication date, the language, the favicon and the address of the cover image (only the address; the image isn't downloaded).

## Open source

marks is free software under the MIT license. Everything on this page can be checked in the code.
