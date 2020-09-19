use super::{
    components::Divider,
    utils::{
        beginning_of_line, context, end_of_line_or_input, lc, take_line_while1,
    },
    Span, VimwikiIResult, LC,
};
use nom::{character::complete::char, combinator::verify};

#[inline]
pub fn divider(input: Span) -> VimwikiIResult<LC<Divider>> {
    fn inner(input: Span) -> VimwikiIResult<Divider> {
        let (input, _) = beginning_of_line(input)?;
        let (input, _) = verify(take_line_while1(char('-')), |s: &Span| {
            s.fragment().len() >= 4
        })(input)?;
        let (input, _) = end_of_line_or_input(input)?;

        Ok((input, Divider))
    }

    context("Divider", lc(inner))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::Span;

    #[test]
    fn divider_should_fail_if_not_at_beginning_of_line() {
        let input = Span::from(" ----");
        assert!(divider(input).is_err());
    }

    #[test]
    fn divider_should_fail_if_not_at_least_four_hyphens() {
        let input = Span::from("---");
        assert!(divider(input).is_err());
    }

    #[test]
    fn divider_should_fail_if_not_only_hyphens_within_line() {
        let input = Span::from("----a");
        assert!(divider(input).is_err());
    }

    #[test]
    fn divider_should_succeed_if_four_hyphens_at_start_of_line() {
        let input = Span::from("----");
        let (input, d) = divider(input).unwrap();
        assert!(input.fragment().is_empty(), "Divider not consumed");
        assert_eq!(d.region.start.line, 1);
        assert_eq!(d.region.start.column, 1);
        assert_eq!(d.region.end.line, 1);
        assert_eq!(d.region.end.column, 4);
    }

    #[test]
    fn divider_should_succeed_if_more_than_four_hyphens_at_start_of_line() {
        let input = Span::from("-----");
        let (input, d) = divider(input).unwrap();
        assert!(input.fragment().is_empty(), "Divider not consumed");
        assert_eq!(d.region.start.line, 1);
        assert_eq!(d.region.start.column, 1);
        assert_eq!(d.region.end.line, 1);
        assert_eq!(d.region.end.column, 5);
    }

    #[test]
    fn divider_should_consume_end_of_line() {
        let input = Span::from("----\nabcd");
        let (input, d) = divider(input).unwrap();
        assert_eq!(input.fragment_str(), "abcd");
        assert_eq!(d.region.start.line, 1);
        assert_eq!(d.region.start.column, 1);
        assert_eq!(d.region.end.line, 1);
        assert_eq!(d.region.end.column, 5);
    }
}
