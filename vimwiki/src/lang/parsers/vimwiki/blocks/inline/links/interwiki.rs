use super::{link_anchor, link_description, link_path};
use crate::lang::{
    elements::{
        Anchor, Description, IndexedInterWikiLink, InterWikiLink, Located,
        NamedInterWikiLink, WikiLink,
    },
    parsers::{
        utils::{
            capture, context, cow_str, locate, not_contains, surround_in_line1,
            take_line_until1,
        },
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, map_parser, map_res, opt},
    sequence::{delimited, pair, preceded},
};
use std::{borrow::Cow, path::Path};

#[inline]
pub fn inter_wiki_link(input: Span) -> IResult<Located<InterWikiLink>> {
    fn inner(input: Span) -> IResult<InterWikiLink> {
        // InterWikiLinks are specialized links that start with either an
        // index or name
        alt((
            map(
                pair(indexed_link_index, rest_of_link),
                |(index, (path, maybe_anchor, maybe_description))| {
                    InterWikiLink::from(IndexedInterWikiLink::new(
                        index,
                        WikiLink::new(path, maybe_description, maybe_anchor),
                    ))
                },
            ),
            map(
                pair(named_link_name, rest_of_link),
                |(name, (path, maybe_anchor, maybe_description))| {
                    InterWikiLink::from(NamedInterWikiLink::new(
                        name,
                        WikiLink::new(path, maybe_description, maybe_anchor),
                    ))
                },
            ),
        ))(input)
    }

    context(
        "InterWikiLink",
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

fn named_link_name<'a>(input: Span<'a>) -> IResult<Cow<'a, str>> {
    cow_str(delimited(tag("wn."), take_line_until1(":"), tag(":")))(input)
}

fn rest_of_link<'a>(
    input: Span<'a>,
) -> IResult<(Cow<'a, Path>, Option<Anchor<'a>>, Option<Description<'a>>)> {
    // After the specialized start, a valid path must follow
    let (input, path) = link_path(input)?;

    // Next, check if there are any anchors
    let (input, maybe_anchor) = opt(link_anchor)(input)?;

    // Finally, check if there is a description (preceding with |), where
    // a special case is wrapped in {{...}} as a URL
    let (input, maybe_description) =
        opt(preceded(tag("|"), link_description))(input)?;

    Ok((input, (path, maybe_anchor, maybe_description)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::{Anchor, Description};
    use std::path::PathBuf;

    #[test]
    fn inter_wiki_link_with_index_should_support_numbered_prefix() {
        let input = Span::from("[[wiki1:This is a link]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(link.description(), None);
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_index_should_support_description() {
        let input = Span::from(
            "[[wiki1:This is a link source|Description of the link]]",
        );
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(
            link.path().to_path_buf(),
            PathBuf::from("This is a link source")
        );
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_index_should_support_anchors() {
        let input = Span::from("[[wiki1:This is a link source#anchor]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(
            link.path().to_path_buf(),
            PathBuf::from("This is a link source")
        );
        assert_eq!(link.description(), None,);
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }

    #[test]
    fn inter_wiki_link_with_index_should_support_description_and_anchors() {
        let input = Span::from(
            "[[wiki1:This is a link source#anchor|Description of the link]]",
        );
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(
            link.path().to_path_buf(),
            PathBuf::from("This is a link source")
        );
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_named_wikis() {
        let input = Span::from("[[wn.My Name:This is a link]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(link.description(), None);
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_description() {
        let input =
            Span::from("[[wn.My Name:This is a link|Description of the link]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_anchors() {
        let input = Span::from("[[wn.My Name:This is a link#anchor]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(link.description(), None);
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_description_and_anchors() {
        let input = Span::from(
            "[[wn.My Name:This is a link#anchor|Description of the link]]",
        );
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }
}
