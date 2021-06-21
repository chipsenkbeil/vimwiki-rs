use super::{
    blockquotes::arrow_blockquote, code::code_block,
    definitions::definition_list, dividers::divider, headers::header,
    inline::inline_element_container, lists::list, math::math_block,
    placeholders::placeholder, tables::table,
};
use crate::lang::{
    elements::{InlineElementContainer, Located, Paragraph},
    parsers::{
        utils::{blank_line, capture, context, end_of_line_or_input, locate},
        IResult, Span,
    },
};
use nom::{
    character::complete::space0,
    combinator::{map, not},
    multi::many1,
    sequence::delimited,
};

/// Parses a vimwiki paragraph, returning the associated paragraph is successful
#[inline]
pub fn paragraph(input: Span) -> IResult<Located<Paragraph>> {
    fn inner(input: Span) -> IResult<Paragraph> {
        // Continuously take content until we encounter another type of
        // element
        let (input, lines) = context(
            "Paragraph",
            many1(delimited(
                continue_paragraph,
                paragraph_line,
                end_of_line_or_input,
            )),
        )(input)?;

        // Transform contents into the paragraph itself
        let paragraph = Paragraph::new(lines);

        Ok((input, paragraph))
    }

    context("Paragraph", locate(capture(inner)))(input)
}

fn paragraph_line(input: Span) -> IResult<InlineElementContainer> {
    let (input, _) = space0(input)?;

    map(
        inline_element_container,
        |l: Located<InlineElementContainer>| l.into_inner(),
    )(input)
}

// TODO: Optimize by adjusting paragraph parser to be a tuple that
//       includes an Option<BlockElement> so that we don't waste
//       the processing spent
fn continue_paragraph(input: Span) -> IResult<()> {
    let (input, _) = not(header)(input)?;
    let (input, _) = not(definition_list)(input)?;
    let (input, _) = not(list)(input)?;
    let (input, _) = not(table)(input)?;
    let (input, _) = not(code_block)(input)?;
    let (input, _) = not(math_block)(input)?;
    let (input, _) = not(blank_line)(input)?;
    let (input, _) = not(arrow_blockquote)(input)?;
    let (input, _) = not(divider)(input)?;
    let (input, _) = not(placeholder)(input)?;
    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::{
        DecoratedText, DecoratedTextContent, InlineElement, Link, MathInline,
        Text,
    };
    use indoc::indoc;
    use std::convert::TryFrom;
    use uriparse::URIReference;

    #[test]
    fn paragraph_should_fail_if_on_blank_line() {
        let input = Span::from(" ");
        assert!(paragraph(input).is_err());
    }

    #[test]
    fn paragraph_should_parse_single_line() {
        let input = Span::from(indoc! {"
        Some paragraph with *decorations*, [[links]], $math$, and more
        "});
        let (input, p) = paragraph(input).unwrap();
        assert!(input.is_empty(), "Did not consume paragraph");

        assert_eq!(
            p[0].iter()
                .map(|c| c.as_inner().clone())
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Text(Text::from("Some paragraph with ")),
                InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "decorations"
                    )))
                ])),
                InlineElement::Text(Text::from(", ")),
                InlineElement::Link(Link::new_wiki_link(
                    URIReference::try_from("links").unwrap(),
                    None
                )),
                InlineElement::Text(Text::from(", ")),
                InlineElement::Math(MathInline::from("math")),
                InlineElement::Text(Text::from(", and more")),
            ],
        );
    }

    #[test]
    fn paragraph_should_parse_multiple_lines() {
        let input = Span::from(indoc! {"
        Some paragraph with *decorations*,
        [[links]], $math$, and more
        "});
        let (input, p) = paragraph(input).unwrap();
        assert!(input.is_empty(), "Did not consume paragraph");

        assert_eq!(
            p[0].iter()
                .map(|c| c.as_inner().clone())
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Text(Text::from("Some paragraph with ")),
                InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "decorations"
                    )))
                ])),
                InlineElement::Text(Text::from(",")),
            ],
        );

        assert_eq!(
            p[1].iter()
                .map(|c| c.as_inner().clone())
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Link(Link::new_wiki_link(
                    URIReference::try_from("links").unwrap(),
                    None
                )),
                InlineElement::Text(Text::from(", ")),
                InlineElement::Math(MathInline::from("math")),
                InlineElement::Text(Text::from(", and more")),
            ],
        );
    }

    #[test]
    fn paragraph_should_support_whitespace_at_beginning_of_all_following_lines()
    {
        let input = Span::from(indoc! {"
        Some paragraph with *decorations*,
          [[links]], $math$, and more
        "});
        let (input, p) = paragraph(input).unwrap();
        assert!(input.is_empty(), "Did not consume paragraph");

        assert_eq!(
            p[0].iter()
                .map(|c| c.as_inner().clone())
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Text(Text::from("Some paragraph with ")),
                InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "decorations"
                    )))
                ])),
                InlineElement::Text(Text::from(",")),
            ],
        );

        assert_eq!(
            p[1].iter()
                .map(|c| c.as_inner().clone())
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Link(Link::new_wiki_link(
                    URIReference::try_from("links").unwrap(),
                    None
                )),
                InlineElement::Text(Text::from(", ")),
                InlineElement::Math(MathInline::from("math")),
                InlineElement::Text(Text::from(", and more")),
            ],
        );
    }

    #[test]
    fn paragraph_should_stop_at_a_blank_line() {
        let input = Span::from(indoc! {"
        Some paragraph with *decorations*,
        [[links]], $math$, and more

        And this would be a second paragraph
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "\nAnd this would be a second paragraph\n",
            "Unexpected consumption of input"
        );

        assert_eq!(
            p[0].iter()
                .map(|c| c.as_inner().clone())
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Text(Text::from("Some paragraph with ")),
                InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "decorations"
                    )))
                ],)),
                InlineElement::Text(Text::from(",")),
            ],
        );

        assert_eq!(
            p[1].iter()
                .map(|c| c.as_inner().clone())
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Link(Link::new_wiki_link(
                    URIReference::try_from("links").unwrap(),
                    None
                )),
                InlineElement::Text(Text::from(", ")),
                InlineElement::Math(MathInline::from("math")),
                InlineElement::Text(Text::from(", and more")),
            ],
        );
    }

    #[test]
    fn paragraph_should_stop_after_encountering_a_list() {
        let input = Span::from(indoc! {"
            some paragraph
            of text
            - list item
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "- list item\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }

    #[test]
    fn paragraph_should_stop_after_encountering_a_header() {
        let input = Span::from(indoc! {"
            some paragraph
            of text
            = header =
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "= header =\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }

    #[test]
    fn paragraph_should_stop_after_encountering_a_definition_list() {
        let input = Span::from(indoc! {"
            some paragraph
            of text
            term:: def
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "term:: def\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }

    #[test]
    fn paragraph_should_stop_after_encountering_a_table() {
        let input = Span::from(indoc! {"
            some paragraph
            of text
            |cell|
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "|cell|\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }

    #[test]
    fn paragraph_should_stop_after_encountering_a_code_block() {
        let input = Span::from(indoc! {"
            some paragraph
            of text
            {{{
            code block
            }}}
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "{{{\ncode block\n}}}\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }

    #[test]
    fn paragraph_should_stop_after_encountering_a_math_block() {
        let input = Span::from(indoc! {"
            some paragraph
            of text
            {{$
            math block
            }}$
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "{{$\nmath block\n}}$\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }

    #[test]
    fn paragraph_should_stop_after_encountering_a_blank_line() {
        let input = Span::from(indoc! {"
            some paragraph
            of text

            more text
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nmore text\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }

    #[test]
    fn paragraph_should_continue_consuming_when_encountering_an_indented_blockquote(
    ) {
        let input = Span::from(indoc! {"
            some paragraph
            of text
                some blockquote
        "});
        let (input, p) = paragraph(input).unwrap();
        assert!(input.is_empty(), "Unexpectedly failed to consume input");

        // NOTE: Paragraph trims whitespace from beginning of lines, so
        //       the indented blockquote becomes just another line
        assert_eq!(p.to_string(), "some paragraph\nof text\nsome blockquote");
    }

    #[test]
    fn paragraph_should_stop_after_encountering_an_arrow_blockquote() {
        let input = Span::from(indoc! {"
            some paragraph
            of text
            > some blockquote
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "> some blockquote\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }

    #[test]
    fn paragraph_should_stop_after_encountering_a_divider() {
        let input = Span::from(indoc! {"
            some paragraph
            of text
            ----
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "----\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }

    #[test]
    fn paragraph_should_stop_after_encountering_a_placeholder() {
        let input = Span::from(indoc! {"
            some paragraph
            of text
            %title some title
        "});
        let (input, p) = paragraph(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "%title some title\n");

        assert_eq!(p.to_string(), "some paragraph\nof text");
    }
}
