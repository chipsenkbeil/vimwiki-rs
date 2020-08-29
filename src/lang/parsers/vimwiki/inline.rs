use super::{
    components::{
        self, DecoratedText, Decoration, InlineComponent, Keyword, Link,
        MathInline, Tag, TagSequence,
    },
    Span, VimwikiIResult, LC,
};
use nom::{branch::alt, combinator::map};
use nom_locate::position;

/// Parses an inline component, which can only exist on a single line
#[inline]
pub fn inline_component(input: Span) -> VimwikiIResult<LC<InlineComponent>> {
    // NOTE: Ordering matters here as the first match is used as the
    //       component. This means that we want to ensure that text,
    //       which can match any character, is the last of our components.
    alt((
        map(math_inline, |c| c.map(InlineComponent::from)),
        map(tag_sequence, |c| c.map(InlineComponent::from)),
        map(link, |c| c.map(InlineComponent::from)),
        map(decorated_text, |c| c.map(InlineComponent::from)),
        map(keyword, |c| c.map(InlineComponent::from)),
        map(text, |c| c.map(InlineComponent::from)),
    ))(input)
}

#[inline]
fn text(input: Span) -> VimwikiIResult<LC<String>> {
    let (input, pos) = position(input)?;

    // TODO: Text as an inline component should continue until it encounters
    //       something different (math, keyword, link, etc); so, text should
    //       use all other inline components other than itself as not(...)
    //       in a pattern of recoginize(multi1(...))
    panic!("TODO: Implement");
}

#[inline]
fn decorated_text(input: Span) -> VimwikiIResult<LC<DecoratedText>> {
    let (input, pos) = position(input)?;

    // TODO: Decorated text can include include keywords, links, and regular
    //       text but not tags or math
    panic!("TODO: Implement");
}

#[inline]
fn keyword(input: Span) -> VimwikiIResult<LC<Keyword>> {
    let (input, pos) = position(input)?;

    // TODO: Single word matching one from a series of options
    panic!("TODO: Implement");
}

#[inline]
fn link(input: Span) -> VimwikiIResult<LC<Link>> {
    let (input, pos) = position(input)?;

    // TODO: Links can only be on a single line and descriptions can be
    //       either text or an embedded url (no decorated text)
    panic!("TODO: Implement");
}

#[inline]
fn tag_sequence(input: Span) -> VimwikiIResult<LC<TagSequence>> {
    let (input, pos) = position(input)?;

    // TODO: Tag sequences are just :tag1:tag2:...: on a single line
    panic!("TODO: Implement");
}

#[inline]
fn math_inline(input: Span) -> VimwikiIResult<LC<MathInline>> {
    let (input, pos) = position(input)?;

    // TODO: Cannot be on multiple lines and is $...$ with anything inbetween
    panic!("TODO: Implement");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_component_should_parse_math_inline() {
        panic!("TODO: Implement");
    }

    #[test]
    fn inline_component_should_parse_tag_sequence() {
        panic!("TODO: Implement");
    }

    #[test]
    fn inline_component_should_parse_link() {
        panic!("TODO: Implement");
    }

    #[test]
    fn inline_component_should_parse_keyword() {
        panic!("TODO: Implement");
    }

    #[test]
    fn inline_component_should_parse_decorated_text() {
        panic!("TODO: Implement");
    }

    #[test]
    fn inline_component_should_parse_text() {
        panic!("TODO: Implement");
    }
}
