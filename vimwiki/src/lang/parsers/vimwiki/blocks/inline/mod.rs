use super::{
    elements::{self, InlineElement, InlineElementContainer},
    utils::{self, context, le, VimwikiIResult},
    Span, LE,
};
use nom::{branch::alt, combinator::map, multi::many1};

pub mod links;
pub mod math;
pub mod tags;
pub mod typefaces;

/// Parses one or more inline elements and wraps it in a container; note
/// that this does NOT consume a line termination
#[inline]
pub fn inline_element_container(
    input: Span,
) -> VimwikiIResult<LE<InlineElementContainer>> {
    context(
        "Inline Element Container",
        le(map(many1(inline_element), InlineElementContainer::from)),
    )(input)
}

/// Parses an inline element, which can only exist on a single line
#[inline]
pub fn inline_element(input: Span) -> VimwikiIResult<LE<InlineElement>> {
    // NOTE: Ordering matters here as the first match is used as the
    //       element. This means that we want to ensure that text,
    //       which can match any character, is the last of our elements.
    context(
        "Inline Element",
        alt((
            map(math::math_inline, |c| c.map(InlineElement::from)),
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
    use crate::{
        elements::{
            DecoratedText, DecoratedTextContent, Decoration, InlineElement,
            Keyword, Link, MathInline, Tags, WikiLink,
        },
        lang::utils::Span,
    };
    use std::path::PathBuf;

    #[test]
    fn inline_element_container_should_correctly_identify_elements() {
        let input = Span::from(
            "*item 1* has a [[link]] with :tag: and $formula$ is DONE",
        );
        let (input, mut container) = inline_element_container(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume all of input");
        assert_eq!(
            container
                .elements
                .drain(..)
                .map(|c| c.element)
                .collect::<Vec<InlineElement>>(),
            vec![
                InlineElement::DecoratedText(DecoratedText::new(
                    vec![LE::from(DecoratedTextContent::Text(
                        "item 1".to_string()
                    ))],
                    Decoration::Bold
                )),
                InlineElement::Text(" has a ".to_string()),
                InlineElement::Link(Link::from(WikiLink::from(PathBuf::from(
                    "link"
                )))),
                InlineElement::Text(" with ".to_string()),
                InlineElement::Tags(Tags::from("tag")),
                InlineElement::Text(" and ".to_string()),
                InlineElement::Math(MathInline::new("formula".to_string())),
                InlineElement::Text(" is ".to_string()),
                InlineElement::Keyword(Keyword::DONE),
            ]
        );
    }
}
