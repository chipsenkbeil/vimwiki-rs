use super::{
    components::{IndexedInterWikiLink, InterWikiLink, NamedInterWikiLink},
    utils::{new_nom_error, take_line_while1},
    wiki::wiki_link,
    Span, VimwikiIResult, LC,
};
use nom::{bytes::complete::tag, combinator::not, sequence::delimited};
use std::path::PathBuf;

#[inline]
pub fn inter_wiki_link(input: Span) -> VimwikiIResult<LC<InterWikiLink>> {
    let (input, mut link) = wiki_link(input)?;
    let path = link.path.to_str().ok_or_else(|| {
        nom::Err::Error(new_nom_error(input, "Not interwiki link"))
    })?;

    if let Some((path, index)) = parse_index_from_path(path) {
        // Update path of link after removal of prefix
        link.path = PathBuf::from(path);

        return Ok((
            input,
            link.map(|c| {
                InterWikiLink::from(IndexedInterWikiLink::new(index, c))
            }),
        ));
    }

    if let Some((path, name)) = parse_name_from_path(path) {
        // Update path of link after removal of prefix
        link.path = PathBuf::from(path);

        return Ok((
            input,
            link.map(|c| InterWikiLink::from(NamedInterWikiLink::new(name, c))),
        ));
    }

    Err(nom::Err::Error(new_nom_error(input, "not interwiki link")))
}

fn parse_index_from_path(path: &str) -> Option<(&str, u32)> {
    delimited(tag("wiki"), take_line_while1(not(tag(":"))), tag(":"))(
        Span::new(path),
    )
    .ok()
    .map(|x| {
        x.1.fragment()
            .parse::<u32>()
            .ok()
            .map(|n| (*x.0.fragment(), n))
    })
    .flatten()
}

fn parse_name_from_path(path: &str) -> Option<(&str, String)> {
    delimited(tag("wn."), take_line_while1(not(tag(":"))), tag(":"))(Span::new(
        path,
    ))
    .ok()
    .map(|x| (*x.0.fragment(), x.1.fragment().to_string()))
}

#[cfg(test)]
mod tests {
    use super::super::components::{Anchor, Description};
    use super::*;

    #[test]
    fn inter_wiki_link_with_index_should_support_numbered_prefix() {
        let input = Span::new("[[wiki1:This is a link]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.index(), Some(1), "Wrong index detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(link.description(), None);
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_index_should_support_description() {
        let input = Span::new(
            "[[wiki1:This is a link source|Description of the link]]",
        );
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
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
        let input = Span::new("[[wiki1:This is a link source#anchor]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
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
        let input = Span::new(
            "[[wiki1:This is a link source#anchor|Description of the link]]",
        );
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
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
        let input = Span::new("[[wn.My Name:This is a link]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(link.description(), None);
        assert_eq!(link.anchor(), None);
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_description() {
        let input =
            Span::new("[[wn.My Name:This is a link|Description of the link]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
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
        let input = Span::new("[[wn.My Name:This is a link#anchor]]");
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(link.description(), None);
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }

    #[test]
    fn inter_wiki_link_with_name_should_support_description_and_anchors() {
        let input = Span::new(
            "[[wn.My Name:This is a link#anchor|Description of the link]]",
        );
        let (input, link) = inter_wiki_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.name(), Some("My Name"), "Wrong name detected");
        assert_eq!(link.path().to_path_buf(), PathBuf::from("This is a link"));
        assert_eq!(
            link.description(),
            Some(&Description::from("Description of the link"))
        );
        assert_eq!(link.anchor(), Some(&Anchor::from("anchor")));
    }
}
