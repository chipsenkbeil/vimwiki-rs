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
    character::complete::char,
    combinator::{map, not},
    error::context,
    multi::many1,
    sequence::delimited,
};

#[inline]
pub fn text(input: Span) -> VimwikiIResult<LC<String>> {
    // NOTE: Text as an inline component should continue until it encounters
    //       something different (math, keyword, link, etc); so, text should
    //       use all other inline components other than itself as not(...)
    //       in a pattern of recoginize(multi1(...))
    context(
        "Text",
        lc(pstring(take_line_while1(alt((
            not(math_inline),
            not(tags),
            not(link),
            not(decorated_text),
            not(keyword),
        ))))),
    )(input)
}

#[inline]
pub fn decorated_text(input: Span) -> VimwikiIResult<LC<DecoratedText>> {
    macro_rules! parser {
        ($name:expr, $start:expr, $end:expr, $decoration:ident) => {
            context(
                $name,
                delimited(
                    $start,
                    map(
                        many1(alt((
                            map(link, |c| c.map(DecoratedTextContent::from)),
                            map(keyword, |c| c.map(DecoratedTextContent::from)),
                            map(text, |c| c.map(DecoratedTextContent::from)),
                        ))),
                        |components| {
                            DecoratedText::new(
                                components,
                                Decoration::$decoration,
                            )
                        },
                    ),
                    $end,
                ),
            )
        };
        ($name:expr, $start_end:expr, $decoration:ident) => {
            parser!($name, $start_end, $start_end, $decoration)
        };
    }

    lc(alt((
        parser!("Bold Text", char('*'), Bold),
        parser!("Italic Text", char('_'), Italic),
        parser!("Bold Italic Text", tag("_*"), tag("*_"), BoldItalic),
        parser!("Italic Bold Text", tag("*_"), tag("_*"), BoldItalic),
        parser!("Strikeout Text", tag("~~"), Strikeout),
        parser!("Code Text", char('`'), Code),
        parser!("Super Script Text", char('^'), Superscript),
        parser!("Sub Script Text", tag(",,"), Subscript),
    )))(input)
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
    use super::*;

    #[test]
    fn text_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(text(input).is_err());
    }

    #[test]
    fn text_should_consume_until_encountering_inline_math() {
        let input = Span::new("abc123$math$");
        let (input, t) = text(input).unwrap();
        assert_eq!(*input.fragment(), "$math$", "Unexpected input consumption");
        assert_eq!(&t.component, "abc123");
    }

    #[test]
    fn text_should_consume_until_encountering_a_tag() {
        panic!("TODO: Implement");
    }

    #[test]
    fn text_should_consume_until_encountering_a_link() {
        panic!("TODO: Implement");
    }

    #[test]
    fn text_should_consume_until_encountering_decorated_text() {
        panic!("TODO: Implement");
    }

    #[test]
    fn text_should_consume_until_encountering_a_keyword() {
        panic!("TODO: Implement");
    }

    #[test]
    fn text_should_consume_until_reaching_end_of_line() {
        panic!("TODO: Implement");
    }

    #[test]
    fn text_should_consume_until_reaching_end_of_input() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(decorated_text(input).is_err());
    }

    #[test]
    fn decorated_text_should_fail_if_start_is_followed_by_whitespace() {
        let input = Span::new("* bold text*");
        assert!(decorated_text(input).is_err());
    }

    #[test]
    fn decorated_text_should_fail_if_end_is_preceded_by_whitespace() {
        let input = Span::new("*bold text *");
        assert!(decorated_text(input).is_err());
    }

    #[test]
    fn decorated_text_should_fail_if_start_and_end_separated_by_newline() {
        let input = Span::new("*bold text\n*");
        assert!(decorated_text(input).is_err());
    }

    #[test]
    fn decorated_text_should_support_bold() {
        let input = Span::new("*bold text*");
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
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_support_bold_italic() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_support_strikeout() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_support_code() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_support_superscript() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_support_subscript() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_support_links() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_support_keywords() {
        panic!("TODO: Implement");
    }

    #[test]
    fn keyword_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(keyword(input).is_err());
    }

    #[test]
    fn keyword_should_fail_if_not_a_matching_identifier() {
        let input = Span::new("NOTHING");
        assert!(keyword(input).is_err());
    }

    #[test]
    fn keyword_should_consume_specific_keywords() {
        let input = Span::new("DONE");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::DONE);

        let input = Span::new("FIXED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::FIXED);

        let input = Span::new("FIXME");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::FIXME);

        let input = Span::new("STARTED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::STARTED);

        let input = Span::new("TODO");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::TODO);

        let input = Span::new("XXX");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.component, Keyword::XXX);
    }
}
