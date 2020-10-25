use crate::lang::{
    elements::{Description, Located, TransclusionLink},
    parsers::{
        utils::{
            capture, context, cow_str, locate, take_line_until,
            take_line_until1, take_line_until_one_of_two,
            take_line_until_one_of_two1, uri,
        },
        IResult, Span,
    },
};
use nom::{
    bytes::complete::tag,
    combinator::{map, map_parser, opt},
    multi::separated_nonempty_list,
    sequence::{delimited, preceded, separated_pair},
};
use std::{borrow::Cow, collections::HashMap};

pub fn transclusion_link(input: Span) -> IResult<Located<TransclusionLink>> {
    fn inner(input: Span) -> IResult<TransclusionLink> {
        let (input, _) = tag("{{")(input)?;
        let (input, link_uri) =
            map_parser(take_line_until_one_of_two1("|", "}}"), uri)(input)?;
        let (input, maybe_description) = opt(map(
            map_parser(
                preceded(tag("|"), take_line_until_one_of_two("|", "}}")),
                cow_str,
            ),
            Description::from,
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

    context("Transclusion Link", locate(capture(inner)))(input)
}

/// Parser for property pairs separated by | in the form of
///
/// key1="value1"|key2="value2"|...
fn transclusion_properties<'a>(
    input: Span<'a>,
) -> IResult<HashMap<Cow<'a, str>, Cow<'a, str>>> {
    map(
        separated_nonempty_list(
            tag("|"),
            map_parser(
                take_line_until_one_of_two1("|", "}}"),
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
    fn transclusion_link_should_support_local_relative_uri() {
        let input = Span::from("{{file:../../images/vimwiki_logo.png}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.uri.scheme().as_str(), "file");
        assert_eq!(link.uri.path(), "../../images/vimwiki_logo.png");
        assert_eq!(link.description, None);
        assert!(link.properties.is_empty(), "Unexpectedly found property");
    }

    #[test]
    fn transclusion_link_should_support_local_absolute_uri() {
        let input = Span::from("{{file:/some/path/images/vimwiki_logo.png}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
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
        assert!(input.is_empty(), "Did not consume link");
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
        assert!(input.is_empty(), "Did not consume link");
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
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.uri.scheme().as_str(), "http");
        assert_eq!(
            link.uri.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.uri.path(), "/vimwiki_logo.png");
        assert_eq!(link.description, Some(Description::from("cool stuff")));
        assert_eq!(
            link.properties,
            vec![(Cow::from("style"), Cow::from("width:150px;height:120px;"))]
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
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.uri.scheme().as_str(), "http");
        assert_eq!(
            link.uri.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.uri.path(), "/vimwiki_logo.png");
        assert_eq!(link.description, Some(Description::from("")));
        assert_eq!(
            link.properties,
            vec![(Cow::from("class"), Cow::from("center flow blabla"))]
                .drain(..)
                .collect()
        );
    }
}
