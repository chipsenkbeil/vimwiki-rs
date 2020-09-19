use super::{
    components::{
        self, BlockComponent, InlineComponent, InlineComponentContainer, Page,
    },
    utils::{self, context, lc, range, scan, VimwikiIResult},
    Span, LC,
};
use nom::{
    branch::alt,
    combinator::{all_consuming, map, value},
    multi::{many0, many1},
    InputLength, Slice,
};

pub mod blockquotes;
pub mod comments;
pub mod definitions;
pub mod dividers;
pub mod headers;
pub mod links;
pub mod lists;
pub mod math;
pub mod paragraphs;
pub mod placeholders;
pub mod preformatted;
pub mod tables;
pub mod tags;
pub mod typefaces;

/// Parses entire vimwiki page
pub fn page(input: Span) -> VimwikiIResult<LC<Page>> {
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

        // Third, continuously parse input for new block components until we
        // have nothing left (or we fail)
        let (_, components) = context(
            "Page Components",
            // NOTE: all_consuming will yield an Eof error if input len != 0
            all_consuming(many0(block_component)),
        )(input_2)?;

        // Fourth, return a page wrapped in a location that comprises the
        // entire input
        let comments = ranges_and_comments.drain(..).map(|x| x.1).collect();
        let input = input.slice(input.input_len()..);
        Ok((input, Page::new(components, comments)))
    }

    context("Page", lc(inner))(input)
}

/// Parses a block component
pub fn block_component(input: Span) -> VimwikiIResult<LC<BlockComponent>> {
    context(
        "Block Component",
        alt((
            map(headers::header, |c| c.map(BlockComponent::from)),
            map(definitions::definition_list, |c| {
                c.map(BlockComponent::from)
            }),
            map(lists::list, |c| c.map(BlockComponent::from)),
            map(tables::table, |c| c.map(BlockComponent::from)),
            map(preformatted::preformatted_text, |c| {
                c.map(BlockComponent::from)
            }),
            map(math::math_block, |c| c.map(BlockComponent::from)),
            map(blockquotes::blockquote, |c| c.map(BlockComponent::from)),
            map(dividers::divider, |c| c.map(BlockComponent::from)),
            map(placeholders::placeholder, |c| c.map(BlockComponent::from)),
            map(paragraphs::paragraph, |c| c.map(BlockComponent::from)),
            map(tags::tags, |c| c.map(BlockComponent::from)),
            // NOTE: Parses a single line to end, failing if contains non-whitespace
            map(blank_line, |c| LC::new(BlockComponent::BlankLine, c.region)),
            // NOTE: Parses a single line to end; final type because will match
            //       anychar and consume the line
            map(non_blank_line, |c| c.map(BlockComponent::from)),
        )),
    )(input)
}

/// Parses one or more inline components and wraps it in a container; note
/// that this does NOT consume a line termination
#[inline]
pub fn inline_component_container(
    input: Span,
) -> VimwikiIResult<LC<InlineComponentContainer>> {
    context(
        "Inline Component Container",
        lc(map(many1(inline_component), InlineComponentContainer::from)),
    )(input)
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
    context("Blank Line", lc(value((), utils::blank_line)))(input)
}

/// Parses a non-blank line
fn non_blank_line(input: Span) -> VimwikiIResult<LC<String>> {
    context("Non Blank Line", lc(utils::non_blank_line))(input)
}

#[cfg(test)]
mod tests {
    use super::super::{
        components::{
            Comment, DecoratedText, DecoratedTextContent, Decoration,
            InlineComponent, Keyword, LineComment, Link, MathInline,
            MultiLineComment, Paragraph, Tags, WikiLink,
        },
        Region,
    };
    use super::*;
    use crate::lang::utils::Span;
    use std::path::PathBuf;

    #[test]
    fn page_should_support_blank_lines() {
        let input = Span::from("\n\n");
        let (input, page) = page(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume all of input");
        assert!(page.comments.is_empty());
        assert_eq!(
            page.components,
            vec![BlockComponent::BlankLine, BlockComponent::BlankLine]
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
            page.components,
            vec![BlockComponent::BlankLine, BlockComponent::BlankLine]
        );
    }

    #[test]
    fn page_should_parse_block_components() {
        let input = Span::from("some text with % signs");
        let (input, page) = page(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume all of input");
        assert!(page.comments.is_empty(), "Unexpected parsed comment");
        assert_eq!(
            page.components,
            vec![BlockComponent::from(Paragraph::from(vec![LC::from(
                InlineComponent::Text("some text with % signs".to_string())
            )]))]
        );
    }

    #[test]
    fn page_should_properly_translate_line_and_column_of_block_components_with_comments(
    ) {
        let input =
            Span::from("%%comment\nSome %%+comment+%%more%%+\ncomment+%% text");
        let (input, page) = page(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume all of input");

        let comment = &page.comments[0];
        assert_eq!(
            comment.component,
            Comment::from(LineComment("comment".to_string()))
        );
        assert_eq!(comment.region, Region::from((1, 1, 1, 9)));

        let comment = &page.comments[1];
        assert_eq!(
            comment.component,
            Comment::from(MultiLineComment(vec!["comment".to_string()])),
        );
        assert_eq!(comment.region, Region::from((2, 6, 2, 18)));

        let comment = &page.comments[2];
        assert_eq!(
            comment.component,
            Comment::from(MultiLineComment(vec![
                "".to_string(),
                "comment".to_string(),
            ])),
        );
        assert_eq!(comment.region, Region::from((2, 23, 3, 10)));

        let component = &page.components[0];
        assert_eq!(component.component, BlockComponent::BlankLine);
        assert_eq!(component.region, Region::from((1, 10, 1, 10)));

        let component = &page.components[1];
        assert_eq!(
            component.component,
            BlockComponent::from(Paragraph::from(vec![LC::from(
                InlineComponent::Text("Some more text".to_string())
            )]))
        );
        assert_eq!(component.region, Region::from((2, 1, 3, 15)));
    }

    #[test]
    fn inline_component_container_should_correctly_identify_components() {
        let input = Span::from(
            "*item 1* has a [[link]] with :tag: and $formula$ is DONE",
        );
        let (input, mut container) = inline_component_container(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume all of input");
        assert_eq!(
            container
                .components
                .drain(..)
                .map(|c| c.component)
                .collect::<Vec<InlineComponent>>(),
            vec![
                InlineComponent::DecoratedText(DecoratedText::new(
                    vec![LC::from(DecoratedTextContent::Text(
                        "item 1".to_string()
                    ))],
                    Decoration::Bold
                )),
                InlineComponent::Text(" has a ".to_string()),
                InlineComponent::Link(Link::from(WikiLink::from(
                    PathBuf::from("link")
                ))),
                InlineComponent::Text(" with ".to_string()),
                InlineComponent::Tags(Tags::from("tag")),
                InlineComponent::Text(" and ".to_string()),
                InlineComponent::Math(MathInline::new("formula".to_string())),
                InlineComponent::Text(" is ".to_string()),
                InlineComponent::Keyword(Keyword::DONE),
            ]
        );
    }
}
