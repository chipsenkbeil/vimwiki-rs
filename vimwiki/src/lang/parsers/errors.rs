use super::{utils, Span, VimwikiNomError};
use derive_more::{Constructor, Display, Error};

/// Represents an encapsulated error that is encountered
#[derive(Constructor, Clone, Debug, Eq, PartialEq, Display, Error)]
pub struct LangParserError {
    pub msg: String,
}

impl From<&str> for LangParserError {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
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
