use super::{
    components::PreformattedText,
    utils::{
        any_line, beginning_of_line, end_of_line_or_input, position,
        take_line_while, take_line_while1,
    },
    Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::tag,
    character::complete::{char, space0},
    combinator::{map, not},
    multi::{many1, separated_list},
    sequence::{delimited, preceded, separated_pair},
};
use std::collections::HashMap;

#[inline]
pub fn preformatted_text(input: Span) -> VimwikiIResult<LC<PreformattedText>> {
    let (input, pos) = position(input)?;

    let (input, metadata) = preformatted_text_start(input)?;
    let (input, lines) =
        many1(preceded(not(preformatted_text_end), any_line))(input)?;
    let (input, _) = preformatted_text_end(input)?;

    Ok((
        input,
        LC::from((PreformattedText::new(metadata, lines), pos, input)),
    ))
}

#[inline]
fn preformatted_text_start(
    input: Span,
) -> VimwikiIResult<HashMap<String, String>> {
    // First, verify we have the start of a block and consume it
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("{{{")(input)?;

    // Second, look for optional metadata and consume it
    let (input, mut pairs) = separated_list(
        char(';'),
        map(
            separated_pair(
                take_line_while1(not(char('='))),
                char('='),
                delimited(
                    char('"'),
                    take_line_while(not(char('"'))),
                    char('"'),
                ),
            ),
            |(k, v): (Span, Span)| {
                (k.fragment().to_string(), v.fragment().to_string())
            },
        ),
    )(input)?;

    // Third, consume end of line
    let (input, _) = space0(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    let mut metadata = HashMap::new();
    for (k, v) in pairs.drain(..) {
        metadata.insert(k, v);
    }

    Ok((input, metadata))
}

#[inline]
fn preformatted_text_end(input: Span) -> VimwikiIResult<()> {
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("}}}")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn preformatted_text_should_fail_if_does_not_have_starting_line() {
        let input = Span::new(indoc! {r"
            some code
            }}}
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_fail_if_starting_block_not_on_own_line() {
        let input = Span::new(indoc! {r"
            {{{some code
            }}}
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_fail_if_does_not_have_ending_line() {
        let input = Span::new(indoc! {r"
            {{{
            some code
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_fail_if_ending_block_not_on_own_line() {
        let input = Span::new(indoc! {r"
            {{{
            some code}}}
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_fail_if_does_not_have_lines_inbetween() {
        let input = Span::new(indoc! {r"
            {{{
            }}}
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_parse_all_lines_between() {
        let input = Span::new(indoc! {r"
            {{{
            Tyger! Tyger! burning bright
             In the forests of the night,
              What immortal hand or eye
               Could frame thy fearful symmetry?
            In what distant deeps or skies
             Burnt the fire of thine eyes?
              On what wings dare he aspire?
               What the hand dare sieze the fire?
            }}}
        "});
        let (input, p) = preformatted_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume preformatted block"
        );
        assert_eq!(
            p.lines,
            vec![
                "Tyger! Tyger! burning bright",
                " In the forests of the night,",
                "  What immortal hand or eye",
                "   Could frame thy fearful symmetry?",
                "In what distant deeps or skies",
                " Burnt the fire of thine eyes?",
                "  On what wings dare he aspire?",
                "   What the hand dare sieze the fire?",
            ]
        );
        assert!(p.metadata.is_empty(), "Has unexpected metadata");
    }

    #[test]
    fn preformatted_text_should_support_single_metadata() {
        let input = Span::new(indoc! {r#"
            {{{class="brush: python"
            def hello(world):
                for x in range(10):
                    print("Hello {0} number {1}".format(world, x))
            }}}
        "#});
        let (input, p) = preformatted_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume preformatted block"
        );
        assert_eq!(
            p.lines,
            vec![
                r#"def hello(world):"#,
                r#"    for x in range(10):"#,
                r#"        print("Hello {0} number {1}".format(world, x))"#,
            ]
        );
        assert_eq!(p.metadata.get("class"), Some(&"brush: python".to_string()));
    }

    #[test]
    fn preformatted_text_should_support_multiple_metadata() {
        let input = Span::new(indoc! {r#"
            {{{class="brush: python";style="position: relative"
            def hello(world):
                for x in range(10):
                    print("Hello {0} number {1}".format(world, x))
            }}}
        "#});
        let (input, p) = preformatted_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume preformatted block"
        );
        assert_eq!(
            p.lines,
            vec![
                r#"def hello(world):"#,
                r#"    for x in range(10):"#,
                r#"        print("Hello {0} number {1}".format(world, x))"#,
            ]
        );
        assert_eq!(p.metadata.get("class"), Some(&"brush: python".to_string()));
        assert_eq!(
            p.metadata.get("style"),
            Some(&"position: relative".to_string())
        );
    }
}
