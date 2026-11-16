//! Language resolution: pure, no I/O.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Es,
    En,
    Pseudo,
}

impl Locale {
    /// Tag of the catalog it loads: the pseudo-locale starts from English.
    pub fn tag(self) -> &'static str {
        match self {
            Locale::Es => "es",
            Locale::En | Locale::Pseudo => "en",
        }
    }
}

/// Language (no region) of a tag such as `es-MX` or `en_GB`.
fn language(tag: &str) -> &str {
    tag.split(['-', '_', '.', '@']).next().unwrap_or("")
}

/// 1. `override_` (`es`, `en`, `pseudo`); 2. the first supported tag in `preferred`; 3. `En`.
pub fn resolve(override_: Option<&str>, preferred: &[String]) -> Locale {
    let parse = |s: &str| match language(s).to_ascii_lowercase().as_str() {
        "es" => Some(Locale::Es),
        "en" => Some(Locale::En),
        _ => None,
    };
    if let Some(o) = override_ {
        if o.eq_ignore_ascii_case("pseudo") {
            return Locale::Pseudo;
        }
        if let Some(l) = parse(o) {
            return l;
        }
    }
    preferred
        .iter()
        .find_map(|p| parse(p))
        .unwrap_or(Locale::En)
}

/// Languages from `LC_ALL`, `LC_MESSAGES` and `LANG`, in that order, skipping empty, `C` and `POSIX`.
pub fn posix_preferred(get: impl Fn(&str) -> Option<String>) -> Vec<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|k| get(k))
        .filter(|v| !v.is_empty() && v != "C" && v != "POSIX")
        .map(|v| language(&v).to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_wins() {
        assert_eq!(resolve(Some("pseudo"), &["es".into()]), Locale::Pseudo);
    }
    #[test]
    fn region_is_ignored() {
        assert_eq!(resolve(None, &["es-MX".into()]), Locale::Es);
        assert_eq!(resolve(None, &["en-GB".into()]), Locale::En);
    }
    #[test]
    fn first_supported_wins() {
        assert_eq!(resolve(None, &["fr-FR".into(), "es-ES".into()]), Locale::Es);
    }
    #[test]
    fn unsupported_falls_back_to_english() {
        assert_eq!(resolve(None, &["fr-FR".into()]), Locale::En);
        assert_eq!(resolve(None, &[]), Locale::En);
    }
    #[test]
    fn posix_c_is_skipped() {
        let env = |k: &str| match k {
            "LC_ALL" => Some("C".into()),
            "LANG" => Some("es_ES.UTF-8".into()),
            _ => None,
        };
        assert_eq!(posix_preferred(env), vec!["es".to_string()]);
    }
    #[test]
    fn posix_strips_modifier_and_skips_empty_and_posix() {
        let env = |k: &str| match k {
            "LC_ALL" => Some("".into()),
            "LC_MESSAGES" => Some("POSIX".into()),
            "LANG" => Some("es_ES.UTF-8@euro".into()),
            _ => None,
        };
        assert_eq!(posix_preferred(env), vec!["es".to_string()]);
    }
    #[test]
    fn unknown_override_falls_through() {
        assert_eq!(resolve(Some("fr"), &["es-ES".into()]), Locale::Es);
    }
}
