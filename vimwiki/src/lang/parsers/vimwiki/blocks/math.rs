use super::{
    elements::MathBlock,
    utils::{
        any_line, beginning_of_line, context, end_of_line_or_input, lc,
        pstring, take_line_while1,
    },
    Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::tag,
    character::complete::{char, line_ending, space0},
    combinator::{not, opt},
    multi::many1,
    sequence::{delimited, preceded},
};

#[inline]
pub fn math_block(input: Span) -> VimwikiIResult<LC<MathBlock>> {
    fn inner(input: Span) -> VimwikiIResult<MathBlock> {
        // First, look for the beginning section including an optional environment
        let (input, environment) = beginning_of_math_block(input)?;

        // Second, parse all lines while we don't encounter the closing block
        let (input, lines) =
            many1(preceded(not(end_of_math_block), any_line))(input)?;

        // Third, parse the closing block
        let (input, _) = end_of_math_block(input)?;

        let math_block = MathBlock::new(lines, environment);
        Ok((input, math_block))
    }

    context("Math Block", lc(inner))(input)
}

fn beginning_of_math_block(input: Span) -> VimwikiIResult<Option<String>> {
    let environment_parser = pstring(delimited(
        char('%'),
        take_line_while1(not(char('%'))),
        char('%'),
    ));

    let (input, _) = beginning_of_line(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("{{$")(input)?;
    let (input, environment) = opt(environment_parser)(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = line_ending(input)?;

    Ok((input, environment))
}

fn end_of_math_block(input: Span) -> VimwikiIResult<()> {
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("}}$")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = end_of_line_or_input(input)?;
    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::Span;
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
    fn math_block_should_consume_all_lines_between_as_formula() {
        let input = Span::from(indoc! {r"
            {{$
            \sum_i a_i^2
            =
            1
            }}$
        "});
        let (input, m) = math_block(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume math block");
        assert_eq!(m.lines, vec![r"\sum_i a_i^2", "=", "1"]);
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
        assert!(input.fragment().is_empty(), "Did not consume math block");
        assert_eq!(m.lines, vec![r"\sum_i a_i^2 &= 1 + 1 \\", r"&= 2."]);
        assert_eq!(m.environment, Some("align".to_string()));
    }
}
