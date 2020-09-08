use super::{
    components::{MathBlock, MathInline},
    utils::{beginning_of_line, position, take_line_while1},
    Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::tag,
    character::complete::{char, line_ending, not_line_ending},
    combinator::{map, not, opt, recognize},
    error::context,
    multi::many1,
    sequence::{delimited, pair, preceded, terminated},
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
    let (input, pos) = position(input)?;

    let environment_parser = map(
        delimited(char('%'), take_line_while1(not(char('%'))), char('%')),
        |s: Span| s.fragment().to_string(),
    );

    // First, look for the beginning section including an optional environment
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = tag("{{$")(input)?;
    let (input, environment) = opt(environment_parser)(input)?;
    let (input, _) = line_ending(input)?;

    // Second, parse all lines while we don't encounter the closing block
    let (input, lines) = many1(map(
        preceded(
            not(delimited(beginning_of_line, tag("$}}"), line_ending)),
            terminated(not_line_ending, line_ending),
        ),
        |s| s.fragment().to_string(),
    ))(input)?;

    // Third, parse the closing block
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = tag("$}}")(input)?;
    let (input, _) = line_ending(input)?;

    let math_block = MathBlock::new(lines, environment);
    Ok((input, LC::from((math_block, pos, input))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn math_inline_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_fail_if_does_not_start_with_dollar_sign() {
        let input = Span::new(r"\sum_i a_i^2 = 1$");
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_fail_if_does_not_end_with_dollar_sign() {
        let input = Span::new(r"$\sum_i a_i^2 = 1");
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_fail_if_end_is_on_next_line() {
        let input = Span::new(indoc! {r"
            $\sum_i a_i^2 = 1
            $
        "});
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_consume_all_text_between_dollar_signs_as_formula() {
        let input = Span::new(r"$\sum_i a_i^2 = 1$");
        let (input, m) = math_inline(input).unwrap();
        assert!(input.fragment().is_empty(), "Math inline not consumed");
        assert_eq!(m.formula, r"\sum_i a_i^2 = 1");
    }

    #[test]
    fn math_block_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_fail_if_does_not_start_with_dedicated_line() {
        let input = Span::new(indoc! {r"
                \sum_i a_i^2
            $}}
        "});
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_fail_if_does_not_end_with_dedicated_line() {
        let input = Span::new(indoc! {r"
            {{$
                \sum_i a_i^2
        "});
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_consume_all_lines_between_as_formula() {
        let input = Span::new(indoc! {r"
            {{$
            \sum_i a_i^2
            =
            1
            $}}
        "});
        let (input, m) = math_block(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume math block");
        assert_eq!(m.lines, vec![r"\sum_i a_i^2", "=", "1"]);
        assert_eq!(m.environment, None);
    }

    #[test]
    fn math_block_should_fail_if_environment_delimiters_not_used_correctly() {
        let input = Span::new(indoc! {r"
            {{$%align
            \sum_i a_i^2
            =
            1
            $}}
        "});
        assert!(math_block(input).is_err());

        let input = Span::new(indoc! {r"
            {{$align%
            \sum_i a_i^2
            =
            1
            $}}
        "});
        assert!(math_block(input).is_err());

        let input = Span::new(indoc! {r"
            {{$%%
            \sum_i a_i^2
            =
            1
            $}}
        "});
        assert!(math_block(input).is_err());
    }

    #[test]
    fn math_block_should_accept_optional_environment_specifier() {
        let input = Span::new(indoc! {r"
             {{$%align%
             \sum_i a_i^2 &= 1 + 1 \\
             &= 2.
             $}}
        "});
        let (input, m) = math_block(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume math block");
        assert_eq!(m.lines, vec![r"\sum_i a_i^2 &= 1 + 1 \\", r"&= 2."]);
        assert_eq!(m.environment, Some("align".to_string()));
    }
}
