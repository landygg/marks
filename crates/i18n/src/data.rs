//! Names that core writes into data (export, import, notes and the first profile).

use crate::Locale;
use crate::loader::{Localize, loader, localize_in};
use i18n_embed_fl::fl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataText {
    UntaggedFolder,
    ReadingListTag,
    NotesHeading,
    DefaultProfileName,
    DefaultProfileInitials,
}

impl Localize for DataText {
    fn localize(&self) -> String {
        let l = loader();
        match self {
            DataText::UntaggedFolder => fl!(l, "data-untagged-folder"),
            DataText::ReadingListTag => fl!(l, "data-reading-list-tag"),
            DataText::NotesHeading => fl!(l, "data-notes-heading"),
            DataText::DefaultProfileName => fl!(l, "data-default-profile-name"),
            DataText::DefaultProfileInitials => fl!(l, "data-default-profile-initials"),
        }
    }
}

impl DataText {
    /// The text in each language, in `[En, Es]` order, without duplicates.
    pub fn all_locales(self) -> Vec<String> {
        let mut all: Vec<String> = Vec::new();
        for locale in [Locale::En, Locale::Es] {
            let text = localize_in(locale, &self);
            if !all.contains(&text) {
                all.push(text);
            }
        }
        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_locales_lists_en_then_es_without_repeats() {
        assert_eq!(
            DataText::UntaggedFolder.all_locales(),
            ["Untagged", "Sin etiqueta"]
        );
        assert_eq!(
            DataText::NotesHeading.all_locales(),
            ["## Notes", "## Notas"]
        );
        assert_eq!(
            localize_in(Locale::Es, &DataText::ReadingListTag),
            "lista de lectura"
        );
        assert_eq!(
            localize_in(Locale::En, &DataText::ReadingListTag),
            "reading list"
        );
    }
}
