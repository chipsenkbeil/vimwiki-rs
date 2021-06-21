use super::link_data;
use crate::lang::{
    elements::{Link, Located},
    parsers::{
        utils::{capture, context, locate, not_contains, surround_in_line1},
        IResult, Span,
    },
};
use nom::combinator::map_parser;

pub fn transclusion_link(input: Span) -> IResult<Located<Link>> {
    fn inner(input: Span) -> IResult<Link> {
        let (input, data) = link_data(input)?;
        Ok((input, Link::Transclusion { data }))
    }

    context(
        "Transclusion Link",
        locate(capture(map_parser(
            not_contains("%%", surround_in_line1("{{", "}}")),
            inner,
        ))),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Description;
    use std::borrow::Cow;

    #[test]
    fn transclusion_link_should_support_local_relative_uri() {
        let input = Span::from("{{file:../../images/vimwiki_logo.png}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme().unwrap(), "file");
        assert_eq!(link.data().uri_ref.path(), "../../images/vimwiki_logo.png");
        assert_eq!(link.description(), None);
        assert!(link.properties().is_none(), "Unexpectedly found property");
    }

    #[test]
    fn transclusion_link_should_support_local_absolute_uri() {
        let input = Span::from("{{file:/some/path/images/vimwiki_logo.png}}");
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme().unwrap(), "file");
        assert_eq!(
            link.data().uri_ref.path(),
            "/some/path/images/vimwiki_logo.png"
        );
        assert_eq!(link.description(), None);
        assert!(link.properties().is_none(), "Unexpectedly found property");
    }

    #[test]
    fn transclusion_link_should_support_universal_uri() {
        let input = Span::from(
            "{{http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png}}",
        );
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme().unwrap(), "http");
        assert_eq!(
            link.data().uri_ref.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.data().uri_ref.path(), "/hg/images/vimwiki_logo.png");
        assert_eq!(link.description(), None);
        assert!(link.properties().is_none(), "Unexpectedly found property");
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
        assert_eq!(link.scheme().unwrap(), "http");
        assert_eq!(
            link.data().uri_ref.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.data().uri_ref.path(), "/hg/images/vimwiki_logo.png");
        assert_eq!(link.description(), Some(&Description::from("Vimwiki")));
        assert!(link.properties().is_none(), "Unexpectedly found property");
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
        assert_eq!(link.scheme().unwrap(), "http");
        assert_eq!(
            link.data().uri_ref.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.data().uri_ref.path(), "/vimwiki_logo.png");
        assert_eq!(link.description(), Some(&Description::from("cool stuff")));
        assert_eq!(
            link.properties(),
            Some(
                &vec![(
                    Cow::from("style"),
                    Cow::from("width:150px;height:120px;")
                )]
                .drain(..)
                .collect()
            )
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
        assert_eq!(link.scheme().unwrap(), "http");
        assert_eq!(
            link.data().uri_ref.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.data().uri_ref.path(), "/vimwiki_logo.png");
        assert_eq!(link.description(), Some(&Description::from("")));
        assert_eq!(
            link.properties(),
            Some(
                &vec![(Cow::from("class"), Cow::from("center flow blabla"))]
                    .drain(..)
                    .collect()
            )
        );
    }

    #[test]
    fn transclusion_link_should_support_multiple_properties() {
        // in HTML:
        //
        // <img src="http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png"
        // alt="" class="center flow blabla"/>
        //
        let input = Span::from(
            "{{http://vimwiki.googlecode.com/vimwiki_logo.png||class=\"center flow blabla\" style=\"something\"}}",
        );
        let (input, link) = transclusion_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme().unwrap(), "http");
        assert_eq!(
            link.data().uri_ref.host().unwrap().to_string(),
            "vimwiki.googlecode.com"
        );
        assert_eq!(link.data().uri_ref.path(), "/vimwiki_logo.png");
        assert_eq!(link.description(), Some(&Description::from("")));

        assert_eq!(
            link.data().get_property_str("class"),
            Some("center flow blabla")
        );
        assert_eq!(link.data().get_property_str("style"), Some("something"));
    }
}
