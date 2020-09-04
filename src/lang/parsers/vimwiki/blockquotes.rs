use super::{
    components::Blockquote,
    utils::{beginning_of_line, blank_line, end_of_line_or_input, position},
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    character::complete::{not_line_ending, space0},
    combinator::{map, value, verify},
    error::context,
    multi::{many0, many1},
    sequence::pair,
};

#[inline]
pub fn blockquote(input: Span) -> VimwikiIResult<LC<Blockquote>> {
    let (input, pos) = position(input)?;

    let (input, lines) = context(
        "Blockquote",
        alt((
            // NOTE: Indented blockquotes do not allow blank lines
            many1(blockquote_line_1),
            // NOTE: > blockquotes allow blank lines inbetween
            map(
                pair(
                    many1(blockquote_line_2),
                    many0(alt((
                        blockquote_line_2,
                        value(String::new(), blank_line),
                    ))),
                ),
                |(head, rest)| vec![head, rest].concat(),
            ),
        )),
    )(input)?;

    Ok((input, LC::from((Blockquote::new(lines), pos, input))))
}

/// Parses a blockquote line that begins with four or more spaces
#[inline]
fn blockquote_line_1(input: Span) -> VimwikiIResult<String> {
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = verify(space0, |s: &Span| s.fragment().len() >= 4)(input)?;
    let (input, text) =
        map(not_line_ending, |s: Span| s.fragment().to_string())(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, text))
}

/// Parses a blockquote line that begins with >
#[inline]
fn blockquote_line_2(input: Span) -> VimwikiIResult<String> {
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = verify(space0, |s: &Span| s.fragment().len() >= 4)(input)?;
    let (input, text) =
        map(not_line_ending, |s: Span| s.fragment().to_string())(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blockquote_should_fail_if_not_starting_with_correct_prefix() {
        todo!();
    }

    #[test]
    fn blockquote_should_fail_if_not_enough_spaces_at_beginning() {
        todo!();
    }

    #[test]
    fn blockquote_should_stop_if_using_indented_format_and_reach_blank_line() {
        todo!();
    }

    #[test]
    fn blockquote_should_consume_blank_lines_if_using_angle_prefix() {
        todo!();
    }
}
