use super::{Position, Region, Span, LC};
use nom::{
    error::{VerboseError, VerboseErrorKind},
    Err,
};

mod parsers;
pub use parsers::*;

mod ports;
pub use ports::*;

/// Alias to the type of error to use with vimwiki parsing using nom
pub type VimwikiNomError<'a> = VerboseError<Span<'a>>;

/// Constructs a new nom error using the given input when the error was
/// encountered and a custom context string to describe the error
pub fn new_nom_error<'a>(
    input: Span<'a>,
    context: &'static str,
) -> VimwikiNomError<'a> {
    VimwikiNomError {
        errors: vec![(input, VerboseErrorKind::Context(context))],
    }
}

/// Alias to an IResult using VimwikiNomError
pub type VimwikiIResult<'a, O> =
    Result<(Span<'a>, O), Err<VimwikiNomError<'a>>>;
