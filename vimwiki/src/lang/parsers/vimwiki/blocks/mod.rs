use super::{
    elements::{self, BlockElement},
    utils::{self, context, lc, VimwikiIResult},
    Span, LE,
};
use nom::{
    branch::alt,
    combinator::{map, value},
};

pub mod blockquotes;
pub mod definitions;
pub mod dividers;
pub mod headers;
pub mod inline;
pub mod lists;
pub mod math;
pub mod paragraphs;
pub mod placeholders;
pub mod preformatted;
pub mod tables;

/// Parses a block element
pub fn block_element(input: Span) -> VimwikiIResult<LE<BlockElement>> {
    context(
        "Block Element",
        alt((
            map(headers::header, |c| c.map(BlockElement::from)),
            map(definitions::definition_list, |c| {
                c.map(BlockElement::from)
            }),
            map(lists::list, |c| c.map(BlockElement::from)),
            map(tables::table, |c| c.map(BlockElement::from)),
            map(preformatted::preformatted_text, |c| {
                c.map(BlockElement::from)
            }),
            map(math::math_block, |c| c.map(BlockElement::from)),
            map(blank_line, |c| LE::new(BlockElement::BlankLine, c.region)),
            map(blockquotes::blockquote, |c| c.map(BlockElement::from)),
            map(dividers::divider, |c| c.map(BlockElement::from)),
            map(placeholders::placeholder, |c| c.map(BlockElement::from)),
            map(paragraphs::paragraph, |c| c.map(BlockElement::from)),
            // NOTE: Parses a single line to end; final type because will match
            //       anychar and consume the line; used as our fallback in
            //       case we don't match any other type
            map(non_blank_line, |c| c.map(BlockElement::from)),
        )),
    )(input)
}

/// Parses a blank line
fn blank_line(input: Span) -> VimwikiIResult<LE<()>> {
    context("Blank Line", lc(value((), utils::blank_line)))(input)
}

/// Parses a non-blank line
fn non_blank_line(input: Span) -> VimwikiIResult<LE<String>> {
    context("Non Blank Line", lc(utils::non_blank_line))(input)
}
