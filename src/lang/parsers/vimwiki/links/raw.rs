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

    #[test]
    fn raw_link_should_support_http_scheme() {
        // http://example.com
        todo!();
    }

    #[test]
    fn raw_link_should_support_https_scheme() {
        // https://example.com
        todo!();
    }

    #[test]
    fn raw_link_should_support_no_scheme_with_www() {
        // www.example.com -> https://www.example.com
        todo!();
    }

    #[test]
    fn raw_link_should_support_ftp_scheme() {
        // ftp://example.com
        todo!();
    }

    #[test]
    fn raw_link_should_support_file_scheme() {
        // file://some/path
        todo!();
    }

    #[test]
    fn raw_link_should_support_local_scheme() {
        // local://some/path
        todo!();
    }

    #[test]
    fn raw_link_should_support_mailto_scheme() {
        // mailto:person@example.com
        todo!();
    }
}
