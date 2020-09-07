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
    fn list_should_return_definition_list_where_appropriate() {
        todo!();
    }

    #[test]
    fn list_should_return_regular_list_where_appropriate() {
        todo!();
    }
}
