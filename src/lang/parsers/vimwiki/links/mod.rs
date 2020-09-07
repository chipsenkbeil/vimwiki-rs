use super::{
    components::{self, Link},
    utils, Span, VimwikiIResult, LC,
};
use nom::{branch::alt, combinator::map, error::context};

mod diary;
mod external;
mod interwiki;
mod raw;
mod transclusion;
mod wiki;

/// Inspecting vimwiki source code, there are a couple of link utils
///
/// 1. s:is_web_link = https | http | www | ftp | file | mailto
/// 2. s:is_img_link = .png | .jpg | .gif | .jpeg
///
/// TRANSCLUSIONS
/// NOTE: Can include additional attributes beyond description
///
/// {{imgurl|arg1|arg2}}         -> ???
/// {{imgurl}}                   -> <img src="imgurl"/>
/// {{imgurl|descr|style="A"}}   -> <img src="imgurl" alt="descr" style="A" />
/// {{imgurl|descr|class="B"}}   -> <img src="imgurl" alt="descr" class="B" />
///
/// WIKILINKS
/// NOTE: According to below, only need to worry about transclusion other than
///       string in a description
///
/// [url]]                       -> <a href="url.html">url</a>
/// [[url|descr]]                -> <a href="url.html">descr</a>
/// [[url|{{...}}]]              -> <a href="url.html"> ... </a>
/// [[fileurl.ext|descr]]        -> <a href="fileurl.ext">descr</a>
/// [[dirurl/|descr]]            -> <a href="dirurl/index.html">descr</a>
/// [[url#a1#a2]]                -> <a href="url.html#a1-a2">url#a1#a2</a>
/// [[#a1#a2]]                   -> <a href="#a1-a2">#a1#a2</a>
///
#[inline]
pub fn link(input: Span) -> VimwikiIResult<LC<Link>> {
    context(
        "Link",
        alt((
            // NOTE: We reuse the wiki_link logic for other links and then
            //       do a second pass to determine if external, diary, or
            //       interwiki versus wiki; so, we need to run the other
            //       parsers first to avoid wiki_link consuming other types
            //
            // TODO: This could be better optimized for diary and interwiki by
            //       duplicating the [[ ]] delimited check and then parsing
            //       the beginning, which is unique to diary/interwiki,
            //       avoiding another complete parsing
            map(external::external_file_link, |c| c.map(Link::from)),
            map(diary::diary_link, |c| c.map(Link::from)),
            map(interwiki::inter_wiki_link, |c| c.map(Link::from)),
            map(wiki::wiki_link, |c| c.map(Link::from)),
            map(raw::raw_link, |c| c.map(Link::from)),
            map(transclusion::transclusion_link, |c| c.map(Link::from)),
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_should_return_external_link_where_appropriate() {
        let input = Span::new("[[file:/home/somebody/a/b/c/music.mp3]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.component, Link::ExternalFile(_)));
    }

    #[test]
    fn link_should_return_diary_link_where_appropriate() {
        let input = Span::new("[[diary:2012-03-05]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.component, Link::Diary(_)));
    }

    #[test]
    fn link_should_return_interwiki_link_where_appropriate() {
        let input = Span::new("[[wiki1:Some Link]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.component, Link::InterWiki(_)));

        let input = Span::new("[[wn.My Name:Some Link]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.component, Link::InterWiki(_)));
    }

    #[test]
    fn link_should_return_wiki_link_where_appropriate() {
        let input = Span::new("[[Some Link]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.component, Link::Wiki(_)));
    }

    #[test]
    fn link_should_return_raw_link_where_appropriate() {
        let input = Span::new("https://example.com");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.component, Link::Raw(_)));
    }

    #[test]
    fn link_should_return_transclusion_link_where_appropriate() {
        let input = Span::new("{{https://example.com/img.jpg}}");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.component, Link::Transclusion(_)));
    }
}
