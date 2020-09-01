use super::{
    components::Divider, utils::beginning_of_line, Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::take_while, character::complete::line_ending,
    combinator::verify,
};
use nom_locate::position;

#[inline]
pub fn divider(input: Span) -> VimwikiIResult<LC<Divider>> {
    let (input, pos) = position(input)?;

    let (input, _) = beginning_of_line(input)?;
    let (input, _) =
        verify(take_while(|c| c == '-'), |s: &Span| s.fragment().len() >= 4)(
            input,
        )?;
    let (input, _) = line_ending(input)?;

    Ok((input, LC::from((Divider, pos))))
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
