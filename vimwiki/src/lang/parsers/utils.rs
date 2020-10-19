use crate::lang::{
    elements::{Located, Region},
    parsers::{Captured, Error, IResult, Span},
};
use memchr::{memchr, memchr_iter};
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while},
    character::complete::{anychar, crlf, line_ending, space0, space1},
    combinator::{map, map_res, not, recognize, rest, rest_len, value, verify},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
    AsBytes, InputLength, InputTake,
};
use std::{borrow::Cow, convert::TryFrom, path::Path};
use uriparse::URI;

/// Wraps a parser in a contextual label, which makes it easier to identify
/// where parsing failures occur
#[cfg(not(feature = "timekeeper"))]
pub use nom::error::context;

/// Wraps a parser in a contextual label, which makes it easier to identify
/// where parsing failures occur. This implementation also logs to a
/// timekeeper table, which can be printed out to evaluate the time spent
/// within each parser wrapped in a context.
#[cfg(feature = "timekeeper")]
pub fn context<'a, T>(
    ctx: &'static str,
    f: impl Fn(Span<'a>) -> IResult<'a, T>,
) -> impl Fn(Span<'a>) -> IResult<'a, T> {
    crate::timekeeper::parsers::context(ctx, f)
}

/// Parser that transforms a `Captured<T>` to a `Located<T>`, which involves
/// calculating the line and column information; so, this is expensive!
pub fn locate<'a, T>(
    parser: impl Fn(Span<'a>) -> IResult<Captured<T>>,
) -> impl Fn(Span<'a>) -> IResult<Located<T>> {
    context("Locate", move |input: Span| {
        let (input, c) = parser(input)?;

        // Construct a region that does NOT compute the line & column
        let region = Region::from(c.input());

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

/// Parser that transforms the result of one parser to that of `Cow<'a, str>`
/// where the lifetime is bound to the resulting `Span<'a>`
pub fn cow_str<'a>(
    parser: impl Fn(Span<'a>) -> IResult<Span<'a>>,
) -> impl Fn(Span<'a>) -> IResult<Cow<'a, str>> {
    context("Cow Str", map(parser, |s: Span<'a>| s.into()))
}

/// Parser that transforms the result of one parser to that of `Cow<'a, Path>`
/// where the lifetime is bound to the resulting `Span<'a>`
pub fn cow_path<'a>(
    parser: impl Fn(Span<'a>) -> IResult<Span<'a>>,
) -> impl Fn(Span<'a>) -> IResult<Cow<'a, Path>> {
    context("Cow Path", map(parser, |s: Span<'a>| s.into()))
}

/// Parser that will consume an end of line (\n or \r\n) or do nothing if
/// the input is empty
pub fn end_of_line_or_input(input: Span) -> IResult<()> {
    fn inner(input: Span) -> IResult<()> {
        if input.is_empty() {
            return Ok((input, ()));
        }

        let (input, _) = line_ending(input)?;
        Ok((input, ()))
    }

    context("End of Line/Input", inner)(input)
}

/// Parser that consumes input inside the surrounding left and right sides,
/// failing if not starting with the left or if the right is not found prior
/// to the end of a line. The result is the content WITHIN the surroundings.
/// Will not match right side if it follows immediately from the left.
///
/// Note that the left and right must be non-empty.
pub fn surround_in_line1<'a>(
    left: &'static str,
    right: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    fn inner<'a>(
        left: &'static str,
        right: &'static str,
    ) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
        move |input: Span| {
            let (input, _) = tag(left)(input)?;
            let input_bytes = input.as_bytes();

            // First, figure out where our next line will be
            let maybe_newline_pos = memchr(b'\n', input_bytes);

            // Second, look for the starting byte of the right side of our
            // surround wrapper
            for pos in memchr_iter(right.as_bytes()[0], input_bytes) {
                // If we've reached the end of the line, return an error
                if let Some(newline_pos) = maybe_newline_pos {
                    if pos >= newline_pos {
                        return Err(nom::Err::Error(Error::from_ctx(
                            &input,
                            "end of line reached before right side",
                        )));
                    }
                }

                // If there would be nothing in the surroundings, continue
                if pos == 0 {
                    continue;
                }

                // Grab everything but the possible start of the right
                let (input, content) = input.take_split(pos);

                // Verify that the right would be next, and if so return our
                // result, otherwise continue
                let (input, right_span) = take(right.len())(input)?;
                if right_span.as_bytes() == right.as_bytes() {
                    return Ok((input, content));
                } else {
                    continue;
                }
            }

            // There was no match of the right side
            Err(nom::Err::Error(Error::from_ctx(
                &input,
                "right side not found",
            )))
        }
    }

    context("Surround in Line", inner(left, right))
}

/// Parser that consumes input while the pattern succeeds or we reach the
/// end of the line. Note that this does NOT consume the line termination.
pub fn take_line_while<'a, T>(
    parser: impl Fn(Span<'a>) -> IResult<T>,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    fn single_char<'a, T>(
        parser: impl Fn(Span<'a>) -> IResult<T>,
    ) -> impl Fn(Span<'a>) -> IResult<char> {
        move |input: Span| {
            let (input, _) = not(end_of_line_or_input)(input)?;

            // NOTE: This is the same as peek(parser), but avoids the issue
            //       of variable being moved out of captured Fn(...)
            let (_, _) = parser(input)?;

            anychar(input)
        }
    }

    context("Take Line While", recognize(many0(single_char(parser))))
}

/// Parser that consumes input while the pattern succeeds or we reach the
/// end of the line. Note that this does NOT consume the line termination.
pub fn take_line_while1<'a, T>(
    parser: impl Fn(Span<'a>) -> IResult<T>,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    context(
        "Take Line While 1",
        verify(take_line_while(parser), |s| !s.is_empty()),
    )
}

/// Parser that will consume the remainder of a line (or end of input)
pub fn take_until_end_of_line_or_input(input: Span) -> IResult<Span> {
    fn inner(input: Span) -> IResult<Span> {
        match memchr(b'\n', input.as_bytes()) {
            Some(pos) => Ok(input.take_split(pos)),
            _ => rest(input),
        }
    }

    context("Take Until End of Line or Input", inner)(input)
}

/// Parser that will consume input until the specified byte is found,
/// failing if the byte is never found
pub fn take_until_byte<'a>(byte: u8) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    move |input: Span| {
        if let Some(pos) = memchr(byte, input.as_bytes()) {
            Ok(input.take_split(pos))
        } else {
            Err(nom::Err::Error(Error::from_ctx(
                &input,
                "Unable to find byte",
            )))
        }
    }
}

/// Parser that will consume input until the specified byte is found,
/// consuming the entire input if the byte is not found; fails if does
/// not consume at least 1 byte
pub fn take_until_byte1<'a>(
    byte: u8,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    context(
        "Take Until Byte 1",
        verify(take_until_byte(byte), |output| !output.is_empty()),
    )
}

/// Parser that will succeed if input is at the beginning of a line; input
/// will not be consumed
pub fn beginning_of_line(input: Span) -> IResult<()> {
    fn inner(input: Span) -> IResult<()> {
        let l = input.consumed_len();

        // If we have consumed nothing or the last consumed byte was a newline,
        // we are at the beginning of the line now
        if l == 0 || input.as_consumed()[l - 1] == b'\n' {
            Ok((input, ()))
        } else {
            Err(nom::Err::Error(Error::from_ctx(
                &input,
                "Not at beginning of line",
            )))
        }
    }

    context("Beginning of Line", inner)(input)
}

/// Parser that will consume a line if it is blank, which means that it is
/// comprised of nothing but whitespace and line termination
pub fn blank_line<'a>(input: Span<'a>) -> IResult<Span<'a>> {
    context(
        "Blank Line",
        preceded(
            beginning_of_line,
            alt((
                terminated(space1, end_of_line_or_input),
                terminated(space0, line_ending),
            )),
        ),
    )(input)
}

/// Parser that will consume any line, returning the line's content as output
pub fn any_line<'a>(input: Span<'a>) -> IResult<Span<'a>> {
    fn inner<'a>(input: Span<'a>) -> IResult<Span<'a>> {
        let (input, _) = beginning_of_line(input)?;
        let (input, content) = take_until_end_of_line_or_input(input)?;
        let (input, _) = end_of_line_or_input(input)?;
        Ok((input, content))
    }

    context("Any Line", inner)(input)
}

/// Parser that consumes a single multispace that could be \r\n, \n, \t, or
/// a space character
pub fn single_multispace(input: Span) -> IResult<()> {
    context(
        "Single Multispace",
        value((), alt((crlf, tag("\n"), tag("\t"), tag(" ")))),
    )(input)
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

/// Counts the spaces & tabs that are trailing in our input
pub fn count_trailing_whitespace(input: Span) -> IResult<usize> {
    fn inner(input: Span) -> IResult<usize> {
        let mut cnt = 0;

        // Count whitespace in reverse so we know how many are trailing
        for b in input.as_bytes().iter().rev() {
            if !nom::character::is_space(*b) {
                break;
            }
            cnt += 1;
        }

        Ok((input, cnt))
    }

    context("Count Trailing Whitespace", inner)(input)
}

/// Trims the trailing whitespace from input, essentially working backwards
/// to cut off part of the input
pub fn trim_trailing_whitespace(input: Span) -> IResult<()> {
    fn inner(input: Span) -> IResult<()> {
        use nom::Slice;
        let (input, len) = rest_len(input)?;
        let (input, cnt) = count_trailing_whitespace(input)?;
        Ok((input.slice(..(len - cnt)), ()))
    }

    context("Trim Trailing Whitespace", inner)(input)
}

/// Trims the leading and trailing whitespace from input
pub fn trim_whitespace(input: Span) -> IResult<()> {
    fn inner(input: Span) -> IResult<()> {
        let (input, _) = space0(input)?;
        let (input, _) = trim_trailing_whitespace(input)?;
        Ok((input, ()))
    }

    context("Trim Whitespace", inner)(input)
}

/// Takes from the end instead of the beginning
pub fn take_end<'a, C>(count: C) -> impl Fn(Span<'a>) -> IResult<Span<'a>>
where
    C: nom::ToUsize,
{
    use nom::{
        error::{ErrorKind, ParseError},
        Err,
    };
    let cnt = count.to_usize();
    context("Take End", move |input: Span| {
        let len = input.input_len();
        if cnt > len {
            Err(Err::Error(Error::from_error_kind(input, ErrorKind::Eof)))
        } else {
            let (end, input) = input.take_split(len - cnt);
            Ok((input, end))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::character::complete::char;

    #[inline]
    fn take_and_toss(n: usize) -> impl Fn(Span) -> IResult<()> {
        move |input: Span| {
            nom::combinator::value((), nom::bytes::complete::take(n))(input)
        }
    }

    #[test]
    fn end_of_line_or_input_should_succeed_if_line_ending() {
        assert!(end_of_line_or_input(Span::from("\n")).is_ok());
        assert!(end_of_line_or_input(Span::from("\r\n")).is_ok());
    }

    #[test]
    fn end_of_line_or_input_should_succeed_if_input_empty() {
        assert!(end_of_line_or_input(Span::from("")).is_ok());
    }

    #[test]
    fn beginning_of_line_should_fail_if_not_at_beginning_of_line() {
        let input = Span::from("1234");
        let (input, _) =
            take_and_toss(1)(input).expect("Failed to take a character");
        assert!(beginning_of_line(input).is_err());
    }

    #[test]
    fn beginning_of_line_should_succeed_if_at_beginning_of_first_line() {
        let input = Span::from("1234");
        let (input, _) = beginning_of_line(input)
            .expect("Unexpectedly think not at beginning of line");

        // Input shouldn't be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "1234");
    }

    #[test]
    fn beginning_of_line_should_succeed_if_at_beginning_of_any_line() {
        let input = Span::from("abc\n1234");
        let (input, _) =
            take_and_toss(4)(input).expect("Failed to take a character");
        let (input, _) = beginning_of_line(input)
            .expect("Unexpectedly think not at beginning of line");

        // Input shouldn't be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "1234");
    }

    #[test]
    fn blank_line_should_fail_if_line_contains_non_whitespace() {
        let input = Span::from("1234");
        assert!(blank_line(input).is_err());
    }

    #[test]
    fn blank_line_should_fail_if_input_empty_and_at_beginning_of_line() {
        let input = Span::from("");
        assert!(blank_line(input).is_err());
    }
    #[test]
    fn blank_line_should_succeed_if_has_whitespace_but_no_line_termination() {
        let input = Span::from(" ");
        let (input, s) = blank_line(input).expect("Failed to parse blank line");
        assert!(input.is_empty(), "Did not consume blank line");
        assert_eq!(s, " ");
    }

    #[test]
    fn blank_line_should_succeed_if_line_empty() {
        let input = Span::from("\nabcd");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "abcd");
    }

    #[test]
    fn blank_line_should_succeed_if_line_only_has_whitespace() {
        let input = Span::from(" \t\nabcd");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "abcd");
    }

    #[test]
    fn blank_line_should_succeed_if_on_last_line_and_only_whitespace() {
        let input = Span::from(" \t");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(input.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn any_line_should_fail_if_not_at_beginning_of_line() {
        let input = Span::from("abc");
        let (input, _) =
            take_and_toss(1)(input).expect("Failed to take a character");
        assert!(any_line(input).is_err());
    }

    #[test]
    fn any_line_should_return_empty_if_nothing_in_line() {
        let input = Span::from("\nabcd");
        let (input, content) =
            any_line(input).expect("Failed to parse any line");
        assert_eq!(input.as_unsafe_remaining_str(), "abcd");
        assert!(content.is_empty());
    }

    #[test]
    fn any_line_should_return_all_content_update_to_newline() {
        let input = Span::from("test\nabcd");
        let (input, line) = any_line(input).expect("Failed to parse any line");
        assert_eq!(input.as_unsafe_remaining_str(), "abcd");
        assert_eq!(line, "test");
    }

    #[test]
    fn any_line_should_return_all_content_remaining_if_no_more_newline() {
        let input = Span::from("test");
        let (input, line) = any_line(input).expect("Failed to parse any line");
        assert_eq!(input.as_unsafe_remaining_str(), "");
        assert_eq!(line, "test");
    }

    #[test]
    fn single_multispace_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(single_multispace(input).is_err());
    }

    #[test]
    fn single_multispace_should_fail_if_not_multispace_character() {
        let input = Span::from("a");
        assert!(single_multispace(input).is_err());
    }

    #[test]
    fn single_multispace_should_succeed_if_tab() {
        let input = Span::from("\t abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_space() {
        let input = Span::from("  abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_crlf() {
        let input = Span::from("\r\n abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_newline() {
        let input = Span::from("\n abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), " abc");
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

    #[test]
    fn take_line_while_should_yield_empty_if_empty_input() {
        let input = Span::from("");
        let (_, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(taken.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_line_termination_next() {
        let input = Span::from("\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "");

        let input = Span::from("\r\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\r\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_stops_without_ever_succeeding() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(char('c'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "aabb\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn take_line_while_should_take_until_provided_parser_fails() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(char('a'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "bb\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aa");
    }

    #[test]
    fn take_line_while_should_take_until_line_termination_reached() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aabb");
    }

    #[test]
    fn take_line_while_should_count_condition_parser_towards_consumption() {
        // NOTE: Using an ODD number of characters as otherwise we wouldn't
        //       catch the error which was happening where we would use the
        //       parser, char('-'), which would consume a character since it
        //       was not a not(...) and then try to use an anychar, so we
        //       would end up consuming TWO parsers instead of one
        let input = Span::from("-----");
        let (input, taken) = take_line_while(char('-'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "");
        assert_eq!(taken.as_unsafe_remaining_str(), "-----");
    }

    #[test]
    fn take_line_while1_should_fail_if_empty_input() {
        let input = Span::from("");
        assert!(take_line_while1(anychar)(input).is_err());
    }

    #[test]
    fn take_line_while1_should_fail_if_line_termination_next() {
        let input = Span::from("\nabcd");
        assert!(take_line_while1(anychar)(input).is_err());

        let input = Span::from("\r\nabcd");
        assert!(take_line_while1(anychar)(input).is_err());
    }

    #[test]
    fn take_line_while1_should_fail_if_stops_without_ever_succeeding() {
        let input = Span::from("aabb\nabcd");
        assert!(take_line_while1(char('c'))(input).is_err());
    }

    #[test]
    fn take_line_while1_should_take_until_provided_parser_fails() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while1(char('a'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "bb\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aa");
    }

    #[test]
    fn take_line_while1_should_take_until_line_termination_reached() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while1(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aabb");
    }

    #[test]
    fn take_line_while1_should_count_condition_parser_towards_consumption() {
        // NOTE: Using an ODD number of characters as otherwise we wouldn't
        //       catch the error which was happening where we would use the
        //       parser, char('-'), which would consume a character since it
        //       was not a not(...) and then try to use an anychar, so we
        //       would end up consuming TWO parsers instead of one
        let input = Span::from("-----");
        let (input, taken) = take_line_while1(char('-'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "");
        assert_eq!(taken.as_unsafe_remaining_str(), "-----");
    }
}
