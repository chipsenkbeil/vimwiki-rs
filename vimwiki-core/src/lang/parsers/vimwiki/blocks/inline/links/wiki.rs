use super::link_data;
use crate::lang::{
    elements::{Link, Located},
    parsers::{
        utils::{capture, context, locate, not_contains, surround_in_line1},
        IResult, Span,
    },
};
use nom::combinator::map_parser;

#[inline]
pub fn wiki_link(input: Span) -> IResult<Located<Link>> {
    fn inner(input: Span) -> IResult<Link> {
        let (input, data) = link_data(input)?;
        Ok((input, Link::Wiki { data }))
    }

    context(
        "Wiki Link",
        locate(capture(map_parser(
            not_contains("%%", surround_in_line1("[[", "]]")),
            inner,
        ))),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::{Anchor, Description};
    use std::{borrow::Cow, convert::TryFrom};
    use uriparse::URIReference;

    #[test]
    fn wiki_link_should_fail_if_does_not_have_proper_prefix() {
        let input = Span::from("link]]");
        assert!(wiki_link(input).is_err());
    }

    #[test]
    fn wiki_link_should_fail_if_does_not_have_proper_suffix() {
        let input = Span::from("[[link");
        assert!(wiki_link(input).is_err());
    }

    #[test]
    fn wiki_link_should_not_consume_across_lines() {
        let input = Span::from("[[link\n]]");
        assert!(wiki_link(input).is_err());
    }

    #[test]
    fn wiki_link_should_support_plain_link() {
        let input = Span::from("[[This is a link]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "This%20is%20a%20link");
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn wiki_link_should_support_description() {
        let input =
            Span::from("[[This is a link source|Description of the link]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "This%20is%20a%20link%20source");
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn wiki_link_should_support_thumbnail_description() {
        let input = Span::from(
            "[[This is a link source|{{https://example.com/img.jpg}}]]",
        );
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "This%20is%20a%20link%20source");
        assert_eq!(
            link.description(),
            Some(&Description::from(
                URIReference::try_from("https://example.com/img.jpg")
                    .unwrap()
                    .into_owned()
            ))
        );
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn wiki_link_should_support_sources_in_subdirectories() {
        let input = Span::from("[[projects/Important Project 1]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(
            link.data().uri_ref.path(),
            "projects/Important%20Project%201"
        );
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn wiki_link_should_support_relative_sources() {
        let input = Span::from("[[../index]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "../index");
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn wiki_link_should_support_absolute_source_for_wiki_root() {
        let input = Span::from("[[/index]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "/index");
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn wiki_link_should_support_source_being_subdirectory() {
        let input = Span::from("[[a subdirectory/|Other files]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert!(link.data().is_path_dir(), "Not detected as subdirectory");
        assert_eq!(link.data().uri_ref.path(), "a%20subdirectory/");
        assert_eq!(link.description(), Some(&Description::from("Other files")));
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn wiki_link_should_support_an_anchor() {
        let input = Span::from("[[Todo List#Tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "Todo%20List");
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), Some(Anchor::from("Tomorrow")));
    }

    #[test]
    fn wiki_link_should_support_multiple_anchors() {
        let input = Span::from("[[Todo List#Tomorrow#Later]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "Todo%20List");
        assert_eq!(link.description(), None);
        assert_eq!(
            link.to_anchor(),
            Some(Anchor::new(vec![Cow::from("Tomorrow"), Cow::from("Later")]))
        );
    }

    #[test]
    fn wiki_link_should_support_an_anchor_and_a_description() {
        let input = Span::from("[[Todo List#Tomorrow|Tasks for tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "Todo%20List");
        assert_eq!(
            link.description(),
            Some(&Description::from("Tasks for tomorrow"))
        );
        assert_eq!(link.to_anchor(), Some(Anchor::from("Tomorrow")));
    }

    #[test]
    fn wiki_link_should_support_multiple_anchors_and_a_description() {
        let input =
            Span::from("[[Todo List#Tomorrow#Later|Tasks for tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "Todo%20List");
        assert_eq!(
            link.description(),
            Some(&Description::from("Tasks for tomorrow"))
        );
        assert_eq!(
            link.to_anchor(),
            Some(Anchor::new(vec![Cow::from("Tomorrow"), Cow::from("Later")]))
        );
    }

    #[test]
    fn wiki_link_should_support_anchor_only() {
        let input = Span::from("[[#Tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.data().uri_ref.path(), "");
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), Some(Anchor::from("Tomorrow")));
    }
}
