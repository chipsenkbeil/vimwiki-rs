use super::{
    components::{Description, TransclusionLink},
    utils::{position, take_line_while, take_line_while1},
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, map_parser, map_res, not, opt},
    multi::separated_nonempty_list,
    sequence::{delimited, preceded, separated_pair},
};
use std::collections::HashMap;
use url::Url;

#[inline]
pub fn transclusion_link(input: Span) -> VimwikiIResult<LC<TransclusionLink>> {
    let (input, pos) = position(input)?;

    let (input, _) = tag("{{")(input)?;
    let (input, link_url) = map_res(
        take_line_while1(not(alt((tag("|"), tag("}}"))))),
        |s: Span| Url::parse(s.fragment()),
    )(input)?;
    let (input, maybe_description) = opt(map(
        preceded(tag("|"), take_line_while(not(alt((tag("|"), tag("}}")))))),
        |s: Span| Description::from(s.fragment().to_string()),
    ))(input)?;
    let (input, maybe_properties) =
        opt(preceded(tag("|"), transclusion_properties))(input)?;

    Ok((
        input,
        LC::from((
            TransclusionLink::new(
                link_url,
                maybe_description,
                maybe_properties.unwrap_or_default(),
            ),
            pos,
            input,
        )),
    ))
}

/// Parser for property pairs separated by | in the form of
///
///     key1="value1"|key2="value2"|...
#[inline]
fn transclusion_properties(
    input: Span,
) -> VimwikiIResult<HashMap<String, String>> {
    map(
        separated_nonempty_list(
            tag("|"),
            map_parser(
                take_line_while1(not(alt((tag("|"), tag("}}"))))),
                separated_pair(
                    map(take_line_while1(not(tag("="))), |s: Span| {
                        s.fragment().to_string()
                    }),
                    tag("="),
                    map(
                        delimited(
                            tag("\""),
                            take_line_while(not(tag("\""))),
                            tag("\""),
                        ),
                        |s: Span| s.fragment().to_string(),
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
