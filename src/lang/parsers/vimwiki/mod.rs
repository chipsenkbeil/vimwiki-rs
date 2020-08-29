use super::{
    components::{self, BlockComponent, Page},
    utils::{self, VimwikiIResult},
    LangParserError, Span, LC,
};
use nom::{
    branch::alt,
    combinator::{all_consuming, map},
    multi::many0,
};
use nom_locate::position;

mod headers;
mod inline;
mod paragraphs;

/// Parses str slice into a wiki page
pub fn parse_str(text: &str) -> Result<LC<Page>, LangParserError> {
    let input = Span::new(text);
    Ok(page(input)
        .map_err(|x| LangParserError::from((input, x)))?
        .1)
}

/// Parses entire vimwiki page
fn page(input: Span) -> VimwikiIResult<LC<Page>> {
    // Continuously parse input for new block components until we have
    // nothing left (or we fail)
    let (input, pos) = position(input)?;
    map(all_consuming(many0(block_component)), move |c| {
        LC::from((Page::new(c), pos))
    })(input)
}

//
// CHIP CHIP CHIP: To ensure parsing works okay, inline components including
// text are limited to a single line, even if text extends to the next line.
// It may or may not be a good idea to examine inline components once fully
// parsed to see if two text components exist next to one another and - if so -
// join their contents and regions together
//

/// Parses a block component
fn block_component(input: Span) -> VimwikiIResult<LC<BlockComponent>> {
    alt((
        map(headers::header, |c| c.map(BlockComponent::from)),
        map(paragraphs::paragraph, |c| c.map(BlockComponent::from)),
    ))(input)
}
