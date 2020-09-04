use super::{
    components::{DecoratedText, DecoratedTextContent, Decoration, Keyword},
    links::link,
    math::math_inline,
    tags::tag_sequence,
    utils::{end_of_line_or_input, position},
    Span, VimwikiIResult, LC,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char},
    combinator::{map, not, recognize},
    error::context,
    multi::many1,
    sequence::{delimited, tuple},
};

#[inline]
pub fn text(input: Span) -> VimwikiIResult<LC<String>> {
    let (input, pos) = position(input)?;

    // NOTE: Text as an inline component should continue until it encounters
    //       something different (math, keyword, link, etc); so, text should
    //       use all other inline components other than itself as not(...)
    //       in a pattern of recoginize(multi1(...))
    let (input, text) = context(
        "Text",
        map(
            recognize(many1(map(
                // TODO: Extract this logic to a separate helper parser whose
                //       purpose is to apply not(...) around N parsers and if
                //       all not(...) pass, then apply some parser
                tuple((
                    not(math_inline),
                    not(tag_sequence),
                    not(link),
                    not(decorated_text),
                    not(keyword),
                    not(end_of_line_or_input),
                    anychar,
                )),
                |x: (_, _, _, _, _, _, char)| x.6,
            ))),
            |s: Span| s.fragment().to_string(),
        ),
    )(input)?;

    Ok((input, LC::from((text, pos, input))))
}

#[inline]
pub fn decorated_text(input: Span) -> VimwikiIResult<LC<DecoratedText>> {
    let (input, pos) = position(input)?;

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

    let (input, decorated_text) = alt((
        parser!("Bold Text", char('*'), Bold),
        parser!("Italic Text", char('_'), Italic),
        parser!("Bold Italic Text", tag("_*"), tag("*_"), BoldItalic),
        parser!("Italic Bold Text", tag("*_"), tag("_*"), BoldItalic),
        parser!("Strikeout Text", tag("~~"), Strikeout),
        parser!("Code Text", char('`'), Code),
        parser!("Super Script Text", char('^'), Superscript),
        parser!("Sub Script Text", tag(",,"), Subscript),
    ))(input)?;

    Ok((input, LC::from((decorated_text, pos, input))))
}

#[inline]
pub fn keyword(input: Span) -> VimwikiIResult<LC<Keyword>> {
    let (input, pos) = position(input)?;

    // TODO: Generate using strum to iterate over all keyword items,
    //       forming a tag based on the string version and parsing the
    //       string back into the keyword in a map (or possibly using
    //       the keyword enum variant directly if we can iterate over
    //       the variants themselves)
    let (input, keyword) = context(
        "Keyword",
        alt((
            map(tag("DONE"), |_| Keyword::DONE),
            map(tag("FIXED"), |_| Keyword::FIXED),
            map(tag("FIXME"), |_| Keyword::FIXME),
            map(tag("STARTED"), |_| Keyword::STARTED),
            map(tag("TODO"), |_| Keyword::TODO),
            map(tag("XXX"), |_| Keyword::XXX),
        )),
    )(input)?;

    Ok((input, LC::from((keyword, pos, input))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_should_fail_if_input_empty() {
        panic!("TODO: Implement");
    }

    #[test]
    fn text_should_consume_until_encountering_inline_math() {
        panic!("TODO: Implement");
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
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_fail_if_start_is_followed_by_whitespace() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_fail_if_end_is_preceded_by_whitespace() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_fail_if_start_and_end_separated_by_newline() {
        panic!("TODO: Implement");
    }

    #[test]
    fn decorated_text_should_support_bold() {
        panic!("TODO: Implement");
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
        panic!("TODO: Implement");
    }

    #[test]
    fn keyword_should_consume_specific_keywords() {
        // map(tag("DONE"), |_| Keyword::DONE),
        // map(tag("FIXED"), |_| Keyword::FIXED),
        // map(tag("FIXME"), |_| Keyword::FIXME),
        // map(tag("STARTED"), |_| Keyword::STARTED),
        // map(tag("TODO"), |_| Keyword::TODO),
        // map(tag("XXX"), |_| Keyword::XXX),
        panic!("TODO: Implement");
    }
}
