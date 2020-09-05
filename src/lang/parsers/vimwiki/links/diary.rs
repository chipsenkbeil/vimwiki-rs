use super::{components::DiaryLink, utils::position, Span, VimwikiIResult, LC};
use nom::{branch::alt, combinator::map, error::context};

#[inline]
pub fn diary_link(input: Span) -> VimwikiIResult<LC<DiaryLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    panic!("TODO: Implement");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diary_link_should_support_diary_scheme() {
        // [[diary:2012-03-05]]
        todo!();
    }

    #[test]
    fn diary_link_should_support_anchors() {
        // [[diary:2020-03-05#Tomorrow|Tasks for tomorrow]]
        todo!();
    }
}
