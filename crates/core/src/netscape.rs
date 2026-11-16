//! Netscape bookmark format (`bookmarks.html`): a tolerant reader and a writer. Does not touch the store.
//!
//! Facts from real files (2026-11-13, v0.7 spec):
//! - Chrome: header with a comment and an outer `<DL>`; the toolbar is the `<H3>` with
//!   `PERSONAL_TOOLBAR_FOLDER="true"`, named in Chrome's language; "Other bookmarks" sits at the
//!   top level; `ADD_DATE` in seconds and `ICON` on every link; no `TAGS`.
//! - Safari: inside `<HTML>` with no outer `<DL>`; the `Favourites` and `Bookmarks Menu` roots carry no
//!   attributes and sit at the same level as the user's folders; no `ADD_DATE`, `ICON` or `TAGS`;
//!   unescaped `&` in `HREF`, and `&amp;` and `&#x2013;` in titles.
//! - Safari Reading List: a separate file, with `<H3 id="com.apple.ReadingList">`.

use crate::extract::{MAX_NAME_CHARS, clean};
use std::rc::Rc;

/// Root folder of the marks export.
pub const MARKS_FOLDER: &str = "marks";

/// An `<A>` in the file, with the user's folder path (no browser roots or marks structure).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Trimmed, entities decoded.
    pub href: String,
    /// The `<A>` text, entities decoded.
    pub title: String,
    /// Seconds; `None` if missing or outside `0 < v < 100_000_000_000`.
    pub add_date: Option<i64>,
    pub path: Vec<String>,
    /// Inside the Safari Reading List.
    pub reading_list: bool,
    /// Comma-separated `TAGS` attribute, trimmed, empties dropped.
    pub tags: Vec<String>,
}

/// marks structure marker on a folder (`MARKS="root"` / `MARKS="untagged"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    Root,
    Untagged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Folder {
        name: String,
        marker: Option<Marker>,
        children: Vec<Node>,
    },
    Link {
        href: String,
        title: String,
        add_date: i64,
        tags: Vec<String>,
    },
}

/// Safari roots, which carry no attributes: they only count at the top level.
const SAFARI_ROOTS: [&str; 3] = ["Favorites", "Favourites", "Bookmarks Menu"]; // i18n-allow: file format
const TOOLBAR_ATTR: &str = "PERSONAL_TOOLBAR_FOLDER";
const TRUE_VALUE: &str = "true";
const ID_ATTR: &str = "ID";
const READING_LIST_ID: &str = "com.apple.ReadingList";
const MARKS_ATTR: &str = "MARKS";
const MARKS_ROOT: &str = "root";
const MARKS_UNTAGGED: &str = "untagged";
const HREF_ATTR: &str = "HREF";
const ADD_DATE_ATTR: &str = "ADD_DATE";
const TAGS_ATTR: &str = "TAGS";
const TAG_SEPARATOR: char = ',';
/// `ADD_DATE` is valid if `0 < v < MAX_ADD_DATE` (seconds; rules out milliseconds and microseconds).
const MAX_ADD_DATE: i64 = 100_000_000_000;
/// Folders that count in the path; deeper ones add nothing.
const MAX_DEPTH: usize = 64;
const COMMENT_START: &str = "<!--";
const COMMENT_END: &str = "-->";
const HEADER: &str = // i18n-allow: file format
    "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n\
<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n\
<TITLE>Bookmarks</TITLE>\n\
<H1>Bookmarks</H1>\n";
const LIST_OPEN: &str = "<DL><p>";
const LIST_CLOSE: &str = "</DL><p>";
const INDENT: &str = "    ";
/// Named entities that are decoded; any other `&…;` is left as is.
const NAMED_ENTITIES: [(&str, char); 5] = [
    ("amp", '&'),
    ("lt", '<'),
    ("gt", '>'),
    ("quot", '"'),
    ("apos", '\''),
];

type Attrs = Vec<(String, String)>;

/// A tag or a run of text. `name` is uppercase and `attrs` unparsed.
enum Token<'a> {
    Text(&'a str),
    Tag {
        name: String,
        close: bool,
        attrs: &'a str,
    },
}

struct Tokens<'a> {
    html: &'a str,
    pos: usize,
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        loop {
            let rest = &self.html[self.pos..];
            if rest.is_empty() {
                return None;
            }
            if rest.starts_with(COMMENT_START) {
                self.pos += rest
                    .find(COMMENT_END)
                    .map_or(rest.len(), |i| i + COMMENT_END.len());
                continue;
            }
            let bytes = rest.as_bytes();
            let close = bytes.get(1) == Some(&b'/');
            let name_start = if close { 2 } else { 1 };
            let is_tag = bytes[0] == b'<'
                && bytes
                    .get(name_start)
                    .is_some_and(|b| b.is_ascii_alphabetic());
            // `<!DOCTYPE …>` and `<?…>` are skipped whole.
            if bytes[0] == b'<' && matches!(bytes.get(1), Some(b'!' | b'?')) {
                self.pos += rest.find('>').map_or(rest.len(), |i| i + 1);
                continue;
            }
            if !is_tag {
                // Text up to the next `<`; a `<` that does not open a tag is text too.
                let end = bytes[1..]
                    .iter()
                    .position(|b| *b == b'<')
                    .map_or(rest.len(), |i| i + 1);
                self.pos += end;
                return Some(Token::Text(&rest[..end]));
            }
            let name_end = name_start
                + bytes[name_start..]
                    .iter()
                    .take_while(|b| b.is_ascii_alphanumeric())
                    .count();
            let Some(end) = tag_end(&rest[name_end..]) else {
                self.pos = self.html.len();
                return None;
            };
            self.pos += name_end + end + 1;
            return Some(Token::Tag {
                name: rest[name_start..name_end].to_ascii_uppercase(),
                close,
                attrs: &rest[name_end..name_end + end],
            });
        }
    }
}

/// Position of the `>` that closes the tag, ignoring those inside quoted values.
fn tag_end(s: &str) -> Option<usize> {
    let mut quote = None;
    let mut after_eq = false;
    for (i, b) in s.bytes().enumerate() {
        match quote {
            Some(q) if b == q => quote = None,
            Some(_) => continue,
            None if b == b'>' => return Some(i),
            None if after_eq && (b == b'"' || b == b'\'') => quote = Some(b),
            None => {}
        }
        if !b.is_ascii_whitespace() {
            after_eq = b == b'=';
        }
    }
    None
}

/// `NAME="v"`, `NAME='v'`, `NAME=v` and `NAME`, with the name uppercased and the value decoded.
fn parse_attrs(s: &str) -> Attrs {
    let b = s.as_bytes();
    let skip_ws = |mut i: usize| {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        i
    };
    let mut out = Vec::new();
    let mut i = 0;
    loop {
        while i < b.len() && (b[i].is_ascii_whitespace() || b[i] == b'/') {
            i += 1;
        }
        if i >= b.len() {
            return out;
        }
        let start = i;
        while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'=' {
            i += 1;
        }
        let name = s[start..i].to_ascii_uppercase();
        let mut value = "";
        let j = skip_ws(i);
        if b.get(j) == Some(&b'=') {
            let vs = skip_ws(j + 1);
            if let Some(&q) = b.get(vs).filter(|q| **q == b'"' || **q == b'\'') {
                let ve = s[vs + 1..].find(q as char).map_or(b.len(), |k| vs + 1 + k);
                value = &s[vs + 1..ve];
                i = (ve + 1).min(b.len());
            } else {
                i = vs;
                while i < b.len() && !b[i].is_ascii_whitespace() {
                    i += 1;
                }
                value = &s[vs..i];
            }
        }
        out.push((name, decode_entities(value)));
    }
}

fn attr<'a>(attrs: &'a Attrs, name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.as_str())
}

/// Decodes the named entities in `NAMED_ENTITIES` and numeric ones; any other `&` is left as is,
/// also without `;` (Safari writes unescaped `&` inside `HREF`).
fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i + 1..];
        let len = rest
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'#')
            .count();
        let decoded = (rest.as_bytes().get(len) == Some(&b';'))
            .then(|| entity(&rest[..len]))
            .flatten();
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &rest[len + 1..];
            }
            None => out.push('&'),
        }
    }
    out.push_str(rest);
    out
}

fn entity(name: &str) -> Option<char> {
    if let Some((_, c)) = NAMED_ENTITIES.iter().find(|(n, _)| *n == name) {
        return Some(*c);
    }
    let num = name.strip_prefix('#')?;
    let code = match num.strip_prefix(['x', 'X']) {
        Some(hex) => u32::from_str_radix(hex, 16).ok()?,
        None => num.parse().ok()?,
    };
    char::from_u32(code)
}

/// Contents of a `<DL>`: accumulated path, reading-list flag and the `H3` folders above (roots included).
#[derive(Clone, Default)]
struct Frame {
    /// Shared between frames with the same path, so it is not copied for every `<DL>`.
    path: Rc<Vec<String>>,
    reading_list: bool,
    level: usize,
    /// marks root (`MARKS="root"` or `marks`), to recognize the untagged folder right inside it.
    root: bool,
}

/// The frame of the `<DL>` that opens `folder` (name and attributes of the preceding `<H3>`) inside `top`.
fn classify(folder: Option<(String, Attrs)>, top: &Frame, untagged_folders: &[String]) -> Frame {
    let Some((name, attrs)) = folder else {
        return top.clone();
    };
    let mut frame = Frame {
        level: top.level + 1,
        root: false,
        ..top.clone()
    };
    let marker =
        |value: &str| attr(&attrs, MARKS_ATTR).is_some_and(|m| m.eq_ignore_ascii_case(value));
    let toolbar = attr(&attrs, TOOLBAR_ATTR).is_some_and(|v| v.eq_ignore_ascii_case(TRUE_VALUE));
    if toolbar || (top.level == 0 && SAFARI_ROOTS.contains(&name.as_str())) {
    } else if marker(MARKS_ROOT) || name == MARKS_FOLDER {
        frame.path = Rc::default();
        frame.root = true;
    } else if marker(MARKS_UNTAGGED)
        || name.is_empty()
        || (top.root && untagged_folders.contains(&name))
    {
    } else if attr(&attrs, ID_ATTR) == Some(READING_LIST_ID) {
        frame.reading_list = true;
    } else if top.path.len() < MAX_DEPTH {
        let mut path = (*top.path).clone();
        path.push(name);
        frame.path = Rc::new(path);
    }
    frame
}

fn entry(attrs: &Attrs, title: String, frame: &Frame) -> Entry {
    Entry {
        href: attr(attrs, HREF_ATTR)
            .unwrap_or_default()
            .trim()
            .to_string(),
        title,
        add_date: attr(attrs, ADD_DATE_ATTR)
            .and_then(|v| v.trim().parse().ok())
            .filter(|v| (1..MAX_ADD_DATE).contains(v)),
        path: (*frame.path).clone(),
        reading_list: frame.reading_list,
        tags: attr(attrs, TAGS_ATTR)
            .unwrap_or_default()
            .split(TAG_SEPARATOR)
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(String::from)
            .collect(),
    }
}

/// An open `<A>` or `<H3>`, with the text read so far.
struct Open {
    link: bool,
    attrs: Attrs,
    text: String,
}

/// `untagged_folders` holds the "untagged" folder name in every language: without the marks
/// marker, a folder with one of those names right inside the root does not count as a tag.
/// Tolerant: unclosed HTML, any letter case and valueless attributes are not errors.
pub fn parse(html: &str, untagged_folders: &[String]) -> Vec<Entry> {
    let mut stack = vec![Frame::default()];
    let mut folder: Option<(String, Attrs)> = None;
    let mut open: Option<Open> = None;
    let mut entries = Vec::new();
    // The text of an `<A>` or `<H3>` ends at the next tag, closed or not.
    let mut finish = |open: Option<Open>, stack: &[Frame], folder: &mut Option<(String, Attrs)>| {
        let Some(o) = open else { return };
        // One line and `PageMeta`'s name cap, for titles and folders.
        let text = clean(Some(decode_entities(&o.text)), MAX_NAME_CHARS, true).unwrap_or_default();
        if o.link {
            if let Some(top) = stack.last() {
                entries.push(entry(&o.attrs, text, top));
            }
        } else {
            *folder = Some((text, o.attrs));
        }
    };
    for token in (Tokens { html, pos: 0 }) {
        let (name, close, attrs) = match token {
            Token::Text(t) => {
                if let Some(o) = &mut open {
                    o.text.push_str(t);
                }
                continue;
            }
            Token::Tag { name, close, attrs } => (name, close, attrs),
        };
        finish(open.take(), &stack, &mut folder);
        match (name.as_str(), close) {
            ("A" | "H3", false) => {
                // An `<A>` after an `<H3>` with no `<DL>`: that folder was empty.
                folder = None;
                open = Some(Open {
                    link: name == "A",
                    attrs: parse_attrs(attrs),
                    text: String::new(),
                });
            }
            ("DL", false) => {
                let frame = stack
                    .last()
                    .map(|top| classify(folder.take(), top, untagged_folders));
                stack.extend(frame);
            }
            ("DL", true) if stack.len() > 1 => {
                stack.pop();
            }
            _ => {}
        }
    }
    finish(open, &stack, &mut folder);
    entries
}

/// Escapes `&`, `<`, `>` and `"` for text and attribute values.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            // Chrome reads the file line by line: a line break inside an `<A>` would lose it.
            '\n' | '\r' if out.ends_with(' ') => {}
            '\n' | '\r' => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

fn render_list(out: &mut String, nodes: &[Node], depth: usize) {
    let pad = INDENT.repeat(depth);
    out.push_str(&format!("{pad}{LIST_OPEN}\n"));
    for node in nodes {
        match node {
            Node::Folder {
                name,
                marker,
                children,
            } => {
                let mark = match marker {
                    Some(Marker::Root) => format!(" {MARKS_ATTR}=\"{MARKS_ROOT}\""),
                    Some(Marker::Untagged) => format!(" {MARKS_ATTR}=\"{MARKS_UNTAGGED}\""),
                    None => String::new(),
                };
                out.push_str(&format!(
                    "{pad}{INDENT}<DT><H3{mark}>{}</H3>\n",
                    escape(name)
                ));
                render_list(out, children, depth + 1);
            }
            Node::Link {
                href,
                title,
                add_date,
                tags,
            } => {
                let tags = if tags.is_empty() {
                    String::new()
                } else {
                    let joined = tags.join(&TAG_SEPARATOR.to_string());
                    format!(" {TAGS_ATTR}=\"{}\"", escape(&joined))
                };
                out.push_str(&format!(
                    "{pad}{INDENT}<DT><A {HREF_ATTR}=\"{}\" {ADD_DATE_ATTR}=\"{add_date}\"{tags}>{}</A>\n",
                    escape(href),
                    escape(title)
                ));
            }
        }
    }
    out.push_str(&format!("{pad}{LIST_CLOSE}\n"));
}

/// Whole file: DOCTYPE, UTF-8 META charset, "Bookmarks" TITLE and H1, and the nodes in a <DL><p>.
pub fn render(nodes: &[Node]) -> String {
    let mut out = String::from(HEADER);
    render_list(&mut out, nodes, 0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNTAGGED: &str = "Sin etiqueta";

    fn parse_t(html: &str) -> Vec<Entry> {
        parse(html, &[UNTAGGED.to_string()])
    }

    const CHROME: &str = include_str!("../fixtures/bookmarks-chrome.html");
    const SAFARI: &str = include_str!("../fixtures/bookmarks-safari.html");
    const READING_LIST: &str = include_str!("../fixtures/readinglist-safari.html");

    /// Title and `/`-joined path of each entry, for comparison at a glance.
    fn paths(entries: &[Entry]) -> Vec<(String, String)> {
        entries
            .iter()
            .map(|e| (e.title.clone(), e.path.join("/")))
            .collect()
    }

    fn expected(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(t, p)| (t.to_string(), p.to_string()))
            .collect()
    }

    fn by_title<'a>(entries: &'a [Entry], title: &str) -> &'a Entry {
        entries
            .iter()
            .find(|e| e.title == title)
            .unwrap_or_else(|| panic!("sin entrada {title:?}"))
    }

    #[test]
    fn parses_chrome_fixture() {
        let entries = parse_t(CHROME);
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().all(|e| e.path.is_empty()));
        let first = &entries[0];
        assert_eq!(first.href, "https://agent.example.dev/");
        assert_eq!(
            first.title,
            "Example Agent | A sample coding agent for any model"
        );
        assert_eq!(first.add_date, Some(1767225600));
    }

    #[test]
    fn parses_safari_fixture() {
        let entries = parse_t(SAFARI);
        assert_eq!(entries.len(), 66);
        assert_eq!(entries.iter().filter(|e| e.path.is_empty()).count(), 12);
        assert!(
            entries
                .iter()
                .all(|e| e.add_date.is_none() && !e.reading_list)
        );
        assert_eq!(
            by_title(&entries, "Google").href,
            "https://www.google.com/?client=safari&channel=mac_bm"
        );
        by_title(
            &entries,
            "ExampleServe – Fast, Open-Source Model & Media Serving",
        );
        by_title(&entries, "🏡 Home / Example WebUI");
        assert_eq!(by_title(&entries, "Example Tables").path, ["Dev"]);
        assert_eq!(
            by_title(&entries, "Example Components").path,
            ["Dev", "Rust"]
        );
        let course = entries
            .iter()
            .find(|e| e.title == "Example Academy" && e.href.contains("/learn/"))
            .expect("Example Academy de /learn/");
        assert_eq!(course.path, ["Courses", "GenAI", "Starter"]);
    }

    #[test]
    fn parses_safari_reading_list_fixture() {
        let entries = parse_t(READING_LIST);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.reading_list && e.path.is_empty()));
        by_title(
            &entries,
            "Sample resume builder with PDF & Word downloads | Example Resume",
        );
        let aws = by_title(&entries, "Example Cloud Sign-In");
        assert!(aws.href.contains("&client_id="));
        assert!(READING_LIST.contains(&format!("HREF=\"{}\"", aws.href)));
    }

    #[test]
    fn browser_roots_are_not_paths() {
        let html = r#"<DL><p>
<DT><H3 PERSONAL_TOOLBAR_FOLDER="true">Barra de marcadores</H3>
<DL><p><DT><A HREF="https://a.com/">a</A></DL><p>
<DT><H3>Favorites</H3><DL><p><DT><A HREF="https://b.com/">b</A></DL><p>
<DT><H3>Favourites</H3><DL><p><DT><A HREF="https://c.com/">c</A></DL><p>
<DT><H3>Bookmarks Menu</H3><DL><p><DT><A HREF="https://d.com/">d</A></DL><p>
<DT><H3>AI</H3><DL><p>
  <DT><H3>Favorites</H3><DL><p><DT><A HREF="https://e.com/">e</A></DL><p>
</DL><p>
</DL><p>"#;
        assert_eq!(
            paths(&parse_t(html)),
            expected(&[
                ("a", ""),
                ("b", ""),
                ("c", ""),
                ("d", ""),
                ("e", "AI/Favorites")
            ])
        );
    }

    #[test]
    fn tolerates_unclosed_tags_and_case() {
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<!-- <A HREF="https://comentario.com/">no</A> -->
<title>Bookmarks</title>
<dt><h3 add_date="1">Uno</h3>
<dl><p>
<dt><a href="https://a.com/" data-x="a>b">A
<hr>
<dt><h3 folded>Dos</h3>
<dl><p>
<dt><a href=https://b.com/ add_date='5'>B</a>
</dl><p>
<dt><a HREF="https://c.com/">C</a>
</dl>
<dt><a href="https://d.com/">D</a>
"#;
        let entries = parse_t(html);
        assert_eq!(
            paths(&entries),
            expected(&[("A", "Uno"), ("B", "Uno/Dos"), ("C", "Uno"), ("D", "")])
        );
        assert_eq!(entries[0].href, "https://a.com/");
        assert_eq!(entries[1].href, "https://b.com/");
        assert_eq!(entries[1].add_date, Some(5));
    }

    #[test]
    fn survives_truncated_files() {
        // A text starting with a multi-byte character used to break byte-based slicing.
        for html in [CHROME, SAFARI, READING_LIST] {
            for (i, _) in html.char_indices() {
                parse_t(&html[..i]);
            }
        }
    }

    #[test]
    fn decodes_entities() {
        let html = r#"<DT><A HREF="https://x.com/?q=&quot;a&quot;&amp;r=&#233;">T &amp; &quot;q&quot; &#39;s&#39; &lt;b&gt; caf&#233; &#x2013; fin</A>
<DT><A HREF="https://x.com/?a=1&b=2">x &amp y&nbsp;z</A>"#;
        let entries = parse_t(html);
        assert_eq!(entries[0].href, "https://x.com/?q=\"a\"&r=é");
        assert_eq!(entries[0].title, "T & \"q\" 's' <b> café – fin");
        assert_eq!(entries[1].href, "https://x.com/?a=1&b=2");
        assert_eq!(entries[1].title, "x &amp y&nbsp;z");
    }

    #[test]
    fn reads_tags_and_dates() {
        let html = r#"<DT><A HREF="https://a.com/" ADD_DATE="1700000000" TAGS=" a, b ,,">a</A>
<DT><A HREF="https://b.com/" ADD_DATE="1700000000000000">b</A>
<DT><A HREF="https://c.com/" ADD_DATE="x">c</A>"#;
        let entries = parse_t(html);
        assert_eq!(entries[0].tags, ["a", "b"]);
        assert_eq!(entries[0].add_date, Some(1700000000));
        assert!(entries[1].tags.is_empty());
        assert_eq!(entries[1].add_date, None);
        assert_eq!(entries[2].add_date, None);
    }

    #[test]
    fn marks_structure_is_not_a_path() {
        let html = r#"<DL><p>
<DT><H3>marks</H3><DL><p>
  <DT><H3>Dev</H3><DL><p>
    <DT><H3>rust</H3><DL><p><DT><A HREF="https://1.com/">1</A></DL><p>
    <DT><H3>Sin etiqueta</H3><DL><p><DT><A HREF="https://6.com/">6</A></DL><p>
  </DL><p>
  <DT><H3>Sin etiqueta</H3><DL><p><DT><A HREF="https://3.com/">3</A></DL><p>
</DL><p>
<DT><H3>Importado</H3><DL><p><DT><H3>marks</H3><DL><p>
  <DT><H3>Dev</H3><DL><p><DT><H3>rust</H3><DL><p><DT><A HREF="https://2.com/">2</A></DL><p></DL><p>
</DL><p></DL><p>
<DT><H3>Otra</H3><DL><p><DT><H3 MARKS="root">Exportado</H3><DL><p>
  <DT><H3>Dev</H3><DL><p><DT><A HREF="https://4.com/">4</A></DL><p>
  <DT><H3 MARKS="untagged">Nada</H3><DL><p><DT><A HREF="https://5.com/">5</A></DL><p>
</DL><p></DL><p>
</DL><p>"#;
        assert_eq!(
            paths(&parse_t(html)),
            expected(&[
                ("1", "Dev/rust"),
                ("6", "Dev/Sin etiqueta"),
                ("3", ""),
                ("2", "Dev/rust"),
                ("4", "Dev"),
                ("5", ""),
            ])
        );
    }

    #[test]
    fn render_escapes_and_round_trips() {
        let link = Node::Link {
            href: "https://e.com/?a=1&b=<2>".into(),
            title: "x <&>\" 🦀".into(),
            add_date: 1700000000,
            tags: vec!["rust".into(), "a & b".into()],
        };
        let untagged = Node::Link {
            href: "https://f.com/".into(),
            title: "f".into(),
            add_date: 1,
            tags: Vec::new(),
        };
        let tree = vec![Node::Folder {
            name: MARKS_FOLDER.into(),
            marker: Some(Marker::Root),
            children: vec![
                Node::Folder {
                    name: "A & \"B\"".into(),
                    marker: None,
                    children: vec![Node::Folder {
                        name: "rust".into(),
                        marker: None,
                        children: vec![link],
                    }],
                },
                Node::Folder {
                    name: UNTAGGED.into(),
                    marker: Some(Marker::Untagged),
                    children: vec![untagged],
                },
            ],
        }];
        let entries = parse_t(&render(&tree));
        assert_eq!(
            entries,
            vec![
                Entry {
                    href: "https://e.com/?a=1&b=<2>".into(),
                    title: "x <&>\" 🦀".into(),
                    add_date: Some(1700000000),
                    path: vec!["A & \"B\"".into(), "rust".into()],
                    reading_list: false,
                    tags: vec!["rust".into(), "a & b".into()],
                },
                Entry {
                    href: "https://f.com/".into(),
                    title: "f".into(),
                    add_date: Some(1),
                    path: Vec::new(),
                    reading_list: false,
                    tags: Vec::new(),
                },
            ]
        );
    }

    #[test]
    fn render_keeps_each_link_on_one_line() {
        let html = render(&[Node::Folder {
            name: "x\r\ny".into(),
            marker: None,
            children: vec![Node::Link {
                href: "https://a.com/".into(),
                title: "a\nb".into(),
                add_date: 1,
                tags: Vec::new(),
            }],
        }]);
        assert!(html.lines().any(|l| l.ends_with("<H3>x y</H3>")));
        assert!(html.lines().any(|l| l.ends_with(">a b</A>")));
    }

    #[test]
    fn titles_and_folders_are_one_capped_line() {
        let long = format!("{}\n{}", "x".repeat(200), "y".repeat(199));
        let html =
            format!("<DT><H3>a\n  b</H3><DL><p><DT><A HREF=\"https://a.com/\">{long}</A></DL><p>");
        let entries = parse_t(&html);
        assert_eq!(entries[0].title.chars().count(), MAX_NAME_CHARS);
        assert!(!entries[0].title.contains('\n'));
        assert_eq!(entries[0].path, ["a b"]);
    }

    #[test]
    fn empty_folder_names_keep_the_parent_path() {
        let html = r#"<DT><H3>A</H3><DL><p>
<DT><H3>  </H3><DL><p><DT><A HREF="https://a.com/">a</A></DL><p>
</DL><p>"#;
        assert_eq!(paths(&parse_t(html)), expected(&[("a", "A")]));
    }

    #[test]
    fn deep_nesting_is_capped() {
        let html = format!(
            "{}<DT><A HREF=\"https://a.com/\">a</A>",
            "<DT><H3>f</H3><DL><p>".repeat(5_000)
        );
        let entries = parse_t(&html);
        assert_eq!(entries[0].path.len(), MAX_DEPTH);
    }

    #[test]
    fn unbalanced_list_ends_are_ignored() {
        let html = r#"</DL></DL>
<DT><H3>A</H3><DL><p><DT><A HREF="https://a.com/">a</A></DL><p></DL><p></DL><p>
<DT><H3>B</H3><DL><p><DT><A HREF="https://b.com/">b</A></DL><p>
<DT><A HREF="https://c.com/">c</A>"#;
        assert_eq!(
            paths(&parse_t(html)),
            expected(&[("a", "A"), ("b", "B"), ("c", "")])
        );
    }

    #[test]
    fn render_has_header() {
        let html = render(&[]);
        assert!(html.starts_with("<!DOCTYPE NETSCAPE-Bookmark-file-1>"));
        assert!(
            html.contains(r#"<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">"#)
        );
    }
}
