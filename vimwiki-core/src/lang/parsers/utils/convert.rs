use super::{context, single_multispace};
use crate::lang::{
    elements::{ElementLike, Located, Region},
    parsers::{Captured, IResult, Span},
};
use nom::{
    character::complete::anychar,
    combinator::{map_res, not, recognize},
    multi::many1,
    sequence::pair,
};
use std::{borrow::Cow, convert::TryFrom};
use uriparse::URIReference;

/// Parser that wraps a span in a deeper depth
pub fn deeper<'a, T>(
    mut parser: impl FnMut(Span<'a>) -> IResult<T>,
) -> impl FnMut(Span<'a>) -> IResult<T> {
    context("Deeper", move |input: Span<'a>| {
        let (input, x) = parser(input.with_deeper_depth())?;
        Ok((input.with_shallower_depth(), x))
    })
}

/// Parser that transforms a `Captured<T>` to a `Located<T>`.
///
/// If the feature "location" is enabled, this will also compute the line
/// and column information for the captured input.
pub fn locate<'a, T>(
    mut parser: impl FnMut(Span<'a>) -> IResult<Captured<T>>,
) -> impl FnMut(Span<'a>) -> IResult<Located<T>>
where
    T: ElementLike,
{
    context("Locate", move |input: Span<'a>| {
        let (input, c) = parser(input)?;
        let region = Region::from(c.input());
        Ok((input, Located::new(c.into_inner(), region)))
    })
}

/// Parser that captures the input used to create the output of provided the parser
pub fn capture<'a, T>(
    mut parser: impl FnMut(Span<'a>) -> IResult<T>,
) -> impl FnMut(Span<'a>) -> IResult<Captured<T>> {
    context("Capture", move |input: Span<'a>| {
        let start = input;
        let (input, x) = parser(input)?;
        let start =
            start.with_length(input.start_offset() - start.start_offset());

        Ok((input, Captured::new(x, start)))
    })
}

/// Parser that transforms the input to that of `Cow<'a, str>`
/// where the lifetime is bound to the resulting `Span<'a>`
pub fn cow_str<'a>(input: Span<'a>) -> IResult<Cow<'a, str>> {
    Ok((input, input.into()))
}

/// Parser for a general purpose URI Reference. Will consume until whitespace
/// is encountered as unescaped whitespace is not part of a URI.
///
/// ### Regular cases
///
/// 1. https (https://example.com)
/// 2. http (http://example.com)
/// 3. ftp (ftp:)
/// 4. file (file:relative/path)
/// 5. local (local:relative/path)
/// 6. mailto (mailto:someone@example.com)
///
/// ### Special cases
///
/// 1. www (www.example.com) -> (https://www.example.com)
/// 2. // (//some/abs/path) -> (file:/some/abs/path)
pub fn uri_ref<'a>(input: Span<'a>) -> IResult<URIReference<'a>> {
    // TODO: Support special cases, which involves allocating a new string
    //       or providing some alternative structure to a URI
    context(
        "Normal URI Reference",
        map_res(
            recognize(many1(pair(not(single_multispace), anychar))),
            URIReference::try_from,
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::bytes::complete::tag;

    #[derive(Debug, PartialEq, Eq)]
    struct FakeElement(u32);
    impl ElementLike for FakeElement {}

    #[test]
    fn locate_should_return_parser_result_with_consumed_input_location() {
        let input = Span::from("123abc");
        let (input, located) =
            locate(capture(map_res(tag("123"), |s: Span| {
                s.as_unsafe_remaining_str().parse::<u32>().map(FakeElement)
            })))(input)
            .unwrap();
        assert_eq!(input, "abc");
        assert_eq!(located.region().offset(), 0);
        assert_eq!(located.region().len(), 3);
        assert_eq!(located.into_inner(), FakeElement(123));
    }

    #[test]
    fn capture_should_return_parser_result_with_consumed_input() {
        let input = Span::from("123abc");
        let (input, captured) = capture(map_res(tag("123"), |s: Span| {
            s.as_unsafe_remaining_str().parse::<u32>().map(FakeElement)
        }))(input)
        .unwrap();
        assert_eq!(input, "abc");
        assert_eq!(captured.input(), "123");
        assert_eq!(captured.into_inner(), FakeElement(123));
    }

    #[test]
    fn cow_str_should_return_input_as_cow_str() {
        let input = Span::from("abc");
        let (input, result) = cow_str(input).unwrap();
        assert_eq!(input, "abc");
        assert_eq!(result, Cow::from("abc"));
    }

    #[test]
    fn uri_ref_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(uri_ref(input).is_err());
    }

    #[test]
    fn uri_ref_should_succeed_even_if_no_scheme_or_subdomain_as_long_as_url() {
        let input = Span::from("example.com");
        let (input, u) = uri_ref(input).expect("Failed to parse uri ref");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), None);
        assert_eq!(u.host(), None);
        assert_eq!(u.path(), "example.com");
    }

    #[test]
    fn uri_ref_should_succeed_even_if_no_scheme_as_long_as_url() {
        let input = Span::from("www.example.com");
        let (input, u) = uri_ref(input).expect("Failed to parse uri ref");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), None);
        assert_eq!(u.host(), None);
        assert_eq!(u.path(), "www.example.com");
    }

    #[test]
    fn uri_ref_should_succeed_if_starts_with_network_path() {
        let input = Span::from("//some/network/path");
        let (input, u) = uri_ref(input).expect("Failed to parse uri ref");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), None);
        assert_eq!(u.host().map(ToString::to_string), Some("some".to_string()));
        assert_eq!(u.path(), "/network/path");
    }

    #[test]
    fn uri_ref_should_succeed_if_starts_with_scheme() {
        let input = Span::from("https://github.com/vimwiki/vimwiki.git");
        let (input, u) = uri_ref(input).expect("Failed to parse uri ref");
        assert!(input.is_empty());
        assert_eq!(u.scheme().unwrap(), "https");
        assert_eq!(u.host().unwrap().to_string(), "github.com");
        assert_eq!(u.path(), "/vimwiki/vimwiki.git");

        let input = Span::from("mailto:habamax@gmail.com");
        let (input, u) = uri_ref(input).expect("Failed to parse uri ref");
        assert!(input.is_empty());
        assert_eq!(u.scheme().unwrap(), "mailto");
        assert_eq!(u.path(), "habamax@gmail.com");

        let input = Span::from("ftp://vim.org");
        let (input, u) = uri_ref(input).expect("Failed to parse uri ref");
        assert!(input.is_empty());
        assert_eq!(u.scheme().unwrap(), "ftp");
        assert_eq!(u.host().unwrap().to_string(), "vim.org");
    }
}
