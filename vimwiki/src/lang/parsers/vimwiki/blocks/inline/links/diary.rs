use super::{link_anchor, link_description};
use crate::lang::{
    elements::{Link, Located},
    parsers::{
        utils::{
            capture, context, locate, not_contains, surround_in_line1,
            take_line_until_one_of_two1,
        },
        IResult, Span,
    },
};
use chrono::NaiveDate;
use nom::{
    bytes::complete::tag,
    combinator::{map_parser, map_res, opt},
};

pub fn diary_link(input: Span) -> IResult<Located<Link>> {
    fn inner(input: Span) -> IResult<Link> {
        // Diary is a specialized link that must start with diary:
        let (input, _) = tag("diary:")(input)?;

        // After the specialized start, a valid date must follow before the
        // end of a link, start of anchor, or start of a description
        let (input, date) =
            map_res(take_line_until_one_of_two1("|", "#"), |span| {
                NaiveDate::parse_from_str(
                    span.as_unsafe_remaining_str(),
                    "%Y-%m-%d",
                )
            })(input)?;

        // Check for an optional anchor that we will need to parse
        let (input, maybe_anchor) = opt(link_anchor)(input)?;

        // Finally, check if there is a description (preceding with |)
        let (input, maybe_description) = opt(link_description)(input)?;

        Ok((
            input,
            Link::new_diary_link(date, maybe_description, maybe_anchor),
        ))
    }

    context(
        "Diary Link",
        locate(capture(map_parser(
            not_contains("%%", surround_in_line1("[[", "]]")),
            inner,
        ))),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::{Anchor, Description};

    #[test]
    fn diary_link_should_fail_if_not_using_diary_scheme() {
        let input = Span::from("[[notdiary:2012-03-05]]");
        assert!(diary_link(input).is_err());
    }

    #[test]
    fn diary_link_should_fail_if_not_using_correct_date_format() {
        let input = Span::from("[[diary:2012/03/05]]");
        assert!(diary_link(input).is_err());
    }

    #[test]
    fn diary_link_should_support_diary_scheme() {
        let input = Span::from("[[diary:2012-03-05]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.date(), Some(NaiveDate::from_ymd(2012, 3, 5)));
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn diary_link_should_support_a_description() {
        let input = Span::from("[[diary:2012-03-05|some description]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.date(), Some(NaiveDate::from_ymd(2012, 3, 5)));
        assert_eq!(
            link.description(),
            Some(&Description::from("some description"))
        );
        assert_eq!(link.to_anchor(), None);
    }

    #[test]
    fn diary_link_should_support_an_anchor() {
        let input = Span::from("[[diary:2012-03-05#Tomorrow]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.date(), Some(NaiveDate::from_ymd(2012, 3, 5)));
        assert_eq!(link.description(), None);
        assert_eq!(link.to_anchor(), Some(Anchor::from("Tomorrow")));
    }

    #[test]
    fn diary_link_should_support_an_anchor_and_description() {
        let input =
            Span::from("[[diary:2012-03-05#Tomorrow|Tasks for tomorrow]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.date(), Some(NaiveDate::from_ymd(2012, 3, 5)));
        assert_eq!(
            link.description(),
            Some(&Description::from("Tasks for tomorrow"))
        );
        assert_eq!(link.to_anchor(), Some(Anchor::from("Tomorrow")));
    }
}
