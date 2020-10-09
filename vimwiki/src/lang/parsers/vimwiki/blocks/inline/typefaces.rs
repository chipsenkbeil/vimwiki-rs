use super::{
    code::code_inline,
    elements::{DecoratedText, DecoratedTextContent, Keyword, Text},
    links::link,
    math::math_inline,
    tags::tags,
    utils::{context, le, pstring, take_line_while1},
    Span, VimwikiIResult, LE,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, not},
    multi::many1,
};

#[inline]
pub fn text(input: Span) -> VimwikiIResult<LE<Text>> {
    fn is_text(input: Span) -> VimwikiIResult<()> {
        let (input, _) = not(code_inline)(input)?;
        let (input, _) = not(math_inline)(input)?;
        let (input, _) = not(tags)(input)?;
        let (input, _) = not(link)(input)?;
        let (input, _) = not(decorated_text)(input)?;
        let (input, _) = not(keyword)(input)?;
        Ok((input, ()))
    }

    context(
        "Text",
        le(map(pstring(take_line_while1(is_text)), Text::new)),
    )(input)
}

#[inline]
pub fn decorated_text(input: Span) -> VimwikiIResult<LE<DecoratedText>> {
    /// Parses inner content of decorated text
    fn dtc(
        start: &'static str,
        end: &'static str,
    ) -> impl Fn(Span) -> VimwikiIResult<Vec<LE<DecoratedTextContent>>> {
        move |input: Span| {
            fn is_other(
                end: &'static str,
            ) -> impl Fn(Span) -> VimwikiIResult<()> {
                move |input: Span| {
                    let (input, _) = not(link)(input)?;
                    let (input, _) = not(keyword)(input)?;
                    let (input, _) = not(tag(end))(input)?;
                    Ok((input, ()))
                }
            }

            fn other(
                end: &'static str,
            ) -> impl Fn(Span) -> VimwikiIResult<LE<Text>> {
                le(map(pstring(take_line_while1(is_other(end))), Text::new))
            }

            let (input, _) = tag(start)(input)?;
            let (input, contents) = many1(alt((
                map(link, |c| c.map(DecoratedTextContent::from)),
                map(keyword, |c| c.map(DecoratedTextContent::from)),
                map(other(end), |c| c.map(DecoratedTextContent::from)),
            )))(input)?;
            let (input, _) = tag(end)(input)?;
            Ok((input, contents))
        }
    }

    context(
        "Decorated Text",
        le(alt((
            map(dtc("_*", "*_"), DecoratedText::BoldItalic),
            map(dtc("*_", "_*"), DecoratedText::BoldItalic),
            map(dtc("*", "*"), DecoratedText::Bold),
            map(dtc("_", "_"), DecoratedText::Italic),
            map(dtc("~~", "~~"), DecoratedText::Strikeout),
            map(dtc("^", "^"), DecoratedText::Superscript),
            map(dtc(",,", ",,"), DecoratedText::Subscript),
        ))),
    )(input)
}

#[inline]
pub fn keyword(input: Span) -> VimwikiIResult<LE<Keyword>> {
    // TODO: Generate using strum to iterate over all keyword items,
    //       forming a tag based on the string version and parsing the
    //       string back into the keyword in a map (or possibly using
    //       the keyword enum variant directly if we can iterate over
    //       the variants themselves)
    context(
        "Keyword",
        le(alt((
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
    use super::super::elements::{Link, WikiLink};
    use super::*;
    use crate::lang::utils::Span;
    use std::path::PathBuf;

    #[test]
    fn text_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(text(input).is_err());
    }

    #[test]
    fn text_should_consume_until_encountering_inline_math() {
        let input = Span::from("abc123$math$");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.fragment_str(),
            "$math$",
            "Unexpected input consumption"
        );
        assert_eq!(t.element, Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_encountering_a_tag() {
        let input = Span::from("abc123:tag:");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.fragment_str(),
            ":tag:",
            "Unexpected input consumption"
        );
        assert_eq!(t.element, Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_encountering_a_link() {
        let input = Span::from("abc123[[some link]]");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.fragment_str(),
            "[[some link]]",
            "Unexpected input consumption"
        );
        assert_eq!(t.element, Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_encountering_decorated_text() {
        let input = Span::from("abc123*bold text*");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.fragment_str(),
            "*bold text*",
            "Unexpected input consumption"
        );
        assert_eq!(t.element, Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_encountering_a_keyword() {
        let input = Span::from("abc123 TODO");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.fragment_str(),
            "TODO",
            "Unexpected input consumption"
        );
        assert_eq!(t.element, Text::from("abc123 "));
    }

    #[test]
    fn text_should_consume_until_reaching_end_of_line() {
        let input = Span::from("abc123\nsome other text");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.fragment_str(),
            "\nsome other text",
            "Unexpected input consumption"
        );
        assert_eq!(t.element, Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_reaching_end_of_input() {
        let input = Span::from("abc123");
        let (input, t) = text(input).unwrap();
        assert_eq!(input.fragment_str(), "", "Unexpected input consumption");
        assert_eq!(t.element, Text::from("abc123"));
    }

    #[test]
    fn decorated_text_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(decorated_text(input).is_err());
    }

    #[test]
    fn decorated_text_should_fail_if_start_and_end_separated_by_newline() {
        let input = Span::from("*bold text\n*");
        assert!(decorated_text(input).is_err());
    }

    #[test]
    fn decorated_text_should_support_bold() {
        let input = Span::from("*bold text*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.element,
            DecoratedText::Bold(vec![LE::from(DecoratedTextContent::Text(
                Text::from("bold text")
            ))])
        );
    }

    #[test]
    fn decorated_text_should_support_italic() {
        let input = Span::from("_italic text_");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.element,
            DecoratedText::Italic(vec![LE::from(DecoratedTextContent::Text(
                Text::from("italic text")
            ))])
        );
    }

    #[test]
    fn decorated_text_should_support_bold_italic_1() {
        let input = Span::from("_*bold italic text*_");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.element,
            DecoratedText::BoldItalic(vec![LE::from(
                DecoratedTextContent::Text(Text::from("bold italic text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_bold_italic_2() {
        let input = Span::from("*_bold italic text_*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.element,
            DecoratedText::BoldItalic(vec![LE::from(
                DecoratedTextContent::Text(Text::from("bold italic text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_strikeout() {
        let input = Span::from("~~strikeout text~~");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.element,
            DecoratedText::Strikeout(vec![LE::from(
                DecoratedTextContent::Text(Text::from("strikeout text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_superscript() {
        let input = Span::from("^superscript text^");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.element,
            DecoratedText::Superscript(vec![LE::from(
                DecoratedTextContent::Text(Text::from("superscript text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_subscript() {
        let input = Span::from(",,subscript text,,");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.element,
            DecoratedText::Subscript(vec![LE::from(
                DecoratedTextContent::Text(Text::from("subscript text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_links() {
        let input = Span::from("*[[some link]]*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.element,
            DecoratedText::Bold(vec![LE::from(DecoratedTextContent::Link(
                Link::Wiki(WikiLink::from(PathBuf::from("some link")))
            ))])
        );
    }

    #[test]
    fn decorated_text_should_support_keywords() {
        let input = Span::from("*TODO*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume decorated text"
        );
        assert_eq!(
            dt.element,
            DecoratedText::Bold(vec![LE::from(DecoratedTextContent::Keyword(
                Keyword::TODO
            ))])
        );
    }

    #[test]
    fn keyword_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(keyword(input).is_err());
    }

    #[test]
    fn keyword_should_fail_if_not_a_matching_identifier() {
        let input = Span::from("NOTHING");
        assert!(keyword(input).is_err());
    }

    #[test]
    fn keyword_should_consume_specific_keywords() {
        let input = Span::from("DONE");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.element, Keyword::DONE);

        let input = Span::from("FIXED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.element, Keyword::FIXED);

        let input = Span::from("FIXME");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.element, Keyword::FIXME);

        let input = Span::from("STARTED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.element, Keyword::STARTED);

        let input = Span::from("TODO");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.element, Keyword::TODO);

        let input = Span::from("XXX");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.element, Keyword::XXX);
    }
}
