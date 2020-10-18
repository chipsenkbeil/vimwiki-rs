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
            CodeInline, Comment, DecoratedText, DecoratedTextContent,
            InlineElement, Keyword, LineComment, Link, MathInline,
            MultiLineComment, Tags, Text, WikiLink,
        },
        parsers::Span,
    };
    use std::path::PathBuf;

    #[test]
    fn inline_element_container_should_prioritize_comments_over_bold_text() {
        let input = Span::from(r"*not %%bold*");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"*not "))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(LineComment::from(r"bold*").into())
        );
    }

    #[test]
    fn inline_element_container_should_prioritize_comments_over_italic_text() {
        let input = Span::from(r"_not %%italic_");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"_not "))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(LineComment::from(r"italic_").into())
        );
    }

    #[test]
    fn inline_element_container_should_prioritize_comments_over_strikeout_text()
    {
        let input = Span::from(r"~~not %%strikeout~~");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"~~not "))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(LineComment::from(r"strikeout~~").into())
        );
    }

    #[test]
    fn inline_element_container_should_prioritize_comments_over_superscript_text(
    ) {
        let input = Span::from(r"^not %%superscript^");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"^not "))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(LineComment::from(r"superscript^").into())
        );
    }

    #[test]
    fn inline_element_container_should_prioritize_comments_over_subscript_text()
    {
        let input = Span::from(r",,not %%subscript,,");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r",,not "))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(LineComment::from(r"subscript,,").into())
        );
    }

    #[test]
    fn inline_element_container_should_prioritize_comments_over_math() {
        let input = Span::from(r"$not %%math$");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"$not "))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(LineComment::from(r"math$").into())
        );
    }

    #[test]
    fn inline_element_container_should_prioritize_comments_over_code() {
        let input = Span::from(r"`not %%code`");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"`not "))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(LineComment::from(r"code`").into())
        );
    }

    #[test]
    fn inline_element_container_should_prioritize_comments_over_link() {
        let input = Span::from(r"[[link|not %%link]]");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"[[link|not "))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(LineComment::from(r"link]]").into())
        );
    }

    #[test]
    fn inline_element_container_should_prioritize_comments_over_keyword() {
        let input = Span::from(r"TO%%+DO+%%DO");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"TO"))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(MultiLineComment::from(r"DO").into())
        );
        assert_eq!(
            container.elements[2],
            InlineElement::from(Text::from(r"DO"))
        );
    }

    #[test]
    fn inline_element_container_should_prioritize_comments_over_text() {
        let input = Span::from(r"some text%%comment");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"some text"))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(LineComment::from(r"comment").into())
        );

        let input = Span::from(r"some%%+comment+%%text");
        let (input, container) = inline_element_container(input).unwrap();
        assert!(input.is_empty(), "Did not consume all of input");
        assert_eq!(
            container.elements[0],
            InlineElement::from(Text::from(r"some"))
        );
        assert_eq!(
            container.elements[1],
            InlineElement::Comment(MultiLineComment::from(r"comment").into())
        );
        assert_eq!(
            container.elements[2],
            InlineElement::from(Text::from(r"text"))
        );
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
