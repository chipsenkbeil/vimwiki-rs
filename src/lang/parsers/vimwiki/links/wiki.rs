use super::{
    components::{Anchor, Description, WikiLink},
    utils::{new_nom_error, position, take_line_while1, url},
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, map_parser, not, opt, rest},
    error::context,
    multi::separated_list,
    sequence::{delimited, preceded},
};
use std::path::PathBuf;

#[inline]
pub fn wiki_link(input: Span) -> VimwikiIResult<LC<WikiLink>> {
    let (input, pos) = position(input)?;
    let (input, link) = context(
        "WikiLink",
        delimited(tag("[["), wiki_link_internal, tag("]]")),
    )(input)?;

    Ok((input, LC::from((link, pos, input))))
}

/// Parser for wiki link content within [[...]]
#[inline]
pub(super) fn wiki_link_internal(input: Span) -> VimwikiIResult<WikiLink> {
    // First, check that the start is not an anchor, then grab all content
    // leading up to | (for description), # (for start of anchor), or
    // ]] (for end of link); if it is the start of an anchor, we won't have
    // a path
    let (input, maybe_path) = opt(preceded(
        not(tag("#")),
        map(
            take_line_while1(not(alt((tag("|"), tag("#"), tag("]]"))))),
            |s| PathBuf::from(s.fragment()),
        ),
    ))(input)?;

    // Next, check if there are any anchors
    let (input, maybe_anchor) = opt(preceded(
        tag("#"),
        map(
            separated_list(
                tag("#"),
                take_line_while1(not(alt((tag("|"), tag("#"), tag("]]"))))),
            ),
            |mut x| {
                Anchor::new(
                    x.drain(..).map(|a| a.fragment().to_string()).collect(),
                )
            },
        ),
    ))(input)?;

    // Finally, check if there is a description (preceding with |), where
    // a special case is wrapped in {{...}} as a URL
    let (input, maybe_description) = opt(preceded(
        tag("|"),
        map_parser(
            take_line_while1(not(tag("]]"))),
            alt((
                map(delimited(tag("{{"), url, tag("}}")), Description::from),
                map(rest, |s: Span| {
                    Description::from(s.fragment().to_string())
                }),
            )),
        ),
    ))(input)?;

    match maybe_path {
        Some(path) => {
            Ok((input, WikiLink::new(path, maybe_description, maybe_anchor)))
        }
        None if maybe_anchor.is_some() => Ok((
            input,
            WikiLink::new(PathBuf::new(), maybe_description, maybe_anchor),
        )),
        None => Err(nom::Err::Error(new_nom_error(
            input,
            "Missing path and anchor",
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_link_should_fail_if_does_not_have_proper_prefix() {
        let input = Span::new("link]]");
        assert!(wiki_link(input).is_err());
    }

    #[test]
    fn wiki_link_should_fail_if_does_not_have_proper_suffix() {
        let input = Span::new("[[link");
        assert!(wiki_link(input).is_err());
    }

    #[test]
    fn wiki_link_should_not_consume_across_lines() {
        let input = Span::new("[[link\n]]");
        let result = wiki_link(input);

        // No input should have been consumed
        assert_eq!(*input.fragment(), "[[link\n]]");

        assert!(result.is_err());
    }

    #[test]
    fn wiki_link_should_support_plain_link() {
        let input = Span::new("[[This is a link]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert!(link.path.is_relative(), "Not detected as relative");
        assert_eq!(link.path.to_str().unwrap(), "This is a link");
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_description() {
        let input =
            Span::new("[[This is a link source|Description of the link]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert!(link.path.is_relative(), "Not detected as relative");
        assert_eq!(link.path.to_str().unwrap(), "This is a link source");
        assert_eq!(
            link.description,
            Some(Description::Text("Description of the link".to_string()))
        );
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_sources_in_subdirectories() {
        let input = Span::new("[[projects/Important Project 1]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert!(link.path.is_relative(), "Not detected as relative");
        assert_eq!(link.path.to_str().unwrap(), "projects/Important Project 1");
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_relative_sources() {
        let input = Span::new("[[../index]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert!(link.path.is_relative(), "Not detected as relative");
        assert_eq!(link.path.to_str().unwrap(), "../index");
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_absolute_source_for_wiki_root() {
        let input = Span::new("[[/index]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert!(link.path.is_absolute(), "Not detected as absolute");
        assert_eq!(link.path.to_str().unwrap(), "/index");
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_source_being_subdirectory() {
        let input = Span::new("[[a subdirectory/|Other files]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert!(link.is_path_dir(), "Not detected as subdirectory");
        assert_eq!(link.path.to_str().unwrap(), "a subdirectory/");
        assert_eq!(
            link.description,
            Some(Description::Text("Other files".to_string()))
        );
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn wiki_link_should_support_an_anchor() {
        let input = Span::new("[[Todo List#Tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.path.to_str().unwrap(), "Todo List");
        assert_eq!(link.description, None);
        assert_eq!(
            link.anchor,
            Some(Anchor::new(vec!["Tomorrow".to_string()]))
        );
    }

    #[test]
    fn wiki_link_should_support_multiple_anchors() {
        let input = Span::new("[[Todo List#Tomorrow#Later]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.path.to_str().unwrap(), "Todo List");
        assert_eq!(link.description, None);
        assert_eq!(
            link.anchor,
            Some(Anchor::new(vec![
                "Tomorrow".to_string(),
                "Later".to_string()
            ]))
        );
    }

    #[test]
    fn wiki_link_should_support_an_anchor_and_a_description() {
        let input = Span::new("[[Todo List#Tomorrow|Tasks for tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.path.to_str().unwrap(), "Todo List");
        assert_eq!(
            link.description,
            Some(Description::Text("Tasks for tomorrow".to_string()))
        );
        assert_eq!(
            link.anchor,
            Some(Anchor::new(vec!["Tomorrow".to_string()]))
        );
    }

    #[test]
    fn wiki_link_should_support_multiple_anchors_and_a_description() {
        let input =
            Span::new("[[Todo List#Tomorrow#Later|Tasks for tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.path.to_str().unwrap(), "Todo List");
        assert_eq!(
            link.description,
            Some(Description::Text("Tasks for tomorrow".to_string()))
        );
        assert_eq!(
            link.anchor,
            Some(Anchor::new(vec![
                "Tomorrow".to_string(),
                "Later".to_string()
            ]))
        );
    }

    #[test]
    fn wiki_link_should_support_anchor_only() {
        let input = Span::new("[[#Tomorrow]]");
        let (input, link) =
            wiki_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert!(link.is_local_anchor(), "Not detected as local anchor");
        assert_eq!(link.path.to_str().unwrap(), "");
        assert_eq!(link.description, None,);
        assert_eq!(
            link.anchor,
            Some(Anchor::new(vec!["Tomorrow".to_string()]))
        );
    }
}
