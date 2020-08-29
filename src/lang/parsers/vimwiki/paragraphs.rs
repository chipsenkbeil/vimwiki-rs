use super::{
    components::Paragraph,
    inline::inline_component,
    utils::{beginning_of_line, blank_line},
    Span, VimwikiIResult, LC,
};
use nom::{
    combinator::{map, not},
    error::context,
    multi::many1,
    sequence::pair,
};
use nom_locate::position;

/// Parses a vimwiki paragraph, returning the associated paragraph is successful
#[inline]
pub fn paragraph(input: Span) -> VimwikiIResult<LC<Paragraph>> {
    let (input, pos) = position(input)?;

    // Ensure that we are starting at the beginning of a line
    let (input, _) = beginning_of_line(input)?;

    // Continuously take content until we reach a blank line
    let (input, components) = context(
        "Paragraph",
        many1(map(pair(not(blank_line), inline_component), |(_, c)| c)),
    )(input)?;

    // Transform contents into the paragraph itself
    let paragraph = Paragraph::from(components);

    Ok((input, LC::from((paragraph, pos))))
}

#[cfg(test)]
mod tests {
    use super::super::super::utils::convert_error;
    use super::*;
    use nom::Err;

    fn parse_and_eval<'a>(
        input: Span<'a>,
        f: impl Fn((Span<'a>, LC<Paragraph>)),
    ) {
        match paragraph(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!("{}", convert_error(input, e))
            }
            Err(Err::Incomplete(needed)) => panic!("Incomplete: {:?}", needed),
            Ok(result) => f(result),
        }
    }

    fn parse_and_test(input_str: &str) {
        let input = Span::new(input_str);
        parse_and_eval(input, |result| {
            assert!(
                result.0.fragment().is_empty(),
                "Entire input not consumed! Input: '{}' | Remainder: '{}'",
                input,
                result.0,
            );
        });
    }

    #[test]
    fn paragraph_should_parse_single_line() {
        panic!("TODO: Implement");
    }

    #[test]
    fn paragraph_should_parse_multiple_lines() {
        panic!("TODO: Implement");
    }

    #[test]
    fn paragraph_should_stop_at_a_blank_line() {
        panic!("TODO: Implement");
    }
}
