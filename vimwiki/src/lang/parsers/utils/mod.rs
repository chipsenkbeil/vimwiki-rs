use super::{LangParserError, Region, Span, LC};
use nom::Err;

mod parsers;
pub use parsers::*;

/// Alias to the type of error to use with vimwiki parsing using nom
pub type VimwikiNomError = LangParserError;

/// Alias to an IResult using VimwikiNomError
pub type VimwikiIResult<O> = Result<(Span, O), Err<VimwikiNomError>>;
