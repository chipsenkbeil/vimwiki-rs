use super::{
    code::code_inline, comments::comment, links::link, math::math_inline,
    tags::tags,
};
use crate::lang::{
    elements::{
        DecoratedText, DecoratedTextContent, Keyword, Link, Located, Text,
    },
    parsers::{
        utils::{
            capture, context, cow_str, deeper, locate, not_contains,
            surround_in_line1,
        },
        IResult, Span,
    },
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{crlf, newline},
    combinator::{map, map_parser, not, recognize},
    multi::many1,
    sequence::preceded,
};

#[inline]
pub fn text(input: Span) -> IResult<Located<Text>> {
    // Uses combination of short-circuiting and full checks to ensure we
    // can continue consuming text
    fn is_text(input: Span) -> IResult<()> {
        // Check for \n and \r\n
        let (input, _) = not(newline)(input)?;
        let (input, _) = not(crlf)(input)?;

        // Check for all other inline element types
        let (input, _) = not(comment)(input)?;
        let (input, _) = not(code_inline)(input)?;
        let (input, _) = not(math_inline)(input)?;
        let (input, _) = not(tags)(input)?;
        let (input, _) = not(link)(input)?;
        let (input, _) = not(decorated_text)(input)?;
        let (input, _) = not(keyword)(input)?;

        Ok((input, ()))
    }

    fn text_line(input: Span) -> IResult<Span> {
        recognize(many1(preceded(is_text, take(1usize))))(input)
    }

    context(
        "Text",
        locate(capture(map(map_parser(text_line, cow_str), Text::new))),
    )(input)
}

#[inline]
pub fn decorated_text(input: Span) -> IResult<Located<DecoratedText>> {
    context(
        "Decorated Text",
        locate(capture(alt((
            bold_text,
            italic_text,
            strikeout_text,
            superscript_text,
            subscript_text,
        )))),
    )(input)
}

fn italic_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Italic Decorated Text",
        map(
            map_parser(
                not_contains("%%", surround_in_line1("_", "_")),
                deeper(decorated_text_contents),
            ),
            DecoratedText::Italic,
        ),
    )(input)
}

fn bold_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Bold Decorated Text",
        map(
            map_parser(
                not_contains("%%", surround_in_line1("*", "*")),
                deeper(decorated_text_contents),
            ),
            DecoratedText::Bold,
        ),
    )(input)
}

fn strikeout_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Strikeout Decorated Text",
        map(
            map_parser(
                not_contains("%%", surround_in_line1("~~", "~~")),
                deeper(decorated_text_contents),
            ),
            DecoratedText::Strikeout,
        ),
    )(input)
}

fn superscript_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Superscript Decorated Text",
        map(
            map_parser(
                not_contains("%%", surround_in_line1("^", "^")),
                deeper(decorated_text_contents),
            ),
            DecoratedText::Superscript,
        ),
    )(input)
}

fn subscript_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Subscript Decorated Text",
        map(
            map_parser(
                not_contains("%%", surround_in_line1(",,", ",,")),
                deeper(decorated_text_contents),
            ),
            DecoratedText::Subscript,
        ),
    )(input)
}

fn decorated_text_contents<'a>(
    input: Span<'a>,
) -> IResult<Vec<Located<DecoratedTextContent<'a>>>> {
    fn inner(input: Span) -> IResult<Vec<Located<DecoratedTextContent>>> {
        many1(alt((
            map(link, |l: Located<Link>| l.map(DecoratedTextContent::from)),
            map(keyword, |l: Located<Keyword>| {
                l.map(DecoratedTextContent::from)
            }),
            map(decorated_text, |l: Located<DecoratedText>| {
                l.map(DecoratedTextContent::from)
            }),
            map(text, |l: Located<Text>| l.map(DecoratedTextContent::from)),
        )))(input)
    }

    context("Decorated Text Contents", inner)(input)
}

#[inline]
pub fn keyword(input: Span) -> IResult<Located<Keyword>> {
    // TODO: Generate using strum to iterate over all keyword items,
    //       forming a tag based on the string version and parsing the
    //       string back into the keyword in a map (or possibly using
    //       the keyword enum variant directly if we can iterate over
    //       the variants themselves)
    context(
        "Keyword",
        locate(capture(alt((
            map(tag("DONE"), |_| Keyword::Done),
            map(tag("FIXED"), |_| Keyword::Fixed),
            map(tag("FIXME"), |_| Keyword::Fixme),
            map(tag("STARTED"), |_| Keyword::Started),
            map(tag("TODO"), |_| Keyword::Todo),
            map(tag("XXX"), |_| Keyword::Xxx),
        )))),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::Link;
    use std::convert::TryFrom;
    use uriparse::URIReference;

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
            input.as_unsafe_remaining_str(),
            "$math$",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_encountering_a_tag() {
        let input = Span::from("abc123 :tag:");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            ":tag:",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123 "));
    }

    #[test]
    fn text_should_consume_until_encountering_a_wiki_link() {
        let input = Span::from("abc123[[some link]]");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "[[some link]]",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_encountering_a_transclusion_link() {
        let input = Span::from("abc123{{some link}}");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "{{some link}}",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_encountering_a_raw_link() {
        let input = Span::from("abc123 https://example.com/");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "https://example.com/",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123 "));
    }

    #[test]
    fn text_should_consume_until_encountering_decorated_text() {
        let input = Span::from("abc123*bold text*");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "*bold text*",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_encountering_a_keyword() {
        let input = Span::from("abc123 TODO");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "TODO",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123 "));
    }

    #[test]
    fn text_should_consume_until_reaching_end_of_line() {
        // Support \n line termination
        let input = Span::from("abc123\nsome other text");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "\nsome other text",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123"));

        // Support \r\n line termination
        let input = Span::from("abc123\r\nsome other text");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "\r\nsome other text",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_reaching_end_of_input() {
        let input = Span::from("abc123");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123"));
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
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::Bold(vec![Located::from(
                DecoratedTextContent::from(Text::from("bold text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_italic() {
        let input = Span::from("_italic text_");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::Italic(vec![Located::from(
                DecoratedTextContent::from(Text::from("italic text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_strikeout() {
        let input = Span::from("~~strikeout text~~");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::Strikeout(vec![Located::from(
                DecoratedTextContent::from(Text::from("strikeout text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_superscript() {
        let input = Span::from("^superscript text^");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::Superscript(vec![Located::from(
                DecoratedTextContent::from(Text::from("superscript text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_subscript() {
        let input = Span::from(",,subscript text,,");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::Subscript(vec![Located::from(
                DecoratedTextContent::from(Text::from("subscript text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_links() {
        let input = Span::from("*[[some link]]*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::Bold(vec![Located::from(
                DecoratedTextContent::from(Link::new_wiki_link(
                    URIReference::try_from("some%20link").unwrap(),
                    None
                ))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_keywords() {
        let input = Span::from("*TODO*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::Bold(vec![Located::from(
                DecoratedTextContent::from(Keyword::Todo)
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_nested_decorated_text() {
        let input = Span::from("*bold _italic_*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::Bold(vec![
                Located::from(DecoratedTextContent::from(Text::from("bold "))),
                Located::from(DecoratedTextContent::from(
                    DecoratedText::Italic(vec![Located::from(
                        DecoratedTextContent::from(Text::from("italic"))
                    )])
                ))
            ])
        );
    }

    #[test]
    fn decorated_text_should_properly_adjust_depth_for_content() {
        let input = Span::from(
            "*bold _italic_ DONE ~~strikeout~~ ^^superscript^^ ,,subscript,, [[some link]]*",
        );
        let (_, dt) = decorated_text(input).unwrap();

        assert_eq!(dt.depth(), 0, "Decorated text was at wrong level");
        for content in dt.iter() {
            assert_eq!(
                content.depth(),
                1,
                "Decorated text content depth was at wrong level"
            );
        }
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
        assert_eq!(k.into_inner(), Keyword::Done);

        let input = Span::from("FIXED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::Fixed);

        let input = Span::from("FIXME");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::Fixme);

        let input = Span::from("STARTED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::Started);

        let input = Span::from("TODO");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::Todo);

        let input = Span::from("XXX");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::Xxx);
    }
}
