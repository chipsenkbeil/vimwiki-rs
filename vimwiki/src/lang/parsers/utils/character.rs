use super::context;
use crate::lang::parsers::{IResult, Span};
use nom::character::complete::line_ending;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_of_line_or_input_should_fail_if_input_not_line_ending_or_empty() {
        assert!(end_of_line_or_input(Span::from("a")).is_err());
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
}
