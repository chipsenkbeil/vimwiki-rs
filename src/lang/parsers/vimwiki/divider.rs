use super::{
    components::Divider,
    utils::{
        beginning_of_line, end_of_line_or_input, position, take_line_while1,
    },
    Span, VimwikiIResult, LC,
};
use nom::{character::complete::char, combinator::verify};

#[inline]
pub fn divider(input: Span) -> VimwikiIResult<LC<Divider>> {
    let (input, pos) = position(input)?;

    let (input, _) = beginning_of_line(input)?;
    println!("PREPARING TO TAKE FROM: '{}'", input.fragment());
    let (input, _) = verify(take_line_while1(char('-')), |s: &Span| {
        s.fragment().len() >= 4
    })(input)?;
    println!("REMAINING: '{}'", input.fragment());
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, LC::from((Divider, pos, input))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_should_fail_if_not_at_beginning_of_line() {
        let input = Span::new(" ----");
        assert!(divider(input).is_err());
    }

    #[test]
    fn divider_should_fail_if_not_at_least_four_hyphens() {
        let input = Span::new("---");
        assert!(divider(input).is_err());
    }

    #[test]
    fn divider_should_fail_if_not_only_hyphens_within_line() {
        let input = Span::new("----a");
        assert!(divider(input).is_err());
    }

    #[test]
    fn divider_should_succeed_if_four_hyphens_at_start_of_line() {
        let input = Span::new("----");
        let (input, d) = divider(input).unwrap();
        assert!(input.fragment().is_empty(), "Divider not consumed");
        assert_eq!(d.region.start.line, 0);
        assert_eq!(d.region.start.column, 0);
        assert_eq!(d.region.end.line, 0);
        assert_eq!(d.region.end.column, 3);
    }

    #[test]
    fn divider_should_succeed_if_more_than_four_hyphens_at_start_of_line() {
        let input = Span::new("-----");
        let (input, d) = divider(input).unwrap();
        assert!(input.fragment().is_empty(), "Divider not consumed");
        assert_eq!(d.region.start.line, 0);
        assert_eq!(d.region.start.column, 0);
        assert_eq!(d.region.end.line, 0);
        assert_eq!(d.region.end.column, 4);
    }

    #[test]
    fn divider_should_consume_end_of_line() {
        let input = Span::new("----\nabcd");
        let (input, d) = divider(input).unwrap();
        assert_eq!(*input.fragment(), "abcd");
        assert_eq!(d.region.start.line, 0);
        assert_eq!(d.region.start.column, 0);
        assert_eq!(d.region.end.line, 0);
        assert_eq!(d.region.end.column, 4);
    }
}
