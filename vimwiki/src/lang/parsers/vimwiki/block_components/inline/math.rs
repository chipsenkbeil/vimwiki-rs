use super::{
    components::MathInline,
    utils::{context, lc, pstring, take_line_while1},
    Span, VimwikiIResult, LC,
};
use nom::{
    character::complete::char,
    combinator::{map, not},
    sequence::delimited,
};

#[inline]
pub fn math_inline(input: Span) -> VimwikiIResult<LC<MathInline>> {
    fn inner(input: Span) -> VimwikiIResult<MathInline> {
        // TODO: Is there any way to escape a $ inside a formula? If so, we will
        //       need to support detecting that rather than using take_till1
        map(
            pstring(delimited(
                char('$'),
                take_line_while1(not(char('$'))),
                char('$'),
            )),
            MathInline::new,
        )(input)
    }

    context("Math Inline", lc(inner))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::Span;
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
        assert!(input.fragment().is_empty(), "Math inline not consumed");
        assert_eq!(m.formula, r"\sum_i a_i^2 = 1");
    }
}
