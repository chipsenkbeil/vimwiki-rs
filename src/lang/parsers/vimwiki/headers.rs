use super::{
    components::Header,
    utils::{
        beginning_of_line, end_of_line_or_input, position, take_line_while1,
    },
    Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::take,
    character::complete::{char, space0},
    combinator::{map, verify},
    multi::many0_count,
};

/// Parses a vimwiki header, returning the associated header if successful
#[inline]
pub fn header(input: Span) -> VimwikiIResult<LC<Header>> {
    let (input, pos) = position(input)?;

    // Header must start at the beginning of a line
    let (input, _) = beginning_of_line(input)?;

    // First, check if the header is indented at all; if so, then it is centered
    let (input, centered) =
        map(space0, |s: Span| !s.fragment().is_empty())(input)?;

    // Second, determine the potential level of the header (the number of =)
    let (input, level) = verify(
        map(take_line_while1(char('=')), |s: Span| s.fragment().len()),
        |level| *level >= Header::MIN_LEVEL && *level <= Header::MAX_LEVEL,
    )(input)?;

    // Third, get the content of the header by collecting all text until we
    // find a closing set of = matching our expected level
    let (input, header) = map(
        take_line_while1(verify(many0_count(char('=')), |count| {
            *count < level
        })),
        |s: Span| Header::new(level, s.fragment().to_string(), centered),
    )(input)?;

    // Fourth, take the right-side of the header's = boundary
    let (input, _) = take(level)(input)?;

    // Fifth, be nice and consume any additional spaces as well as the end
    // of the line to mark the conclusion of the header
    let (input, _) = space0(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, LC::from((header, pos, input))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_should_parse_level_1_header() {
        let input = Span::new("=test header=");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 1, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::new(" =test header= ");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 1, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_2_header() {
        let input = Span::new("==test header==");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 2, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::new(" ==test header== ");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 2, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_3_header() {
        let input = Span::new("===test header===");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 3, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::new(" ===test header=== ");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 3, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_4_header() {
        let input = Span::new("====test header====");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 4, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::new(" ====test header==== ");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 4, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_5_header() {
        let input = Span::new("=====test header=====");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 5, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::new(" =====test header===== ");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 5, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_parse_level_6_header() {
        let input = Span::new("======test header======");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 6, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, false, "Wrong centered status");

        let input = Span::new(" ======test header====== ");
        let (input, h) = header(input).unwrap();
        assert_eq!(*input.fragment(), "", "Did not consume header");
        assert_eq!(h.level, 6, "Wrong header level");
        assert_eq!(h.text, "test header", "Wrong header text");
        assert_eq!(h.centered, true, "Wrong centered status");
    }

    #[test]
    fn header_should_fail_if_level_greater_than_6() {
        let input = Span::new("=======test header=======");
        assert!(header(input).is_err(), "Header succeeded above max level");

        let input = Span::new(" =======test header======= ");
        assert!(header(input).is_err(), "Header succeeded above max level");
    }
}
