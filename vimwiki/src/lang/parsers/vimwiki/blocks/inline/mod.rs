use super::{
    components::{self, InlineComponent, InlineComponentContainer},
    utils::{self, context, lc, VimwikiIResult},
    Span, LC,
};
use nom::{branch::alt, combinator::map, multi::many1};

pub mod links;
pub mod math;
pub mod tags;
pub mod typefaces;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{
            Comment, DecoratedText, DecoratedTextContent, Decoration,
            InlineComponent, Keyword, LineComment, Link, MathInline,
            MultiLineComment, Paragraph, Tags, WikiLink,
        },
        lang::utils::Span,
        Region,
    };
    use std::path::PathBuf;

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
