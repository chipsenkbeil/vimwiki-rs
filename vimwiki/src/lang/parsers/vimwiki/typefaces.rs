use super::{
    components::{DecoratedText, DecoratedTextContent, Decoration, Keyword},
    links::link,
    math::math_inline,
    tags::tags,
    utils::{lc, pstring, take_line_while1},
    Span, VimwikiIResult, LC,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, not},
    error::context,
    multi::many1,
};

#[inline]
pub fn text(input: Span) -> VimwikiIResult<LC<String>> {
    fn is_text(input: Span) -> VimwikiIResult<()> {
        let (input, _) = not(math_inline)(input)?;
        let (input, _) = not(tags)(input)?;
        let (input, _) = not(link)(input)?;
        let (input, _) = not(decorated_text)(input)?;
        let (input, _) = not(keyword)(input)?;
        Ok((input, ()))
    }

    context("Text", lc(pstring(take_line_while1(is_text))))(input)
}

#[inline]
pub fn decorated_text(input: Span) -> VimwikiIResult<LC<DecoratedText>> {
    #[inline]
    fn dt(
        start: &'static str,
        end: &'static str,
        decoration: Decoration,
    ) -> impl Fn(Span) -> VimwikiIResult<DecoratedText> {
        move |input: Span| {
            fn is_other(
                end: &'static str,
            ) -> impl Fn(Span) -> VimwikiIResult<()> {
                move |input: Span| {
                    let (input, _) = not(link)(input)?;
                    let (input, _) = not(keyword)(input)?;
                    let (input, _) = not(decorated_text)(input)?;
                    let (input, _) = not(tag(end))(input)?;
                    Ok((input, ()))
                }
            }

            fn other<'a>(
                end: &'static str,
            ) -> impl Fn(Span<'a>) -> VimwikiIResult<LC<String>> {
                lc(pstring(take_line_while1(is_other(end))))
            }

            let (input, _) = tag(start)(input)?;
            let (input, contents) = many1(alt((
                map(link, |c| c.map(DecoratedTextContent::from)),
                map(keyword, |c| c.map(DecoratedTextContent::from)),
                map(decorated_text, |c| c.map(DecoratedTextContent::from)),
                map(other(end), |c| c.map(DecoratedTextContent::from)),
            )))(input)?;
            let (input, _) = tag(end)(input)?;
            Ok((input, DecoratedText::new(contents, decoration)))
        }
    }

    context(
        "Decorated Text",
        lc(alt((
            dt("_*", "*_", Decoration::BoldItalic),
            dt("*_", "_*", Decoration::BoldItalic),
            dt("*", "*", Decoration::Bold),
            dt("_", "_", Decoration::Italic),
            dt("~~", "~~", Decoration::Strikeout),
            dt("`", "`", Decoration::Code),
            dt("^", "^", Decoration::Superscript),
            dt(",,", ",,", Decoration::Subscript),
        ))),
    )(input)
}

#[inline]
pub fn keyword(input: Span) -> VimwikiIResult<LC<Keyword>> {
    // TODO: Generate using strum to iterate over all keyword items,
    //       forming a tag based on the string version and parsing the
    //       string back into the keyword in a map (or possibly using
    //       the keyword enum variant directly if we can iterate over
    //       the variants themselves)
    context(
        "Keyword",
        lc(alt((
            map(tag("DONE"), |_| Keyword::DONE),
            map(tag("FIXED"), |_| Keyword::FIXED),
            map(tag("FIXME"), |_| Keyword::FIXME),
            map(tag("STARTED"), |_| Keyword::STARTED),
            map(tag("TODO"), |_| Keyword::TODO),
            map(tag("XXX"), |_| Keyword::XXX),
        ))),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::super::components::{Link, WikiLink};
    use super::*;
    use crate::lang::utils::new_span;
    use std::path::PathBuf;

    #[test]
    fn text_should_fail_if_input_empty() {
        let input = new_span("");
        assert!(text(input).is_err());
    }

    #[test]
    fn text_should_consume_until_encountering_inline_math() {
        let input = new_span("abc123$math$");
        let (input, t) = text(input).unwrap();
        assert_eq!(*input.fragment(), "$math$", "Unexpected input consumption");
        assert_eq!(&t.component, "abc123");
    }

    #[test]
    fn text_should_consume_until_encountering_a_tag() {
        let input = new_span("abc123:tag:");
        let (input, t) = text(input).unwrap();
        assert_eq!(*input.fragment(), ":tag:", "Unexpected input consumption");
        assert_eq!(&t.component, "abc123");
    }

    #[test]
    fn text_should_consume_until_encountering_a_link() {
        let input = new_span("abc123[[some link]]");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            *input.fragment(),
            "[[some link]]",
            "Unexpected input consumption"
        );
        assert_eq!(&t.component, "abc123");
    }

    #[test]
    fn text_should_consume_until_encountering_decorated_text() {
        let input = new_span("abc123*bold text*");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            *input.fragment(),
            "*bold text*",
            "Unexpected input consumption"
        );
        assert_eq!(&t.component, "abc123");
    }

    #[test]
    fn text_should_consume_until_encountering_a_keyword() {
        let input = new_span("abc123 TODO");
        let (input, t) = text(input).unwrap();
        assert_eq!(*input.fragment(), "TODO", "Unexpected input consumption");
        assert_eq!(&t.component, "abc123 ");
    }

    #[test]
    fn text_should_consume_until_reaching_end_of_line() {
        let input = new_span("abc123\nsome other text");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            *input.fragment(),
            "\nsome other text",
            "Unexpected input consumption"
        );
        assert_eq!(&t.component, "abc123");
    }

    #[test]
    fn text_should_consume_until_reaching_end_of_input() {
        let input = new_span("abc123");
        let (input, t) = text(input).unwrap();
        assert_eq!(*input.fragment(), "", "Unexpected input consumption");
        assert_eq!(&t.component, "abc123");
    }

    #[test]
    fn decorated_text_should_fail_if_input_empty() {
        let input = new_span("");
        assert!(decorated_text(input).is_err());
    }

    #[test]
    fn decorated_text_should_fail_if_start_and_end_separated_by_newline() {
        let input = new_span("*bold text\n*");
        assert!(decorated_text(input).is_err());
    }

    #[test]
    fn decorated_text_should_support_bold() {
        let input = new_span("*bold text*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Text(
                    "bold text".to_string()
                ))],
                Decoration::Bold
            )
        );
    }

    #[test]
    fn decorated_text_should_support_italic() {
        let input = new_span("_italic text_");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Text(
                    "italic text".to_string()
                ))],
                Decoration::Italic
            )
        );
    }

    #[test]
    fn decorated_text_should_support_bold_italic_1() {
        let input = new_span("_*bold italic text*_");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Text(
                    "bold italic text".to_string()
                ))],
                Decoration::BoldItalic
            )
        );
    }

    #[test]
    fn decorated_text_should_support_bold_italic_2() {
        let input = new_span("*_bold italic text_*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Text(
                    "bold italic text".to_string()
                ))],
                Decoration::BoldItalic
            )
        );
    }

    #[test]
    fn decorated_text_should_support_strikeout() {
        let input = new_span("~~strikeout text~~");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Text(
                    "strikeout text".to_string()
                ))],
                Decoration::Strikeout
            )
        );
    }

    #[test]
    fn decorated_text_should_support_code() {
        let input = new_span("`code text`");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Text(
                    "code text".to_string()
                ))],
                Decoration::Code
            )
        );
    }

    #[test]
    fn decorated_text_should_support_superscript() {
        let input = new_span("^superscript text^");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Text(
                    "superscript text".to_string()
                ))],
                Decoration::Superscript
            )
        );
    }

    #[test]
    fn decorated_text_should_support_subscript() {
        let input = new_span(",,subscript text,,");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Text(
                    "subscript text".to_string()
                ))],
                Decoration::Subscript
            )
        );
    }

    #[test]
    fn decorated_text_should_support_links() {
        let input = new_span("*[[some link]]*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Link(Link::Wiki(
                    WikiLink::from(PathBuf::from("some link"))
                )))],
                Decoration::Bold
            )
        );
    }

    #[test]
    fn decorated_text_should_support_keywords() {
        let input = new_span("*TODO*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![LC::from(DecoratedTextContent::Keyword(Keyword::TODO))],
                Decoration::Bold
            )
        );
    }

    #[test]
    fn decorated_text_should_support_nested_decorations() {
        let input = new_span("*Bold Text ~~Bold Strikeout Text~~*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.component,
            DecoratedText::new(
                vec![
                    LC::from(DecoratedTextContent::Text(
                        "Bold Text ".to_string()
                    )),
                    LC::from(DecoratedTextContent::DecoratedText(
                        DecoratedText::new(
                            vec![LC::from(DecoratedTextContent::Text(
                                "Bold Strikeout Text".to_string()
                            ))],
                            Decoration::Strikeout
                        )
                    ))
                ],
                Decoration::Bold
            )
        );
    }

    #[test]
    fn keyword_should_fail_if_input_empty() {
        let input = new_span("");
        assert!(keyword(input).is_err());
    }

    #[test]
    fn keyword_should_fail_if_not_a_matching_identifier() {
        let input = new_span("NOTHING");
        assert!(keyword(input).is_err());
    }

    #[test]
    fn keyword_should_consume_specific_keywords() {
        let input = new_span("DONE");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::DONE);

        let input = new_span("FIXED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::FIXED);

        let input = new_span("FIXME");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::FIXME);

        let input = new_span("STARTED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::STARTED);

        let input = new_span("TODO");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::TODO);

        let input = new_span("XXX");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::XXX);
    }
}
