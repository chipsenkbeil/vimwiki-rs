use crate::lang::{
    elements::{CodeInline, Located},
    parsers::{
        utils::{capture, context, cow_str, locate, surround_in_line1},
        IResult, Span,
    },
};
use nom::combinator::map;

#[inline]
pub fn code_inline(input: Span) -> IResult<Located<CodeInline>> {
    fn inner(input: Span) -> IResult<CodeInline> {
        map(cow_str(surround_in_line1("`", "`")), CodeInline::new)(input)
    }

    context("Code Inline", locate(capture(inner)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(input.is_empty(), "Code inline not consumed");
        assert_eq!(m.code, r"some code");
    }
}
