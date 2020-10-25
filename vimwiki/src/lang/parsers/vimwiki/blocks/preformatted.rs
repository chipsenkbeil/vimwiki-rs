use crate::lang::{
    elements::{Located, PreformattedText},
    parsers::{
        utils::{
            any_line, beginning_of_line, capture, context, cow_str,
            end_of_line_or_input, locate, take_line_until, take_line_until1,
        },
        IResult, Span,
    },
};
use nom::{
    bytes::complete::tag,
    character::complete::{char, space0},
    combinator::{map_parser, not, opt, verify},
    multi::{many1, separated_list},
    sequence::{delimited, preceded, separated_pair, terminated},
};
use std::{borrow::Cow, collections::HashMap};

type MaybeLang<'a> = Option<Cow<'a, str>>;
type Metadata<'a> = HashMap<Cow<'a, str>, Cow<'a, str>>;

#[inline]
pub fn preformatted_text(input: Span) -> IResult<Located<PreformattedText>> {
    fn inner(input: Span) -> IResult<PreformattedText> {
        let (input, (maybe_lang, metadata)) = preformatted_text_start(input)?;
        let (input, lines) = many1(preceded(
            not(preformatted_text_end),
            map_parser(any_line, cow_str),
        ))(input)?;
        let (input, _) = preformatted_text_end(input)?;

        Ok((input, PreformattedText::new(maybe_lang, metadata, lines)))
    }

    context("Preformatted Text", locate(capture(inner)))(input)
}

#[inline]
fn preformatted_text_start<'a>(
    input: Span<'a>,
) -> IResult<(MaybeLang<'a>, Metadata<'a>)> {
    // First, verify we have the start of a block and consume it
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("{{{")(input)?;

    // Second, look for optional language and consume it
    //
    // e.g. {{{c++ -> Some("c++")
    let (input, maybe_lang) = opt(terminated(
        map_parser(
            verify(take_line_until1(";"), |s: &Span| {
                !s.as_remaining().contains(&b'=')
            }),
            cow_str,
        ),
        opt(char(';')),
    ))(input)?;

    // Third, look for optional metadata and consume it
    let (input, mut pairs) = separated_list(
        char(';'),
        separated_pair(
            map_parser(take_line_until1("="), cow_str),
            char('='),
            delimited(
                char('"'),
                map_parser(take_line_until("\""), cow_str),
                char('"'),
            ),
        ),
    )(input)?;

    // Fourth, consume end of line
    let (input, _) = space0(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    let mut metadata = HashMap::new();
    for (k, v) in pairs.drain(..) {
        metadata.insert(k, v);
    }

    Ok((input, (maybe_lang, metadata)))
}

#[inline]
fn preformatted_text_end(input: Span) -> IResult<()> {
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
        let input = Span::from(indoc! {r"
            some code
            }}}
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_fail_if_starting_block_not_on_own_line() {
        let input = Span::from(indoc! {r"
            {{{some code
            }}}
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_fail_if_does_not_have_ending_line() {
        let input = Span::from(indoc! {r"
            {{{
            some code
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_fail_if_ending_block_not_on_own_line() {
        let input = Span::from(indoc! {r"
            {{{
            some code}}}
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_fail_if_does_not_have_lines_inbetween() {
        let input = Span::from(indoc! {r"
            {{{
            }}}
        "});
        assert!(preformatted_text(input).is_err());
    }

    #[test]
    fn preformatted_text_should_support_lang_shorthand() {
        let input = Span::from(indoc! {r"
            {{{c++
            some code
            }}}
        "});
        let (input, p) = preformatted_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume preformatted block");
        assert_eq!(p.lang, Some(Cow::from("c++")));
        assert_eq!(p.lines, vec![Cow::from("some code")]);
        assert!(p.metadata.is_empty(), "Has unexpected metadata");
    }

    #[test]
    fn preformatted_text_should_support_lang_shorthand_with_metadata() {
        let input = Span::from(indoc! {r#"
            {{{c++;key="value"
            some code
            }}}
        "#});
        let (input, p) = preformatted_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume preformatted block");
        assert_eq!(p.lang, Some(Cow::from("c++")));
        assert_eq!(p.lines, vec![Cow::from("some code")]);
        assert_eq!(p.metadata.get("key"), Some(&Cow::from("value")));
    }

    #[test]
    fn preformatted_text_should_parse_all_lines_between() {
        let input = Span::from(indoc! {r"
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
        assert!(input.is_empty(), "Did not consume preformatted block");
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
        assert!(p.lang.is_none(), "Has unexpected language");
        assert!(p.metadata.is_empty(), "Has unexpected metadata");
    }

    #[test]
    fn preformatted_text_should_support_single_metadata() {
        let input = Span::from(indoc! {r#"
            {{{class="brush: python"
            def hello(world):
                for x in range(10):
                    print("Hello {0} number {1}".format(world, x))
            }}}
        "#});
        let (input, p) = preformatted_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume preformatted block");
        assert_eq!(
            p.lines,
            vec![
                r#"def hello(world):"#,
                r#"    for x in range(10):"#,
                r#"        print("Hello {0} number {1}".format(world, x))"#,
            ]
        );
        assert_eq!(p.metadata.get("class"), Some(&Cow::from("brush: python")));
    }

    #[test]
    fn preformatted_text_should_support_multiple_metadata() {
        let input = Span::from(indoc! {r#"
            {{{class="brush: python";style="position: relative"
            def hello(world):
                for x in range(10):
                    print("Hello {0} number {1}".format(world, x))
            }}}
        "#});
        let (input, p) = preformatted_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume preformatted block");
        assert_eq!(
            p.lines,
            vec![
                r#"def hello(world):"#,
                r#"    for x in range(10):"#,
                r#"        print("Hello {0} number {1}".format(world, x))"#,
            ]
        );
        assert!(p.lang.is_none(), "Has unexpected language");
        assert_eq!(p.metadata.get("class"), Some(&Cow::from("brush: python")));
        assert_eq!(
            p.metadata.get("style"),
            Some(&Cow::from("position: relative"))
        );
    }
}
