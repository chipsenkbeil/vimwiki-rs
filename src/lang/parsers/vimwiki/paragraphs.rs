use super::{
    components::Paragraph,
    inline_component_container,
    utils::{beginning_of_line, blank_line, end_of_line_or_input, position},
    Span, VimwikiIResult, LC,
};
use nom::{
    character::complete::space1,
    combinator::{map, not},
    error::context,
    multi::many1,
    sequence::delimited,
};

/// Parses a vimwiki paragraph, returning the associated paragraph is successful
#[inline]
pub fn paragraph(input: Span) -> VimwikiIResult<LC<Paragraph>> {
    let (input, pos) = position(input)?;

    // Ensure that we are starting at the beginning of a line
    let (input, _) = beginning_of_line(input)?;

    // Paragraph has NO indentation
    let (input, _) = not(space1)(input)?;

    // Continuously take content until we reach a blank line
    let (input, components) = context(
        "Paragraph",
        many1(delimited(
            not(blank_line),
            map(inline_component_container, |c| c.component),
            end_of_line_or_input,
        )),
    )(input)?;

    // Transform contents into the paragraph itself
    let paragraph = Paragraph::new(From::from(components));

    Ok((input, LC::from((paragraph, pos, input))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraph_should_fail_if_on_blank_line() {
        let input = Span::new(" ");
        assert!(paragraph(input).is_err());
    }

    #[test]
    fn paragraph_should_fail_if_line_indented() {
        let input = Span::new(" some text");
        assert!(paragraph(input).is_err());
    }

    #[test]
    fn paragraph_should_parse_single_line() {
        todo!();
    }

    #[test]
    fn paragraph_should_parse_multiple_lines() {
        todo!();
    }

    #[test]
    fn paragraph_should_support_whitespace_at_beginning_of_all_following_lines()
    {
        todo!();
    }

    #[test]
    fn paragraph_should_stop_at_a_blank_line() {
        todo!();
    }
}
