use std::path::PathBuf;
use std::process::{Command, Output};

const URL: &str = "https://example.com/x";

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
            // These tests compare the English output.
            .env("LANG", "en_US.UTF-8")
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

#[test]
fn add_only_enqueues() {
    let db = TempDb::new("add");
    let out = db.run(&["add", URL]);
    assert!(out.status.success(), "{out:?}");
    let row = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(row.contains("[pending]") && row.contains(URL), "{row}");
    // Without a download there is no page title or content: the title is still the URL.
    assert!(row.contains(&format!("  {URL}  {URL}")), "{row}");
}

#[test]
fn no_fetch_is_rejected() {
    let db = TempDb::new("nofetch");
    let out = db.run(&["add", URL, "--no-fetch"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--no-fetch"));
}
