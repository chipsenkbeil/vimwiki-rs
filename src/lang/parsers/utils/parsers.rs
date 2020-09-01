use super::{new_nom_error, Position, Span, VimwikiIResult};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{anychar, crlf, line_ending, space0},
    combinator::{map, map_res, not, opt, recognize, rest_len, value, verify},
    error::context,
    multi::many1,
    sequence::{delimited, pair, terminated, tuple},
};
use url::Url;

/// Parser that will consume an end of line (\n or \r\n) or do nothing if
/// the input is empty
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

/// Parser that will report the total columns consumed since the beginning of
/// the line (0 being none); input will not be consumed
pub fn count_from_beginning_of_line(input: Span) -> VimwikiIResult<usize> {
    Ok((input, Position::from(input).column))
}

/// Parser that will succeed if input is at the beginning of a line; input
/// will not be consumed
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
pub fn non_blank_line(input: Span) -> VimwikiIResult<String> {
    context(
        "Non Blank Line",
        verify(
            map(
                map(
                    tuple((
                        beginning_of_line,
                        recognize(many1(pair(
                            not(end_of_line_or_input),
                            anychar,
                        ))),
                        end_of_line_or_input,
                    )),
                    |x| x.1,
                ),
                |s: Span| s.fragment().to_string(),
            ),
            |s: &str| !s.trim().is_empty(),
        ),
    )(input)
}

/// Parser that will consume any line, returning the line's content as output
/// (not including line termination)
pub fn any_line(input: Span) -> VimwikiIResult<String> {
    alt((non_blank_line, blank_line))(input)
}

/// Parser that consumes a single multispace that could be \r\n, \n, \t, or
/// a space character
pub fn single_multispace(input: Span) -> VimwikiIResult<()> {
    value((), alt((crlf, tag("\n"), tag("\t"), tag(" "))))(input)
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
                alt((scheme, tag("www."), tag("//"))),
                many1(pair(not(single_multispace), anychar)),
            )),
            |s| Url::parse(s.fragment()),
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_of_line_or_input_should_succeed_if_line_ending() {
        todo!();
    }

    #[test]
    fn end_of_line_or_input_should_succeed_if_input_empty() {
        todo!();
    }

    #[test]
    fn count_from_beginning_of_line_should_yield_0_if_at_beginning_of_line() {
        todo!();
    }

    #[test]
    fn count_from_beginning_of_line_should_yield_N_where_N_is_characters_from_beginning_of_line(
    ) {
        todo!();
    }

    #[test]
    fn beginning_of_line_should_fail_if_not_at_beginning_of_line() {
        todo!();
    }

    #[test]
    fn beginning_of_line_should_succeed_if_at_beginning_of_line() {
        todo!();
    }

    #[test]
    fn blank_line_should_fail_if_line_contains_non_whitespace() {
        todo!();
    }

    #[test]
    fn blank_line_should_succeed_if_input_empty_and_at_beginning_of_line() {
        todo!();
    }

    #[test]
    fn blank_line_should_succeed_if_line_empty() {
        todo!();
    }

    #[test]
    fn blank_line_should_succeed_if_line_only_has_whitespace() {
        todo!();
    }

    #[test]
    fn blank_line_should_succeed_if_on_last_line_and_only_whitespace() {
        todo!();
    }

    #[test]
    fn non_blank_line_should_fail_if_input_empty_and_at_beginning_of_line() {
        todo!();
    }

    #[test]
    fn non_blank_line_should_fail_if_line_is_empty() {
        todo!();
    }

    #[test]
    fn non_blank_line_should_succeed_if_line_has_more_than_whitespace() {
        todo!();
    }

    #[test]
    fn non_blank_line_should_succeed_if_on_last_line_and_not_only_whitespace() {
        todo!();
    }

    #[test]
    fn single_multispace_should_fail_if_input_empty() {
        todo!();
    }

    #[test]
    fn single_multispace_should_fail_if_not_multispace_character() {
        todo!();
    }

    #[test]
    fn single_multispace_should_succeed_if_tab() {
        todo!();
    }

    #[test]
    fn single_multispace_should_succeed_if_space() {
        todo!();
    }

    #[test]
    fn single_multispace_should_succeed_if_crlf() {
        todo!();
    }

    #[test]
    fn single_multispace_should_succeed_if_newline() {
        todo!();
    }

    #[test]
    fn url_should_fail_if_input_empty() {
        todo!();
    }

    #[test]
    fn url_should_fail_if_no_scheme_and_not_www() {
        todo!();
    }

    #[test]
    fn url_should_succeed_if_starts_with_www_and_will_add_https_as_scheme() {
        todo!();
    }

    #[test]
    fn url_should_succeed_if_starts_with_scheme() {
        // https://github.com/vimwiki/vimwiki.git
        // mailto:habamax@gmail.com
        // ftp://vim.org
        todo!();
    }
}
