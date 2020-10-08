use super::{
    elements::CodeInline,
    utils::{context, le, pstring, take_line_while1},
    Span, VimwikiIResult, LE,
};
use nom::{
    character::complete::char,
    combinator::{map, not},
    sequence::delimited,
};

#[inline]
pub fn code_inline(input: Span) -> VimwikiIResult<LE<CodeInline>> {
    fn inner(input: Span) -> VimwikiIResult<CodeInline> {
        map(
            pstring(delimited(
                char('`'),
                take_line_while1(not(char('`'))),
                char('`'),
            )),
            CodeInline::new,
        )(input)
    }

    context("Code Inline", le(inner))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::Span;
    use indoc::indoc;

    #[test]
    fn code_inline_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(code_inline(input).is_err());
    }

    #[test]
    fn code_inline_should_fail_if_does_not_start_with_backtick() {
        let input = Span::from(r"some code`");
        assert!(code_inline(input).is_err());
    }

    #[test]
    fn code_inline_should_fail_if_does_not_end_with_backtick() {
        let input = Span::from(r"`some code");
        assert!(code_inline(input).is_err());
    }

    #[test]
    fn code_inline_should_fail_if_end_is_on_next_line() {
        let input = Span::from(indoc! {r"
            `some code
            `
        "});
        assert!(code_inline(input).is_err());
    }

    #[test]
    fn code_inline_should_consume_all_text_between_backticks_as_code() {
        let input = Span::from(r"`some code`");
        let (input, m) = code_inline(input).unwrap();
        assert!(input.fragment().is_empty(), "Code inline not consumed");
        assert_eq!(m.code, r"some code");
    }
}
