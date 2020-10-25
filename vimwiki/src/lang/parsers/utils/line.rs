use super::{context, end_of_line_or_input, take_until_end_of_line_or_input};
use crate::lang::parsers::{Error, IResult, Span};
use nom::{
    branch::alt,
    character::complete::{line_ending, space0, space1},
    sequence::{preceded, terminated},
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[inline]
    fn take_and_toss(n: usize) -> impl Fn(Span) -> IResult<()> {
        move |input: Span| {
            nom::combinator::value((), nom::bytes::complete::take(n))(input)
        }
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
}
