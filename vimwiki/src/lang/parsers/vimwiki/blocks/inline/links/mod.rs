use crate::lang::{
    elements::{Anchor, Description, Link, Located},
    parsers::{
        utils::{
            context, cow_path, cow_str, take_line_until1,
            take_line_until_one_of_three1, uri,
        },
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, map_parser, not, rest},
    multi::separated_list,
    sequence::{delimited, preceded},
};
use std::{borrow::Cow, path::Path};

pub(crate) mod diary;
pub(crate) mod external;
pub(crate) mod interwiki;
pub(crate) mod raw;
pub(crate) mod transclusion;
pub(crate) mod wiki;

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
pub fn link(input: Span) -> IResult<Located<Link>> {
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

/// Extracts the path-portion of a link
fn link_path<'a>(input: Span<'a>) -> IResult<Cow<'a, Path>> {
    preceded(
        not(tag("#")),
        map_parser(take_line_until_one_of_three1("|", "#", "]]"), cow_path),
    )(input)
}

/// Extracts the anchor-portion of a link
fn link_anchor<'a>(input: Span<'a>) -> IResult<Anchor<'a>> {
    let (input, _) = tag("#")(input)?;

    map(
        separated_list(
            tag("#"),
            map_parser(take_line_until_one_of_three1("|", "#", "]]"), cow_str),
        ),
        Anchor::new,
    )(input)
}

/// Extracts the description-portion of a link
fn link_description<'a>(input: Span<'a>) -> IResult<Description<'a>> {
    map_parser(
        take_line_until1("]]"),
        alt((
            description_from_uri,
            map(rest, |s: Span| Description::Text(s.into())),
        )),
    )(input)
}

// NOTE: This function exists purely because we were hitting some nom
//       error about type-length limit being reached and that means that
//       we've nested too many parsers without breaking them up into
//       functions that do NOT take parsers at input
fn description_from_uri<'a>(input: Span<'a>) -> IResult<Description<'a>> {
    map(
        delimited(
            tag("{{"),
            map_parser(take_line_until1("}}"), uri),
            tag("}}"),
        ),
        Description::from,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_should_return_external_link_where_appropriate() {
        let input = Span::from("[[file:/home/somebody/a/b/c/music.mp3]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::ExternalFile(_)));
    }

    #[test]
    fn link_should_return_diary_link_where_appropriate() {
        let input = Span::from("[[diary:2012-03-05]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::Diary(_)));
    }

    #[test]
    fn link_should_return_interwiki_link_where_appropriate() {
        let input = Span::from("[[wiki1:Some Link]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::InterWiki(_)));

        let input = Span::from("[[wn.My Name:Some Link]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::InterWiki(_)));
    }

    #[test]
    fn link_should_return_wiki_link_where_appropriate() {
        let input = Span::from("[[Some Link]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::Wiki(_)));
    }

    #[test]
    fn link_should_return_raw_link_where_appropriate() {
        let input = Span::from("https://example.com");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::Raw(_)));
    }

    #[test]
    fn link_should_return_transclusion_link_where_appropriate() {
        let input = Span::from("{{https://example.com/img.jpg}}");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::Transclusion(_)));
    }
}
