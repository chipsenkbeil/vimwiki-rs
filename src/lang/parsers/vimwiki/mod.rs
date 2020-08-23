use super::{
    components::{self, BlockComponent, InlineComponent, Page},
    LangParserError, Span, LC,
};
use nom::{
    branch::alt,
    combinator::{all_consuming, map},
    error::{ErrorKind, ParseError},
    multi::many0,
    IResult,
};

mod headers;

/// Parses str slice into a wiki page
pub fn parse_str(input: &str) -> Result<LC<Page>, LangParserError> {
    Ok(page::<(Span, ErrorKind)>(Span::new(input))
        .map_err(|x| match x {
            nom::Err::Error(x) => LangParserError::Error {
                remaining: x.0.fragment(),
                error_kind: x.1,
            },
            nom::Err::Failure(x) => LangParserError::Failure {
                remaining: x.0.fragment(),
                error_kind: x.1,
            },
            nom::Err::Incomplete(x) => match x {
                nom::Needed::Unknown => LangParserError::IncompleteUnknown,
                nom::Needed::Size(size) => LangParserError::Incomplete { size },
            },
        })?
        .1)
}

/// Parses entire vimwiki page
fn page<'a, E: ParseError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span<'a>, LC<Page>, E> {
    // Continuously parse input for new block components until we have
    // nothing left (or we fail)
    map(all_consuming(many0(block_component)), Page::new)(input)
}

/// Parses a block component
fn block_component<'a, E: ParseError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span<'a>, LC<BlockComponent>, E> {
    // TODO: Remove duplicate header and add all other block components
    alt((
        map(headers::header, From::from),
        map(headers::header, From::from),
    ))(input)
}

/// Parses an inline component
fn inline_component<'a, E: ParseError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span<'a>, LC<InlineComponent>, E> {
    // TODO: Add all inline component parsers
    panic!("TODO: Implement");
}
