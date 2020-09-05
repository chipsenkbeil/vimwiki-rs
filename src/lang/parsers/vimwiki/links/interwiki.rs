use super::{
    components::{IndexedInterWikiLink, InterWikiLink, NamedInterWikiLink},
    utils::position,
    Span, VimwikiIResult, LC,
};
use nom::{branch::alt, combinator::map, error::context};

#[inline]
pub fn inter_wiki_link(input: Span) -> VimwikiIResult<LC<InterWikiLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    panic!("TODO: Implement");
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
