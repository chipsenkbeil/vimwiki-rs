#[cfg(feature = "html")]
mod html;

#[cfg(feature = "html")]
pub use html::*;

use derive_more::{Display, Error};
use uriparse::{PathError, RelativeReferenceError, URIReferenceError};

/// Represents the ability to convert some data into some other output form
pub trait Output {
    type Formatter;

    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult;
}

/// Friendly wrapper around a result that yields nothing but can have specific
/// errors related to output
pub type OutputResult = Result<(), OutputError>;

#[derive(Debug, Display, Error)]
pub enum OutputError {
    #[cfg(feature = "html")]
    SyntaxOrThemeNotLoaded {
        #[error(source)]
        source: syntect::LoadingError,
    },

    FailedToConstructUri {
        #[error(source)]
        source: URIReferenceError,
    },

    FailedToConstructRelativeReference {
        #[error(source)]
        source: RelativeReferenceError,
    },

    FailedToModifyUriPath {
        #[error(source)]
        source: PathError,
    },

    ThemeMissing(#[error(not(source))] String),

    MissingWikiAtIndex(#[error(not(source))] usize),

    MissingWikiWithName(#[error(not(source))] String),

    TemplateNotLoaded {
        #[error(source)]
        source: std::io::Error,
    },

    Fmt {
        #[error(source)]
        source: std::fmt::Error,
    },
}

impl From<URIReferenceError> for OutputError {
    fn from(source: URIReferenceError) -> Self {
        Self::FailedToConstructUri { source }
    }
}

impl From<RelativeReferenceError> for OutputError {
    fn from(source: RelativeReferenceError) -> Self {
        Self::FailedToConstructRelativeReference { source }
    }
}

impl From<PathError> for OutputError {
    fn from(source: PathError) -> Self {
        Self::FailedToModifyUriPath { source }
    }
}

impl From<std::fmt::Error> for OutputError {
    fn from(source: std::fmt::Error) -> Self {
        Self::Fmt { source }
    }
}

#[cfg(feature = "html")]
impl From<syntect::LoadingError> for OutputError {
    fn from(source: syntect::LoadingError) -> Self {
        Self::SyntaxOrThemeNotLoaded { source }
    }
}
