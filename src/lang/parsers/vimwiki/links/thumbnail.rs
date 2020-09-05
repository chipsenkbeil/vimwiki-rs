use super::{
    components::WikiLink,
    utils::{position, url},
    Span, VimwikiIResult, LC,
};
use nom::{branch::alt, combinator::map, error::context};

#[inline]
pub fn thumbnail_link(input: Span) -> VimwikiIResult<LC<WikiLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;
}
