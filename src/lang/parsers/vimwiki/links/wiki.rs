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
            separated_list(tag("#"), take_line_while1(not(tag("#")))),
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
