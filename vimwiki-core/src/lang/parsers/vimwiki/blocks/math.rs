use crate::lang::{
    elements::{Located, MathBlock},
    parsers::{
        utils::{
            any_line, beginning_of_line, capture, context,
            count_remaining_bytes, cow_str, end_of_line_or_input, locate,
            take_line_until1,
        },
        IResult, Span,
    },
};
use nom::{
    bytes::complete::tag,
    character::complete::{char, line_ending, space0},
    combinator::{map_parser, not, opt},
    multi::many0,
    sequence::{delimited, preceded},
};
use std::borrow::Cow;

pub fn math_block<'a>(input: Span<'a>) -> IResult<Located<MathBlock<'a>>> {
    fn inner(input: Span) -> IResult<MathBlock> {
        // First, look for the beginning section including an optional environment
        let (input, (start_indent_size, environment)) =
            beginning_of_math_block(input)?;

        // Second, parse all lines while we don't encounter the closing block
        let (input, lines) = many0(preceded(
            not(end_of_math_block),
            map_parser(any_line, cow_str),
        ))(input)?;

        // Third, parse the closing block
        let (input, end_indent_size) = end_of_math_block(input)?;

        // We need to adjust the start of each line based on the indentation
        // of the code block start/end and the space at the beginning of a line
        let indent_size = std::cmp::min(start_indent_size, end_indent_size);
        let lines = lines
            .into_iter()
            .map(|mut line| {
                // Figure out total bytes of leading whitespace so we know if
                // the line is at the same level of indentation, further, or
                // earlier
                let cnt = line.len() - line.trim_start().len();
                let cnt_to_remove = std::cmp::min(cnt, indent_size);

                match line {
                    Cow::Borrowed(ref mut x) => *x = &x[cnt_to_remove..],
                    Cow::Owned(ref mut x) => {
                        *x = x[cnt_to_remove..].to_string()
                    }
                }

                line
            })
            .collect();

        let math_block = MathBlock::new(lines, environment);
        Ok((input, math_block))
    }

    context("Math Block", locate(capture(inner)))(input)
}

fn beginning_of_math_block<'a>(
    input: Span<'a>,
) -> IResult<(usize, Option<Cow<'a, str>>)> {
    let environment_parser =
        delimited(char('%'), take_line_until1("%"), char('%'));

    let (input, _) = beginning_of_line(input)?;
    let (input, indent_size) =
        map_parser(space0, count_remaining_bytes)(input)?;
    let (input, _) = tag("{{$")(input)?;
    let (input, environment) =
        opt(map_parser(environment_parser, cow_str))(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = line_ending(input)?;

    Ok((input, (indent_size, environment)))
}

fn end_of_math_block(input: Span) -> IResult<usize> {
    let (input, _) = beginning_of_line(input)?;
    let (input, indent_size) =
        map_parser(space0, count_remaining_bytes)(input)?;
    let (input, _) = tag("}}$")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = end_of_line_or_input(input)?;
    Ok((input, indent_size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn math_block_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_fail_if_does_not_start_with_dedicated_line() {
        let input = Span::from(indoc! {r"
                \sum_i a_i^2
            }}$
        "});
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_fail_if_does_not_end_with_dedicated_line() {
        let input = Span::from(indoc! {r"
            {{$
                \sum_i a_i^2
        "});
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_support_zero_lines_between_as_formula() {
        let input = Span::from(indoc! {r"
            {{$
            }}$
        "});
        let (input, m) = math_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume math block");
        assert!(m.lines.is_empty(), "Has lines unexpectedly");
        assert_eq!(m.environment, None);
    }

    #[test]
    fn math_block_should_consume_all_lines_between_as_formula() {
        let input = Span::from(indoc! {r"
            {{$
            \sum_i a_i^2
            =
            1
            }}$
        "});
        let (input, m) = math_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume math block");
        assert_eq!(
            m.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec![r"\sum_i a_i^2", "=", "1"]
        );
        assert_eq!(m.environment, None);
    }

    #[test]
    fn math_block_should_fail_if_environment_delimiters_not_used_correctly() {
        let input = Span::from(indoc! {r"
            {{$%align
            \sum_i a_i^2
            =
            1
            }}$
        "});
        assert!(math_block(input).is_err());

        let input = Span::from(indoc! {r"
            {{$align%
            \sum_i a_i^2
            =
            1
            }}$
        "});
        assert!(math_block(input).is_err());

        let input = Span::from(indoc! {r"
            {{$%%
            \sum_i a_i^2
            =
            1
            }}$
        "});
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_accept_optional_environment_specifier() {
        let input = Span::from(indoc! {r"
             {{$%align%
             \sum_i a_i^2 &= 1 + 1 \\
             &= 2.
             }}$
        "});
        let (input, m) = math_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume math block");
        assert_eq!(
            m.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec![r"\sum_i a_i^2 &= 1 + 1 \\", r"&= 2."]
        );
        assert_eq!(m.environment.as_deref(), Some("align"));
    }

    #[test]
    fn math_block_should_support_indentation() {
        // Lines are at same level
        let input =
            vec!["  {{$", "  one line", "  two line", "  }}$"].join("\n");
        let input = Span::from(input.as_str());
        let (input, p) = math_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume math block");
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec!["one line", "two line"]
        );

        // Start of math block is deeper
        let input = Span::from(indoc! {"
                {{$
            one line
            two line
            }}$
        "});
        let (input, p) = math_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume math block");
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec!["one line", "two line"]
        );

        // End of math block is deeper
        let input = Span::from(indoc! {"
            {{$
            one line
            two line
                }}$
        "});
        let (input, p) = math_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume math block");
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec!["one line", "two line"]
        );

        // Start & end of math block are deeper
        let input = Span::from(indoc! {"
                {{$
            one line
            two line
                }}$
        "});
        let (input, p) = math_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume math block");
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec!["one line", "two line"]
        );

        // Uneven lines
        let input = Span::from(indoc! {"
                {{$
            one line
                    two line
              three line
                four line
                 five line
                }}$
        "});
        let (input, p) = math_block(input).unwrap();
        assert!(input.is_empty(), "Did not consume math block");
        assert_eq!(
            p.lines.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
            vec![
                "one line",
                "    two line",
                "three line",
                "four line",
                " five line"
            ]
        );
    }
}
