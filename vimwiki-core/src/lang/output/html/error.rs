use super::LinkResolutionError;
use derive_more::{Display, Error};
use uriparse::{PathError, RelativeReferenceError, URIReferenceError};

pub type HtmlOutputResult = Result<(), HtmlOutputError>;

#[derive(Debug, Display, Error)]
pub enum HtmlOutputError {
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

    FailedToResolveLink {
        #[error(source)]
        source: LinkResolutionError,
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

impl From<URIReferenceError> for HtmlOutputError {
    fn from(source: URIReferenceError) -> Self {
        Self::FailedToConstructUri { source }
    }
}

impl From<RelativeReferenceError> for HtmlOutputError {
    fn from(source: RelativeReferenceError) -> Self {
        Self::FailedToConstructRelativeReference { source }
    }
}

impl From<PathError> for HtmlOutputError {
    fn from(source: PathError) -> Self {
        Self::FailedToModifyUriPath { source }
    }
}

impl From<std::fmt::Error> for HtmlOutputError {
    fn from(source: std::fmt::Error) -> Self {
        Self::Fmt { source }
    }
}

impl From<LinkResolutionError> for HtmlOutputError {
    fn from(source: LinkResolutionError) -> Self {
        Self::FailedToResolveLink { source }
    }
}

impl From<syntect::LoadingError> for HtmlOutputError {
    fn from(source: syntect::LoadingError) -> Self {
        Self::SyntaxOrThemeNotLoaded { source }
    }
}
