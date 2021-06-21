use crate::lang::{
    elements::{Blockquote, Located},
    parsers::{
        utils::{
            blank_line, capture, context, cow_str, end_of_line_or_input, locate,
        },
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{not_line_ending, space0},
    combinator::{map, map_parser, value, verify},
    multi::{many0, many1},
    sequence::pair,
};
use std::borrow::Cow;

pub fn blockquote(input: Span) -> IResult<Located<Blockquote>> {
    context("Blockquote", alt((indented_blockquote, arrow_blockquote)))(input)
}

pub fn indented_blockquote(input: Span) -> IResult<Located<Blockquote>> {
    fn inner(input: Span) -> IResult<Blockquote> {
        let (input, lines) = many1(indented_blockquote_line)(input)?;
        Ok((input, Blockquote::new(lines)))
    }

    context("Indented Blockquote", locate(capture(inner)))(input)
}

/// Parses a blockquote line that begins with four or more spaces
#[inline]
fn indented_blockquote_line<'a>(input: Span<'a>) -> IResult<Cow<'a, str>> {
    let (input, _) = verify(space0, |s: &Span| s.remaining_len() >= 4)(input)?;
    let (input, text) = map_parser(
        verify(not_line_ending, |s: &Span<'a>| !s.is_only_whitespace()),
        cow_str,
    )(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, text))
}

pub fn arrow_blockquote(input: Span) -> IResult<Located<Blockquote>> {
    fn inner(input: Span) -> IResult<Blockquote> {
        // NOTE: > blockquotes allow blank lines inbetween
        let (input, lines) = map(
            pair(
                many1(arrow_blockquote_line),
                map(
                    many0(pair(
                        many0(value(Cow::from(""), blank_line)),
                        arrow_blockquote_line,
                    )),
                    |pairs| {
                        pairs
                            .into_iter()
                            .flat_map(|(mut blanks, bq)| {
                                blanks.push(bq);
                                blanks
                            })
                            .collect()
                    },
                ),
            ),
            |(head, rest)| vec![head, rest].concat(),
        )(input)?;
        Ok((input, Blockquote::new(lines)))
    }

    context("Arrow Blockquote", locate(capture(inner)))(input)
}

/// Parses a blockquote line that begins with >
#[inline]
fn arrow_blockquote_line<'a>(input: Span<'a>) -> IResult<Cow<'a, str>> {
    let (input, _) = space0(input)?;
    let (input, _) = tag("> ")(input)?;
    let (input, text) = map_parser(not_line_ending, cow_str)(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn blockquote_should_fail_if_not_starting_with_correct_prefix() {
        let input = Span::from(indoc! {"
            < Wrong prefix
            < on these lines
        "});
        assert!(blockquote(input).is_err());
    }

    #[test]
    fn blockquote_should_fail_if_not_enough_spaces_at_beginning() {
        let input = Span::from(indoc! {"
           Only using
           three spaces
        regular line starts here and is needed for indoc to have a baseline
        "});
        assert!(blockquote(input).is_err());
    }

    #[test]
    fn blockquote_should_fail_if_using_indented_format_and_line_is_empty() {
        let input = Span::from("        ");
        assert!(blockquote(input).is_err());
    }

    #[test]
    fn blockquote_should_stop_if_using_indented_format_and_reach_blank_line() {
        let input = Span::from(indoc! {"
            This is a blockquote
            that is using four spaces

            This is another blockquote
            that is using four spaces
        regular line starts here and is needed for indoc to have a baseline
        "});
        let (input, bq) = blockquote(input).unwrap();

        // Verify that only the first blockquote was consumed
        assert_eq!(
            input.as_unsafe_remaining_str(),
            indoc! {"

                This is another blockquote
                that is using four spaces
            regular line starts here and is needed for indoc to have a baseline
            "}
        );

        // Verify the contents of the blockquote
        assert_eq!(bq.lines.len(), 2, "Wrong number of blockquote lines found");
        assert_eq!(bq[0], "This is a blockquote");
        assert_eq!(bq[1], "that is using four spaces");
    }

    #[test]
    fn blockquote_should_stop_if_using_indented_format_and_reach_unindented_line(
    ) {
        let input = Span::from(indoc! {"
            This is a blockquote
            that is using four spaces
        regular line starts here and is needed for indoc to have a baseline
            This is another blockquote
            that is using four spaces
        "});
        let (input, bq) = blockquote(input).unwrap();

        // Verify that only the first blockquote was consumed
        assert_eq!(
            input.as_unsafe_remaining_str(),
            indoc! {"
            regular line starts here and is needed for indoc to have a baseline
                This is another blockquote
                that is using four spaces
            "}
        );

        // Verify the contents of the blockquote
        assert_eq!(bq.lines.len(), 2, "Wrong number of blockquote lines found");
        assert_eq!(bq[0], "This is a blockquote");
        assert_eq!(bq[1], "that is using four spaces");
    }

    #[test]
    fn blockquote_should_consume_blank_lines_if_using_angle_prefix() {
        let input = Span::from(indoc! {"
        > This is a blockquote
        > that is using prefixes

        > This is another blockquote
        > that is using prefixes

        this is a regular line
        > This is a third blockquote
        > that is using prefixes
        "});
        let (input, bq) = blockquote(input).unwrap();

        // Verify blank lines BETWEEN block quotes are consumed, but not those
        // after no more block quotes
        assert_eq!(
            input.as_unsafe_remaining_str(),
            indoc! {"

            this is a regular line
            > This is a third blockquote
            > that is using prefixes
            "}
        );

        // Verify the contents of the blockquote
        assert_eq!(bq.lines.len(), 5, "Wrong number of blockquote lines found");
        assert_eq!(bq[0], "This is a blockquote");
        assert_eq!(bq[1], "that is using prefixes");
        assert_eq!(bq[2], "");
        assert_eq!(bq[3], "This is another blockquote");
        assert_eq!(bq[4], "that is using prefixes");
    }
}
