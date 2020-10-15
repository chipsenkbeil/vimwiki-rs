use super::{
    blockquotes::blockquote, definitions::definition_list, dividers::divider,
    headers::header, inline::inline_element_container, lists::list,
    math::math_block, placeholders::placeholder,
    preformatted::preformatted_text, tables::table,
};
use crate::lang::{
    elements::{InlineElementContainer, Located, Paragraph},
    parsers::{
        utils::{
            beginning_of_line, blank_line, capture, context,
            end_of_line_or_input, locate,
        },
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
        // Ensure that we are starting at the beginning of a line
        let (input, _) = beginning_of_line(input)?;

        // Continuously take content until we encounter another type of
        // element
        let (input, elements) = context(
            "Paragraph",
            many1(delimited(
                continue_paragraph,
                paragraph_line,
                end_of_line_or_input,
            )),
        )(input)?;

        // Transform contents into the paragraph itself
        let paragraph = Paragraph::new(From::from(elements));

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
    let (input, _) = not(preformatted_text)(input)?;
    let (input, _) = not(math_block)(input)?;
    let (input, _) = not(blank_line)(input)?;
    let (input, _) = not(blockquote)(input)?;
    let (input, _) = not(divider)(input)?;
    let (input, _) = not(placeholder)(input)?;
    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::{
        DecoratedText, DecoratedTextContent, InlineElement, Link, MathInline,
        Text, WikiLink,
    };
    use indoc::indoc;
    use std::path::PathBuf;

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
        let (input, mut p) = paragraph(input).unwrap();
        assert!(input.is_empty(), "Did not consume paragraph");

        assert_eq!(
            p.content
                .elements
                .drain(..)
                .map(|c| c.element)
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Text(Text::from("Some paragraph with ")),
                InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "decorations"
                    )))
                ])),
                InlineElement::Text(Text::from(", ")),
                InlineElement::Link(Link::from(WikiLink::from(PathBuf::from(
                    "links"
                )))),
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
        let (input, mut p) = paragraph(input).unwrap();
        assert!(input.is_empty(), "Did not consume paragraph");

        assert_eq!(
            p.content
                .elements
                .drain(..)
                .map(|c| c.element)
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Text(Text::from("Some paragraph with ")),
                InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "decorations"
                    )))
                ])),
                InlineElement::Text(Text::from(",")),
                InlineElement::Link(Link::from(WikiLink::from(PathBuf::from(
                    "links"
                )))),
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
        let (input, mut p) = paragraph(input).unwrap();
        assert!(input.is_empty(), "Did not consume paragraph");

        assert_eq!(
            p.content
                .elements
                .drain(..)
                .map(|c| c.element)
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Text(Text::from("Some paragraph with ")),
                InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "decorations"
                    )))
                ])),
                InlineElement::Text(Text::from(",")),
                InlineElement::Link(Link::from(WikiLink::from(PathBuf::from(
                    "links"
                )))),
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
        let (input, mut p) = paragraph(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "\nAnd this would be a second paragraph\n",
            "Unexpected consumption of input"
        );

        assert_eq!(
            p.content
                .elements
                .drain(..)
                .map(|c| c.element)
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::Text(Text::from("Some paragraph with ")),
                InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "decorations"
                    )))
                ],)),
                InlineElement::Text(Text::from(",")),
                InlineElement::Link(Link::from(WikiLink::from(PathBuf::from(
                    "links"
                )))),
                InlineElement::Text(Text::from(", ")),
                InlineElement::Math(MathInline::from("math")),
                InlineElement::Text(Text::from(", and more")),
            ],
        );
    }
}
