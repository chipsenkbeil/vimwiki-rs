use super::{Position, Region, Span, VimwikiIResult, LC};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{anychar, crlf, line_ending, space0},
    combinator::{
        map, map_res, not, opt, peek, recognize, rest_len, value, verify,
    },
    error::context,
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, terminated},
};
use url::Url;

/// Alternative to the `position` function of nom_locate that retains the fragment
#[inline]
pub fn position(input: Span) -> VimwikiIResult<Span> {
    Ok((input, input))
}

/// Parser that wraps another parser's output in a LocatedComponent based on
/// the consumed input
#[inline]
pub fn lc<'a, T>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<LC<T>> {
    move |input: Span<'a>| {
        let (input, pos) = position(input)?;
        let (input, x) = parser(input)?;
        Ok((input, LC::new(x, Region::from((pos, input)))))
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
pub fn take_line_while<'a, T>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
    recognize(many0(preceded(
        pair(not(end_of_line_or_input), peek(parser)),
        anychar,
    )))
}

/// Parser that consumes input while the pattern succeeds or we reach the
/// end of the line. Note that this does NOT consume the line termination.
#[inline]
pub fn take_line_while1<'a, T>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<T>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
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
    Ok((input, Position::from(input).column))
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
    //    that would be a blank line at the end of a file
    context(
        "Blank Line",
        map(
            delimited(beginning_of_line, space0, end_of_line_or_input),
            |s| s.fragment().to_string(),
        ),
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
                |s: Span| s.fragment().to_string(),
            ),
            |s: &str| !s.trim().is_empty(),
        ),
    )(input)
}

/// Parser that will consume any line, returning the line's content as output
/// (not including line termination)
#[inline]
pub fn any_line(input: Span) -> VimwikiIResult<String> {
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
pub fn pstring<'a>(
    parser: impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>>,
) -> impl Fn(Span<'a>) -> VimwikiIResult<String> {
    map(parser, |s: Span<'a>| s.fragment().to_string())
}

/// Parser for a general purpose URL.
///
/// ### Regular cases
///
///     1. https (https://example.com)
///     2. http (http://example.com)
///     3. ftp (ftp:)
///     4. file (file:relative/path)
///     5. local (local:relative/path)
///     6. mailto (mailto:someone@example.com)
///
/// ### Special cases
///
///     1. www (www.example.com) -> (https://www.example.com)
///     2. // (//some/abs/path) -> (file:/some/abs/path)
#[inline]
pub fn url(input: Span) -> VimwikiIResult<Url> {
    // URI = scheme:[//authority]path[?query][#fragment]
    // scheme = sequence of characters beginning with a letter and followed
    //          by any combination of letters, digits, plus (+), period (.),
    //          or hyphen (-)
    // authority = [userinfo@]host[:port] where host is a hostname or IP address
    // path = sequence of path segments separated by / with an empty segment
    //        resulting in //
    let scheme = terminated(
        take_while(|c: char| {
            c.is_alphanumeric() || c == '+' || c == '.' || c == '-'
        }),
        tag(":"),
    );

    // TODO: Do we need to support whitespace in our raw URLs?
    context(
        "Url",
        map_res(
            recognize(pair(
                alt((tag("www."), tag("//"), scheme)),
                many1(pair(not(single_multispace), anychar)),
            )),
            |s| match *s.fragment() {
                text if text.starts_with("www.") => {
                    Url::parse(&["https://", text].join(""))
                }
                text if text.starts_with("//") => {
                    Url::parse(&["file:/", text].join(""))
                }
                text => Url::parse(text),
            },
        ),
    )(input)
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
        assert!(end_of_line_or_input(Span::new("\n")).is_ok());
        assert!(end_of_line_or_input(Span::new("\r\n")).is_ok());
    }

    #[test]
    fn end_of_line_or_input_should_succeed_if_input_empty() {
        assert!(end_of_line_or_input(Span::new("")).is_ok());
    }

    #[test]
    fn count_from_beginning_of_line_should_yield_0_if_at_beginning_of_line() {
        let (_, count) =
            count_from_beginning_of_line(Span::new("")).expect("Parser failed");
        assert_eq!(count, 0);

        let (_, count) = count_from_beginning_of_line(Span::new("some text"))
            .expect("Parser failed");
        assert_eq!(count, 0);
    }

    #[test]
    fn count_from_beginning_of_line_should_yield_n_where_n_is_characters_from_beginning_of_line(
    ) {
        let input = Span::new("some text");
        let (input, _) =
            take_and_toss(4)(input).expect("Failed to take N characters");
        let (_, count) = count_from_beginning_of_line(input)
            .expect("Failed to count characters");
        assert_eq!(count, 4);
    }

    #[test]
    fn count_from_beginning_of_line_should_use_only_current_line_progress() {
        let input = Span::new("1234\n1234");
        let (input, _) =
            take_and_toss(5)(input).expect("Failed to take first line");
        let (_, count) = count_from_beginning_of_line(input)
            .expect("Failed to count characters");
        assert_eq!(count, 0);
    }

    #[test]
    fn beginning_of_line_should_fail_if_not_at_beginning_of_line() {
        let input = Span::new("1234");
        let (input, _) =
            take_and_toss(1)(input).expect("Failed to take a character");
        assert!(beginning_of_line(input).is_err());
    }

    #[test]
    fn beginning_of_line_should_succeed_if_at_beginning_of_line() {
        let input = Span::new("1234");
        let (input, _) = beginning_of_line(input)
            .expect("Unexpectedly think not at beginning of line");

        // Input shouldn't be consumed
        assert_eq!(*input.fragment(), "1234");
    }

    #[test]
    fn blank_line_should_fail_if_line_contains_non_whitespace() {
        let input = Span::new("1234");
        assert!(blank_line(input).is_err());
    }

    #[test]
    fn blank_line_should_succeed_if_input_empty_and_at_beginning_of_line() {
        let input = Span::new("");
        assert!(blank_line(input).is_ok());
    }

    #[test]
    fn blank_line_should_succeed_if_line_empty() {
        let input = Span::new("\nabcd");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(*input.fragment(), "abcd");
    }

    #[test]
    fn blank_line_should_succeed_if_line_only_has_whitespace() {
        let input = Span::new(" \t\nabcd");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(*input.fragment(), "abcd");
    }

    #[test]
    fn blank_line_should_succeed_if_on_last_line_and_only_whitespace() {
        let input = Span::new(" \t");
        let (input, _) = blank_line(input).expect("Failed to parse blank line");

        // Line including termination should be consumed
        assert_eq!(*input.fragment(), "");
    }

    #[test]
    fn non_blank_line_should_fail_if_input_empty_and_at_beginning_of_line() {
        let input = Span::new("");
        assert!(non_blank_line(input).is_err());
    }

    #[test]
    fn non_blank_line_should_fail_if_line_is_empty() {
        let input = Span::new("\nabcd");
        assert!(non_blank_line(input).is_err());
    }

    #[test]
    fn non_blank_line_should_succeed_if_line_has_more_than_whitespace() {
        let input = Span::new("  a  \nabcd");
        let (input, line) =
            non_blank_line(input).expect("Failed to parse non blank line");
        assert_eq!(*input.fragment(), "abcd");
        assert_eq!(line, "  a  ");
    }

    #[test]
    fn non_blank_line_should_succeed_if_on_last_line_and_not_only_whitespace() {
        let input = Span::new("  a  ");
        let (input, line) =
            non_blank_line(input).expect("Failed to parse non blank line");
        assert_eq!(*input.fragment(), "");
        assert_eq!(line, "  a  ");
    }

    #[test]
    fn single_multispace_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(single_multispace(input).is_err());
    }

    #[test]
    fn single_multispace_should_fail_if_not_multispace_character() {
        let input = Span::new("a");
        assert!(single_multispace(input).is_err());
    }

    #[test]
    fn single_multispace_should_succeed_if_tab() {
        let input = Span::new("\t abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(*input.fragment(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_space() {
        let input = Span::new("  abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(*input.fragment(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_crlf() {
        let input = Span::new("\r\n abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(*input.fragment(), " abc");
    }

    #[test]
    fn single_multispace_should_succeed_if_newline() {
        let input = Span::new("\n abc");
        let (input, _) = single_multispace(input).unwrap();
        assert_eq!(*input.fragment(), " abc");
    }

    #[test]
    fn url_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(url(input).is_err());
    }

    #[test]
    fn url_should_fail_if_no_scheme_and_not_www_or_absolute_path() {
        let input = Span::new("example.com");
        assert!(url(input).is_err());
    }

    #[test]
    fn url_should_succeed_if_starts_with_www_and_will_add_https_as_scheme() {
        let input = Span::new("www.example.com");
        let (input, u) = url(input).expect("Failed to parse url");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "https");
        assert_eq!(u.host_str(), Some("www.example.com"));
    }

    #[test]
    fn url_should_succeed_if_starts_with_absolute_path_and_will_add_file_as_scheme(
    ) {
        let input = Span::new("//some/absolute/path");
        let (input, u) = url(input).expect("Failed to parse url");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "file");
        assert_eq!(u.path(), "/some/absolute/path");
    }

    #[test]
    fn url_should_succeed_if_starts_with_scheme() {
        let input = Span::new("https://github.com/vimwiki/vimwiki.git");
        let (input, u) = url(input).expect("Failed to parse url");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "https");
        assert_eq!(u.host_str(), Some("github.com"));
        assert_eq!(u.path(), "/vimwiki/vimwiki.git");

        let input = Span::new("mailto:habamax@gmail.com");
        let (input, u) = url(input).expect("Failed to parse url");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "mailto");
        assert_eq!(u.path(), "habamax@gmail.com");

        let input = Span::new("ftp://vim.org");
        let (input, u) = url(input).expect("Failed to parse url");
        assert!(input.fragment().is_empty());
        assert_eq!(u.scheme(), "ftp");
        assert_eq!(u.host_str(), Some("vim.org"));
    }

    #[test]
    fn take_line_while_should_yield_empty_if_empty_input() {
        let input = Span::new("");
        let (_, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(*taken.fragment(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_line_termination_next() {
        let input = Span::new("\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(*input.fragment(), "\nabcd");
        assert_eq!(*taken.fragment(), "");

        let input = Span::new("\r\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(*input.fragment(), "\r\nabcd");
        assert_eq!(*taken.fragment(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_stops_without_ever_succeeding() {
        let input = Span::new("aabb\nabcd");
        let (input, taken) = take_line_while(char('c'))(input).unwrap();
        assert_eq!(*input.fragment(), "aabb\nabcd");
        assert_eq!(*taken.fragment(), "");
    }

    #[test]
    fn take_line_while_should_take_until_provided_parser_fails() {
        let input = Span::new("aabb\nabcd");
        let (input, taken) = take_line_while(char('a'))(input).unwrap();
        assert_eq!(*input.fragment(), "bb\nabcd");
        assert_eq!(*taken.fragment(), "aa");
    }

    #[test]
    fn take_line_while_should_take_until_line_termination_reached() {
        let input = Span::new("aabb\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(*input.fragment(), "\nabcd");
        assert_eq!(*taken.fragment(), "aabb");
    }

    #[test]
    fn take_line_while_should_count_condition_parser_towards_consumption() {
        // NOTE: Using an ODD number of characters as otherwise we wouldn't
        //       catch the error which was happening where we would use the
        //       parser, char('-'), which would consume a character since it
        //       was not a not(...) and then try to use an anychar, so we
        //       would end up consuming TWO parsers instead of one
        let input = Span::new("-----");
        let (input, taken) = take_line_while(char('-'))(input).unwrap();
        assert_eq!(*input.fragment(), "");
        assert_eq!(*taken.fragment(), "-----");
    }

    #[test]
    fn take_line_while1_should_fail_if_empty_input() {
        let input = Span::new("");
        assert!(take_line_while1(anychar)(input).is_err());
    }

    #[test]
    fn take_line_while1_should_fail_if_line_termination_next() {
        let input = Span::new("\nabcd");
        assert!(take_line_while1(anychar)(input).is_err());

        let input = Span::new("\r\nabcd");
        assert!(take_line_while1(anychar)(input).is_err());
    }

    #[test]
    fn take_line_while1_should_fail_if_stops_without_ever_succeeding() {
        let input = Span::new("aabb\nabcd");
        assert!(take_line_while1(char('c'))(input).is_err());
    }

    #[test]
    fn take_line_while1_should_take_until_provided_parser_fails() {
        let input = Span::new("aabb\nabcd");
        let (input, taken) = take_line_while1(char('a'))(input).unwrap();
        assert_eq!(*input.fragment(), "bb\nabcd");
        assert_eq!(*taken.fragment(), "aa");
    }

    #[test]
    fn take_line_while1_should_take_until_line_termination_reached() {
        let input = Span::new("aabb\nabcd");
        let (input, taken) = take_line_while1(anychar)(input).unwrap();
        assert_eq!(*input.fragment(), "\nabcd");
        assert_eq!(*taken.fragment(), "aabb");
    }

    #[test]
    fn take_line_while1_should_count_condition_parser_towards_consumption() {
        // NOTE: Using an ODD number of characters as otherwise we wouldn't
        //       catch the error which was happening where we would use the
        //       parser, char('-'), which would consume a character since it
        //       was not a not(...) and then try to use an anychar, so we
        //       would end up consuming TWO parsers instead of one
        let input = Span::new("-----");
        let (input, taken) = take_line_while1(char('-'))(input).unwrap();
        assert_eq!(*input.fragment(), "");
        assert_eq!(*taken.fragment(), "-----");
    }
}
