use super::{link_description, link_uri_ref};
use crate::lang::{
    elements::{Link, Located},
    parsers::{
        utils::{
            capture, context, cow_str, locate, not_contains, surround_in_line1,
            take_line_until1,
        },
        IResult, Span,
    },
};
use nom::{
    bytes::complete::tag,
    combinator::{map_parser, map_res, opt},
    sequence::delimited,
};
use std::borrow::Cow;

pub fn indexed_interwiki_link(input: Span) -> IResult<Located<Link>> {
    fn inner(input: Span) -> IResult<Link> {
        // First, grab the index of the link
        let (input, index) = indexed_link_index(input)?;

        // Second, grab uri of link
        let (input, uri_ref) = link_uri_ref(input)?;

        // Third, grab optional description of link
        let (input, maybe_description) = opt(link_description)(input)?;

        Ok((
            input,
            Link::new_indexed_interwiki_link(index, uri_ref, maybe_description),
        ))
    }

    context(
        "Indexed Interwiki Link",
        locate(capture(map_parser(
            not_contains("%%", surround_in_line1("[[", "]]")),
            inner,
        ))),
    )(input)
}

fn indexed_link_index(input: Span) -> IResult<u32> {
    map_res(
        delimited(tag("wiki"), take_line_until1(":"), tag(":")),
        |s| s.as_unsafe_remaining_str().parse::<u32>(),
    )(input)
}

pub fn named_interwiki_link(input: Span) -> IResult<Located<Link>> {
    fn inner(input: Span) -> IResult<Link> {
        // First, grab the name of the link
        let (input, name) = named_link_name(input)?;

        // Second, grab uri of link
        let (input, uri_ref) = link_uri_ref(input)?;

        // Third, grab optional description of link
        let (input, maybe_description) = opt(link_description)(input)?;

        Ok((
            input,
            Link::new_named_interwiki_link(name, uri_ref, maybe_description),
        ))
    }

    context(
        "Named Interwiki Link",
        locate(capture(map_parser(
            not_contains("%%", surround_in_line1("[[", "]]")),
            inner,
        ))),
    )(input)
}

fn named_link_name<'a>(input: Span<'a>) -> IResult<Cow<'a, str>> {
    map_parser(
        delimited(tag("wn."), take_line_until1(":"), tag(":")),
        cow_str,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::{Anchor, Description};

    #[test]
    fn indexed_interwiki_link_should_support_numbered_prefix() {
        let input = Span::from("[[wiki1:This is a link]]");
        let (input, link) = indexed_interwiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(link.data().uri_ref().path(), "This is a link");
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn indexed_interwiki_link_should_support_description() {
        let input = Span::from(
            "[[wiki1:This is a link source|Description of the link]]",
        );
        let (input, link) = indexed_interwiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(link.data().uri_ref().path(), "This is a link source");
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn indexed_interwiki_link_should_support_anchors() {
        let input = Span::from("[[wiki1:This is a link source#anchor]]");
        let (input, link) = indexed_interwiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(link.data().uri_ref().path(), "This is a link source");
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), Some(Anchor::from("anchor")));
    }

    #[test]
    fn indexed_interwiki_link_should_support_description_and_anchors() {
        let input = Span::from(
            "[[wiki1:This is a link source#anchor|Description of the link]]",
        );
        let (input, link) = indexed_interwiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(link.data().uri_ref().path(), "This is a link source");
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.to_anchor(), Some(Anchor::from("anchor")));
    }

    #[test]
    fn named_interwiki_link_should_support_named_wikis() {
        let input = Span::from("[[wn.My Name:This is a link]]");
        let (input, link) = named_interwiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.data().uri_ref().path(), "This is a link");
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn named_interwiki_link_should_support_description() {
        let input =
            Span::from("[[wn.My Name:This is a link|Description of the link]]");
        let (input, link) = named_interwiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.data().uri_ref().path(), "This is a link");
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn named_interwiki_link_should_support_anchors() {
        let input = Span::from("[[wn.My Name:This is a link#anchor]]");
        let (input, link) = named_interwiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.data().uri_ref().path(), "This is a link");
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), Some(Anchor::from("anchor")));
    }

    #[test]
    fn named_interwiki_link_should_support_description_and_anchors() {
        let input = Span::from(
            "[[wn.My Name:This is a link#anchor|Description of the link]]",
        );
        let (input, link) = named_interwiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.data().uri_ref().path(), "This is a link");
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.to_anchor(), Some(Anchor::from("anchor")));
    }
}
