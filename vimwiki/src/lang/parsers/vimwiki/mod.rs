use super::{
    elements::{self, BlockElement, Comment, Page},
    utils::{
        self, blank_line, context, range, scan_with_step, take_until_byte1,
        VimwikiIResult, VimwikiNomError,
    },
    Span, LE,
};
use nom::{
    branch::alt,
    bytes::complete::take,
    combinator::{all_consuming, map, value},
    multi::many0,
};
use std::ops::Range;

pub mod blocks;
pub mod comments;

/// Parses entire vimwiki page
pub fn page(mut s: String) -> Result<LE<Page>, nom::Err<VimwikiNomError>> {
    // First, parse the page for comments and remove all from input,
    // skipping over any character that is not a comment
    let mut ranges_and_comments = {
        let input = Span::from(s.as_bytes());
        page_comments(input)?.1
    };

    // Second, modify our original input to remove the comments
    for segment in ranges_and_comments.iter().map(|x| x.0.to_owned()).rev() {
        s.replace_range(segment, "");
    }

    // Third, continuously parse input for new block elements until we
    // have nothing left (or we fail)
    let (_, elements) = page_elements(Span::from(s.as_bytes()))?;

    // Fourth, return a page wrapped in a location that comprises the
    // entire input
    let comments = ranges_and_comments.drain(..).map(|x| x.1).collect();
    Ok(LE::from(Page::new(elements, comments)))
}

fn page_comments(
    input: Span,
) -> VimwikiIResult<Vec<(Range<usize>, LE<Comment>)>> {
    context(
        "Page Comments",
        scan_with_step(
            range(comments::comment),
            value((), alt((take_until_byte1(b'%'), take(1usize)))),
        ),
    )(input)
}

fn page_elements(input: Span) -> VimwikiIResult<Vec<LE<BlockElement>>> {
    fn inner(input: Span) -> VimwikiIResult<Vec<LE<BlockElement>>> {
        // Parses one or more lines, either eating blank lines or producing
        // a block element
        fn maybe_block_element(
            input: Span,
        ) -> VimwikiIResult<Option<LE<BlockElement>>> {
            alt((value(None, blank_line), map(blocks::block_element, Some)))(
                input,
            )
        }

        map(all_consuming(many0(maybe_block_element)), |mut elements| {
            elements.drain(..).flatten().collect()
        })(input)
    }

    context("Page Elements", inner)(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        elements::{
            BlockElement, Comment, InlineElement, LineComment,
            MultiLineComment, Paragraph, Text,
        },
        lang::utils::Span,
        Region,
    };

    #[test]
    fn page_should_skip_blank_lines_not_within_block_elements() {
        let page = page(String::from("\n\n")).unwrap();
        assert!(page.comments.is_empty());
        assert!(page.elements.is_empty());
    }

    #[test]
    #[ignore]
    fn page_should_parse_comments() {
        let page = page(String::from("%%comment\n%%+comment2+%%\n%%comment3"))
            .unwrap();
        assert_eq!(
            page.comments,
            vec![
                Comment::from(LineComment("comment".to_string())),
                Comment::from(MultiLineComment(vec!["comment2".to_string()])),
                Comment::from(LineComment("comment3".to_string())),
            ],
        );
        assert!(page.elements.is_empty());
    }

    #[test]
    fn page_should_parse_blocks() {
        let page = page(String::from("some text with % signs")).unwrap();
        assert!(page.comments.is_empty(), "Unexpected parsed comment");
        assert_eq!(
            page.elements,
            vec![BlockElement::from(Paragraph::from(vec![LE::from(
                InlineElement::Text(Text::from("some text with % signs"))
            )]))]
        );
    }

    #[test]
    #[ignore]
    fn page_should_properly_translate_line_and_column_of_blocks_with_comments()
    {
        let page = page(String::from(
            "%%comment\nSome %%+comment+%%more%%+\ncomment+%% text",
        ))
        .unwrap();

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
        assert_eq!(
            element.element,
            BlockElement::from(Paragraph::from(vec![LE::from(
                InlineElement::Text(Text::from("Some more text"))
            )]))
        );
        assert_eq!(element.region, Region::from((2, 1, 3, 15)));
    }
}
