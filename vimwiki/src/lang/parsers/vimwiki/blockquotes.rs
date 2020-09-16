use super::{
    components::Blockquote,
    utils::{beginning_of_line, blank_line, end_of_line_or_input, position},
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
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
                    map(
                        many0(pair(
                            many0(value(String::new(), blank_line)),
                            blockquote_line_2,
                        )),
                        |mut pairs| {
                            pairs
                                .drain(..)
                                .flat_map(|(mut blanks, bq)| {
                                    blanks.push(bq);
                                    blanks
                                })
                                .collect()
                        },
                    ),
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
    let (input, _) = tag("> ")(input)?;
    let (input, text) =
        map(not_line_ending, |s: Span| s.fragment().to_string())(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::new_span;
    use indoc::indoc;

    #[test]
    fn blockquote_should_fail_if_not_starting_with_correct_prefix() {
        let input = new_span(indoc! {"
            < Wrong prefix
            < on these lines
        "});
        assert!(blockquote(input).is_err());
    }

    #[test]
    fn blockquote_should_fail_if_not_enough_spaces_at_beginning() {
        let input = new_span(indoc! {"
           Only using
           three spaces
        regular line starts here and is needed for indoc to have a baseline
        "});
        assert!(blockquote(input).is_err());
    }

    #[test]
    fn blockquote_should_stop_if_using_indented_format_and_reach_blank_line() {
        let input = new_span(indoc! {"
            This is a blockquote
            that is using four spaces

            This is another blockquote
            that is using four spaces
        regular line starts here and is needed for indoc to have a baseline
        "});
        let (input, bq) = blockquote(input).unwrap();

        // Verify that only the first blockquote was consumed
        assert_eq!(
            *input.fragment(),
            indoc! {"

                This is another blockquote
                that is using four spaces
            regular line starts here and is needed for indoc to have a baseline
            "}
        );

        // Verify the contents of the blockquote
        assert_eq!(bq.lines.len(), 2, "Wrong number of blockquote lines found");
        assert_eq!(bq.lines[0], "This is a blockquote");
        assert_eq!(bq.lines[1], "that is using four spaces");
    }

    #[test]
    fn blockquote_should_stop_if_using_indented_format_and_reach_unindented_line(
    ) {
        let input = new_span(indoc! {"
            This is a blockquote
            that is using four spaces
        regular line starts here and is needed for indoc to have a baseline
            This is another blockquote
            that is using four spaces
        "});
        let (input, bq) = blockquote(input).unwrap();

        // Verify that only the first blockquote was consumed
        assert_eq!(
            *input.fragment(),
            indoc! {"
            regular line starts here and is needed for indoc to have a baseline
                This is another blockquote
                that is using four spaces
            "}
        );

        // Verify the contents of the blockquote
        assert_eq!(bq.lines.len(), 2, "Wrong number of blockquote lines found");
        assert_eq!(bq.lines[0], "This is a blockquote");
        assert_eq!(bq.lines[1], "that is using four spaces");
    }

    #[test]
    fn blockquote_should_consume_blank_lines_if_using_angle_prefix() {
        let input = new_span(indoc! {"
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
            *input.fragment(),
            indoc! {"

            this is a regular line
            > This is a third blockquote
            > that is using prefixes
            "}
        );

        // Verify the contents of the blockquote
        assert_eq!(bq.lines.len(), 5, "Wrong number of blockquote lines found");
        assert_eq!(bq.lines[0], "This is a blockquote");
        assert_eq!(bq.lines[1], "that is using prefixes");
        assert_eq!(bq.lines[2], "");
        assert_eq!(bq.lines[3], "This is another blockquote");
        assert_eq!(bq.lines[4], "that is using prefixes");
    }
}
