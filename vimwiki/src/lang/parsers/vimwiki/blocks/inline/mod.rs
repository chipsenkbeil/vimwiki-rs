use crate::lang::{
    elements::{InlineElement, InlineElementContainer, Located},
    parsers::{
        utils::{capture, context, locate},
        IResult, Span,
    },
};
use nom::{branch::alt, combinator::map, multi::many1};

pub mod code;
pub mod comments;
pub mod links;
pub mod math;
pub mod tags;
pub mod typefaces;

/// Parses one or more inline elements and wraps it in a container; note
/// that this does NOT consume a line termination
#[inline]
pub fn inline_element_container(
    input: Span,
) -> IResult<Located<InlineElementContainer>> {
    context(
        "Inline Element Container",
        locate(capture(map(
            many1(inline_element),
            InlineElementContainer::from,
        ))),
    )(input)
}

/// Parses an inline element, which can only exist on a single line
#[inline]
pub fn inline_element(input: Span) -> IResult<Located<InlineElement>> {
    // NOTE: Ordering matters here as the first match is used as the
    //       element. This means that we want to ensure that text,
    //       which can match any character, is the last of our elements.
    //       Additionally, we place comments first as they take priority
    //       over any other type.
    context(
        "Inline Element",
        alt((
            map(comments::comment, |c| c.map(InlineElement::from)),
            map(math::math_inline, |c| c.map(InlineElement::from)),
            map(code::code_inline, |c| c.map(InlineElement::from)),
            map(tags::tags, |c| c.map(InlineElement::from)),
            map(links::link, |c| c.map(InlineElement::from)),
            map(typefaces::decorated_text, |c| c.map(InlineElement::from)),
            map(typefaces::keyword, |c| c.map(InlineElement::from)),
            map(typefaces::text, |c| c.map(InlineElement::from)),
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{
        elements::{
            CodeInline, DecoratedText, DecoratedTextContent, InlineElement,
            Keyword, Link, MathInline, Tags, Text, WikiLink,
        },
        parsers::Span,
    };
    use indoc::indoc;
    use std::path::PathBuf;

    #[test]
    fn inline_element_container_should_prioritize_comments_over_other_ongoing_elements(
    ) {
        // CHIP CHIP CHIP
        // An inline element container should look ahead for comments starting
        // with %% (regardless of single line or multi line) and split input
        // to before and after so that it can apply the normal inline loop
        // to the before and just apply a comment to the after
        let input = Span::from(indoc! {r#"
            *not %%bold*
            _not %%italic_
            ~~not %%strikeout~~
            ^not %%superscript^
            ,,not %%subscript,,
            $not %%math$
            `not %%code`
            [[link|not %%link]]
            TO%%+DO+%%DO
            some text%%comment
            some%%+comment+%%text
        "#});

        todo!();
    }

    #[test]
    fn inline_element_container_should_correctly_identify_elements() {
        let input = Span::from(
            "*item 1* has a [[link]] with `code` and :tag: and $formula$ is DONE",
        );
        let (input, mut container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container
                .elements
                .drain(..)
                .map(|c| c.element)
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "item 1"
                    )))
                ],)),
                InlineElement::Text(Text::from(" has a ")),
                InlineElement::Link(Link::from(WikiLink::from(PathBuf::from(
                    "link"
                )))),
                InlineElement::Text(Text::from(" with ")),
                InlineElement::Code(CodeInline::from("code")),
                InlineElement::Text(Text::from(" and ")),
                InlineElement::Tags(Tags::from("tag")),
                InlineElement::Text(Text::from(" and ")),
                InlineElement::Math(MathInline::from("formula")),
                InlineElement::Text(Text::from(" is ")),
                InlineElement::Keyword(Keyword::DONE),
            ]
        );
    }
}
