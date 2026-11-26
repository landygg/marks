## errors-store

error-chain = { $head }: { $rest }

error-no-content = no content
error-bookmark-not-found = bookmark not found
error-store-poisoned = the store became unusable after an earlier failure
error-stores-poisoned = the map of open stores became unusable after an earlier failure
error-unknown-status = unknown status: { $value }
error-unknown-error-kind = unknown error kind: { $value }
error-open-database = could not open { $path }
error-database-too-new = This database ({ $path }) comes from a newer version of marks, and this version can't open it. Install the latest marks, or restore the copy made before the migration.
error-database-unknown = marks doesn't recognize this database ({ $path }): it wasn't created by marks 1.0 or later. It was left untouched.
error-database-unreadable = marks can't read this database ({ $path }): it isn't an SQLite database, or it's damaged. It was left untouched. If marks backed it up before an update, the copies are in the backups folder next to it.
error-backup = Couldn't copy the database to { $path } before updating it, so it was left untouched. Check that the disk has free space and open marks again.
error-invalid-url = invalid URL
error-group-name-empty = the group name cannot be empty
error-group-exists = a group with that name already exists
error-group-not-found = group not found
error-profile-not-found = profile not found
error-profile-name = the profile name must be between 1 and { $max } characters
error-profile-initials = the initials must be 1 or { $max } characters
error-profile-exists = a profile with that name already exists
error-default-profile = the first profile cannot be deleted
error-same-profile = the bookmark is already in that profile
error-url-in-profile = that bookmark is already in the profile “{ $profile }”
error-file-too-large = the file is over { $max_mb } MB
error-not-utf8 = the file is not UTF-8

## errors-fetch

error-blocked = the page asked for an anti-bot check
error-blocked-bookmark = the page asked for an anti-bot check: save it with the browser extension
error-unsupported-scheme = only http or https URLs can be saved
error-destination = destination not allowed: only remote http(s)
error-not-html = the response is not an HTML page
error-too-many-redirects = too many redirects
error-not-image = the response is not a supported image
error-favicon-too-large = the favicon is over the maximum size
error-no-location = redirect without a destination
error-fetch-task = the download task ended unexpectedly
error-thin-result = the new extraction had too little content: nothing was changed

## errors-ai

error-extract-timeout = AI extraction did not finish in time
error-embed-chunk-count = the model returned a different number of vectors than chunks
error-embed-dim = the model returned empty vectors or vectors of different sizes
error-vectors-len-mismatch = the number of chunks and vectors does not match
error-vectors-dim-mismatch = the vector size does not match the index size
error-vectors-no-index = the vector index is not initialized
error-vectors-bad-dim = the stored index size is not valid
error-no-json = the model response does not contain a JSON object
error-invalid-json = the model JSON does not have the expected shape
error-api-key-required = this provider requires an API key
error-anthropic-no-embeddings = Anthropic does not offer embeddings: choose another provider
error-client-build = could not create the model client
error-chat-failed = the chat model call failed
error-embed-failed = the embeddings model call failed
error-empty-chat = the model returned no text
error-embed-batch-count = the model returned a different number of vectors than texts
error-ollama-too-long = the input does not fit in the context marks asks Ollama for: the page is too long for this model
error-ollama-status = Ollama responded with error { $status }: { $detail }
error-ollama-bad-response = the Ollama response does not have the expected shape
error-endpoint-required = this provider requires an endpoint
error-empty-model = model names cannot be empty
error-invalid-settings = the saved AI settings are not valid
error-empty-extraction-model = the extraction model cannot be empty
error-keyring-entry = could not open the Keychain entry
error-keyring-read = could not read the API key from the Keychain
error-keyring-write = could not save the API key to the Keychain
error-keyring-delete = could not delete the API key from the Keychain
error-token-random = could not generate the API token

## errors-app

error-webview-url = URL not allowed in the webview: only remote http(s)
error-webview-timeout = the webview did not finish loading the page in time: { $stage }
error-webview-decode = the webview returned a response that could not be decoded
error-webview-closed-loading = the webview closed before it finished loading
error-webview-closed-html = the webview closed before returning the HTML
error-parser-page = the parser window did not return a valid page
error-parser-clean = the parser window did not return valid clean HTML
error-parser-timeout = the parser window did not finish extracting in time: { $stage }
error-parser-closed-loading = the parser window closed before it finished loading
error-parser-closed-page = the parser window closed before returning the page
error-no-pending-import = there is no import ready
error-import-other-profile = the import was prepared for another profile
error-import-read = could not read { $path }
error-no-model = no AI model is configured
error-no-extraction-model = no extraction model is configured
error-export-write = could not write the exported file
error-stage-open = opening the window
error-stage-load = waiting for the page to finish loading
error-stage-settle = waiting for the page to settle
error-stage-eval = waiting for the script to respond
error-stage-ready = waiting for the extraction module to be ready

## data

data-untagged-folder = Untagged
data-reading-list-tag = reading list
data-notes-heading = ## Notes
data-default-profile-name = Personal
data-default-profile-initials = PE

## app
app-window-title = marks
tray-open = Open marks
tray-add = Add URL…
tray-quit = Quit
app-import-dialog-title = Import bookmarks
app-export-dialog-title = Export bookmarks
app-html-filter = HTML file
app-model-unavailable = AI is on, but the model could not be prepared
app-profiles-failed = could not read the profile registry
app-store-failed = the store failed while processing pending bookmarks
app-favicons-failed = could not save the favicons
app-profile-open-failed = could not open the profile
app-api-port-busy = the local API for the extension could not open port { $port }
app-api-stopped = the local API for the extension stopped
app-empty-notification = error with no message
app-update-available = A new version of marks is available: { $version }.
app-update-check-failed = Couldn't check for a new version. Check your connection.
app-update-install-failed = Couldn't install the new version.
app-update-needs-macos = marks { $version } needs macOS { $minimum } or later, and this Mac has macOS { $current }. marks stays on this version.
menu-about = About marks
menu-services = Services
menu-hide = Hide marks
menu-hide-others = Hide Others
menu-quit = Quit marks
menu-close-window = Close Window
menu-undo = Undo
menu-redo = Redo
menu-cut = Cut
menu-copy = Copy
menu-paste = Paste
menu-select-all = Select All
menu-fullscreen = Enter Full Screen
menu-minimize = Minimize
menu-maximize = Zoom
menu-file = File
menu-edit = Edit
menu-view = View
menu-window = Window
menu-help = Help

## cli
cli-about = Local bookmarks with full-text search
cli-help-db = Path to the SQLite database (variable { $env }; default: { $default })
cli-help-add = Queues the URL; the app downloads and extracts it when it opens
cli-help-title = Manual title (otherwise the app uses the page's)
cli-help-notes = User notes
cli-help-tag = Tag (repeatable)
cli-help-profile = Profile to work on; the first one by default
cli-help-profiles = Lists the profiles
cli-help-list = Lists all bookmarks
cli-help-search = Full-text search
cli-help-export = Exports one .md per bookmark into the directory
cli-help-import = Imports a bookmarks file (Netscape HTML); they stay unextracted until you ask for it in the app
cli-help-export-html = Exports all bookmarks to a bookmarks file (Netscape HTML)
cli-help-stats = Counts bookmarks by status
cli-usage-root = marks [OPTIONS] <COMMAND>
cli-usage-add = marks add [OPTIONS] <URL>
cli-usage-profiles = marks profiles [OPTIONS]
cli-usage-list = marks list [OPTIONS]
cli-usage-search = marks search [OPTIONS] [QUERY]...
cli-usage-export = marks export [OPTIONS] <DIR>
cli-usage-import = marks import [OPTIONS] <FILE>
cli-usage-export-html = marks export-html [OPTIONS] <FILE>
cli-usage-stats = marks stats [OPTIONS]
cli-value-command = COMMAND
cli-value-db = PATH
cli-value-profile = NAME
cli-value-title = TITLE
cli-value-notes = NOTES
cli-value-tag = TAG
cli-value-url = URL
cli-value-query = QUERY
cli-value-dir = DIR
cli-value-file = FILE
cli-exported =
    { $n ->
        [one] { $n } bookmark exported to { $path }
       *[other] { $n } bookmarks exported to { $path }
    }
cli-import-summary =
    { $new ->
        [one] { $new } new
       *[other] { $new } new
    }, { $existing ->
        [one] { $existing } already present
       *[other] { $existing } already present
    }, { $skipped ->
        [one] { $skipped } not importable
       *[other] { $skipped } not importable
    }
cli-row-error = (error: { $error })
cli-status-pending = pending
cli-status-pending-embed = waiting to be indexed
cli-status-ready = ready
cli-status-failed = failed
cli-status-imported = not extracted
cli-stats-total = total: { $n }
cli-stats-pending = pending: { $n }
cli-stats-failed = failed: { $n }
cli-stats-unsummarized = unsummarized: { $n }
cli-stats-imported = not extracted: { $n }
cli-usage = Usage:
cli-arguments-heading = Arguments
cli-options-heading = Options
cli-commands-heading = Commands
cli-help-flag = Print help
cli-version-flag = Print version
cli-error-unknown-argument = unknown argument: { $arg }
cli-error-invalid-subcommand = unknown subcommand: { $arg }
cli-error-missing-required-argument = missing required arguments: { $arg }
cli-error-invalid-value = invalid value for { $arg }
cli-error-missing-value = missing value for { $arg }
cli-error-value-validation = the value of { $arg } is not valid
cli-error-too-many-values = too many values for { $arg }
cli-error-argument-conflict = { $arg } cannot be used with the other arguments
cli-error-argument-repeated = { $arg } cannot be used more than once
cli-error-missing-subcommand = { $arg } requires a subcommand
cli-error-invalid-utf8 = an argument is not valid UTF-8
cli-error-generic = the arguments are not valid
cli-error-hint = For more information, try '--help'.
cli-error-did-you-mean = Did you mean '{ $suggestion }'?

## locale

error-invalid-locale = unsupported language
