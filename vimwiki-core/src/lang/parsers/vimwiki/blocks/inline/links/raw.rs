use crate::lang::{
    elements::{Link, Located},
    parsers::{
        utils::{capture, context, locate, uri_ref},
        IResult, Span,
    },
};
use nom::combinator::verify;

pub fn raw_link(input: Span) -> IResult<Located<Link>> {
    fn inner(input: Span) -> IResult<Link> {
        // This will match any URI, but we only want to allow a certain set
        // to ensure that we don't mistake some text preceding a tag
        //
        // NOTE: We don't use link_uri_ref because we don't want to auto-escape
        //       spaces or other characters. For raw links, that is up to the
        //       user to do so
        let (input, uri_ref) = verify(uri_ref, |uri_ref| {
            uri_ref.scheme().map_or(false, |scheme| {
                ["http", "https", "ftp", "file", "local", "mailto"]
                    .contains(&scheme.as_str())
            })
        })(input)?;

        Ok((input, Link::new_raw_link(uri_ref)))
    }

    context("Raw Link", locate(capture(inner)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_link_should_support_http_scheme() {
        let input = Span::from("http://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.scheme().unwrap(), "http");
        assert_eq!(
            link.data().uri_ref.host().unwrap().to_string(),
            "example.com"
        );
    }

    #[test]
    fn raw_link_should_support_https_scheme() {
        let input = Span::from("https://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.scheme().unwrap(), "https");
        assert_eq!(
            link.data().uri_ref.host().unwrap().to_string(),
            "example.com"
        );
    }

    #[test]
    #[ignore]
    fn raw_link_should_support_no_scheme_with_www() {
        let input = Span::from("www.example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.scheme().unwrap(), "https");
        assert_eq!(
            link.data().uri_ref.host().unwrap().to_string(),
            "www.example.com"
        );
    }

    #[test]
    fn raw_link_should_support_ftp_scheme() {
        let input = Span::from("ftp://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.scheme().unwrap(), "ftp");
        assert_eq!(
            link.data().uri_ref.host().unwrap().to_string(),
            "example.com"
        );
    }

    #[test]
    fn raw_link_should_support_file_scheme() {
        let input = Span::from("file:///some/path");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.scheme().unwrap(), "file");
        assert_eq!(link.data().uri_ref.path(), "/some/path");
    }

    #[test]
    fn raw_link_should_support_local_scheme() {
        let input = Span::from("local:///some/path");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.scheme().unwrap(), "local");
        assert_eq!(link.data().uri_ref.path(), "/some/path");
    }

    #[test]
    fn raw_link_should_support_mailto_scheme() {
        let input = Span::from("mailto:person@example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.scheme().unwrap(), "mailto");
        assert_eq!(link.data().uri_ref.path(), "person@example.com");
    }
}
