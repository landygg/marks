//! CLI texts: help, output and argument errors.

use crate::loader::{Localize, loader};
use i18n_embed_fl::fl;
use marks_core::store::Status;

/// clap's error kind, without depending on clap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClapErrorKind {
    UnknownArgument,
    InvalidSubcommand,
    MissingRequiredArgument,
    InvalidValue,
    /// An option without the value it needs.
    MissingValue,
    ValueValidation,
    TooManyValues,
    ArgumentConflict,
    /// A single-value option given twice.
    ArgumentRepeated,
    MissingSubcommand,
    InvalidUtf8,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliText {
    About,
    HelpDb {
        env: String,
        default: String,
    },
    HelpAdd,
    HelpTitle,
    HelpNotes,
    HelpTag,
    HelpProfile,
    HelpProfiles,
    HelpList,
    HelpSearch,
    HelpExport,
    HelpImport,
    HelpExportHtml,
    HelpStats,
    UsageRoot,
    UsageAdd,
    UsageProfiles,
    UsageList,
    UsageSearch,
    UsageExport,
    UsageImport,
    UsageExportHtml,
    UsageStats,
    ValueCommand,
    ValueDb,
    ValueProfile,
    ValueTitle,
    ValueNotes,
    ValueTag,
    ValueUrl,
    ValueQuery,
    ValueDir,
    ValueFile,
    Exported {
        n: u64,
        path: String,
    },
    ImportSummary {
        new: u64,
        existing: u64,
        skipped: u64,
    },
    RowError {
        error: String,
    },
    Status(Status),
    StatsTotal {
        n: u64,
    },
    StatsPending {
        n: u64,
    },
    StatsFailed {
        n: u64,
    },
    StatsUnsummarized {
        n: u64,
    },
    StatsImported {
        n: u64,
    },
    Usage,
    ArgumentsHeading,
    OptionsHeading,
    CommandsHeading,
    HelpFlag,
    VersionFlag,
    ClapError {
        kind: ClapErrorKind,
        arg: Option<String>,
    },
    ErrorHint,
    /// clap's "did you mean": the closest subcommand, option or value.
    DidYouMean {
        suggestion: String,
    },
}

impl Localize for CliText {
    fn localize(&self) -> String {
        let l = loader();
        match self {
            CliText::About => fl!(l, "cli-about"),
            CliText::HelpDb { env, default } => fl!(
                l,
                "cli-help-db",
                env = env.as_str(),
                default = default.as_str()
            ),
            CliText::HelpAdd => fl!(l, "cli-help-add"),
            CliText::HelpTitle => fl!(l, "cli-help-title"),
            CliText::HelpNotes => fl!(l, "cli-help-notes"),
            CliText::HelpTag => fl!(l, "cli-help-tag"),
            CliText::HelpProfile => fl!(l, "cli-help-profile"),
            CliText::HelpProfiles => fl!(l, "cli-help-profiles"),
            CliText::HelpList => fl!(l, "cli-help-list"),
            CliText::HelpSearch => fl!(l, "cli-help-search"),
            CliText::HelpExport => fl!(l, "cli-help-export"),
            CliText::HelpImport => fl!(l, "cli-help-import"),
            CliText::HelpExportHtml => fl!(l, "cli-help-export-html"),
            CliText::HelpStats => fl!(l, "cli-help-stats"),
            CliText::UsageRoot => fl!(l, "cli-usage-root"),
            CliText::UsageAdd => fl!(l, "cli-usage-add"),
            CliText::UsageProfiles => fl!(l, "cli-usage-profiles"),
            CliText::UsageList => fl!(l, "cli-usage-list"),
            CliText::UsageSearch => fl!(l, "cli-usage-search"),
            CliText::UsageExport => fl!(l, "cli-usage-export"),
            CliText::UsageImport => fl!(l, "cli-usage-import"),
            CliText::UsageExportHtml => fl!(l, "cli-usage-export-html"),
            CliText::UsageStats => fl!(l, "cli-usage-stats"),
            CliText::ValueCommand => fl!(l, "cli-value-command"),
            CliText::ValueDb => fl!(l, "cli-value-db"),
            CliText::ValueProfile => fl!(l, "cli-value-profile"),
            CliText::ValueTitle => fl!(l, "cli-value-title"),
            CliText::ValueNotes => fl!(l, "cli-value-notes"),
            CliText::ValueTag => fl!(l, "cli-value-tag"),
            CliText::ValueUrl => fl!(l, "cli-value-url"),
            CliText::ValueQuery => fl!(l, "cli-value-query"),
            CliText::ValueDir => fl!(l, "cli-value-dir"),
            CliText::ValueFile => fl!(l, "cli-value-file"),
            CliText::Exported { n, path } => {
                fl!(l, "cli-exported", n = n, path = path.as_str())
            }
            CliText::ImportSummary {
                new,
                existing,
                skipped,
            } => fl!(
                l,
                "cli-import-summary",
                new = new,
                existing = existing,
                skipped = skipped
            ),
            CliText::RowError { error } => fl!(l, "cli-row-error", error = error.as_str()),
            CliText::Status(status) => match status {
                Status::Pending => fl!(l, "cli-status-pending"),
                Status::PendingEmbed => fl!(l, "cli-status-pending-embed"),
                Status::Ready => fl!(l, "cli-status-ready"),
                Status::Failed => fl!(l, "cli-status-failed"),
                Status::Imported => fl!(l, "cli-status-imported"),
            },
            CliText::StatsTotal { n } => fl!(l, "cli-stats-total", n = n),
            CliText::StatsPending { n } => fl!(l, "cli-stats-pending", n = n),
            CliText::StatsFailed { n } => fl!(l, "cli-stats-failed", n = n),
            CliText::StatsUnsummarized { n } => fl!(l, "cli-stats-unsummarized", n = n),
            CliText::StatsImported { n } => fl!(l, "cli-stats-imported", n = n),
            CliText::Usage => fl!(l, "cli-usage"),
            CliText::ArgumentsHeading => fl!(l, "cli-arguments-heading"),
            CliText::OptionsHeading => fl!(l, "cli-options-heading"),
            CliText::CommandsHeading => fl!(l, "cli-commands-heading"),
            CliText::HelpFlag => fl!(l, "cli-help-flag"),
            CliText::VersionFlag => fl!(l, "cli-version-flag"),
            CliText::ErrorHint => fl!(l, "cli-error-hint"),
            CliText::DidYouMean { suggestion } => {
                fl!(
                    l,
                    "cli-error-did-you-mean",
                    suggestion = suggestion.as_str()
                )
            }
            CliText::ClapError { kind, arg } => {
                let arg = arg.as_deref().unwrap_or_default();
                match kind {
                    ClapErrorKind::UnknownArgument => {
                        fl!(l, "cli-error-unknown-argument", arg = arg)
                    }
                    ClapErrorKind::InvalidSubcommand => {
                        fl!(l, "cli-error-invalid-subcommand", arg = arg)
                    }
                    ClapErrorKind::MissingRequiredArgument => {
                        fl!(l, "cli-error-missing-required-argument", arg = arg)
                    }
                    ClapErrorKind::InvalidValue => fl!(l, "cli-error-invalid-value", arg = arg),
                    ClapErrorKind::MissingValue => fl!(l, "cli-error-missing-value", arg = arg),
                    ClapErrorKind::ValueValidation => {
                        fl!(l, "cli-error-value-validation", arg = arg)
                    }
                    ClapErrorKind::TooManyValues => {
                        fl!(l, "cli-error-too-many-values", arg = arg)
                    }
                    ClapErrorKind::ArgumentConflict => {
                        fl!(l, "cli-error-argument-conflict", arg = arg)
                    }
                    ClapErrorKind::ArgumentRepeated => {
                        fl!(l, "cli-error-argument-repeated", arg = arg)
                    }
                    ClapErrorKind::MissingSubcommand => {
                        fl!(l, "cli-error-missing-subcommand", arg = arg)
                    }
                    ClapErrorKind::InvalidUtf8 => fl!(l, "cli-error-invalid-utf8"),
                    ClapErrorKind::Other => fl!(l, "cli-error-generic"),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, localize_in};

    #[test]
    fn plurals_pick_one_only_for_one() {
        let summary = |new, existing, skipped| CliText::ImportSummary {
            new,
            existing,
            skipped,
        };
        assert_eq!(
            localize_in(Locale::Es, &summary(1, 0, 2)),
            "1 nuevo, 0 ya estaban, 2 no importables"
        );
        assert_eq!(
            localize_in(Locale::Es, &summary(0, 1, 1)),
            "0 nuevos, 1 ya estaba, 1 no importable"
        );
        let exported = |n| CliText::Exported {
            n,
            path: "/tmp".into(),
        };
        assert_eq!(
            localize_in(Locale::En, &exported(1)),
            "1 bookmark exported to /tmp"
        );
        assert_eq!(
            localize_in(Locale::En, &exported(4)),
            "4 bookmarks exported to /tmp"
        );
    }

    #[test]
    fn clap_errors_carry_the_argument() {
        let e = |kind| CliText::ClapError {
            kind,
            arg: Some("--nope".into()),
        };
        assert_eq!(
            localize_in(Locale::Es, &e(ClapErrorKind::UnknownArgument)),
            "argumento desconocido: --nope"
        );
        assert_eq!(
            localize_in(Locale::En, &e(ClapErrorKind::UnknownArgument)),
            "unknown argument: --nope"
        );
        assert!(!localize_in(Locale::En, &e(ClapErrorKind::Other)).contains("--nope"));
    }
}
