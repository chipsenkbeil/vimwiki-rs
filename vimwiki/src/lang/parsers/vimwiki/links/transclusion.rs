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
    let (input, _) = tag("}}")(input)?;

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
    #[ignore]
    fn transclusion_link_should_support_local_relative_url() {
        let input = Span::new("{{file:../../images/vimwiki_logo.png}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.url.scheme(), "file");

        // Currently failing due to not handling relative URLs as expected:
        //
        // - https://github.com/servo/rust-url/issues/641
        // - https://github.com/vimwiki/vimwiki/issues/989#issuecomment-687900789
        //
        // Given that the Url class is following the URL spec instead of URI,
        // the path is being resolved inline. Couple of options are as follows:
        //
        // 1. Use uriparse-rs (it doesn't support serde and hasn't had a
        //    stable release, only nightly)
        // 2. Provide a wrapper class around Url that captures the raw input
        //    and can provide it back along with the Url (maybe even trim
        //    the scheme on the front)
        // 3. Store a PathBuf for specific schemes like file: and local: and
        //    have different handling throughout
        assert_eq!(link.url.as_str(), "../../images/vimwiki_logo.png");
        assert_eq!(link.description, None);
        assert!(link.properties.is_empty(), "Unexpectedly found property");
    }

    #[test]
    fn transclusion_link_should_support_local_absolute_url() {
        let input = Span::new("{{file:/some/path/images/vimwiki_logo.png}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.url.scheme(), "file");
        assert_eq!(link.url.path(), "/some/path/images/vimwiki_logo.png");
        assert_eq!(link.description, None);
        assert!(link.properties.is_empty(), "Unexpectedly found property");
    }

    #[test]
    fn transclusion_link_should_support_universal_url() {
        let input = Span::new(
            "{{http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png}}",
        );
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.url.scheme(), "http");
        assert_eq!(link.url.host_str(), Some("vimwiki.googlecode.com"));
        assert_eq!(link.url.path(), "/hg/images/vimwiki_logo.png");
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
        let input = Span::new("{{http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png|Vimwiki}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.url.scheme(), "http");
        assert_eq!(link.url.host_str(), Some("vimwiki.googlecode.com"));
        assert_eq!(link.url.path(), "/hg/images/vimwiki_logo.png");
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
        let input = Span::new("{{http://vimwiki.googlecode.com/vimwiki_logo.png|cool stuff|style=\"width:150px;height:120px;\"}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.url.scheme(), "http");
        assert_eq!(link.url.host_str(), Some("vimwiki.googlecode.com"));
        assert_eq!(link.url.path(), "/vimwiki_logo.png");
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
        let input = Span::new(
            "{{http://vimwiki.googlecode.com/vimwiki_logo.png||class=\"center flow blabla\"}}",
        );
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.url.scheme(), "http");
        assert_eq!(link.url.host_str(), Some("vimwiki.googlecode.com"));
        assert_eq!(link.url.path(), "/vimwiki_logo.png");
        assert_eq!(link.description, Some(Description::from("")));
        assert_eq!(
            link.properties,
            vec![("class".to_string(), "center flow blabla".to_string())]
                .drain(..)
                .collect()
        );
    }
}
