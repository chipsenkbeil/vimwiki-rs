use super::{
    components::{self, List},
    inline_component_container, utils, Span, VimwikiIResult, LC,
};
use nom::{branch::alt, combinator::map, error::context};

mod definition;
mod regular;

#[inline]
pub fn list(input: Span) -> VimwikiIResult<LC<List>> {
    context(
        "List",
        alt((
            map(definition::definition_list, |c| c.map(List::from)),
            map(regular::regular_list, |c| c.map(List::from)),
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_should_fail_if_not_at_beginning_of_line() {
        todo!();
    }

    #[test]
    fn divider_should_fail_if_not_at_least_four_hyphens() {
        todo!();
    }

    #[test]
    fn divider_should_fail_if_not_only_hyphens_within_line() {
        todo!();
    }

    #[test]
    fn divider_should_succeed_if_four_or_more_hyphens_at_start_of_line() {
        todo!();
    }
}
