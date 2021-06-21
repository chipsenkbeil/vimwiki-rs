use super::context;
use crate::lang::parsers::{Error, IResult, Span};
use nom::{bytes::complete::take_while1, character::complete::line_ending};

/// Parser that consumes next word only if is a whole word, meaning that this
/// parser is not used in the middle of a word; in regex this would be similar
/// to `\bword\b`
pub fn whole_word(input: Span) -> IResult<Span> {
    let consumed = input.as_consumed();
    let preceeding_is_okay =
        consumed.last().map_or(true, |c| c.is_ascii_whitespace());

    if !preceeding_is_okay {
        return Err(nom::Err::Error(Error::from_ctx(
            &input,
            "Preceeding character is not whitespace",
        )));
    }

    take_while1(|c: u8| !c.is_ascii_whitespace())(input)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_word_should_succeed_if_nonwhitespace_starting_at_beginning_of_line(
    ) {
        let input = Span::from("\nword").advance_start_by(1);
        let (_, word) = whole_word(input).unwrap();
        assert_eq!(word, "word");
    }

    #[test]
    fn whole_word_should_succeed_if_whitespace_preceeding_and_currently_nonwhitespace(
    ) {
        let input = Span::from(" word").advance_start_by(1);
        let (_, word) = whole_word(input).unwrap();
        assert_eq!(word, "word");
    }

    #[test]
    fn whole_word_should_succeed_if_nonwhitespace_at_very_beginning_of_span() {
        let input = Span::from("word");
        let (_, word) = whole_word(input).unwrap();
        assert_eq!(word, "word");
    }

    #[test]
    fn whole_word_should_fail_if_currently_whitespace() {
        let input = Span::from(" word");
        assert!(whole_word(input).is_err());

        let input = Span::from("\tword");
        assert!(whole_word(input).is_err());

        let input = Span::from("\rword");
        assert!(whole_word(input).is_err());
    }

    #[test]
    fn whole_word_should_fail_if_character_preceeding() {
        let input = Span::from("aword").advance_start_by(1);
        assert!(whole_word(input).is_err());
    }

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
