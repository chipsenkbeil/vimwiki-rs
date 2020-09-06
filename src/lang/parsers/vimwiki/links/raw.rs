use super::{
    components::RawLink,
    utils::{position, url},
    Span, VimwikiIResult, LC,
};

#[inline]
pub fn raw_link(input: Span) -> VimwikiIResult<LC<RawLink>> {
    let (input, pos) = position(input)?;

    let (input, url) = url(input)?;

    Ok((input, LC::from((RawLink::from(url), pos, input))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn raw_link_should_support_http_scheme() {
        let input = Span::new("http://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse url");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.url.scheme(), "http");
        assert_eq!(link.url.host_str(), Some("example.com"));
    }

    #[test]
    fn raw_link_should_support_https_scheme() {
        let input = Span::new("https://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse url");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.url.scheme(), "https");
        assert_eq!(link.url.host_str(), Some("example.com"));
    }

    #[test]
    fn raw_link_should_support_no_scheme_with_www() {
        let input = Span::new("www.example.com");
        let (input, link) = raw_link(input).expect("Failed to parse url");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.url.scheme(), "https");
        assert_eq!(link.url.host_str(), Some("www.example.com"));
    }

    #[test]
    fn raw_link_should_support_ftp_scheme() {
        let input = Span::new("ftp://example.com");
        let (input, link) = raw_link(input).expect("Failed to parse url");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.url.scheme(), "ftp");
        assert_eq!(link.url.host_str(), Some("example.com"));
    }

    #[test]
    fn raw_link_should_support_file_scheme() {
        let input = Span::new("file:///some/path");
        let (input, link) = raw_link(input).expect("Failed to parse url");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.url.scheme(), "file");
        assert_eq!(
            link.url.to_file_path().unwrap(),
            PathBuf::from("/some/path")
        );
    }

    #[test]
    fn raw_link_should_support_local_scheme() {
        let input = Span::new("local:///some/path");
        let (input, link) = raw_link(input).expect("Failed to parse url");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.url.scheme(), "local");
        assert_eq!(
            link.url.to_file_path().unwrap(),
            PathBuf::from("/some/path")
        );
    }

    #[test]
    fn raw_link_should_support_mailto_scheme() {
        let input = Span::new("mailto:person@example.com");
        let (input, link) = raw_link(input).expect("Failed to parse url");

        // Link should be consumed
        assert!(input.fragment().is_empty());

        assert_eq!(link.url.scheme(), "mailto");
        assert_eq!(link.url.path(), "person@example.com");
    }
}
