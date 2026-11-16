# Updates

[Español](../es/updates.md) · **English**

## The app

marks updates from the versions published on GitHub, at <https://github.com/landygg/marks/releases>. There is a single channel: the stable releases.

- **When it checks:** 30 seconds after it opens and, while it stays open, once a day. Also by hand, with “Check for updates” in Settings → Updates.
- **How it tells you:** with a notification, “A new version of marks is available: …”. Its Settings button leads to Settings → Updates.
- **It asks first:** it downloads and installs nothing until you press “Install … and restart”. Then it downloads the new version, checks its signature, installs it and restarts marks.
- **It can be turned off:** the “Check automatically” switch, in Settings → Updates. With it off, marks doesn't contact GitHub unless you press “Check for updates”.
- **Apple silicon Macs only.** On an Intel Mac, the section tells you to download new versions from GitHub, and it doesn't check on its own. If an automatic check fails (offline, for example), it doesn't tell you: it tries again the next day.

To update by hand, download the new version and replace marks in the Applications folder. Your data isn't inside the app: it stays in `~/Library/Application Support/marks/`. Every version you download by hand carries the download mark, so macOS blocks it again the first time you open it: open it with “Open Anyway” again (see [Installation](installation.md#the-first-time-open-anyway)). On an Intel Mac, which doesn't update itself, this happens on every update.

### What happens to your data when you update

A new version may change the database format (a migration). marks does it when it opens, without asking, with two safeguards:

- It makes a **copy of each database before migrating it**.
- An **older** version of marks refuses to open a newer database it can't read, and says so with a clear message.

**There is no automatic way back.** If you want to return to the previous version after updating, you have to restore the copy made before the migration; anything saved afterwards won't be in it. See [Data and backups](data-and-backups.md#automatic-copies-before-a-migration).

Before a big update, a manual copy of the data folder doesn't hurt.

### What connects to the network

The update check contacts GitHub. It is the only connection marks makes without you saving something. See [Privacy](privacy.md).

## The Safari extension

It's experimental: Safari only loads it with “Allow unsigned extensions”. See [Browser extension](browser-extension.md#safari-experimental).

In v1.0 it ships inside marks.app, so **it updates with the app**. It has its own version, and the app tells it whether it is compatible with its API version.

After updating marks, if Safari still shows the old extension, quit Safari and open it again.

If they aren't compatible, the extension says so when you test the connection or save:

- “This version of marks is too old for the extension. Update marks.”: update the app (Settings → Updates, or download it from [GitHub](https://github.com/landygg/marks/releases)).
- “The extension is too old for this version of marks. Update it.”: in Chromium, unzip the new version into the same folder, replacing the old one, and press “Reload” in `chrome://extensions`; in Safari, quit Safari and open it again, since the extension comes with the app.

Later on, if the Apple Developer Program is paid for (the Mac App Store requires it), the extension will move to its own app there and update from it, on its own cycle.

## The Chromium extension

- Loaded unpacked: it doesn't update itself. Unzip the new version into the same folder, replacing the old one, and click “Reload” in `chrome://extensions`. That way the token is kept: the browser identifies an unpacked extension by its folder, and loading it from another one makes it a new extension with no token.

v1.0 isn't on the Chrome Web Store.

## Finding out which version you have

- **The app:** marks menu → “About marks”.
- **The command line:** `marks --version`.

- **The extension** (Safari or Chromium): at the bottom of its options, “Version 1.0.0”.

What's new in each version is in [CHANGELOG.md](../../CHANGELOG.md) and in each version's notes on [GitHub Releases](https://github.com/landygg/marks/releases).
