//! Guard: no user text in `.rs` files outside the Fluent catalogs.
//! Same rule as `apps/desktop/src/i18n.guard.test.ts`.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, Expr, Token};

/// Macros whose arguments are for the developer.
const DEV_MACROS: [&str; 11] = [
    "trace",
    "debug",
    "info",
    "warn",
    "error",
    "log",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "unreachable",
    "panic",
];
/// Macros whose first argument is a path or a variable.
const FIRST_ARG_MACROS: [&str; 2] = ["include_str", "env"];
/// Yields a single literal: checked whole, so split SQL counts as SQL.
const CONCAT: &str = "concat";
const SQL_WORDS: [&str; 9] = [
    "SELECT", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "PRAGMA", "WITH",
];
const MARKER: &str = "i18n-allow:";
/// Punctuation around a lone word (`Resultado:`, `¿Seguro?`).
const WRAPPING_PUNCTUATION: [char; 3] = ['¿', '¡', '…'];
const FORBIDDEN_REASON: &str = "temporal";

/// Strips `{…}` format placeholders.
fn strip_braces(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in s.chars() {
        match c {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

fn is_sql(s: &str) -> bool {
    let first = s.split_whitespace().next().unwrap_or("");
    SQL_WORDS.contains(&first)
}

/// Two runs of letters separated only by spaces.
fn has_two_words(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    (1..chars.len()).any(|i| {
        chars[i].is_whitespace() && chars[i - 1].is_alphabetic() && {
            let rest = chars[i..].iter().find(|c| !c.is_whitespace());
            rest.is_some_and(|c| c.is_alphabetic())
        }
    })
}

/// A word with an initial capital followed by lowercase (`Ajustes`, `Open`, `Resultado:`).
fn is_capitalized_word(s: &str) -> bool {
    let s = s.trim_matches(|c: char| c.is_ascii_punctuation() || WRAPPING_PUNCTUATION.contains(&c));
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_uppercase())
        && s.chars().count() > 1
        && chars.all(|c| c.is_lowercase())
}

fn is_user_text(s: &str) -> bool {
    let s = strip_braces(s);
    let s = s.trim();
    if is_sql(s) {
        return false;
    }
    s.chars().any(|c| !c.is_ascii() && c.is_alphabetic())
        || has_two_words(s)
        || is_capitalized_word(s)
}

/// The reason of an `i18n-allow:` marker on the line, if any (empty if missing).
fn allow_reason(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once(MARKER)?;
    let rest = rest.trim();
    let rest = rest
        .strip_suffix("-->")
        .or(rest.strip_suffix("*/"))
        .unwrap_or(rest);
    Some(rest.trim())
}

fn is_valid_reason(reason: &str) -> bool {
    !reason.is_empty() && reason != FORBIDDEN_REASON
}

fn has_attr(attrs: &[Attribute], name: &str, arg: Option<&str>) -> bool {
    attrs.iter().any(|a| {
        let last = a.path().segments.last().map(|s| s.ident.to_string());
        if last.as_deref() != Some(name) {
            return false;
        }
        // Only plain `cfg(test)`: `cfg(not(test))` is app code.
        match arg {
            None => true,
            Some(arg) => a
                .meta
                .require_list()
                .is_ok_and(|l| l.tokens.to_string() == arg),
        }
    })
}

fn is_test(attrs: &[Attribute]) -> bool {
    has_attr(attrs, "test", None) || has_attr(attrs, "cfg", Some("test"))
}

struct Scan<'a> {
    file: &'a Path,
    lines: Vec<&'a str>,
    /// Line of the `const`, `static` or `let` that wraps the literal.
    decl: Option<usize>,
    hits: Vec<String>,
    /// Lines whose marker let some literal through.
    used: BTreeSet<usize>,
}

impl Scan<'_> {
    fn allowed(&mut self, line: usize) -> bool {
        let marked = [Some(line), self.decl].into_iter().flatten().find(|l| {
            self.lines
                .get(l.wrapping_sub(1))
                .and_then(|text| allow_reason(text))
                .is_some_and(is_valid_reason)
        });
        marked.map(|l| self.used.insert(l)).is_some()
    }

    fn check(&mut self, value: &str, line: usize) {
        if is_user_text(value) && !self.allowed(line) {
            self.hits
                .push(format!("{}:{line}: {value:?}", self.file.display()));
        }
    }

    fn with_decl(&mut self, line: usize, f: impl FnOnce(&mut Self)) {
        let outer = self.decl.replace(line);
        f(self);
        self.decl = outer;
    }

    /// The literals of a `concat!` joined, with `{}` for everything else, on the first one's line.
    fn concat(&mut self, args: &Punctuated<Expr, Token![,]>) {
        let mut text = String::new();
        let mut line = None;
        for arg in args {
            match arg {
                Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => {
                    line.get_or_insert(s.span().start().line);
                    text.push_str(&s.value());
                }
                _ => text.push_str("{}"),
            }
        }
        if let Some(line) = line {
            self.check(&text, line);
        }
    }

    /// Loose literals from tokens that cannot be parsed as expressions.
    fn tokens(&mut self, tokens: proc_macro2::TokenStream) {
        let mut trees = tokens.into_iter().peekable();
        while let Some(tree) = trees.next() {
            match tree {
                // `#[doc = "…"]` and other attributes.
                proc_macro2::TokenTree::Punct(p) if p.as_char() == '#' => {
                    trees.next_if(|t| matches!(t, proc_macro2::TokenTree::Group(_)));
                }
                proc_macro2::TokenTree::Group(g) => self.tokens(g.stream()),
                proc_macro2::TokenTree::Literal(lit) => {
                    if let Ok(s) = syn::parse_str::<syn::LitStr>(&lit.to_string()) {
                        self.check(&s.value(), lit.span().start().line);
                    }
                }
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for Scan<'_> {
    fn visit_attribute(&mut self, a: &'ast Attribute) {
        if a.path().is_ident("doc") {
            return;
        }
        // syn does not descend into the tokens of `#[arg(help = "…")]` or `#[error("…")]`.
        match &a.meta {
            syn::Meta::List(l) => self.tokens(l.tokens.clone()),
            _ => visit::visit_attribute(self, a),
        }
    }

    fn visit_item(&mut self, item: &'ast syn::Item) {
        let attrs = match item {
            syn::Item::Fn(i) => &i.attrs,
            syn::Item::Mod(i) => &i.attrs,
            syn::Item::Impl(i) => &i.attrs,
            syn::Item::Const(i) => &i.attrs,
            syn::Item::Static(i) => &i.attrs,
            syn::Item::Struct(i) => &i.attrs,
            syn::Item::Enum(i) => &i.attrs,
            syn::Item::Use(i) => &i.attrs,
            syn::Item::Macro(i) => &i.attrs,
            syn::Item::Trait(i) => &i.attrs,
            _ => return visit::visit_item(self, item),
        };
        if !is_test(attrs) {
            visit::visit_item(self, item);
        }
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        if !is_test(&f.attrs) {
            visit::visit_impl_item_fn(self, f);
        }
    }

    fn visit_item_const(&mut self, i: &'ast syn::ItemConst) {
        self.with_decl(i.const_token.span.start().line, |s| {
            visit::visit_item_const(s, i)
        });
    }

    fn visit_impl_item_const(&mut self, i: &'ast syn::ImplItemConst) {
        self.with_decl(i.const_token.span.start().line, |s| {
            visit::visit_impl_item_const(s, i)
        });
    }

    fn visit_item_static(&mut self, i: &'ast syn::ItemStatic) {
        self.with_decl(i.static_token.span.start().line, |s| {
            visit::visit_item_static(s, i)
        });
    }

    fn visit_local(&mut self, l: &'ast syn::Local) {
        self.with_decl(l.let_token.span.start().line, |s| visit::visit_local(s, l));
    }

    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = m
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if DEV_MACROS.contains(&name.as_str()) {
            return;
        }
        let skip = usize::from(FIRST_ARG_MACROS.contains(&name.as_str()));
        match m.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated) {
            Ok(args) if name == CONCAT => self.concat(&args),
            Ok(args) => args.iter().skip(skip).for_each(|e| self.visit_expr(e)),
            Err(_) => self.tokens(m.tokens.clone()),
        }
    }

    fn visit_lit_str(&mut self, lit: &'ast syn::LitStr) {
        self.check(&lit.value(), lit.span().start().line);
    }

    fn visit_lit_byte_str(&mut self, lit: &'ast syn::LitByteStr) {
        self.check(
            &String::from_utf8_lossy(&lit.value()),
            lit.span().start().line,
        );
    }
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The `.rs` files in `crates/*/src` and `apps/desktop/src-tauri/src`.
fn sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for krate in std::fs::read_dir(root.join("crates"))
        .expect("crates")
        .flatten()
    {
        rs_files(&krate.path().join("src"), &mut files);
    }
    rs_files(&root.join("apps/desktop/src-tauri/src"), &mut files);
    files.sort();
    files
}

#[test]
fn rule() {
    for s in [
        "perfil no encontrado",
        "Ajustes",
        "Añadir URL…",
        "Open",
        "Resultado:",
        "¿Seguro?",
    ] {
        assert!(is_user_text(s), "{s}");
    }
    for s in [
        "library-changed",
        "pending_embed",
        "HTML",
        "main",
        "⌘N",
        "{n}",
        "SELECT id FROM t",
        "/v1/ping",
    ] {
        assert!(!is_user_text(s), "{s}");
    }
}

#[test]
fn marker_needs_a_reason() {
    assert_eq!(allow_reason("x // i18n-allow: formato"), Some("formato"));
    assert_eq!(
        allow_reason("<!-- i18n-allow: formato -->"),
        Some("formato")
    );
    assert!(!allow_reason("x // i18n-allow:").is_some_and(is_valid_reason));
    assert!(!allow_reason("x // i18n-allow: temporal").is_some_and(is_valid_reason));
    assert_eq!(allow_reason("x // nada"), None);
}

/// A file's findings (literals and markers without a reason or unused) and how many markers it has.
fn scan_text(file: &Path, text: &str) -> (Vec<String>, usize) {
    let lines: Vec<&str> = text.lines().collect();
    let ast = syn::parse_file(text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    let mut scan = Scan {
        file,
        lines,
        decl: None,
        hits: Vec::new(),
        used: BTreeSet::new(),
    };
    scan.visit_file(&ast);
    let mut markers = 0;
    for (i, line) in scan.lines.iter().enumerate() {
        let Some(reason) = allow_reason(line) else {
            continue;
        };
        markers += 1;
        let problem = if !is_valid_reason(reason) {
            "i18n-allow sin motivo válido"
        } else if !scan.used.contains(&(i + 1)) {
            "i18n-allow que no deja pasar nada"
        } else {
            continue;
        };
        scan.hits
            .push(format!("{}:{}: {problem}", file.display(), i + 1));
    }
    (scan.hits, markers)
}

#[test]
fn attribute_lists_and_cfg_are_scanned() {
    let file = Path::new("probe.rs");
    let source = r#"
#[derive(clap::Parser)]
#[command(about = "Guarda bookmarks")]
struct Cli {
    #[arg(help = "Ruta del fichero")]
    path: String,
}
#[cfg(not(test))]
const LIVE: &str = "Texto de la app";
#[cfg(test)]
const TEST: &str = "Texto de test";
/// Comentario de documentación.
const USED: &str = "permitido aquí"; // i18n-allow: formato
const UNUSED: &str = "id"; // i18n-allow: formato
"#;
    let (hits, markers) = scan_text(file, source);
    assert_eq!(markers, 2);
    let lines: Vec<&str> = hits
        .iter()
        .map(|h| h.split(": ").nth(1).unwrap_or(""))
        .collect();
    assert_eq!(
        lines,
        [
            "\"Guarda bookmarks\"",
            "\"Ruta del fichero\"",
            "\"Texto de la app\"",
            "i18n-allow que no deja pasar nada"
        ],
        "{hits:?}"
    );
}

#[test]
fn no_user_literals() {
    let mut hits = Vec::new();
    let mut markers = 0;
    for file in sources() {
        let text = std::fs::read_to_string(&file).expect("read");
        let (found, marked) = scan_text(&file, &text);
        hits.extend(found);
        markers += marked;
    }
    // Outside libtest's capture, so it shows up in `scripts/check.sh`.
    let _ = writeln!(std::io::stderr(), "i18n-allow: {markers}");
    assert!(
        hits.is_empty(),
        "texto de usuario fuera del catálogo:\n{}",
        hits.join("\n")
    );
}
