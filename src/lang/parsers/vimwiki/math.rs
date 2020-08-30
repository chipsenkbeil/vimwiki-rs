use super::{
    components::{Math, MathBlock, MathInline},
    utils::beginning_of_line,
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{char, line_ending, not_line_ending},
    combinator::{map, not, opt, recognize},
    error::context,
    multi::many1,
    sequence::{delimited, pair, tuple},
};
use nom_locate::position;

#[inline]
pub fn math(input: Span) -> VimwikiIResult<LC<Math>> {
    alt((
        map(math_inline, |c| c.map(Math::from)),
        map(math_block, |c| c.map(Math::from)),
    ))(input)
}

#[inline]
pub fn math_inline(input: Span) -> VimwikiIResult<LC<MathInline>> {
    let (input, pos) = position(input)?;

    // TODO: Is there any way to escape a $ inside a formula? If so, we will
    //       need to support detecting that rather than using take_till1
    let (input, math) = context(
        "Math Inline",
        map(
            delimited(char('$'), take_till1(|c| c == '$'), char('$')),
            |s: Span| MathInline::new(s.fragment().to_string()),
        ),
    )(input)?;

    Ok((input, LC::from((math, pos))))
}

#[inline]
pub fn math_block(input: Span) -> VimwikiIResult<LC<MathBlock>> {
    let (input, pos) = position(input)?;

    let environment_parser = map(
        delimited(char('%'), take_till1(|c| c == '%'), char('%')),
        |s: Span| s.fragment().to_string(),
    );

    let (input, environment) = map(
        tuple((
            beginning_of_line,
            tag("{{$"),
            opt(environment_parser),
            line_ending,
        )),
        |x| x.2,
    )(input)?;
    let (input, lines) = many1(map(
        recognize(pair(
            not(tuple((beginning_of_line, tag("$}}"), line_ending))),
            not_line_ending,
        )),
        |s| s.fragment().to_string(),
    ))(input)?;
    let (input, _) =
        tuple((beginning_of_line, tag("$}}"), line_ending))(input)?;

    let math_block = MathBlock::new(lines, environment);
    Ok((input, LC::from((math_block, pos))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_inline_should_fail_if_input_empty() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_inline_should_fail_if_does_not_start_with_dollar_sign() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_inline_should_fail_if_does_not_end_with_dollar_sign() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_inline_should_fail_if_end_is_on_next_line() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_inline_should_consume_all_text_between_dollar_signs_as_formula() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_block_should_fail_if_input_empty() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_block_should_fail_if_does_not_start_with_dedicated_line() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_block_should_fail_if_does_not_end_with_dedicated_line() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_block_should_consume_all_lines_between_as_formula() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_block_should_fail_if_environment_delimiters_not_used_correctly() {
        panic!("TODO: Implement");
    }

    #[test]
    fn math_block_should_accept_optional_environment_specifier() {
        panic!("TODO: Implement");
    }
}
