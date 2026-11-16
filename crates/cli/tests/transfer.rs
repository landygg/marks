use std::path::PathBuf;
use std::process::{Command, Output};

/// Temporary database with a unique name; deletes the database and its profiles folder on drop.
struct TempDb(PathBuf);

impl TempDb {
    fn new(tag: &str) -> Self {
        let name = format!("marks-cli-{tag}-{}.db", std::process::id());
        Self(std::env::temp_dir().join(name))
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_marks"))
            .env("MARKS_DB", &self.0)
            // These tests compare the Spanish texts.
            .env("LANG", "es_ES.UTF-8")
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

fn text(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn import_then_export_html() {
    let db = TempDb::new("transfer");
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../core/fixtures/bookmarks-chrome.html"
    );
    let out = db.run(&["import", fixture]);
    assert!(out.status.success(), "{out:?}");
    let summary = text(&out);
    assert!(summary.contains("nuevos"), "{summary}");
    let new = summary
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    assert!(new.parse::<usize>().is_ok_and(|n| n > 0), "{summary}");

    let stats = text(&db.run(&["stats"]));
    assert!(stats.contains(&format!("sin extraer: {new}")), "{stats}");
    assert!(!text(&db.run(&["list"])).contains("[pendiente]"));

    let dir = std::env::temp_dir().join(format!("marks-cli-transfer-out-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("crear carpeta temporal");
    let file = dir.join("out.html");
    let out = db.run(&["export-html", &file.to_string_lossy()]);
    assert!(out.status.success(), "{out:?}");
    let html = std::fs::read_to_string(&file).expect("leer la exportación");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        html.starts_with("<!DOCTYPE NETSCAPE-Bookmark-file-1>"),
        "{html}"
    );
}

#[test]
fn counts_use_singular_for_one_and_plural_otherwise() {
    let db = TempDb::new("singular");
    let dir = std::env::temp_dir().join(format!("marks-cli-singular-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("crear carpeta temporal");
    let one = dir.join("one.html");
    std::fs::write(
        &one,
        "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n<DL><p>\n<DT><A HREF=\"https://example.com/a\">A</A>\n</DL><p>\n",
    )
    .expect("escribir fixture");
    let one = one.to_string_lossy().into_owned();

    let first = text(&db.run(&["import", &one]));
    assert!(
        first.contains("1 nuevo, 0 ya estaban, 0 no importables"),
        "{first}"
    );
    let again = text(&db.run(&["import", &one]));
    assert!(
        again.contains("0 nuevos, 1 ya estaba, 0 no importables"),
        "{again}"
    );

    let out = dir.join("out.html");
    let exported = text(&db.run(&["export-html", &out.to_string_lossy()]));
    assert!(
        exported.starts_with("1 bookmark exportado en "),
        "{exported}"
    );

    let chrome = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../core/fixtures/bookmarks-chrome.html"
    );
    db.run(&["import", chrome]);
    let exported = text(&db.run(&["export-html", &out.to_string_lossy()]));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        exported.starts_with("4 bookmarks exportados en "),
        "{exported}"
    );
}
