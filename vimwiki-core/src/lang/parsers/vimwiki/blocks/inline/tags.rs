use crate::lang::{
    elements::{Located, Tag, Tags},
    parsers::{
        utils::{
            capture, context, cow_str, locate, take_line_until_one_of_three1,
        },
        IResult, Span,
    },
};
use nom::{
    character::complete::char, combinator::map_parser, multi::many1,
    sequence::terminated,
};

#[inline]
pub fn tags(input: Span) -> IResult<Located<Tags>> {
    fn inner(input: Span) -> IResult<Tags> {
        let (input, _) = char(':')(input)?;
        let (input, contents) =
            many1(terminated(tag_content, char(':')))(input)?;

        Ok((input, Tags::new(contents)))
    }

    context("Tags", locate(capture(inner)))(input)
}

fn tag_content(input: Span) -> IResult<Tag> {
    let (input, s) = map_parser(
        take_line_until_one_of_three1(":", " ", "\t"),
        cow_str,
    )(input)?;
    Ok((input, Tag::new(s)))
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
        assert_eq!(
            tags.into_inner().into_iter().collect::<Vec<Tag>>(),
            vec![Tag::from("tag-example")]
        );
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
        assert_eq!(
            tags.into_inner().into_iter().collect::<Vec<Tag>>(),
            vec![Tag::from("tag-example")]
        );
    }

    #[test]
    fn tags_should_yield_multiple_tags_if_many_colons_with_text() {
        let input = Span::from(":tag-one:tag-two:");
        let (input, tags) = tags(input).unwrap();
        assert!(input.is_empty(), "Did not consume tags");
        assert_eq!(
            tags.into_inner().into_iter().collect::<Vec<Tag>>(),
            vec![Tag::from("tag-one"), Tag::from("tag-two")]
        );
    }
}
