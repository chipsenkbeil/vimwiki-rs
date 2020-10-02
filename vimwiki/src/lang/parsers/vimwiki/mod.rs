use super::{
    elements::{self, Page},
    utils::{self, context, lc, range, scan, VimwikiIResult},
    Span, LE,
};
use nom::{combinator::all_consuming, multi::many0, InputLength, Slice};

pub mod blocks;
pub mod comments;

/// Parses entire vimwiki page
pub fn page(input: Span) -> VimwikiIResult<LE<Page>> {
    fn inner(input: Span) -> VimwikiIResult<Page> {
        // Used for our second pass, needs to be done here given that we
        // will be breaking up into segments based on raw input
        let input_2 = input.clone();

        // First, parse the page for comments and remove all from input,
        // skipping over any character that is not a comment
        let (input, mut ranges_and_comments) =
            context("Page Comments", scan(range(comments::comment)))(input)?;

        // Second, produce a new custom span that skips over commented regions
        let segments =
            ranges_and_comments.iter().map(|x| x.0.to_owned()).collect();
        let input_2 = input_2.without_segments(segments);

        // Third, continuously parse input for new block elements until we
        // have nothing left (or we fail)
        let (_, elements) = context(
            "Page Elements",
            // NOTE: all_consuming will yield an Eof error if input len != 0
            all_consuming(many0(blocks::block_element)),
        )(input_2)?;

        // Fourth, return a page wrapped in a location that comprises the
        // entire input
        let comments = ranges_and_comments.drain(..).map(|x| x.1).collect();
        let input = input.slice(input.input_len()..);
        Ok((input, Page::new(elements, comments)))
    }

    context("Page", lc(inner))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        elements::{
            BlockElement, Comment, InlineElement, LineComment,
            MultiLineComment, Paragraph,
        },
        lang::utils::Span,
        Region,
    };

    #[test]
    fn page_should_support_blank_lines() {
        let input = Span::from("\n\n");
        let (input, page) = page(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume all of input");
        assert!(page.comments.is_empty());
        assert_eq!(
            page.elements,
            vec![BlockElement::BlankLine, BlockElement::BlankLine]
        );
    }

    #[test]
    fn page_should_parse_comments() {
        let input = Span::from("%%comment\n%%+comment2+%%\n%%comment3");
        let (input, page) = page(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume all of input");
        assert_eq!(
            page.comments,
            vec![
                Comment::from(LineComment("comment".to_string())),
                Comment::from(MultiLineComment(vec!["comment2".to_string()])),
                Comment::from(LineComment("comment3".to_string())),
            ],
        );
        assert_eq!(
            page.elements,
            vec![BlockElement::BlankLine, BlockElement::BlankLine]
        );
    }

    #[test]
    fn page_should_parse_blocks() {
        let input = Span::from("some text with % signs");
        let (input, page) = page(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume all of input");
        assert!(page.comments.is_empty(), "Unexpected parsed comment");
        assert_eq!(
            page.elements,
            vec![BlockElement::from(Paragraph::from(vec![LE::from(
                InlineElement::Text("some text with % signs".to_string())
            )]))]
        );
    }

    #[test]
    fn page_should_properly_translate_line_and_column_of_blocks_with_comments()
    {
        let input =
            Span::from("%%comment\nSome %%+comment+%%more%%+\ncomment+%% text");
        let (input, page) = page(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume all of input");

        let comment = &page.comments[0];
        assert_eq!(
            comment.element,
            Comment::from(LineComment("comment".to_string()))
        );
        assert_eq!(comment.region, Region::from((1, 1, 1, 9)));

        let comment = &page.comments[1];
        assert_eq!(
            comment.element,
            Comment::from(MultiLineComment(vec!["comment".to_string()])),
        );
        assert_eq!(comment.region, Region::from((2, 6, 2, 18)));

        let comment = &page.comments[2];
        assert_eq!(
            comment.element,
            Comment::from(MultiLineComment(vec![
                "".to_string(),
                "comment".to_string(),
            ])),
        );
        assert_eq!(comment.region, Region::from((2, 23, 3, 10)));

        let element = &page.elements[0];
        assert_eq!(element.element, BlockElement::BlankLine);
        assert_eq!(element.region, Region::from((1, 10, 1, 10)));

        let element = &page.elements[1];
        assert_eq!(
            element.element,
            BlockElement::from(Paragraph::from(vec![LE::from(
                InlineElement::Text("Some more text".to_string())
            )]))
        );
        assert_eq!(element.region, Region::from((2, 1, 3, 15)));
    }
}
