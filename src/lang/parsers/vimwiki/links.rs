use super::{
    components::{
        DiaryLink, ExternalLink, InterWikiLink, Link, RawLink,
        TransclusionLink, WikiLink,
    },
    utils::{position, url},
    Span, VimwikiIResult, LC,
};
use nom::{branch::alt, combinator::map, error::context};

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

    Ok((input, LC::from((RawLink::from(url), pos, input))))
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

    #[test]
    fn diary_wiki_link_should_support_diary_scheme() {
        // [[diary:2012-03-05]]
        todo!();
    }

    #[test]
    fn diary_wiki_link_should_support_anchors() {
        // [[diary:2020-03-05#Tomorrow|Tasks for tomorrow]]
        todo!();
    }

    #[test]
    fn external_link_should_support_absolute_path_with_no_scheme() {
        // [[//absolute_path]]
        // [[///tmp/in_root_tmp]]
        // [[//~/in_home_dir]]
        // [[//$HOME/in_home_dir]]
        todo!();
    }

    #[test]
    fn external_link_should_support_file_scheme() {
        // [[file:/home/somebody/a/b/c/music.mp3]]
        // [[file:C:/Users/somebody/d/e/f/music.mp3]]
        // [[file:~/a/b/c/music.mp3]]
        // [[file:../assets/data.csv|Important Data]]
        // [[file:/home/user/documents/|Link to a directory]]
        todo!();
    }

    #[test]
    fn external_link_should_support_local_scheme() {
        // [[local:C:/Users/somebody/d/e/f/music.mp3]]
        todo!();
    }

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

    #[test]
    fn thumbnail_link_should_support_image_types() {
        // [[http://someaddr.com/bigpicture.jpg|{{http://someaddr.com/thumbnail.jpg}}]]
        // in HTML:
        // <a href="http://someaddr.com/ ... /.jpg">
        // <img src="http://../thumbnail.jpg /></a>
        //
        // Vimwiki validates against .png | .jpg | .gif | .jpeg
        todo!();
    }
}
