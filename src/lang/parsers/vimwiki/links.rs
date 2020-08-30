use super::{
    components::{
        DiaryLink, ExternalLink, InterWikiLink, Link, RawLink,
        TransclusionLink, WikiLink,
    },
    utils::url,
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_while},
    character::complete::{anychar, char as nomchar, crlf, newline, tab},
    combinator::{map, map_res, not, recognize},
    error::context,
    multi::many1,
    sequence::{delimited, pair, terminated, tuple},
};
use nom_locate::position;
use url::Url;

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
            map(wiki_link, |c| c.map(Link::from)),
            map(inter_wiki_link, |c| c.map(Link::from)),
            map(diary_link, |c| c.map(Link::from)),
            map(raw_link, |c| c.map(Link::from)),
            map(external_link, |c| c.map(Link::from)),
            map(transclusion_link, |c| c.map(Link::from)),
        )),
    )(input)
}

#[inline]
fn wiki_link(input: Span) -> VimwikiIResult<LC<WikiLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    panic!("TODO: Implement");
}

#[inline]
fn inter_wiki_link(input: Span) -> VimwikiIResult<LC<InterWikiLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    panic!("TODO: Implement");
}

#[inline]
fn diary_link(input: Span) -> VimwikiIResult<LC<DiaryLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    panic!("TODO: Implement");
}

#[inline]
fn raw_link(input: Span) -> VimwikiIResult<LC<RawLink>> {
    let (input, pos) = position(input)?;

    let (input, url) = url(input)?;

    Ok((input, LC::from((RawLink::from(url), pos))))
}

#[inline]
fn external_link(input: Span) -> VimwikiIResult<LC<ExternalLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    panic!("TODO: Implement");
}

#[inline]
fn transclusion_link(input: Span) -> VimwikiIResult<LC<TransclusionLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    panic!("TODO: Implement");
}
