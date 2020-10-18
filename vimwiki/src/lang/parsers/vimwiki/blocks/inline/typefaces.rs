use super::{
    code::code_inline, comments::comment, links::link, math::math_inline,
    tags::tags,
};
use crate::lang::{
    elements::{
        DecoratedText, DecoratedTextContent, Keyword, Link, Located, Text,
    },
    parsers::{
        utils::{capture, context, cow_str, locate, surround_in_line1},
        IResult, Span,
    },
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till1},
    character::complete::newline,
    combinator::{map, map_parser, not, recognize},
    multi::many1,
    sequence::preceded,
};

#[inline]
pub fn text(input: Span) -> IResult<Located<Text>> {
    // Uses combination of short-circuiting and full checks to ensure we
    // can continue consuming text
    fn is_text(input: Span) -> IResult<()> {
        let (input, _) = not(newline)(input)?;
        let (input, _) = not(comment)(input)?;
        let (input, _) = not(code_inline)(input)?;
        let (input, _) = not(math_inline)(input)?;
        let (input, _) = not(tags)(input)?;
        let (input, _) = not(link)(input)?;
        let (input, _) = not(decorated_text)(input)?;
        let (input, _) = not(keyword)(input)?;
        Ok((input, ()))
    }

    /// Checks for a byte that is the start of anything inline that would not
    /// be regular text
    #[inline]
    fn start_of_non_text(b: u8) -> bool {
        b == b'\n'
            || b == b'%'
            || b == b'`'
            || b == b'$'
            || b == b':'
            || b == b'['
            || b == b'*'
            || b == b'_'
            || b == b'~'
            || b == b'^'
            || b == b','
            || b == b'D'
            || b == b'F'
            || b == b'S'
            || b == b'T'
            || b == b'X'
    }

    fn text_line(input: Span) -> IResult<Span> {
        recognize(many1(alt((
            take_till1(start_of_non_text),
            preceded(is_text, take(1usize)),
        ))))(input)
    }

    context("Text", locate(capture(map(cow_str(text_line), Text::new))))(input)
}

#[inline]
pub fn decorated_text(input: Span) -> IResult<Located<DecoratedText>> {
    context(
        "Decorated Text",
        locate(capture(alt((
            bold_italic_text_1,
            bold_italic_text_2,
            bold_text,
            italic_text,
            strikeout_text,
            superscript_text,
            subscript_text,
        )))),
    )(input)
}

fn bold_italic_text_1(input: Span) -> IResult<DecoratedText> {
    context(
        "Bold Italic 1 Decorated Text",
        map(
            map_parser(surround_in_line1("_*", "*_"), decorated_text_contents),
            DecoratedText::BoldItalic,
        ),
    )(input)
}

fn bold_italic_text_2(input: Span) -> IResult<DecoratedText> {
    context(
        "Bold Italic 2 Decorated Text",
        map(
            map_parser(surround_in_line1("*_", "_*"), decorated_text_contents),
            DecoratedText::BoldItalic,
        ),
    )(input)
}

fn italic_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Italic Decorated Text",
        map(
            map_parser(surround_in_line1("_", "_"), decorated_text_contents),
            DecoratedText::Italic,
        ),
    )(input)
}

fn bold_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Bold Decorated Text",
        map(
            map_parser(surround_in_line1("*", "*"), decorated_text_contents),
            DecoratedText::Bold,
        ),
    )(input)
}

fn strikeout_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Strikeout Decorated Text",
        map(
            map_parser(surround_in_line1("~~", "~~"), decorated_text_contents),
            DecoratedText::Strikeout,
        ),
    )(input)
}

fn superscript_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Superscript Decorated Text",
        map(
            map_parser(surround_in_line1("^", "^"), decorated_text_contents),
            DecoratedText::Superscript,
        ),
    )(input)
}

fn subscript_text(input: Span) -> IResult<DecoratedText> {
    context(
        "Subscript Decorated Text",
        map(
            map_parser(surround_in_line1(",,", ",,"), decorated_text_contents),
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
            map(tag("DONE"), |_| Keyword::DONE),
            map(tag("FIXED"), |_| Keyword::FIXED),
            map(tag("FIXME"), |_| Keyword::FIXME),
            map(tag("STARTED"), |_| Keyword::STARTED),
            map(tag("TODO"), |_| Keyword::TODO),
            map(tag("XXX"), |_| Keyword::XXX),
        )))),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::{Link, WikiLink};

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
        let input = Span::from("abc123:tag:");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            ":tag:",
            "Unexpected input consumption"
        );
        assert_eq!(t.into_inner(), Text::from("abc123"));
    }

    #[test]
    fn text_should_consume_until_encountering_a_link() {
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
        let input = Span::from("abc123\nsome other text");
        let (input, t) = text(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "\nsome other text",
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
    fn decorated_text_should_support_bold_italic_1() {
        let input = Span::from("_*bold italic text*_");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::BoldItalic(vec![Located::from(
                DecoratedTextContent::from(Text::from("bold italic text"))
            )])
        );
    }

    #[test]
    fn decorated_text_should_support_bold_italic_2() {
        let input = Span::from("*_bold italic text_*");
        let (input, dt) = decorated_text(input).unwrap();
        assert!(input.is_empty(), "Did not consume decorated text");
        assert_eq!(
            dt.into_inner(),
            DecoratedText::BoldItalic(vec![Located::from(
                DecoratedTextContent::from(Text::from("bold italic text"))
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
                DecoratedTextContent::from(Link::Wiki(WikiLink::from(
                    "some link"
                )))
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
                DecoratedTextContent::from(Keyword::TODO)
            )])
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
        assert_eq!(k.into_inner(), Keyword::DONE);

        let input = Span::from("FIXED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::FIXED);

        let input = Span::from("FIXME");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::FIXME);

        let input = Span::from("STARTED");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::STARTED);

        let input = Span::from("TODO");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::TODO);

        let input = Span::from("XXX");
        let (_, k) = keyword(input).unwrap();
        assert_eq!(k.into_inner(), Keyword::XXX);
    }
}
