// Import to make more easily accessible to submodules
use super::{
    components::{self, Page},
    utils::{Position, Region, Span, LC},
};
use derive_more::{Constructor, Display, Error};
use std::path::Path;

mod utils;
use utils::VimwikiNomError;

pub mod vimwiki;

pub trait Parser {
    /// Attempts to parse text as a page
    fn parse_str(text: &str) -> Result<LC<Page>, LangParserError>;

    /// Attempts to read and parse the contents of a file as a page
    fn parse_file(path: impl AsRef<Path>) -> Result<LC<Page>, LangParserError> {
        match std::fs::read_to_string(path) {
            Ok(contents) => Self::parse_str(&contents),
            Err(x) => Err(LangParserError::new(format!("{:?}", x))),
        }
    }
}

/// Represents an encapsulated error that is encountered
#[derive(Constructor, Clone, Debug, Eq, PartialEq, Display, Error)]
pub struct LangParserError {
    pub msg: String,
}

impl<'a> From<VimwikiNomError<'a>> for LangParserError {
    fn from(e: VimwikiNomError<'a>) -> Self {
        Self::new(format!("{:?}", e))
    }
}

impl<'a> From<(Span<'a>, VimwikiNomError<'a>)> for LangParserError {
    fn from(x: (Span<'a>, VimwikiNomError<'a>)) -> Self {
        Self::new(utils::convert_error(x.0, x.1))
    }
}

impl<'a> From<nom::Err<VimwikiNomError<'a>>> for LangParserError {
    fn from(e: nom::Err<VimwikiNomError<'a>>) -> Self {
        match e {
            nom::Err::Error(x) | nom::Err::Failure(x) => Self::from(x),
            nom::Err::Incomplete(x) => Self::new(format!("{:?}", x)),
        }
    }
}
impl<'a> From<(Span<'a>, nom::Err<VimwikiNomError<'a>>)> for LangParserError {
    fn from(x: (Span<'a>, nom::Err<VimwikiNomError<'a>>)) -> Self {
        match x.1 {
            nom::Err::Error(e) | nom::Err::Failure(e) => Self::from((x.0, e)),
            nom::Err::Incomplete(x) => Self::new(format!("{:?}", x)),
        }
    }
}
