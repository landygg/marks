use marks_i18n::{Locale, resolve};
use std::path::PathBuf;
use std::process::{Command, Output};

/// Temporary database with a unique name; deletes the database and its profiles folder on drop.
struct TempDb(PathBuf);

impl TempDb {
    fn new(tag: &str) -> Self {
        let name = format!("marks-cli-locale-{tag}-{}.db", std::process::id());
        Self(std::env::temp_dir().join(name))
    }

    /// Runs `marks` with `LANG` and without the other variables that set the language.
    fn run(&self, lang: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_marks"))
            .env("MARKS_DB", &self.0)
            .env("LANG", lang)
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .env_remove("MARKS_LOCALE")
            .args(args)
            .output()
            .expect("no se pudo ejecutar marks")
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        for ext in ["", "-wal", "-shm"] {
            let mut p = self.0.clone().into_os_string();
            p.push(ext);
            let _ = std::fs::remove_file(p);
        }
        let _ = std::fs::remove_dir_all(self.0.with_extension("").with_file_name(format!(
            "{}-profiles",
            self.0.file_stem().unwrap_or_default().to_string_lossy()
        )));
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// CLI subcommands, to check each one's help.
const COMMANDS: [&str; 8] = [
    "add",
    "profiles",
    "list",
    "search",
    "export",
    "import",
    "export-html",
    "stats",
];

/// English clap pieces that must not appear in the Spanish help.
const ENGLISH: [&str; 13] = [
    "OPTIONS",
    "COMMAND",
    "[env:",
    "[default:",
    "Usage",
    "Options",
    "Commands",
    "Arguments",
    "Print",
    "<DB>",
    "<TITLE>",
    "<NOTES>",
    "[QUERY]",
];
/// Spanish headings and value names that must not appear in the English help.
const SPANISH: [&str; 6] = [
    "Uso:",
    "Opciones",
    "Comandos",
    "Argumentos",
    "OPCIONES",
    "NOMBRE",
];

#[test]
fn help_follows_lang() {
    let db = TempDb::new("help");
    let en = stdout(&db.run("en_US.UTF-8", &["--help"]));
    assert!(en.contains("Usage:"), "{en}");
    assert!(
        en.contains("Queues the URL; the app downloads and extracts it when it opens"),
        "{en}"
    );
    assert!(en.contains("Print help") && en.contains("Options:"), "{en}");
    let es = stdout(&db.run("es_ES.UTF-8", &["--help"]));
    assert!(es.contains("Uso:"), "{es}");
    assert!(
        es.contains("Encola la URL") && es.contains("Opciones:") && es.contains("Comandos:"),
        "{es}"
    );
    let add = stdout(&db.run("es_ES.UTF-8", &["add", "--help"]));
    assert!(
        add.contains("Argumentos:") && add.contains("Etiqueta (repetible)"),
        "{add}"
    );
    for args in std::iter::once(vec!["--help"]).chain(COMMANDS.map(|c| vec![c, "--help"])) {
        let es = stdout(&db.run("es_ES.UTF-8", &args));
        for word in ENGLISH {
            assert!(!es.contains(word), "{word} en {args:?}: {es}");
        }
        let en = stdout(&db.run("en_US.UTF-8", &args));
        for word in SPANISH {
            assert!(!en.contains(word), "{word} en {args:?}: {en}");
        }
    }
}

#[test]
fn stats_in_english() {
    let db = TempDb::new("stats");
    let out = db.run("en_US.UTF-8", &["stats"]);
    assert!(out.status.success(), "{out:?}");
    assert!(stdout(&out).contains("pending: 0"), "{out:?}");
}

#[test]
fn unknown_argument_is_translated() {
    let db = TempDb::new("unknown");
    let out = db.run("es_ES.UTF-8", &["--nope"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let err = stderr(&out);
    assert!(
        err.contains("argumento desconocido")
            && err.contains("--nope")
            && err.contains("Para más información, prueba '--help'."),
        "{err}"
    );
}

#[test]
fn clap_suggestion_is_translated() {
    let db = TempDb::new("suggest");
    let es = db.run("es_ES.UTF-8", &["lst"]);
    assert_eq!(es.status.code(), Some(2), "{es:?}");
    assert!(stderr(&es).contains("¿Querías decir «list»?"), "{es:?}");
    let en = db.run("en_US.UTF-8", &["lst"]);
    assert_eq!(en.status.code(), Some(2), "{en:?}");
    assert!(stderr(&en).contains("Did you mean 'list'?"), "{en:?}");
    let arg = db.run(
        "en_US.UTF-8",
        &["add", "https://example.com/x", "--titel", "a"],
    );
    assert_eq!(arg.status.code(), Some(2), "{arg:?}");
    assert!(stderr(&arg).contains("Did you mean '--title'?"), "{arg:?}");
    // With nothing similar, there is no suggestion.
    let none = db.run("en_US.UTF-8", &["zzzzzz"]);
    assert!(!stderr(&none).contains("Did you mean"), "{none:?}");
}

#[test]
fn missing_profile_is_translated() {
    let db = TempDb::new("profile");
    let out = db.run("en_US.UTF-8", &["--profile", "Nada", "list"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("profile not found"), "{out:?}");
}

#[test]
fn argument_errors_name_the_problem() {
    let db = TempDb::new("args");
    let url = "https://example.com/x";
    let repeated = db.run("en_US.UTF-8", &["add", url, "--title", "a", "--title", "b"]);
    assert!(
        stderr(&repeated).contains("cannot be used more than once"),
        "{repeated:?}"
    );
    let missing = db.run("en_US.UTF-8", &["add", url, "--tag"]);
    assert!(stderr(&missing).contains("missing value"), "{missing:?}");
}

#[test]
fn failed_export_html_is_translated() {
    let db = TempDb::new("export-write");
    let file = std::env::temp_dir().join(format!("marks-no-dir-{}/x.html", std::process::id()));
    let out = db.run("en_US.UTF-8", &["export-html", &file.to_string_lossy()]);
    assert!(!out.status.success(), "{out:?}");
    assert!(
        stderr(&out).contains("could not write the exported file: "),
        "{out:?}"
    );
}

#[test]
fn status_is_translated() {
    let db = TempDb::new("status");
    let row = stdout(&db.run("es_ES.UTF-8", &["add", "https://example.com/x"]));
    assert!(row.contains("[pendiente]"), "{row}");
}

// Assumes macOS: there `sys_locale` reads the system preferences and ignores `LANG`.
#[test]
fn cli_falls_back_to_the_system_locale() {
    let db = TempDb::new("system");
    let help = stdout(&db.run("C", &["--help"]));
    let system: Vec<String> = sys_locale::get_locales().collect();
    let (usage, about) = match resolve(None, &system) {
        Locale::Es => ("Uso:", "Bookmarks locales"),
        _ => ("Usage:", "Local bookmarks"),
    };
    assert!(help.contains(usage) && help.contains(about), "{help}");
}
