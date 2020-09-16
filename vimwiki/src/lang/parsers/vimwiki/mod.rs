use super::{
    components::{
        self, BlockComponent, InlineComponent, InlineComponentContainer, Page,
    },
    utils::{self, lc, position, range, scan, VimwikiIResult},
    Span, SpanFactory, LC,
};
use nom::{
    branch::alt,
    combinator::{all_consuming, map, value},
    error::context,
    multi::{many0, many1},
    InputLength, Slice,
};
use std::ops::Range;

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
    fn inner(input: Span) -> VimwikiIResult<LC<Page>> {
        let (input, pos) = position(input)?;

        // First, parse the page for comments and remove all from input,
        // skipping over any character that is not a comment
        let (_, mut ranges_and_comments) =
            context("Page Comments", scan(range(comments::comment)))(input)?;

        // Second, produce a new custom span that skips over commented regions
        let skippable_ranges: Vec<Range<usize>> =
            ranges_and_comments.iter().map(|x| x.0.to_owned()).collect();
        let shortened_fragment =
            SpanFactory::shorten_fragment(*input.fragment(), &skippable_ranges);
        let factory = SpanFactory::new(
            *input.fragment(),
            &shortened_fragment,
            &skippable_ranges,
        );
        let no_comments_input = factory.make_span();

        // Third, continuously parse input for new block components until we
        // have nothing left (or we fail)
        //
        // TODO: Cannot unwrap error as it returns the input, which has
        //       local references. Need to change the error type to not
        //       pass back the local input. Make it summarize at the point
        //       of error instead? There is also pre-existing blank error
        //       types like (). Maybe we make a custom error type or use
        //       snafu to provide context
        let (_, components) = context(
            "Page Components",
            all_consuming(many0(block_component)),
        )(no_comments_input)?;

        // Fourth, return a page wrapped in a location that comprises the
        // entire input
        let comments = ranges_and_comments.drain(..).map(|x| x.1).collect();
        let input = input.slice(input.input_len()..);
        Ok((
            input,
            LC::from((Page::new(components, comments), pos, input)),
        ))
    }

    context("Page", inner)(input)
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
    use super::super::components::{
        DecoratedText, DecoratedTextContent, Decoration, InlineComponent,
        Keyword, Link, MathInline, Tags, WikiLink,
    };
    use super::*;
    use crate::lang::utils::new_span;
    use std::path::PathBuf;

    #[test]
    fn inline_component_container_should_correctly_identify_components() {
        let input = new_span(
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
