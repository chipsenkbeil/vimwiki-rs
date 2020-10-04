use super::{
    elements::{Header, InlineElementContainer},
    inline::inline_element_container,
    utils::{
        beginning_of_line, context, end_of_line_or_input, le, take_end,
        take_line_while1, take_until_end_of_line_or_input,
        trim_trailing_whitespace, trim_whitespace, unwrap_le,
    },
    Span, VimwikiIResult, LE,
};
use nom::{
    bytes::complete::take,
    character::complete::{char, space0},
    combinator::{map, peek, verify},
};

/// Parses a vimwiki header, returning the associated header if successful
#[inline]
pub fn header(input: Span) -> VimwikiIResult<LE<Header>> {
    fn inner(input: Span) -> VimwikiIResult<Header> {
        // Header must start at the beginning of a line
        let (input, _) = beginning_of_line(input)?;

        // First, check if the header is indented at all; if so, then it is centered
        let (input, centered) =
            map(space0, |s: Span| !s.fragment().is_empty())(input)?;

        // Second, determine the potential level of the header (the number of =)
        let (input, level) = verify(
            map(take_line_while1(char('=')), |s: Span| s.fragment_len()),
            |level| *level >= Header::MIN_LEVEL && *level <= Header::MAX_LEVEL,
        )(input)?;

        // Third, get the content of the header by collecting all text until we
        // find a closing set of = matching our expected level
        let (input, header) = map(header_tail(level), |content| {
            Header::new(level, content, centered)
        })(input)?;

        // Fourth, consume the end of line/input to indicate header complete
        let (input, _) = end_of_line_or_input(input)?;

        Ok((input, header))
    }

    context("Header", le(inner))(input)
}

fn header_tail(
    level: usize,
) -> impl Fn(Span) -> VimwikiIResult<InlineElementContainer> {
    use nom::{AsBytes, InputIter};
    move |input: Span| {
        // Get remainder of line and remove any excess whitespace
        let (input, rest_of_line) = take_until_end_of_line_or_input(input)?;
        let (rest_of_line, _) = trim_trailing_whitespace(rest_of_line)?;

        // Verify that the end of the line (minus whitespace) has the same
        // number of equals signs, and chop them off
        let (rest_of_line, _) = context(
            "Header Tail Equal Levels",
            verify(take_end(level), |end| {
                end.iter_elements().all(|b| b == b'=')
            }),
        )(rest_of_line)?;

        // Verify that there is no equals sign at the beginning or end of the
        // header content, which would imply that we have unbalanced levels
        let (rest_of_line, _) = peek(verify(take(1usize), |start: &Span| {
            start.as_bytes()[0] != b'='
        }))(rest_of_line)?;
        let (rest_of_line, _) =
            peek(verify(take_end(1usize), |end: &Span| {
                end.as_bytes()[0] != b'='
            }))(rest_of_line)?;

        // Remove leading and trailing whitespace within header content
        let (rest_of_line, _) = trim_whitespace(rest_of_line)?;

        // Parse our container of inline elements
        let (_, container) = unwrap_le(inline_element_container)(rest_of_line)?;

        Ok((input, container))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{elements::InlineElement, lang::utils::Span};

    macro_rules! check {
        ($header:expr, $index:expr, $type:ident, $text:expr) => {
            assert!(matches!(
                $header.content[$index].element,
                InlineElement::$type(_)
            ));
            assert_eq!($header.content[$index].to_string(), $text);
        };
    }

    #[test]
    fn header_should_parse_level_1_header() {
        let input = Span::from("=test header=");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 1, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" =test header= ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 1, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_2_header() {
        let input = Span::from("==test header==");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 2, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" ==test header== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 2, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_3_header() {
        let input = Span::from("===test header===");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 3, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" ===test header=== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 3, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_4_header() {
        let input = Span::from("====test header====");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 4, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" ====test header==== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 4, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_5_header() {
        let input = Span::from("=====test header=====");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 5, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" =====test header===== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 5, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_6_header() {
        let input = Span::from("======test header======");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 6, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" ======test header====== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 6, "Wrong header level");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_fail_if_level_greater_than_6() {
        let input = Span::from("=======test header=======");
        assert!(header(input).is_err(), "Header succeeded above max level");

        let input = Span::from(" =======test header======= ");
        assert!(header(input).is_err(), "Header succeeded above max level");
    }

    #[test]
    fn header_should_trim_whitespace_around_text() {
        let input = Span::from("= test header\t=");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.content.to_string(), "test header", "Wrong header text");
    }

    #[test]
    fn header_should_support_equals_signs_within_content() {
        let input = Span::from("=test =header=");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.content.to_string(), "test =header", "Wrong header text");
    }

    #[test]
    fn header_should_support_decorations_within_content() {
        let input =
            Span::from("=*bold* header TODO [[link]] :tag1:tag2: $math$=");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");

        check!(h, 0, DecoratedText, "bold");
        check!(h, 1, Text, " header ");
        check!(h, 2, Keyword, "TODO");
        check!(h, 3, Text, " ");
        check!(h, 4, Link, "link");
        check!(h, 5, Text, " ");
        check!(h, 6, Tags, ":tag1:tag2:");
        check!(h, 7, Text, " ");
        check!(h, 8, Math, "math");
    }
}
