use crate::lang::{
    elements::{Divider, Located},
    parsers::{
        utils::{
            beginning_of_line, capture, context, end_of_line_or_input, locate,
            take_line_while1,
        },
        IResult, Span,
    },
};
use nom::{character::complete::char, combinator::verify};

#[inline]
pub fn divider(input: Span) -> IResult<Located<Divider>> {
    fn inner(input: Span) -> IResult<Divider> {
        let (input, _) = beginning_of_line(input)?;
        let (input, _) = verify(take_line_while1(char('-')), |s: &Span| {
            s.remaining_len() >= 4
        })(input)?;
        let (input, _) = end_of_line_or_input(input)?;

        Ok((input, Divider))
    }

    context("Divider", locate(capture(inner)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let (input, _) = divider(input).unwrap();
        assert!(input.is_empty(), "Divider not consumed");
    }

    #[test]
    fn divider_should_succeed_if_more_than_four_hyphens_at_start_of_line() {
        let input = Span::from("-----");
        let (input, _) = divider(input).unwrap();
        assert!(input.is_empty(), "Divider not consumed");
    }

    #[test]
    fn divider_should_consume_end_of_line() {
        let input = Span::from("----\nabcd");
        let (input, _) = divider(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "abcd");
    }
}
