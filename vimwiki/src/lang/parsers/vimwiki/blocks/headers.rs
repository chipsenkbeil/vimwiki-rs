use super::{
    elements::Header,
    utils::{
        beginning_of_line, context, end_of_line_or_input, lc, take_line_while1,
        take_until_end_of_line_or_input,
    },
    Span, VimwikiIResult, LE,
};
use nom::{
    character::complete::{char, space0},
    combinator::{map, map_res, verify},
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

    context("Header", lc(inner))(input)
}

fn header_tail(level: usize) -> impl Fn(Span) -> VimwikiIResult<String> {
    move |input: Span| {
        map_res(take_until_end_of_line_or_input, |s: Span| {
            let fragment = s.fragment_str().trim_end();
            let suffix = "=".repeat(level);
            match fragment.strip_suffix(&suffix) {
                Some(content) if !content.ends_with('=') => {
                    Ok(content.trim().to_string())
                }
                _ => Err(format!("Did not end in {}", suffix)),
            }
        })(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::Span;

    #[test]
    fn header_should_parse_level_1_header() {
        let input = Span::from("=test header=");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 1, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" =test header= ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 1, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_2_header() {
        let input = Span::from("==test header==");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 2, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" ==test header== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 2, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_3_header() {
        let input = Span::from("===test header===");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 3, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" ===test header=== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 3, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_4_header() {
        let input = Span::from("====test header====");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 4, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" ====test header==== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 4, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_5_header() {
        let input = Span::from("=====test header=====");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 5, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" =====test header===== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 5, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_6_header() {
        let input = Span::from("======test header======");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 6, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::from(" ======test header====== ");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.level, 6, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
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
        assert_eq!(h.text, "test header", "Wrong header text");
    }

    #[test]
    fn header_should_support_equals_signs_within_content() {
        let input = Span::from("=test =header=");
        let (input, h) = header(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume header");
        assert_eq!(h.text, "test =header", "Wrong header text");
    }
}
