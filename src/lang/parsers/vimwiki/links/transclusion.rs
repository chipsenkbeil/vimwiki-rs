use super::{
    components::TransclusionLink,
    utils::{position, url},
    Span, VimwikiIResult, LC,
};
use nom::{branch::alt, combinator::map, error::context};

#[inline]
pub fn transclusion_link(input: Span) -> VimwikiIResult<LC<TransclusionLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transclusion_link_should_support_local_url() {
        // {{file:../../images/vimwiki_logo.png}}
        todo!();
    }

    #[test]
    fn transclusion_link_should_support_universal_url() {
        // {{http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png}}
        todo!();
    }

    #[test]
    fn transclusion_link_should_support_alternate_text() {
        // {{http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png|Vimwiki}}
        //
        // maps to in HTML
        //
        // <img src="http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png"
        // alt="Vimwiki"/>
        todo!();
    }

    #[test]
    fn transclusion_link_should_support_alternate_text_and_style() {
        // {{http://.../vimwiki_logo.png|cool stuff|style="width:150px;height:120px;"}}
        // in HTML:
        // <img src="http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png"
        // alt="cool stuff" style="width:150px; height:120px"/>
        todo!();
    }

    #[test]
    fn transclusion_link_should_support_css_class_without_alternate_text() {
        // {{http://.../vimwiki_logo.png||class="center flow blabla"}}
        // in HTML:
        // <img src="http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png"
        // alt="" class="center flow blabla"/>
        todo!();
    }
}
