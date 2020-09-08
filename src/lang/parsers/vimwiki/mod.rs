use super::{
    components::{
        self, BlockComponent, InlineComponent, InlineComponentContainer, Page,
    },
    utils::{self, position, VimwikiIResult},
    LangParserError, Parser, Span, LC,
};
use nom::{
    branch::alt,
    combinator::{all_consuming, map},
    error::context,
    multi::{many0, many1},
};

mod blockquotes;
mod divider;
mod headers;
mod links;
mod lists;
mod math;
mod paragraphs;
mod preformatted;
mod tables;
mod tags;
mod typefaces;

/// Represents a parser for vimwiki files
pub struct VimwikiParser;

impl Parser for VimwikiParser {
    fn parse_str(text: &str) -> Result<LC<Page>, LangParserError> {
        let input = Span::new(text);
        Ok(page(input)
            .map_err(|x| LangParserError::from((input, x)))?
            .1)
    }
}

/// Parses entire vimwiki page
fn page(input: Span) -> VimwikiIResult<LC<Page>> {
    // Continuously parse input for new block components until we have
    // nothing left (or we fail)
    let (input, pos) = position(input)?;
    let (input, c) = all_consuming(many0(block_component))(input)?;

    Ok((input, LC::from((Page::new(c), pos, input))))
}

/// Parses a block component
fn block_component(input: Span) -> VimwikiIResult<LC<BlockComponent>> {
    alt((
        map(headers::header, |c| c.map(BlockComponent::from)),
        map(paragraphs::paragraph, |c| c.map(BlockComponent::from)),
        map(lists::list, |c| c.map(BlockComponent::from)),
        map(tables::table, |c| c.map(BlockComponent::from)),
        map(preformatted::preformatted_text, |c| {
            c.map(BlockComponent::from)
        }),
        map(math::math_block, |c| c.map(BlockComponent::from)),
        map(blockquotes::blockquote, |c| c.map(BlockComponent::from)),
        map(divider::divider, |c| c.map(BlockComponent::from)),
        map(tags::tags, |c| c.map(BlockComponent::from)),
        // NOTE: Parses a single line to end, failing if contains non-whitespace
        map(blank_line, |c| LC::new(BlockComponent::EmptyLine, c.region)),
        // NOTE: Parses a single line to end; final type because will match
        //       anychar and consume the line
        map(non_blank_line, |c| {
            LC::new(BlockComponent::from(c.component), c.region)
        }),
    ))(input)
}

/// Parses one or more inline components and wraps it in a container; note
/// that this does NOT consume a line termination
#[inline]
pub fn inline_component_container(
    input: Span,
) -> VimwikiIResult<LC<InlineComponentContainer>> {
    let (input, pos) = position(input)?;

    let (input, container) = context(
        "Inline Component Container",
        map(many1(inline_component), InlineComponentContainer::from),
    )(input)?;

    Ok((input, LC::from((container, pos, input))))
}

/// Parses an inline component, which can only exist on a single line
#[inline]
pub fn inline_component(input: Span) -> VimwikiIResult<LC<InlineComponent>> {
    // NOTE: Ordering matters here as the first match is used as the
    //       component. This means that we want to ensure that text,
    //       which can match any character, is the last of our components.
    context(
        "Inline Component",
        alt((
            map(math::math_inline, |c| c.map(InlineComponent::from)),
            map(tags::tags, |c| c.map(InlineComponent::from)),
            map(links::link, |c| c.map(InlineComponent::from)),
            map(typefaces::decorated_text, |c| c.map(InlineComponent::from)),
            map(typefaces::keyword, |c| c.map(InlineComponent::from)),
            map(typefaces::text, |c| c.map(InlineComponent::from)),
        )),
    )(input)
}

/// Parses a blank line
fn blank_line(input: Span) -> VimwikiIResult<LC<()>> {
    let (input, pos) = position(input)?;

    let (input, _) = utils::blank_line(input)?;

    Ok((input, LC::from(((), pos, input))))
}

/// Parses a non-blank line
fn non_blank_line(input: Span) -> VimwikiIResult<LC<String>> {
    let (input, pos) = position(input)?;

    let (input, text) = utils::non_blank_line(input)?;

    Ok((input, LC::from((text, pos, input))))
}
