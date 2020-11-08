use super::{context, single_multispace};
use crate::lang::{
    elements::{Located, Region},
    parsers::{Captured, IResult, Span},
};
use nom::{
    bytes::complete::{tag, take_while},
    character::complete::anychar,
    combinator::{map_res, not, recognize},
    multi::many1,
    sequence::{pair, terminated},
};
use std::{borrow::Cow, convert::TryFrom, path::Path};
use uriparse::URI;

/// Parser that transforms a `Captured<T>` to a `Located<T>`.
///
/// If the feature "location" is enabled, this will also compute the line
/// and column information for the captured input.
pub fn locate<'a, T>(
    parser: impl Fn(Span<'a>) -> IResult<Captured<T>>,
) -> impl Fn(Span<'a>) -> IResult<Located<T>> {
    context("Locate", move |input: Span| {
        let (input, c) = parser(input)?;

        // If enabled, we construct a region that includes line & column
        // information, otherwise we construct a region with just the offset
        // and length of the span
        let region = if cfg!(feature = "location") {
            Region::from_span_with_position(c.input())
        } else {
            Region::from(c.input())
        };

        Ok((input, Located::new(c.into_inner(), region)))
    })
}

/// Parser that captures the input used to create the output of provided the parser
pub fn capture<'a, T>(
    parser: impl Fn(Span<'a>) -> IResult<T>,
) -> impl Fn(Span<'a>) -> IResult<Captured<T>> {
    context("Capture", move |input: Span| {
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

/// Parser that transforms the input to that of `Cow<'a, Path>`
/// where the lifetime is bound to the resulting `Span<'a>`
pub fn cow_path<'a>(input: Span<'a>) -> IResult<Cow<'a, Path>> {
    Ok((input, input.into()))
}

/// Parser for a general purpose URI.
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
pub fn uri<'a>(input: Span<'a>) -> IResult<URI<'a>> {
    // TODO: Support special cases, which involves allocating a new string
    //       or providing some alternative structure to a URI
    context(
        "Normal URI",
        map_res(
            recognize(pair(
                uri_scheme,
                many1(pair(not(single_multispace), anychar)),
            )),
            URI::try_from,
        ),
    )(input)
}

fn uri_scheme<'a>(input: Span<'a>) -> IResult<Span<'a>> {
    context(
        "URI Scheme",
        terminated(
            take_while(|b: u8| {
                let c = char::from(b);
                c.is_alphanumeric() || c == '+' || c == '.' || c == '-'
            }),
            tag(":"),
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn locate_should_return_parser_result_with_consumed_input_location() {
        let input = Span::from("123abc");
        let (input, located) =
            locate(capture(map_res(tag("123"), |s: Span| {
                s.as_unsafe_remaining_str().parse::<u32>()
            })))(input)
            .unwrap();
        assert_eq!(input, "abc");
        assert_eq!(located.region().offset(), 0);
        assert_eq!(located.region().len(), 3);
        assert_eq!(located.into_inner(), 123);
    }

    #[test]
    fn capture_should_return_parser_result_with_consumed_input() {
        let input = Span::from("123abc");
        let (input, captured) = capture(map_res(tag("123"), |s: Span| {
            s.as_unsafe_remaining_str().parse::<u32>()
        }))(input)
        .unwrap();
        assert_eq!(input, "abc");
        assert_eq!(captured.input(), "123");
        assert_eq!(captured.into_inner(), 123);
    }

    #[test]
    fn cow_str_should_return_input_as_cow_str() {
        let input = Span::from("abc");
        let (input, result) = cow_str(input).unwrap();
        assert_eq!(input, "abc");
        assert_eq!(result, Cow::from("abc"));
    }

    #[test]
    fn cow_path_should_return_input_as_cow_path() {
        let input = Span::from("abc");
        let (input, result) = cow_path(input).unwrap();
        assert_eq!(input, "abc");
        assert_eq!(result, Cow::from(PathBuf::from("abc")));
    }

    #[test]
    fn uri_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(uri(input).is_err());
    }

    #[test]
    fn uri_should_fail_if_no_scheme_and_not_www_or_absolute_path() {
        let input = Span::from("example.com");
        assert!(uri(input).is_err());
    }

    #[test]
    #[ignore]
    fn uri_should_succeed_if_starts_with_www_and_will_add_https_as_scheme() {
        let input = Span::from("www.example.com");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "https");
        assert_eq!(u.host().unwrap().to_string(), "www.example.com");
    }

    #[test]
    #[ignore]
    fn uri_should_succeed_if_starts_with_absolute_path_and_will_add_file_as_scheme(
    ) {
        let input = Span::from("//some/absolute/path");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "file");
        assert_eq!(u.path(), "/some/absolute/path");
    }

    #[test]
    fn uri_should_succeed_if_starts_with_scheme() {
        let input = Span::from("https://github.com/vimwiki/vimwiki.git");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "https");
        assert_eq!(u.host().unwrap().to_string(), "github.com");
        assert_eq!(u.path(), "/vimwiki/vimwiki.git");

        let input = Span::from("mailto:habamax@gmail.com");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "mailto");
        assert_eq!(u.path(), "habamax@gmail.com");

        let input = Span::from("ftp://vim.org");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.is_empty());
        assert_eq!(u.scheme(), "ftp");
        assert_eq!(u.host().unwrap().to_string(), "vim.org");
    }
}
