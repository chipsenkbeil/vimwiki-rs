use super::{
    elements::{Tag, Tags},
    utils::{context, le, take_line_while1},
    Span, VimwikiIResult, LE,
};
use nom::{
    character::complete::char, combinator::not, multi::many1,
    sequence::terminated,
};

#[inline]
pub fn tags(input: Span) -> VimwikiIResult<LE<Tags>> {
    fn inner(input: Span) -> VimwikiIResult<Tags> {
        let (input, _) = char(':')(input)?;
        let (input, contents) =
            many1(terminated(tag_content, char(':')))(input)?;

        Ok((input, Tags::new(contents)))
    }

    context("Tags", le(inner))(input)
}

fn tag_content(input: Span) -> VimwikiIResult<Tag> {
    fn has_more(input: Span) -> VimwikiIResult<()> {
        let (input, _) = not(char(':'))(input)?;
        let (input, _) = not(char(' '))(input)?;
        let (input, _) = not(char('\t'))(input)?;
        Ok((input, ()))
    }

    let (input, s) = take_line_while1(has_more)(input)?;
    Ok((input, Tag::from(s.as_unsafe_remaining_str())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(tags(input).is_err());
    }

    #[test]
    fn tags_should_fail_if_not_starting_with_colon() {
        let input = Span::from("tag-example:");
        assert!(tags(input).is_err());
    }

    #[test]
    fn tags_should_fail_if_not_ending_with_colon() {
        let input = Span::from(":tag-example");
        assert!(tags(input).is_err());
    }

    #[test]
    fn tags_should_fail_if_only_comprised_of_colons() {
        let input = Span::from("::");
        assert!(tags(input).is_err());
    }

    #[test]
    fn tags_should_yield_a_single_tag_if_one_pair_of_colons_with_text() {
        let input = Span::from(":tag-example:");
        let (input, tags) = tags(input).unwrap();
        assert!(input.is_empty(), "Did not consume tags");
        assert_eq!(tags.0, vec![Tag::from("tag-example")]);
    }

    #[test]
    fn tags_should_yield_a_single_tag_if_one_pair_of_colons_with_trailing_content(
    ) {
        let input = Span::from(":tag-example:and other text");
        let (input, tags) = tags(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "and other text",
            "Unexpected input consumed"
        );
        assert_eq!(tags.0, vec![Tag::from("tag-example")]);
    }

    #[test]
    fn tags_should_yield_multiple_tags_if_many_colons_with_text() {
        let input = Span::from(":tag-one:tag-two:");
        let (input, tags) = tags(input).unwrap();
        assert!(input.is_empty(), "Did not consume tags");
        assert_eq!(tags.0, vec![Tag::from("tag-one"), Tag::from("tag-two")]);
    }
}
