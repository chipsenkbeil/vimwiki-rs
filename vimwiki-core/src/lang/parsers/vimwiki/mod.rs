use crate::lang::{
    elements::*,
    parsers::{
        utils::{blank_line, context},
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    combinator::{all_consuming, map, value},
    multi::many0,
};

pub mod blocks;

pub fn page<'a>(input: Span<'a>) -> IResult<Page<'a>> {
    fn inner<'a>(input: Span<'a>) -> IResult<Page<'a>> {
        // Parses one or more lines, either eating blank lines or producing
        // a block element
        fn maybe_block_element(
            input: Span,
        ) -> IResult<Option<Located<BlockElement>>> {
            alt((
                value(None, blank_line),
                map(blocks::top_level_block_element, Some),
            ))(input)
        }

        map(all_consuming(many0(maybe_block_element)), |mut elements| {
            Page::new(elements.drain(..).flatten().collect())
        })(input)
    }

    context("Page", inner)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_should_skip_blank_lines_not_within_block_elements() {
        let (_, page) = page(Span::from("\n\n")).unwrap();
        assert!(page.elements().is_empty());
    }

    #[test]
    fn page_should_parse_blocks() {
        let (_, page) = page(Span::from("some text with % signs")).unwrap();
        assert_eq!(
            page.elements(),
            vec![Located::from(BlockElement::from(Paragraph::new(vec![
                InlineElementContainer::new(vec![Located::from(
                    InlineElement::Text(Text::from("some text with % signs"))
                )])
            ])))]
        );
    }
}
