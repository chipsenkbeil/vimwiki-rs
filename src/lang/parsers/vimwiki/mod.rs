use super::{
    components::{self, BlockComponent, InlineComponent, Page},
    utils::VimwikiIResult,
    LangParserError, Span, LC,
};
use nom::{
    branch::alt,
    combinator::{all_consuming, map},
    multi::many0,
};
use nom_locate::position;

mod headers;

/// Parses str slice into a wiki page
pub fn parse_str(text: &str) -> Result<LC<Page>, LangParserError> {
    let input = Span::new(text);
    Ok(page(input)
        .map_err(|x| LangParserError::from((input, x)))?
        .1)
}

/// Parses entire vimwiki page
fn page<'a>(input: Span<'a>) -> VimwikiIResult<Span<'a>, LC<Page>> {
    // Continuously parse input for new block components until we have
    // nothing left (or we fail)
    let (input, pos) = position(input)?;
    map(all_consuming(many0(block_component)), move |c| {
        LC::from((Page::new(c), pos))
    })(input)
}

/// Parses a block component
fn block_component<'a>(
    input: Span<'a>,
) -> VimwikiIResult<Span<'a>, LC<BlockComponent>> {
    // TODO: Remove duplicate header and add all other block components
    alt((
        map(headers::header, |c| c.map(BlockComponent::from)),
        map(headers::header, |c| c.map(BlockComponent::from)),
    ))(input)
}

/// Parses an inline component
fn inline_component<'a>(
    input: Span<'a>,
) -> VimwikiIResult<Span<'a>, LC<InlineComponent>> {
    // TODO: Add all inline component parsers
    panic!("TODO: Implement");
}
