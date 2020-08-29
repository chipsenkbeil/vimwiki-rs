use super::{new_nom_error, Position, Span, VimwikiIResult};
use nom::{
    character::complete::{line_ending, space0},
    combinator::{opt, rest_len, value, verify},
    error::context,
    sequence::{pair, tuple},
};

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
pub fn blank_line(input: Span) -> VimwikiIResult<()> {
    // 1. We must assert (using span) that we're actually at the beginning of
    //    a line, otherwise this could have been used somewhere after some
    //    other content was matched, and we don't want it to succeed
    //
    // 2. We want to eat up all spaces & tabs on that line, followed by a line
    //    termination. If we happen to be at end of input, then that's okay as
    //    that would be a blank line at the end of a file
    context(
        "Blank Line",
        value((), tuple((beginning_of_line, space0, end_of_line_or_input))),
    )(input)
}
