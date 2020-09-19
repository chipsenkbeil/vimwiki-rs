use super::{
    components::{Description, TransclusionLink},
    utils::{context, lc, take_line_while, take_line_while1},
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
use std::convert::TryFrom;
use uriparse::URI;

#[inline]
pub fn transclusion_link(input: Span) -> VimwikiIResult<LC<TransclusionLink>> {
    fn inner(input: Span) -> VimwikiIResult<TransclusionLink> {
        let (input, _) = tag("{{")(input)?;
        let (input, link_uri) = map_res(
            take_line_while1(not(alt((tag("|"), tag("}}"))))),
            |s: Span| {
                URI::try_from(s.fragment_str()).map(|uri| uri.into_owned())
            },
        )(input)?;
        let (input, maybe_description) = opt(map(
            preceded(
                tag("|"),
                take_line_while(not(alt((tag("|"), tag("}}"))))),
            ),
            |s: Span| Description::from(s.fragment_str()),
        ))(input)?;
        let (input, maybe_properties) =
            opt(preceded(tag("|"), transclusion_properties))(input)?;
        let (input, _) = tag("}}")(input)?;

        Ok((
            input,
            TransclusionLink::new(
                link_uri,
                maybe_description,
                maybe_properties.unwrap_or_default(),
            ),
        ))
    }

    context("Transclusion Link", lc(inner))(input)
}

/// Parser for property pairs separated by | in the form of
///
/// key1="value1"|key2="value2"|...
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
                        s.fragment_str().to_string()
                    }),
                    tag("="),
                    map(
                        delimited(
                            tag("\""),
                            take_line_while(not(tag("\""))),
                            tag("\""),
                        ),
                        |s: Span| s.fragment_str().to_string(),
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
    fn transclusion_link_should_support_local_relative_uri() {
        let input = Span::from("{{file:../../images/vimwiki_logo.png}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.uri.scheme().as_str(), "file");
        assert_eq!(link.uri.path(), "../../images/vimwiki_logo.png");
        assert_eq!(link.description, None);
        assert!(link.properties.is_empty(), "Unexpectedly found property");
    }

    #[test]
    fn transclusion_link_should_support_local_absolute_uri() {
        let input = Span::from("{{file:/some/path/images/vimwiki_logo.png}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.uri.scheme().as_str(), "file");
        assert_eq!(link.uri.path(), "/some/path/images/vimwiki_logo.png");
        assert_eq!(link.description, None);
        assert!(link.properties.is_empty(), "Unexpectedly found property");
    }

    #[test]
    fn transclusion_link_should_support_universal_uri() {
        let input = Span::from(
            "{{http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png}}",
        );
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.uri.scheme().as_str(), "http");
        assert_eq!(
            link.uri.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.uri.path(), "/hg/images/vimwiki_logo.png");
        assert_eq!(link.description, None);
        assert!(link.properties.is_empty(), "Unexpectedly found property");
    }

    #[test]
    fn transclusion_link_should_support_alternate_text() {
        // maps to in HTML
        //
        // <img src="http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png"
        // alt="Vimwiki"/>
        //
        let input = Span::from("{{http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png|Vimwiki}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.uri.scheme().as_str(), "http");
        assert_eq!(
            link.uri.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.uri.path(), "/hg/images/vimwiki_logo.png");
        assert_eq!(link.description, Some(Description::from("Vimwiki")));
        assert!(link.properties.is_empty(), "Unexpectedly found property");
    }

    #[test]
    fn transclusion_link_should_support_alternate_text_and_style() {
        // in HTML:
        //
        // <img src="http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png"
        // alt="cool stuff" style="width:150px; height:120px"/>
        //
        let input = Span::from("{{http://vimwiki.googlecode.com/vimwiki_logo.png|cool stuff|style=\"width:150px;height:120px;\"}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.uri.scheme().as_str(), "http");
        assert_eq!(
            link.uri.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.uri.path(), "/vimwiki_logo.png");
        assert_eq!(link.description, Some(Description::from("cool stuff")));
        assert_eq!(
            link.properties,
            vec![(
                "style".to_string(),
                "width:150px;height:120px;".to_string()
            )]
            .drain(..)
            .collect()
        );
    }

    #[test]
    fn transclusion_link_should_support_css_class_without_alternate_text() {
        // in HTML:
        //
        // <img src="http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png"
        // alt="" class="center flow blabla"/>
        //
        let input = Span::from(
            "{{http://vimwiki.googlecode.com/vimwiki_logo.png||class=\"center flow blabla\"}}",
        );
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.uri.scheme().as_str(), "http");
        assert_eq!(
            link.uri.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.uri.path(), "/vimwiki_logo.png");
        assert_eq!(link.description, Some(Description::from("")));
        assert_eq!(
            link.properties,
            vec![("class".to_string(), "center flow blabla".to_string())]
                .drain(..)
                .collect()
        );
    }
}
