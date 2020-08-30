use super::{
    components::{Tag, TagSequence},
    utils::end_of_line_or_input,
    Span, VimwikiIResult, LC,
};
use nom::{
    character::complete::char,
    combinator::{map, not, recognize},
    error::context,
    multi::{many1, separated_nonempty_list},
    sequence::{delimited, pair},
};
use nom_locate::position;

#[inline]
pub fn tag_sequence(input: Span) -> VimwikiIResult<LC<TagSequence>> {
    let (input, pos) = position(input)?;

    // NOTE: Tag sequences are just :tag1:tag2:...: on a single line
    let (input, tags) = context(
        "TagSequence",
        delimited(
            char(':'),
            separated_nonempty_list(
                char(':'),
                map(
                    recognize(many1(pair(
                        not(char(':')),
                        not(end_of_line_or_input),
                    ))),
                    |s| Tag::new(s.fragment().to_string()),
                ),
            ),
            char(':'),
        ),
    )(input)?;

    Ok((input, LC::from((TagSequence::new(tags), pos))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_sequence_should_fail_if_input_empty() {
        panic!("TODO: Implement");
    }

    #[test]
    fn tag_sequence_should_fail_if_not_starting_with_colon() {
        panic!("TODO: Implement");
    }

    #[test]
    fn tag_sequence_should_fail_if_not_ending_with_colon() {
        panic!("TODO: Implement");
    }

    #[test]
    fn tag_sequence_should_fail_if_only_comprised_of_colons() {
        panic!("TODO: Implement");
    }

    #[test]
    fn tag_sequence_should_yield_a_single_tag_if_one_pair_of_colons_with_text()
    {
        panic!("TODO: Implement");
    }

    #[test]
    fn tag_sequence_should_yield_multiple_tags_if_many_colons_with_text() {
        panic!("TODO: Implement");
    }
}
