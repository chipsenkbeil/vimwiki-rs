#[cfg(feature = "html")]
mod html;

#[cfg(feature = "html")]
pub use html::*;

use derive_more::{Display, Error, From};

/// Represents the ability to convert some data into some other output form
pub trait Output {
    type Formatter;

    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult;
}

/// Friendly wrapper around a result that yields nothing but can have specific
/// errors related to output
pub type OutputResult = Result<(), OutputError>;

#[derive(Debug, Display, Error, From)]
pub enum OutputError {
    SyntaxOrThemeNotLoaded {
        #[error(source)]
        source: syntect::LoadingError,
    },

    ThemeMissing(#[error(not(source))] String),

    Fmt {
        #[error(source)]
        source: std::fmt::Error,
    },
}
