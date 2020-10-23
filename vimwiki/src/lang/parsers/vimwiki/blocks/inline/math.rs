use crate::lang::{
    elements::{Located, MathInline},
    parsers::{
        utils::{
            capture, context, cow_str, locate, not_contains, take_line_until1,
        },
        IResult, Span,
    },
};
use nom::{character::complete::char, combinator::map, sequence::delimited};

#[inline]
pub fn math_inline(input: Span) -> IResult<Located<MathInline>> {
    fn inner(input: Span) -> IResult<MathInline> {
        // TODO: Is there any way to escape a $ inside a formula? If so, we will
        //       need to support detecting that
        map(
            cow_str(delimited(
                char('$'),
                not_contains("%%", take_line_until1("$")),
                char('$'),
            )),
            MathInline::new,
        )(input)
    }

    context("Math Inline", locate(capture(inner)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn math_inline_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_fail_if_does_not_start_with_dollar_sign() {
        let input = Span::from(r"\sum_i a_i^2 = 1$");
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_fail_if_does_not_end_with_dollar_sign() {
        let input = Span::from(r"$\sum_i a_i^2 = 1");
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_fail_if_end_is_on_next_line() {
        let input = Span::from(indoc! {r"
            $\sum_i a_i^2 = 1
            $
        "});
        assert!(math_inline(input).is_err());
    }

    #[test]
    fn math_inline_should_consume_all_text_between_dollar_signs_as_formula() {
        let input = Span::from(r"$\sum_i a_i^2 = 1$");
        let (input, m) = math_inline(input).unwrap();
        assert!(input.is_empty(), "Math inline not consumed");
        assert_eq!(m.formula, r"\sum_i a_i^2 = 1");
    }
}
