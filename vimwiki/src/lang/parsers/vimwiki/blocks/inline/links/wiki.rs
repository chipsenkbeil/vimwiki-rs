use crate::lang::{
    elements::{Anchor, Description, Located, WikiLink},
    parsers::{
        utils::{
            capture, context, cow_path, cow_str, locate, take_line_while1, uri,
        },
        Error, IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, map_parser, not, opt, rest},
    multi::separated_list,
    sequence::{delimited, preceded},
};
use std::{borrow::Cow, path::PathBuf};

#[inline]
pub fn wiki_link(input: Span) -> IResult<Located<WikiLink>> {
    context(
        "WikiLink",
        locate(capture(delimited(tag("[["), wiki_link_internal, tag("]]")))),
    )(input)
}

/// Parser for wiki link content within [[...]]
#[inline]
pub(super) fn wiki_link_internal(input: Span) -> IResult<WikiLink> {
    // First, check that the start is not an anchor, then grab all content
    // leading up to | (for description), # (for start of anchor), or
    // ]] (for end of link); if it is the start of an anchor, we won't have
    // a path
    let (input, maybe_path) = opt(preceded(
        not(tag("#")),
        cow_path(take_line_while1(not(alt((tag("|"), tag("#"), tag("]]")))))),
    ))(input)?;

    // Next, check if there are any anchors
    let (input, maybe_anchor) = opt(anchor)(input)?;

    // Finally, check if there is a description (preceding with |), where
    // a special case is wrapped in {{...}} as a URL
    let (input, maybe_description) = opt(description)(input)?;

    match maybe_path {
        Some(path) => {
            Ok((input, WikiLink::new(path, maybe_description, maybe_anchor)))
        }
        None if maybe_anchor.is_some() => Ok((
            input,
            WikiLink::new(
                Cow::from(PathBuf::new()),
                maybe_description,
                maybe_anchor,
            ),
        )),
        None => Err(nom::Err::Error(Error::from_ctx(
            &input,
            "Missing path and anchor",
        ))),
    }
}

// NOTE: This function exists purely because we were hitting some nom
//       error about type-length limit being reached and that means that
//       we've nested too many parsers without breaking them up into
//       functions that do NOT take parsers at input
fn anchor(input: Span) -> IResult<Anchor> {
    preceded(
        tag("#"),
        map(
            separated_list(
                tag("#"),
                cow_str(take_line_while1(not(alt((
                    tag("|"),
                    tag("#"),
                    tag("]]"),
                ))))),
            ),
            Anchor::new,
        ),
    )(input)
}

// NOTE: This function exists purely because we were hitting some nom
//       error about type-length limit being reached and that means that
//       we've nested too many parsers without breaking them up into
//       functions that do NOT take parsers at input
fn description(input: Span) -> IResult<Description> {
    preceded(
        tag("|"),
        map_parser(
            take_line_while1(not(tag("]]"))),
            alt((
                description_from_uri,
                map(rest, |s: Span| {
                    Description::from(s.as_unsafe_remaining_str())
                }),
            )),
        ),
    )(input)
}

// NOTE: This function exists purely because we were hitting some nom
//       error about type-length limit being reached and that means that
//       we've nested too many parsers without breaking them up into
//       functions that do NOT take parsers at input
fn description_from_uri(input: Span) -> IResult<Description> {
    map(
        delimited(
            tag("{{"),
            map_parser(take_line_while1(not(tag("}}"))), uri),
            tag("}}"),
        ),
        Description::from,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use uriparse::URI;

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

        assert!(link.path.is_relative(), "Not detected as relative");
        assert_eq!(link.path.to_str().unwrap(), "This is a link");
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_description() {
        let input =
            Span::from("[[This is a link source|Description of the link]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert!(link.path.is_relative(), "Not detected as relative");
        assert_eq!(link.path.to_str().unwrap(), "This is a link source");
        assert_eq!(
            link.description,
            Some(Description::from("Description of the link"))
        );
        assert_eq!(link.anchor, None);
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

        assert!(link.path.is_relative(), "Not detected as relative");
        assert_eq!(link.path.to_str().unwrap(), "This is a link source");
        assert_eq!(
            link.description,
            Some(Description::from(
                URI::try_from("https://example.com/img.jpg")
                    .unwrap()
                    .into_owned()
            ))
        );
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_sources_in_subdirectories() {
        let input = Span::from("[[projects/Important Project 1]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert!(link.path.is_relative(), "Not detected as relative");
        assert_eq!(link.path.to_str().unwrap(), "projects/Important Project 1");
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_relative_sources() {
        let input = Span::from("[[../index]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert!(link.path.is_relative(), "Not detected as relative");
        assert_eq!(link.path.to_str().unwrap(), "../index");
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_absolute_source_for_wiki_root() {
        let input = Span::from("[[/index]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert!(link.path.is_absolute(), "Not detected as absolute");
        assert_eq!(link.path.to_str().unwrap(), "/index");
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_source_being_subdirectory() {
        let input = Span::from("[[a subdirectory/|Other files]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert!(link.is_path_dir(), "Not detected as subdirectory");
        assert_eq!(link.path.to_str().unwrap(), "a subdirectory/");
        assert_eq!(link.description, Some(Description::from("Other files")));
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_an_anchor() {
        let input = Span::from("[[Todo List#Tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.path.to_str().unwrap(), "Todo List");
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, Some(Anchor::from("Tomorrow")));
    }

    #[test]
    fn wiki_link_should_support_multiple_anchors() {
        let input = Span::from("[[Todo List#Tomorrow#Later]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.path.to_str().unwrap(), "Todo List");
        assert_eq!(link.description, None);
        assert_eq!(
            link.anchor,
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

        assert_eq!(link.path.to_str().unwrap(), "Todo List");
        assert_eq!(
            link.description,
            Some(Description::from("Tasks for tomorrow"))
        );
        assert_eq!(link.anchor, Some(Anchor::from("Tomorrow")));
    }

    #[test]
    fn wiki_link_should_support_multiple_anchors_and_a_description() {
        let input =
            Span::from("[[Todo List#Tomorrow#Later|Tasks for tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.path.to_str().unwrap(), "Todo List");
        assert_eq!(
            link.description,
            Some(Description::from("Tasks for tomorrow"))
        );
        assert_eq!(
            link.anchor,
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

        assert!(link.is_local_anchor(), "Not detected as local anchor");
        assert_eq!(link.path.to_str().unwrap(), "");
        assert_eq!(link.description, None,);
        assert_eq!(link.anchor, Some(Anchor::from("Tomorrow")));
    }
}
