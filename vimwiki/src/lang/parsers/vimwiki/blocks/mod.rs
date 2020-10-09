use super::{
    elements::{self, BlockElement},
    utils::{self, context, VimwikiIResult},
    Span, LE,
};
use nom::{branch::alt, combinator::map};

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
            map(definitions::definition_list, |c| c.map(BlockElement::from)),
            map(lists::list, |c| c.map(BlockElement::from)),
            map(tables::table, |c| c.map(BlockElement::from)),
            map(preformatted::preformatted_text, |c| {
                c.map(BlockElement::from)
            }),
            map(math::math_block, |c| c.map(BlockElement::from)),
            map(blockquotes::blockquote, |c| c.map(BlockElement::from)),
            map(dividers::divider, |c| c.map(BlockElement::from)),
            map(placeholders::placeholder, |c| c.map(BlockElement::from)),
            // NOTE: Final type because will match literally anything in a line
            map(paragraphs::paragraph, |c| c.map(BlockElement::from)),
        )),
    )(input)
}
