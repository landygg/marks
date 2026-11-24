//! marks user texts: language resolution and the Fluent loader.

mod app;
mod cli;
mod data;
mod loader;
mod locale;
mod msg;

pub use app::{AppText, MenuText};
pub use cli::{ClapErrorKind, CliText};
pub use data::DataText;
pub use loader::{Localize, active, chain, init, localize_in};
pub use locale::{Locale, posix_preferred, resolve};
pub use msg::{bookmark_error, error, msg, parts_text};
