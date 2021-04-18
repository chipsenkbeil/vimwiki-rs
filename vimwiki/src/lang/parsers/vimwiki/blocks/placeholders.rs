use crate::lang::{
    elements::{Located, Placeholder},
    parsers::{
        utils::{
            beginning_of_line, capture, context, cow_str, end_of_line_or_input,
            locate, take_line_until_one_of_three1,
            take_until_end_of_line_or_input,
        },
        IResult, Span,
    },
};
use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::{map_parser, map_res, not, verify},
};

#[inline]
pub fn placeholder(input: Span) -> IResult<Located<Placeholder>> {
    fn inner(input: Span) -> IResult<Located<Placeholder>> {
        let (input, _) = beginning_of_line(input)?;
        let (input, le_placeholder) = locate(capture(alt((
            placeholder_title,
            placeholder_nohtml,
            placeholder_template,
            placeholder_date,
            placeholder_other,
        ))))(input)?;
        let (input, _) = end_of_line_or_input(input)?;
        Ok((input, le_placeholder))
    }

    context("Placeholder", inner)(input)
}

fn placeholder_title(input: Span) -> IResult<Placeholder> {
    fn inner(input: Span) -> IResult<Placeholder> {
        let (input, _) = tag("%title")(input)?;
        let (input, _) = space1(input)?;
        let (input, text) = map_parser(
            verify(take_until_end_of_line_or_input, |s: &Span| {
                !s.is_only_whitespace()
            }),
            cow_str,
        )(input)?;
        Ok((input, Placeholder::Title(text)))
    }

    context("Placeholder Title", inner)(input)
}

fn placeholder_nohtml(input: Span) -> IResult<Placeholder> {
    fn inner(input: Span) -> IResult<Placeholder> {
        let (input, _) = tag("%nohtml")(input)?;
        let (input, _) = space0(input)?;
        Ok((input, Placeholder::NoHtml))
    }

    context("Placeholder NoHtml", inner)(input)
}

fn placeholder_template(input: Span) -> IResult<Placeholder> {
    fn inner(input: Span) -> IResult<Placeholder> {
        let (input, _) = tag("%template")(input)?;
        let (input, _) = space1(input)?;
        let (input, text) = map_parser(
            verify(take_until_end_of_line_or_input, |s: &Span| {
                !s.is_only_whitespace()
            }),
            cow_str,
        )(input)?;
        Ok((input, Placeholder::Template(text)))
    }

    context("Placeholder Template", inner)(input)
}

fn placeholder_date(input: Span) -> IResult<Placeholder> {
    fn inner(input: Span) -> IResult<Placeholder> {
        let (input, _) = tag("%date")(input)?;
        let (input, _) = space1(input)?;
        let (input, date) =
            map_res(take_until_end_of_line_or_input, |s: Span| {
                NaiveDate::parse_from_str(
                    s.as_unsafe_remaining_str(),
                    "%Y-%m-%d",
                )
            })(input)?;
        Ok((input, Placeholder::Date(date)))
    }

    context("Placeholder Date", inner)(input)
}

fn placeholder_other(input: Span) -> IResult<Placeholder> {
    fn inner(input: Span) -> IResult<Placeholder> {
        let (input, _) = not(tag("%title"))(input)?;
        let (input, _) = not(tag("%nohtml"))(input)?;
        let (input, _) = not(tag("%template"))(input)?;
        let (input, _) = not(tag("%date"))(input)?;

        let (input, _) = tag("%")(input)?;
        let (input, name) = map_parser(
            take_line_until_one_of_three1(" ", "\t", "%"),
            cow_str,
        )(input)?;
        let (input, _) = space1(input)?;
        let (input, value) = map_parser(
            verify(take_until_end_of_line_or_input, |s: &Span| {
                !s.is_only_whitespace()
            }),
            cow_str,
        )(input)?;
        Ok((input, Placeholder::Other { name, value }))
    }

    context("Placeholder Other", inner)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_fail_title_with_no_text() {
        let input = Span::from("%title");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_succeed_if_title_with_text_input() {
        let input = Span::from("%title some title");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.is_empty(), "Did not consume placeholder");
        assert_eq!(
            placeholder.into_inner(),
            Placeholder::title_from_str("some title")
        );
    }

    #[test]
    fn placeholder_should_fail_if_nohtml_with_text() {
        let input = Span::from("%nohtml something");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_succeed_if_nohtml_with_no_text_input() {
        let input = Span::from("%nohtml");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.is_empty(), "Did not consume placeholder");
        assert_eq!(placeholder.into_inner(), Placeholder::NoHtml);
    }

    #[test]
    fn placeholder_should_fail_if_template_with_no_text() {
        let input = Span::from("%template");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_succeed_if_template_with_text_input() {
        let input = Span::from("%template my_template");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.is_empty(), "Did not consume placeholder");
        assert_eq!(
            placeholder.into_inner(),
            Placeholder::template_from_str("my_template"),
        );
    }

    #[test]
    fn placeholder_should_fail_if_date_with_no_text() {
        let input = Span::from("%date");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_fail_if_date_with_non_date_input() {
        let input = Span::from("%date something");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_should_succeed_if_date_with_date_input() {
        let input = Span::from("%date 2012-03-05");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.is_empty(), "Did not consume placeholder");
        assert_eq!(
            placeholder.into_inner(),
            Placeholder::Date(NaiveDate::from_ymd(2012, 3, 5)),
        );
    }

    #[test]
    fn placeholder_fallback_should_fail_if_double_percent_at_start() {
        let input = Span::from("%%other something else");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_no_space_between_name_and_value() {
        let input = Span::from("%othervalue");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_no_name_provided() {
        let input = Span::from("% value");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_percent_found_in_name() {
        let input = Span::from("%oth%er value");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_percent_found_at_end_of_name() {
        let input = Span::from("%other% value");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_fail_if_no_value_after_name() {
        let input = Span::from("%other");
        assert!(placeholder(input).is_err());
    }

    #[test]
    fn placeholder_fallback_should_succeed_if_percent_followed_by_name_space_and_value(
    ) {
        let input = Span::from("%other something else");
        let (input, placeholder) = placeholder(input).unwrap();
        assert!(input.is_empty(), "Did not consume placeholder");
        assert_eq!(
            placeholder.into_inner(),
            Placeholder::other_from_str("other", "something else"),
        );
    }
}
