use super::{
    components::{MathBlock, MathInline},
    utils::{
        any_line, beginning_of_line, context, end_of_line_or_input, lc,
        position, take_line_while1,
    },
    Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::tag,
    character::complete::{char, line_ending, space0},
    combinator::{map, not, opt},
    multi::many1,
    sequence::{delimited, preceded},
};

#[inline]
pub fn math_inline(input: Span) -> VimwikiIResult<LC<MathInline>> {
    let (input, pos) = position(input)?;

    // TODO: Is there any way to escape a $ inside a formula? If so, we will
    //       need to support detecting that rather than using take_till1
    let (input, math) = context(
        "Math Inline",
        map(
            delimited(char('$'), take_line_while1(not(char('$'))), char('$')),
            |s: Span| MathInline::new(s.fragment().to_string()),
        ),
    )(input)?;

    Ok((input, LC::from((math, pos, input))))
}

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
    let environment_parser = map(
        delimited(char('%'), take_line_while1(not(char('%'))), char('%')),
        |s: Span| s.fragment().to_string(),
    );

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
    use crate::lang::utils::new_span;
    use indoc::indoc;

    #[test]
    fn math_inline_should_fail_if_input_empty() {
        let input = new_span("");
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_fail_if_does_not_start_with_dollar_sign() {
        let input = new_span(r"\sum_i a_i^2 = 1$");
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_fail_if_does_not_end_with_dollar_sign() {
        let input = new_span(r"$\sum_i a_i^2 = 1");
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_fail_if_end_is_on_next_line() {
        let input = new_span(indoc! {r"
            $\sum_i a_i^2 = 1
            $
        "});
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_consume_all_text_between_dollar_signs_as_formula() {
        let input = new_span(r"$\sum_i a_i^2 = 1$");
        let (input, m) = math_inline(input).unwrap();
        assert!(input.fragment().is_empty(), "Math inline not consumed");
        assert_eq!(m.formula, r"\sum_i a_i^2 = 1");
    }

    #[test]
    fn math_block_should_fail_if_input_empty() {
        let input = new_span("");
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_fail_if_does_not_start_with_dedicated_line() {
        let input = new_span(indoc! {r"
                \sum_i a_i^2
            }}$
        "});
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_fail_if_does_not_end_with_dedicated_line() {
        let input = new_span(indoc! {r"
            {{$
                \sum_i a_i^2
        "});
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_consume_all_lines_between_as_formula() {
        let input = new_span(indoc! {r"
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
        let input = new_span(indoc! {r"
            {{$%align
            \sum_i a_i^2
            =
            1
            }}$
        "});
        assert!(math_block(input).is_err());

        let input = new_span(indoc! {r"
            {{$align%
            \sum_i a_i^2
            =
            1
            }}$
        "});
        assert!(math_block(input).is_err());

        let input = new_span(indoc! {r"
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
        let input = new_span(indoc! {r"
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
