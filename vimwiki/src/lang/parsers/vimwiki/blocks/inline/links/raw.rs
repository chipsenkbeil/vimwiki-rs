use crate::lang::{
    elements::{Located, RawLink},
    parsers::{
        utils::{capture, context, locate, uri},
        IResult, Span,
    },
};
use nom::combinator::verify;

#[inline]
pub fn raw_link(input: Span) -> IResult<Located<RawLink>> {
    fn inner(input: Span) -> IResult<RawLink> {
        // This will match any URI, but we only want to allow a certain set
        // to ensure that we don't mistake some text preceding a tag
        let (input, uri) = verify(uri, |uri| {
            vec!["http", "https", "ftp", "file", "local", "mailto"]
                .contains(&uri.scheme().as_str())
        })(input)?;

        Ok((input, RawLink::from(uri)))
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

        assert_eq!(link.uri.scheme(), "http");
        assert_eq!(link.uri.host().unwrap().to_string(), "example.com");
    }

    #[test]
    fn raw_link_should_support_https_scheme() {
        let input = Span::from("https://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.uri.scheme(), "https");
        assert_eq!(link.uri.host().unwrap().to_string(), "example.com");
    }

    #[test]
    #[ignore]
    fn raw_link_should_support_no_scheme_with_www() {
        let input = Span::from("www.example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.uri.scheme(), "https");
        assert_eq!(link.uri.host().unwrap().to_string(), "www.example.com");
    }

    #[test]
    fn raw_link_should_support_ftp_scheme() {
        let input = Span::from("ftp://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.uri.scheme(), "ftp");
        assert_eq!(link.uri.host().unwrap().to_string(), "example.com");
    }

    #[test]
    fn raw_link_should_support_file_scheme() {
        let input = Span::from("file:///some/path");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.uri.scheme(), "file");
        assert_eq!(link.uri.path(), "/some/path");
    }

    #[test]
    fn raw_link_should_support_local_scheme() {
        let input = Span::from("local:///some/path");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.uri.scheme(), "local");
        assert_eq!(link.uri.path(), "/some/path");
    }

    #[test]
    fn raw_link_should_support_mailto_scheme() {
        let input = Span::from("mailto:person@example.com");
        let (input, link) = raw_link(input).expect("Failed to parse uri");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.uri.scheme(), "mailto");
        assert_eq!(link.uri.path(), "person@example.com");
    }
}
