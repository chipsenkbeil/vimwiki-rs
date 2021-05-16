use crate::lang::{
    elements::{Description, Link, LinkData, Located},
    parsers::{
        utils::{
            context, cow_str, take_line_until, take_line_until1,
            take_line_until_one_of_three1,
        },
        Error, IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, map_parser, opt, rest},
    multi::separated_list1,
    sequence::{delimited, separated_pair},
};
use percent_encoding::{percent_encode, AsciiSet, CONTROLS};
use std::{borrow::Cow, collections::HashMap, convert::TryFrom};
use uriparse::URIReference;

mod diary;
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
            diary::diary_link,
            interwiki::indexed_interwiki_link,
            interwiki::named_interwiki_link,
            wiki::wiki_link,
            raw::raw_link,
            transclusion::transclusion_link,
        )),
    )(input)
}

/// Extracts link data from within a link bound by `[[...]]` or `{{...}}`
///
/// Assumes that there is a URI available prior to a description or any
/// other properties
fn link_data<'a>(input: Span<'a>) -> IResult<LinkData<'a>> {
    let (input, uri_ref) = link_uri_ref(input)?;
    let (input, maybe_description) = opt(link_description)(input)?;
    let (input, maybe_properties) = opt(link_properties)(input)?;

    Ok((
        input,
        LinkData::new(uri_ref, maybe_description, maybe_properties),
    ))
}

/// Extracts the uri-portion of a link, supporting converting spaces into
/// %20 encoded characters
///
/// Can either be a text description OR an embeded {{...}} transclusion link
fn link_uri_ref<'a>(input: Span<'a>) -> IResult<URIReference<'a>> {
    /// https://url.spec.whatwg.org/#fragment-percent-encode-set
    const FRAGMENT: &AsciiSet =
        &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

    let (input, uri_span) =
        take_line_until_one_of_three1("|", "]]", "}}")(input)?;

    match URIReference::try_from(uri_span) {
        Ok(uri_ref) => Ok((input, uri_ref)),
        Err(_) => {
            let encoded_uri_str =
                percent_encode(uri_span.as_remaining(), FRAGMENT).to_string();
            let uri_ref = URIReference::try_from(encoded_uri_str.as_str())
                .map_err(|x| {
                    use nom::error::FromExternalError;
                    nom::Err::Error(Error::from_external_error(
                        uri_span,
                        nom::error::ErrorKind::MapRes,
                        x,
                    ))
                })?
                .into_owned();
            Ok((input, uri_ref))
        }
    }
}

/// Extracts the description-portion of a link
///
/// Can either be a text description OR an embeded {{...}} transclusion link
fn link_description<'a>(input: Span<'a>) -> IResult<Description<'a>> {
    map_parser(
        take_line_until_one_of_three1("|", "]]", "}}"),
        alt((
            map(transclusion::transclusion_link, |l| {
                Description::from(l.into_inner().into_data())
            }),
            map(rest, |s: Span| Description::Text(s.into())),
        )),
    )(input)
}

/// Parser for link property pairs separated by | in the form of
///
/// key1="value1"|key2="value2"|...
fn link_properties<'a>(
    input: Span<'a>,
) -> IResult<HashMap<Cow<'a, str>, Cow<'a, str>>> {
    map(
        separated_list1(
            tag("|"),
            map_parser(
                take_line_until_one_of_three1("|", "]]", "}}"),
                separated_pair(
                    map_parser(take_line_until1("="), cow_str),
                    tag("="),
                    map_parser(
                        delimited(tag("\""), take_line_until("\""), tag("\"")),
                        cow_str,
                    ),
                ),
            ),
        ),
        |mut pairs| pairs.drain(..).collect(),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_should_return_diary_link_where_appropriate() {
        let input = Span::from("[[diary:2012-03-05]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::Diary { .. }));
    }

    #[test]
    fn link_should_return_indexed_interwiki_link_where_appropriate() {
        let input = Span::from("[[wiki1:Some Link]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::IndexedInterWiki { .. }));
    }

    #[test]
    fn link_should_return_named_interwiki_link_where_appropriate() {
        let input = Span::from("[[wn.My Name:Some Link]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::NamedInterWiki { .. }));
    }

    #[test]
    fn link_should_return_wiki_link_where_appropriate() {
        let input = Span::from("[[Some Link]]");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::Wiki { .. }));
    }

    #[test]
    fn link_should_return_raw_link_where_appropriate() {
        let input = Span::from("https://example.com");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::Raw { .. }));
    }

    #[test]
    fn link_should_return_transclusion_link_where_appropriate() {
        let input = Span::from("{{https://example.com/img.jpg}}");
        let (_, l) = link(input).unwrap();
        assert!(matches!(l.into_inner(), Link::Transclusion { .. }));
    }
}
