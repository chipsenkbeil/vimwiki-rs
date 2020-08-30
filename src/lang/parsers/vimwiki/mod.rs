use super::{
    components::{self, BlockComponent, InlineComponent, Page},
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
mod links;
mod math;
mod paragraphs;
mod tags;
mod typefaces;

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
        // List(List),
        // Table(Table),
        // PreformattedText(PreformattedText),
        // Math(MathBlock),
        // Blockquote(Blockquote),
        // Divider(Divider),
        map(tags::tag_sequence, |c| c.map(BlockComponent::from)),
        map(blank_line, |c| LC::new(BlockComponent::EmptyLine, c.region)),
    ))(input)
}

/// Parses an inline component, which can only exist on a single line
#[inline]
pub fn inline_component(input: Span) -> VimwikiIResult<LC<InlineComponent>> {
    // NOTE: Ordering matters here as the first match is used as the
    //       component. This means that we want to ensure that text,
    //       which can match any character, is the last of our components.
    alt((
        map(math::math_inline, |c| c.map(InlineComponent::from)),
        map(tags::tag_sequence, |c| c.map(InlineComponent::from)),
        map(links::link, |c| c.map(InlineComponent::from)),
        map(typefaces::decorated_text, |c| c.map(InlineComponent::from)),
        map(typefaces::keyword, |c| c.map(InlineComponent::from)),
        map(typefaces::text, |c| c.map(InlineComponent::from)),
    ))(input)
}

/// Parses a blank line
fn blank_line(input: Span) -> VimwikiIResult<LC<()>> {
    let (input, pos) = position(input)?;

    let (input, _) = utils::blank_line(input)?;

    Ok((input, LC::from(((), pos))))
}
