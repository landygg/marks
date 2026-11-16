use anyhow::Result;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, ArgAction, Command, CommandFactory, FromArgMatches, Parser, Subcommand};
use marks_core::msg::{Msg, MsgExt};
use marks_core::netscape;
use marks_core::profiles::{DEFAULT_PROFILE_ID, Library, ProfileSeed};
use marks_core::store::{Bookmark, NewBookmark, Status, lock};
use marks_core::transfer;
use marks_core::{DB_ENV, default_db_path};
use marks_i18n::{ClapErrorKind, CliText, DataText, Localize};
use std::path::PathBuf;
use std::process::ExitCode;

/// Environment variable that forces the language (`es`, `en` or `pseudo`), for development and tests.
const LOCALE_ENV: &str = "MARKS_LOCALE";
const HELP_ID: &str = "help";
const VERSION_ID: &str = "version";
const SEARCH_LIMIT: usize = 50;

#[derive(Parser)]
#[command(name = "marks", version)]
struct Cli {
    #[arg(long, env = DB_ENV, default_value_os_t = default_db_path())]
    db: PathBuf,
    #[arg(long, global = true)]
    profile: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Add {
        url: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long, default_value = "")]
        notes: String,
        #[arg(long = "tag", action = ArgAction::Append)]
        tags: Vec<String>,
    },
    Profiles,
    List,
    Search {
        query: Vec<String>,
    },
    Export {
        dir: PathBuf,
    },
    Import {
        file: PathBuf,
    },
    #[command(name = "export-html")]
    ExportHtml {
        file: PathBuf,
    },
    Stats,
}

/// The clap definition with the active language's texts: clap only accepts them at build time.
fn command() -> Command {
    let t = |text: CliText| text.localize();
    let template = format!(
        "{{before-help}}{{about-with-newline}}\n{} {{usage}}\n\n{{all-args}}{{after-help}}",
        t(CliText::Usage)
    );
    let (arguments, options) = (t(CliText::ArgumentsHeading), t(CliText::OptionsHeading));
    // Without our own headings, clap would print "Arguments" and "Options".
    let headed = |c: Command| {
        c.help_template(&template).mut_args(|a| {
            let heading = if a.is_positional() {
                &arguments
            } else {
                &options
            };
            a.help_heading(heading.clone())
        })
    };
    // Subcommand, its description, its usage (clap writes `[OPTIONS]` verbatim) and each value name.
    let subcommands = [
        (
            "add",
            CliText::HelpAdd,
            CliText::UsageAdd,
            vec![
                ("url", CliText::ValueUrl),
                ("title", CliText::ValueTitle),
                ("notes", CliText::ValueNotes),
                ("tags", CliText::ValueTag),
            ],
        ),
        (
            "profiles",
            CliText::HelpProfiles,
            CliText::UsageProfiles,
            vec![],
        ),
        ("list", CliText::HelpList, CliText::UsageList, vec![]),
        (
            "search",
            CliText::HelpSearch,
            CliText::UsageSearch,
            vec![("query", CliText::ValueQuery)],
        ),
        (
            "export",
            CliText::HelpExport,
            CliText::UsageExport,
            vec![("dir", CliText::ValueDir)],
        ),
        (
            "import",
            CliText::HelpImport,
            CliText::UsageImport,
            vec![("file", CliText::ValueFile)],
        ),
        (
            "export-html",
            CliText::HelpExportHtml,
            CliText::UsageExportHtml,
            vec![("file", CliText::ValueFile)],
        ),
        ("stats", CliText::HelpStats, CliText::UsageStats, vec![]),
    ];
    let db_help = CliText::HelpDb {
        env: DB_ENV.to_string(),
        default: default_db_path().display().to_string(),
    };
    let mut cmd = Cli::command()
        .about(t(CliText::About))
        .override_usage(t(CliText::UsageRoot))
        .subcommand_help_heading(t(CliText::CommandsHeading))
        .subcommand_value_name(t(CliText::ValueCommand))
        // No `marks help <command>`: help is `--help`, on every subcommand too.
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new(HELP_ID)
                .short('h')
                .long(HELP_ID)
                .global(true)
                .action(ArgAction::Help)
                .help(t(CliText::HelpFlag)),
        )
        .arg(
            Arg::new(VERSION_ID)
                .short('V')
                .long(VERSION_ID)
                .action(ArgAction::Version)
                .help(t(CliText::VersionFlag)),
        )
        // The variable and the default go in the text: clap would write them in English.
        .mut_arg("db", |a| {
            a.help(t(db_help))
                .value_name(t(CliText::ValueDb))
                .hide_env(true)
                .hide_default_value(true)
        })
        .mut_arg("profile", |a| {
            a.help(t(CliText::HelpProfile))
                .value_name(t(CliText::ValueProfile))
        })
        .mut_subcommand("add", |c| {
            c.mut_arg("title", |a| a.help(t(CliText::HelpTitle)))
                .mut_arg("notes", |a| {
                    a.help(t(CliText::HelpNotes)).hide_default_value(true)
                })
                .mut_arg("tags", |a| a.help(t(CliText::HelpTag)))
        });
    for (name, about, usage, values) in subcommands {
        cmd = cmd.mut_subcommand(name, |mut c| {
            for (id, value) in values {
                c = c.mut_arg(id, |a| a.value_name(t(value)));
            }
            headed(c.about(t(about)).override_usage(t(usage)))
        });
    }
    headed(cmd)
}

/// clap's error, translated by kind and with the argument at issue.
fn clap_error(e: &clap::Error) -> CliText {
    let context = |k| e.get(k).map(ToString::to_string);
    let kind = match e.kind() {
        ErrorKind::UnknownArgument => ClapErrorKind::UnknownArgument,
        ErrorKind::InvalidSubcommand => ClapErrorKind::InvalidSubcommand,
        ErrorKind::MissingRequiredArgument => ClapErrorKind::MissingRequiredArgument,
        // clap gives an empty value when the option arrives without one.
        ErrorKind::InvalidValue if context(ContextKind::InvalidValue).as_deref() == Some("") => {
            ClapErrorKind::MissingValue
        }
        ErrorKind::InvalidValue => ClapErrorKind::InvalidValue,
        ErrorKind::ValueValidation => ClapErrorKind::ValueValidation,
        ErrorKind::TooManyValues => ClapErrorKind::TooManyValues,
        // clap reports the argument itself as the prior one when it is repeated.
        ErrorKind::ArgumentConflict
            if context(ContextKind::PriorArg) == context(ContextKind::InvalidArg) =>
        {
            ClapErrorKind::ArgumentRepeated
        }
        ErrorKind::ArgumentConflict => ClapErrorKind::ArgumentConflict,
        ErrorKind::MissingSubcommand => ClapErrorKind::MissingSubcommand,
        ErrorKind::InvalidUtf8 => ClapErrorKind::InvalidUtf8,
        _ => ClapErrorKind::Other,
    };
    let arg = [
        ContextKind::InvalidArg,
        ContextKind::InvalidSubcommand,
        ContextKind::InvalidValue,
    ]
    .into_iter()
    .find_map(context);
    CliText::ClapError { kind, arg }
}

/// clap's suggestion (subcommand, option or value), the closest if there are several.
fn suggestion(e: &clap::Error) -> Option<String> {
    [
        ContextKind::SuggestedSubcommand,
        ContextKind::SuggestedArg,
        ContextKind::SuggestedValue,
    ]
    .into_iter()
    .find_map(|k| match e.get(k)? {
        ContextValue::String(s) => Some(s.clone()),
        // clap sorts them from least to most similar.
        ContextValue::Strings(v) => v.last().cloned(),
        _ => None,
    })
}

fn print_row(b: &Bookmark) {
    let failure = match (&b.status, &b.error) {
        (Status::Failed, Some(e)) => format!(
            "  {}",
            CliText::RowError {
                error: marks_i18n::bookmark_error(e)
            }
            .localize()
        ),
        _ => String::new(),
    };
    println!(
        "#{}  [{}]  {}  {}{failure}",
        b.id,
        CliText::Status(b.status).localize(),
        b.title,
        b.url
    );
}

fn main() -> ExitCode {
    let preferred: Vec<String> = marks_i18n::posix_preferred(|k| std::env::var(k).ok())
        .into_iter()
        .chain(sys_locale::get_locales())
        .collect();
    marks_i18n::init(marks_i18n::resolve(
        std::env::var(LOCALE_ENV).ok().as_deref(),
        &preferred,
    ));
    let parsed = command()
        .try_get_matches()
        .and_then(|m| Cli::from_arg_matches(&m));
    let cli = match parsed {
        Ok(cli) => cli,
        Err(e) => {
            match e.kind() {
                ErrorKind::DisplayHelp
                | ErrorKind::DisplayVersion
                | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                    let _ = e.print();
                }
                _ => {
                    eprintln!("{}", clap_error(&e).localize());
                    if let Some(suggestion) = suggestion(&e) {
                        eprintln!("{}", CliText::DidYouMean { suggestion }.localize());
                    }
                    eprintln!("{}", CliText::ErrorHint.localize());
                }
            }
            return u8::try_from(e.exit_code()).map_or(ExitCode::FAILURE, ExitCode::from);
        }
    };
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", marks_i18n::error(&e));
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let library = Library::open(
        &cli.db,
        &ProfileSeed {
            name: &DataText::DefaultProfileName.localize(),
            initials: &DataText::DefaultProfileInitials.localize(),
        },
    )?;
    if let Cmd::Profiles = cli.cmd {
        for p in library.profiles()? {
            println!("#{}  [{}]  {}", p.id, p.initials, p.name);
        }
        return Ok(());
    }
    let id = match &cli.profile {
        Some(name) => library.find(name)?.msg(Msg::ProfileNotFound)?.id,
        None => DEFAULT_PROFILE_ID,
    };
    // Each branch takes the lock only for its own call.
    let store = library.store(id)?;
    match cli.cmd {
        // Handled above: listing profiles needs no profile's store.
        Cmd::Profiles => {}
        Cmd::Add {
            url,
            title,
            notes,
            tags,
        } => {
            let b = lock(&store)?.add(NewBookmark {
                url,
                title,
                notes,
                tags,
                ..Default::default()
            })?;
            print_row(&b);
        }
        Cmd::List => {
            lock(&store)?.list()?.iter().for_each(print_row);
        }
        Cmd::Search { query } => {
            lock(&store)?
                .search(&query.join(" "), SEARCH_LIMIT)?
                .iter()
                .for_each(print_row);
        }
        Cmd::Export { dir } => {
            let n = lock(&store)?.export_markdown(&dir, &DataText::NotesHeading.localize())?;
            let path = dir.display().to_string();
            println!("{}", CliText::Exported { n: n as u64, path }.localize());
        }
        Cmd::Import { file } => {
            let entries = netscape::parse(
                &transfer::read_file(&file)?,
                &DataText::UntaggedFolder.all_locales(),
            );
            let s = {
                let store = lock(&store)?;
                let tag = DataText::ReadingListTag.localize();
                let preview = transfer::preview(&store, entries, &tag)?;
                transfer::apply(&store, &preview)?
            };
            let summary = CliText::ImportSummary {
                new: s.new as u64,
                existing: s.existing as u64,
                skipped: (s.unsupported + s.invalid) as u64,
            };
            println!("{}", summary.localize());
        }
        Cmd::ExportHtml { file } => {
            let (html, n) = {
                let store = lock(&store)?;
                let untagged = DataText::UntaggedFolder.localize();
                (
                    transfer::export_html(&store, &untagged)?,
                    store.stats()?.total,
                )
            };
            std::fs::write(&file, html).msg(Msg::ExportWrite)?;
            let path = file.display().to_string();
            println!("{}", CliText::Exported { n: n as u64, path }.localize());
        }
        Cmd::Stats => {
            let s = lock(&store)?.stats()?;
            for line in [
                CliText::StatsTotal { n: s.total as u64 },
                CliText::StatsPending {
                    n: s.pending as u64,
                },
                CliText::StatsFailed { n: s.failed as u64 },
                CliText::StatsUnsummarized {
                    n: s.unsummarized as u64,
                },
                CliText::StatsImported {
                    n: s.imported as u64,
                },
            ] {
                println!("{}", line.localize());
            }
        }
    }
    Ok(())
}
