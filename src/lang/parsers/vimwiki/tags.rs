use super::{
    components::{Tag, Tags},
    utils::{position, take_line_while1},
    Span, VimwikiIResult, LC,
};
use nom::{
    character::complete::char,
    combinator::{map, not},
    error::context,
    multi::separated_nonempty_list,
    sequence::delimited,
};

#[inline]
pub fn tags(input: Span) -> VimwikiIResult<LC<Tags>> {
    let (input, pos) = position(input)?;

    let (input, tags) = context(
        "TagSequence",
        delimited(
            char(':'),
            separated_nonempty_list(
                char(':'),
                map(take_line_while1(not(char(':'))), |s| {
                    Tag::new(s.fragment().to_string())
                }),
            ),
            char(':'),
        ),
    )(input)?;

    Ok((input, LC::from((Tags::new(tags), pos, input))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(tags(input).is_err());
    }

    #[test]
    fn tags_should_fail_if_not_starting_with_colon() {
        let input = Span::new("tag-example:");
        assert!(tags(input).is_err());
    }

    #[test]
    fn tags_should_fail_if_not_ending_with_colon() {
        let input = Span::new(":tag-example");
        assert!(tags(input).is_err());
    }

    #[test]
    fn tags_should_fail_if_only_comprised_of_colons() {
        let input = Span::new("::");
        assert!(tags(input).is_err());
    }

    #[test]
    fn tags_should_yield_a_single_tag_if_one_pair_of_colons_with_text() {
        let input = Span::new(":tag-example:");
        let (input, tags) = tags(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume tags");
        assert_eq!(tags.0, vec![Tag::from("tag-example")]);
    }

    #[test]
    fn tags_should_yield_multiple_tags_if_many_colons_with_text() {
        let input = Span::new(":tag-one:tag-two:");
        let (input, tags) = tags(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume tags");
        assert_eq!(tags.0, vec![Tag::from("tag-one"), Tag::from("tag-two")]);
    }
}
