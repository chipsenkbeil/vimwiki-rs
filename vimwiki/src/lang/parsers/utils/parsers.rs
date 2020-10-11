use super::{Region, Span, VimwikiIResult, VimwikiNomError, LE};
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while},
    character::complete::{anychar, crlf, line_ending, space0, space1},
    combinator::{
        map, map_res, not, opt, peek, recognize, rest_len, value, verify,
    },
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, terminated},
    AsBytes, InputLength, InputTake,
};
use std::convert::TryFrom;
use std::ops::Range;
use uriparse::URI;

/// Wraps a parser in a contextual label, which makes it easier to identify
/// where parsing failures occur
#[cfg(not(feature = "timekeeper"))]
pub fn context<T>(
    ctx: &'static str,
    f: impl Fn(Span) -> VimwikiIResult<T>,
) -> impl Fn(Span) -> VimwikiIResult<T> {
    nom::error::context(ctx, f)
}
/// Wraps a parser in a contextual label, which makes it easier to identify
/// where parsing failures occur. This implementation also logs to a
/// timekeeper table, which can be printed out to evaluate the time spent
/// within each parser wrapped in a context.
#[cfg(feature = "timekeeper")]
pub fn context<T>(
    ctx: &'static str,
    f: impl Fn(Span) -> VimwikiIResult<T>,
) -> impl Fn(Span) -> VimwikiIResult<T> {
    crate::timekeeper::parsers::context(ctx, f)
}

/// Parser that wraps another parser's output in a LocatedElement based on
/// the consumed input
#[inline]
pub fn le<T>(
    parser: impl Fn(Span) -> VimwikiIResult<T>,
) -> impl Fn(Span) -> VimwikiIResult<LE<T>> {
    use nom::{Offset, Slice};
    context("LE", move |input: Span| {
        let start_line = input.global_line();
        let start_column = input.global_utf8_column();

        let (input2, x) = parser(input.clone())?;

        // Get offset at end (new start - 1)
        let mut offset = input.offset(&input2);
        if offset > 0 {
            offset -= 1;
        }

        let input = input.slice(offset..);
        let end_line = input.global_line();
        let end_column = input.global_utf8_column();

        Ok((
            input2,
            LE::new(
                x,
                Region::from((start_line, start_column, end_line, end_column)),
            ),
        ))
    })
}

/// Parser that unwraps another parser's output of LocatedElement into the
/// underlying element
pub fn unwrap_le<T>(
    parser: impl Fn(Span) -> VimwikiIResult<LE<T>>,
) -> impl Fn(Span) -> VimwikiIResult<T> {
    context("LE Unwrap", move |input: Span| {
        let (input, le) = parser(input)?;

        Ok((input, le.element))
    })
}

/// Parser that wraps another parser's output in a tuple that also echos out
/// the offset range (starting offset and ending exclusive offset beyond consumed)
#[inline]
pub fn range<T>(
    parser: impl Fn(Span) -> VimwikiIResult<T>,
) -> impl Fn(Span) -> VimwikiIResult<(Range<usize>, T)> {
    move |input: Span| {
        let start = input.local_offset();
        let (input, x) = parser(input)?;
        let end = input.local_offset();
        Ok((input, (start..end, x)))
    }
}

/// Parser that will consume an end of line (\n or \r\n) or do nothing if
/// the input is empty
#[inline]
pub fn end_of_line_or_input(input: Span) -> VimwikiIResult<()> {
    context(
        "End of Line/Input",
        value(
            (),
            verify(pair(opt(line_ending), rest_len), |(end_of_line, len)| {
                *len == 0 || end_of_line.is_some()
            }),
        ),
    )(input)
}

/// Parser that consumes input while the pattern succeeds or we reach the
/// end of the line. Note that this does NOT consume the line termination.
#[inline]
pub fn take_line_while<T>(
    parser: impl Fn(Span) -> VimwikiIResult<T>,
) -> impl Fn(Span) -> VimwikiIResult<Span> {
    recognize(many0(preceded(
        pair(not(end_of_line_or_input), peek(parser)),
        anychar,
    )))
}

/// Parser that consumes input while the pattern succeeds or we reach the
/// end of the line. Note that this does NOT consume the line termination.
#[inline]
pub fn take_line_while1<T>(
    parser: impl Fn(Span) -> VimwikiIResult<T>,
) -> impl Fn(Span) -> VimwikiIResult<Span> {
    verify(take_line_while(parser), |s| !s.fragment().is_empty())
}

/// Parser that will consume the remainder of a line (or end of input)
#[inline]
pub fn take_until_end_of_line_or_input(input: Span) -> VimwikiIResult<Span> {
    take_line_while(anychar)(input)
}

/// Parser that will report the total columns consumed since the beginning of
/// the line (0 being none); input will not be consumed
#[inline]
pub fn count_from_beginning_of_line(input: Span) -> VimwikiIResult<usize> {
    let column = input.local_utf8_column() - 1;
    Ok((input, column))
}

/// Parser that will succeed if input is at the beginning of a line; input
/// will not be consumed
#[inline]
pub fn beginning_of_line(input: Span) -> VimwikiIResult<()> {
    context(
        "Beginning of Line",
        value(
            (),
            verify(count_from_beginning_of_line, |count| *count == 0),
        ),
    )(input)
}

/// Parser that will consume a line if it is blank, which means that it is
/// comprised of nothing but whitespace and line termination
#[inline]
pub fn blank_line(input: Span) -> VimwikiIResult<String> {
    // 1. We must assert (using span) that we're actually at the beginning of
    //    a line, otherwise this could have been used somewhere after some
    //    other content was matched, and we don't want it to succeed
    //
    // 2. We want to eat up all spaces & tabs on that line, followed by a line
    //    termination. If we happen to be at end of input, then that's okay as
    //    long as there was some space as that would be a blank line at the
    //    end of a file
    context(
        "Blank Line",
        pstring(preceded(
            beginning_of_line,
            alt((
                terminated(space1, end_of_line_or_input),
                terminated(space0, line_ending),
            )),
        )),
    )(input)
}

/// Parser that will consume a line if it is not blank, which means that it is
/// comprised of more than just whitespace and line termination
#[inline]
pub fn non_blank_line(input: Span) -> VimwikiIResult<String> {
    context(
        "Non Blank Line",
        verify(
            map(
                delimited(
                    beginning_of_line,
                    recognize(many1(pair(not(end_of_line_or_input), anychar))),
                    end_of_line_or_input,
                ),
                |s: Span| s.fragment_str().to_string(),
            ),
            |s: &str| !s.trim().is_empty(),
        ),
    )(input)
}

/// Parser that will consume any line, returning the line's content as output
/// (not including line termination)
#[inline]
pub fn any_line(input: Span) -> VimwikiIResult<String> {
    // TODO: Use memchr to find end of line, split at that point, and return
    //       it as a span; make a new parser that is any_line_as_string
    //
    //       From there, we can use the span version with the blank and
    //       non_blank parsers above to first verify that there is or is not
    //       a blank line and then allocate a string
    alt((non_blank_line, blank_line))(input)
}

/// Parser that consumes a single multispace that could be \r\n, \n, \t, or
/// a space character
#[inline]
pub fn single_multispace(input: Span) -> VimwikiIResult<()> {
    value((), alt((crlf, tag("\n"), tag("\t"), tag(" "))))(input)
}

/// Parser that transforms the output of a parser into an allocated string
#[inline]
pub fn pstring(
    parser: impl Fn(Span) -> VimwikiIResult<Span>,
) -> impl Fn(Span) -> VimwikiIResult<String> {
    map(parser, |s: Span| s.fragment_str().to_string())
}

/// Parser that scans through the entire input, applying the provided parser
/// and returning a series of results whenever a parser succeeds
#[inline]
pub fn scan<T>(
    parser: impl Fn(Span) -> VimwikiIResult<T>,
) -> impl Fn(Span) -> VimwikiIResult<Vec<T>> {
    move |mut input: Span| {
        fn advance(input: Span) -> VimwikiIResult<()> {
            value((), take(1usize))(input)
        }

        let mut output = Vec::new();

        loop {
            if let Ok((i, item)) = parser(input.clone()) {
                // No advancement happened, so error to prevent infinite loop
                if i == input {
                    return Err(nom::Err::Error(VimwikiNomError::from_ctx(
                        &i,
                        "scan detected infinite loop",
                    )));
                }

                output.push(item);
                input = i;
                continue;
            }

            match advance(input.clone()) {
                Ok((i, _)) => input = i,
                _ => break,
            }
        }

        Ok((input, output))
    }
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
#[inline]
pub fn uri(input: Span) -> VimwikiIResult<URI<'static>> {
    // URI = scheme:[//authority]path[?query][#fragment]
    // scheme = sequence of characters beginning with a letter and followed
    //          by any combination of letters, digits, plus (+), period (.),
    //          or hyphen (-)
    // authority = [userinfo@]host[:port] where host is a hostname or IP address
    // path = sequence of path segments separated by / with an empty segment
    //        resulting in //
    let scheme = terminated(
        take_while(|b: u8| {
            let c = char::from(b);
            c.is_alphanumeric() || c == '+' || c == '.' || c == '-'
        }),
        tag(":"),
    );

    // TODO: Do we need to support whitespace in our raw URIs?
    context(
        "URI",
        map_res(
            recognize(pair(
                alt((tag("www."), tag("//"), scheme)),
                many1(pair(not(single_multispace), anychar)),
            )),
            |s| {
                URI::try_from(
                    match s.fragment_str() {
                        text if text.starts_with("www.") => {
                            ["https://", text].join("")
                        }
                        text if text.starts_with("//") => {
                            ["file:/", text].join("")
                        }
                        text => text.to_string(),
                    }
                    .as_str(),
                )
                .map(|uri| uri.into_owned())
            },
        ),
    )(input)
}

/// Counts the spaces & tabs that are trailing in our input
pub fn count_trailing_whitespace(input: Span) -> VimwikiIResult<usize> {
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

/// Trims the trailing whitespace from input, essentially working backwards
/// to cut off part of the input
pub fn trim_trailing_whitespace(input: Span) -> VimwikiIResult<()> {
    use nom::Slice;
    let (input, len) = rest_len(input)?;
    let (input, cnt) = count_trailing_whitespace(input)?;
    Ok((input.slice(..(len - cnt)), ()))
}

/// Trims the leading and trailing whitespace from input
pub fn trim_whitespace(input: Span) -> VimwikiIResult<()> {
    let (input, _) = space0(input)?;
    let (input, _) = trim_trailing_whitespace(input)?;
    Ok((input, ()))
}

/// Takes from the end instead of the beginning
pub fn take_end<C>(count: C) -> impl Fn(Span) -> VimwikiIResult<Span>
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
            Err(Err::Error(VimwikiNomError::from_error_kind(
                input,
                ErrorKind::Eof,
            )))
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
    fn take_and_toss(n: usize) -> impl Fn(Span) -> VimwikiIResult<()> {
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
    fn count_from_beginning_of_line_should_yield_0_if_at_beginning_of_line() {
        let (_, count) = count_from_beginning_of_line(Span::from(""))
            .expect("Parser failed");
        assert_eq!(count, 0);

        let (_, count) = count_from_beginning_of_line(Span::from("some text"))
            .expect("Parser failed");
        assert_eq!(count, 0);
    }

    #[test]
    fn count_from_beginning_of_line_should_yield_n_where_n_is_characters_from_beginning_of_line(
    ) {
        let input = Span::from("some text");
        let (input, _) =
            take_and_toss(4)(input).expect("Failed to take N characters");
        let (_, count) = count_from_beginning_of_line(input)
            .expect("Failed to count characters");
        assert_eq!(count, 4);
    }

    #[test]
    fn count_from_beginning_of_line_should_use_only_current_line_progress() {
        let input = Span::from("1234\n1234");
        let (input, _) =
            take_and_toss(5)(input).expect("Failed to take first line");
        let (_, count) = count_from_beginning_of_line(input)
            .expect("Failed to count characters");
        assert_eq!(count, 0);
    }

    #[test]
    fn beginning_of_line_should_fail_if_not_at_beginning_of_line() {
        let input = Span::from("1234");
        let (input, _) =
            take_and_toss(1)(input).expect("Failed to take a character");
        assert!(beginning_of_line(input).is_err());
    }

    #[test]
    fn beginning_of_line_should_succeed_if_at_beginning_of_line() {
        let input = Span::from("1234");
        let (input, _) = beginning_of_line(input)
            .expect("Unexpectedly think not at beginning of line");

        // Input shouldn't be consumed
        assert_eq!(input.fragment_str(), "1234");
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
        assert!(input.fragment().is_empty(), "Did not consume blank line");
        assert_eq!(s, " ");
    }

    #[test]
    fn blank_line_should_succeed_if_line_empty() {
        let input = Span::from("\nabcd");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(input.fragment_str(), "abcd");
    }

    #[test]
    fn blank_line_should_succeed_if_line_only_has_whitespace() {
        let input = Span::from(" \t\nabcd");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(input.fragment_str(), "abcd");
    }

    #[test]
    fn blank_line_should_succeed_if_on_last_line_and_only_whitespace() {
        let input = Span::from(" \t");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(input.fragment_str(), "");
    }

    #[test]
    fn non_blank_line_should_fail_if_input_empty_and_at_beginning_of_line() {
        let input = Span::from("");
        assert!(non_blank_line(input).is_err());
    }

    #[test]
    fn non_blank_line_should_fail_if_line_is_empty() {
        let input = Span::from("\nabcd");
        assert!(non_blank_line(input).is_err());
    }

    #[test]
    fn non_blank_line_should_succeed_if_line_has_more_than_whitespace() {
        let input = Span::from("  a  \nabcd");
        let (input, line) =
            non_blank_line(input).expect("Failed to parse non blank line");
        assert_eq!(input.fragment_str(), "abcd");
        assert_eq!(line, "  a  ");
    }

    #[test]
    fn non_blank_line_should_succeed_if_on_last_line_and_not_only_whitespace() {
        let input = Span::from("  a  ");
        let (input, line) =
            non_blank_line(input).expect("Failed to parse non blank line");
        assert_eq!(input.fragment_str(), "");
        assert_eq!(line, "  a  ");
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
        assert_eq!(input.fragment_str(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_space() {
        let input = Span::from("  abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.fragment_str(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_crlf() {
        let input = Span::from("\r\n abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.fragment_str(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_newline() {
        let input = Span::from("\n abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(input.fragment_str(), " abc");
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
    fn uri_should_succeed_if_starts_with_www_and_will_add_https_as_scheme() {
        let input = Span::from("www.example.com");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "https");
        assert_eq!(u.host().unwrap().to_string(), "www.example.com");
    }

    #[test]
    fn uri_should_succeed_if_starts_with_absolute_path_and_will_add_file_as_scheme(
    ) {
        let input = Span::from("//some/absolute/path");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "file");
        assert_eq!(u.path(), "/some/absolute/path");
    }

    #[test]
    fn uri_should_succeed_if_starts_with_scheme() {
        let input = Span::from("https://github.com/vimwiki/vimwiki.git");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "https");
        assert_eq!(u.host().unwrap().to_string(), "github.com");
        assert_eq!(u.path(), "/vimwiki/vimwiki.git");

        let input = Span::from("mailto:habamax@gmail.com");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "mailto");
        assert_eq!(u.path(), "habamax@gmail.com");

        let input = Span::from("ftp://vim.org");
        let (input, u) = uri(input).expect("Failed to parse uri");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "ftp");
        assert_eq!(u.host().unwrap().to_string(), "vim.org");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_empty_input() {
        let input = Span::from("");
        let (_, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(taken.fragment_str(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_line_termination_next() {
        let input = Span::from("\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.fragment_str(), "\nabcd");
        assert_eq!(taken.fragment_str(), "");

        let input = Span::from("\r\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.fragment_str(), "\r\nabcd");
        assert_eq!(taken.fragment_str(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_stops_without_ever_succeeding() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(char('c'))(input).unwrap();
        assert_eq!(input.fragment_str(), "aabb\nabcd");
        assert_eq!(taken.fragment_str(), "");
    }

    #[test]
    fn take_line_while_should_take_until_provided_parser_fails() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(char('a'))(input).unwrap();
        assert_eq!(input.fragment_str(), "bb\nabcd");
        assert_eq!(taken.fragment_str(), "aa");
    }

    #[test]
    fn take_line_while_should_take_until_line_termination_reached() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.fragment_str(), "\nabcd");
        assert_eq!(taken.fragment_str(), "aabb");
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
        assert_eq!(input.fragment_str(), "");
        assert_eq!(taken.fragment_str(), "-----");
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
        assert_eq!(input.fragment_str(), "bb\nabcd");
        assert_eq!(taken.fragment_str(), "aa");
    }

    #[test]
    fn take_line_while1_should_take_until_line_termination_reached() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while1(anychar)(input).unwrap();
        assert_eq!(input.fragment_str(), "\nabcd");
        assert_eq!(taken.fragment_str(), "aabb");
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
        assert_eq!(input.fragment_str(), "");
        assert_eq!(taken.fragment_str(), "-----");
    }

    #[test]
    fn scan_should_fail_if_no_advancement_is_made_with_parser() {
        let input = Span::from("aaa");
        assert!(scan(not(char('b')))(input).is_err());
    }

    #[test]
    fn scan_should_yield_an_empty_vec_if_input_empty() {
        let input = Span::from("");
        let (_, results) = scan(char('a'))(input).unwrap();
        assert!(results.is_empty(), "Unexpectedly found parser results");
    }

    #[test]
    fn scan_should_consume_all_input() {
        let input = Span::from("abc");
        let (input, _) = scan(char('a'))(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "scan did not consume all input"
        );
    }

    #[test]
    fn scan_should_yield_an_empty_vec_if_parser_never_succeeds() {
        let input = Span::from("bbb");
        let (input, results) = scan(char('a'))(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "scan did not consume all input"
        );
        assert!(results.is_empty(), "Unexpectedly found results");
    }

    #[test]
    fn scan_should_yield_a_vec_containing_all_of_parser_successes() {
        let input = Span::from("aba");
        let (input, results) = scan(char('a'))(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "scan did not consume all input"
        );
        assert_eq!(results, vec!['a', 'a']);
    }

    #[test]
    fn range_should_include_the_starting_and_ending_offset_of_consumed_parser()
    {
        let input = Span::from("aba");
        let (input, (r, results)) = range(take(2usize))(input).unwrap();
        assert_eq!(
            input.fragment_str(),
            "a",
            "offset did not consume expected input"
        );
        assert_eq!(r.start, 0, "Start was wrong position");
        assert_eq!(r.end, 2, "End was wrong position");
        assert_eq!(
            results.fragment_str(),
            "ab",
            "Parser did not function properly"
        );
    }
}
