use super::{
    components::WikiLink,
    utils::{position, url},
    Span, VimwikiIResult, LC,
};
use nom::{branch::alt, combinator::map, error::context};

#[inline]
pub fn wiki_link(input: Span) -> VimwikiIResult<LC<WikiLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_link_should_support_plain_link() {
        // [[This is a link]]
        todo!();
    }

    #[test]
    fn wiki_link_should_support_description() {
        // [[This is a link source|Description of the link]]
        todo!();
    }

    #[test]
    fn wiki_link_should_support_sources_in_subdirectories() {
        // [[projects/Important Project 1]]
        todo!();
    }

    #[test]
    fn wiki_link_should_support_relative_sources() {
        // [[../index]]
        todo!();
    }

    #[test]
    fn wiki_link_should_support_absolute_source_for_wiki_root() {
        // [[/index]]
        todo!();
    }

    #[test]
    fn wiki_link_should_support_source_being_subdirectory() {
        // [[a subdirectory/|Other files]]
        todo!();
    }
    #[test]
    fn wiki_link_should_support_anchors() {
        // [[Todo List#Tomorrow|Tasks for tomorrow]]
        todo!();
    }

    #[test]
    fn wiki_link_should_support_anchor_only() {
        // [[#Tomorrow]]
        todo!();
    }
}
