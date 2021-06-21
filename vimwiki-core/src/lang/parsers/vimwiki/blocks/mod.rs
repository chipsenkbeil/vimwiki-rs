use crate::lang::{
    elements::{BlockElement, Located},
    parsers::{utils::context, IResult, Span},
};
use nom::{branch::alt, combinator::map};

pub mod blockquotes;
pub mod code;
pub mod definitions;
pub mod dividers;
pub mod headers;
pub mod inline;
pub mod lists;
pub mod math;
pub mod paragraphs;
pub mod placeholders;
pub mod tables;

/// Parses any block or top-level block element
///
/// Top-level block elements are ones that cannot be nested anywhere else,
/// which include:
///
/// 1. Headers
/// 2. Indented Blockquotes
/// 3. Placeholders
/// 4. Dividers
pub fn top_level_block_element(input: Span) -> IResult<Located<BlockElement>> {
    context(
        "Top Level Block Element",
        alt((
            map(headers::header, |c| c.map(BlockElement::from)),
            map(definitions::definition_list, |c| c.map(BlockElement::from)),
            map(lists::list, |c| c.map(BlockElement::from)),
            map(tables::table, |c| c.map(BlockElement::from)),
            map(code::code_block, |c| c.map(BlockElement::from)),
            map(math::math_block, |c| c.map(BlockElement::from)),
            map(blockquotes::blockquote, |c| c.map(BlockElement::from)),
            map(dividers::divider, |c| c.map(BlockElement::from)),
            map(placeholders::placeholder, |c| c.map(BlockElement::from)),
            // NOTE: Final type because will match literally anything in a line
            map(paragraphs::paragraph, |c| c.map(BlockElement::from)),
        )),
    )(input)
}

/// Parses any block element that can be nested; see [`top_level_block_element`]
/// for an explanation of which elements would or would not show up here
pub fn nested_block_element(input: Span) -> IResult<Located<BlockElement>> {
    context(
        "Block Element",
        alt((
            map(definitions::definition_list, |c| c.map(BlockElement::from)),
            map(lists::list, |c| c.map(BlockElement::from)),
            map(tables::nested_table, |c| c.map(BlockElement::from)),
            map(code::code_block, |c| c.map(BlockElement::from)),
            map(math::math_block, |c| c.map(BlockElement::from)),
            map(blockquotes::arrow_blockquote, |c| c.map(BlockElement::from)),
            // NOTE: Final type because will match literally anything in a line
            map(paragraphs::paragraph, |c| c.map(BlockElement::from)),
        )),
    )(input)
}
