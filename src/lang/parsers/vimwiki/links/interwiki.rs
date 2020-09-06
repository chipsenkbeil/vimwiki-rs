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
    use super::*;

    #[test]
    fn inter_wiki_link_should_support_numbered_prefix() {
        // [[wiki1:This is a link]]
        // [[wiki1:This is a link source|Description of the link]]
        todo!();
    }

    #[test]
    fn inter_wiki_link_should_support_named_wikis() {
        // [[wn.My Name:This is a link]]
        // [[wn.MyWiki:This is a link source|Description of the link]]
        todo!();
    }

    #[test]
    fn inter_wiki_link_should_support_anchors() {
        // [[wiki1:This is a link#Tomorrow]]
        // [[wiki1:This is a link source#Tomorrow|Tasks for tomorrow]]
        // [[wn.My Name:This is a link#Tomorrow|Tasks for tomorrow]]
        // [[wn.MyWiki:This is a link source#Tomrrow|Tasks for tomorrow]]
        todo!();
    }
}
