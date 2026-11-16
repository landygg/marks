//! Splits Markdown into chunks for embeddings.

/// Max chunk size, in characters (not bytes).
pub const MAX_CHARS: usize = 1500;
/// Max chunks per page; the rest of the text is dropped.
pub const MAX_CHUNKS: usize = 40;
const SEPARATOR: &str = "\n\n";

/// Splits `markdown` into chunks of up to `MAX_CHARS` characters, breaking between paragraphs
/// when possible and at spaces when a paragraph is too long.
pub fn chunk(markdown: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let normalized = markdown.replace("\r\n", "\n");
    let pieces = normalized
        .split(SEPARATOR)
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .flat_map(split_long);
    for piece in pieces {
        if chunks.last().is_some_and(|last| fits(last, &piece)) {
            if let Some(last) = chunks.last_mut() {
                last.push_str(SEPARATOR);
                last.push_str(&piece);
            }
        } else if chunks.len() < MAX_CHUNKS {
            chunks.push(piece);
        } else {
            break;
        }
    }
    chunks
}

fn fits(chunk: &str, piece: &str) -> bool {
    chunk.chars().count() + SEPARATOR.len() + piece.chars().count() <= MAX_CHARS
}

/// Splits a paragraph at spaces; a word longer than `MAX_CHARS` is cut by characters as a last resort.
fn split_long(paragraph: &str) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut rest = paragraph;
    while let Some((limit, _)) = rest.char_indices().nth(MAX_CHARS) {
        let cut = rest[..limit].rfind(char::is_whitespace).unwrap_or(limit);
        pieces.push(rest[..cut].trim_end().to_string());
        rest = rest[cut..].trim_start();
    }
    if !rest.is_empty() {
        pieces.push(rest.to_string());
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    fn para(c: char, n: usize) -> String {
        c.to_string().repeat(n)
    }

    #[test]
    fn empty_gives_no_chunks() {
        assert!(chunk("").is_empty());
        assert!(chunk("  \n\n \n").is_empty());
    }

    #[test]
    fn short_text_is_one_chunk() {
        assert_eq!(chunk("hola\n\nmundo"), vec!["hola\n\nmundo"]);
    }

    #[test]
    fn splits_on_paragraphs_under_limit() {
        let text = [para('a', 700), para('b', 700), para('c', 700)].join("\n\n");
        let chunks = chunk(&text);
        assert_eq!(
            chunks,
            vec![
                format!("{}\n\n{}", para('a', 700), para('b', 700)),
                para('c', 700)
            ]
        );
    }

    #[test]
    fn long_paragraph_is_split_at_whitespace() {
        let text = vec!["palabra"; 600].join(" ");
        let chunks = chunk(&text);
        assert!(chunks.len() > 1);
        for c in &chunks {
            assert!(c.chars().count() <= MAX_CHARS);
            assert!(c.split(' ').all(|w| w == "palabra"));
        }
    }

    #[test]
    fn counts_chars_not_bytes() {
        let text = "ñ".repeat(MAX_CHARS);
        assert_eq!(chunk(&text), vec![text]);
    }

    #[test]
    fn caps_at_40_chunks() {
        let text = vec!["palabra"; 14_000].join(" ");
        assert!(text.chars().count() >= 100_000);
        assert_eq!(chunk(&text).len(), MAX_CHUNKS);
    }
}
