use super::{link_anchor, link_description, link_path};
use crate::lang::{
    elements::{DiaryLink, Located},
    parsers::{
        utils::{capture, context, locate, surround_in_line1},
        IResult, Span,
    },
};
use chrono::NaiveDate;
use nom::{
    bytes::complete::tag,
    combinator::{map_parser, map_res, opt},
    sequence::preceded,
};

#[inline]
pub fn diary_link(input: Span) -> IResult<Located<DiaryLink>> {
    fn inner(input: Span) -> IResult<DiaryLink> {
        // Diary is a specialized link that must start with diary:
        let (input, _) = tag("diary:")(input)?;

        // After the specialized start, a valid date must follow
        // TODO: Unsure if this would allocate a new string given that the
        //       path is formed from a valid UTF-8 str; Cow<'_, str> yielded
        //       might just be a pointer to the original slice
        let (input, date) = map_res(link_path, |path| {
            NaiveDate::parse_from_str(&path.to_string_lossy(), "%Y-%m-%d")
        })(input)?;

        // Next, check if there are any anchors
        let (input, maybe_anchor) = opt(link_anchor)(input)?;

        // Finally, check if there is a description (preceding with |), where
        // a special case is wrapped in {{...}} as a URL
        let (input, maybe_description) =
            opt(preceded(tag("|"), link_description))(input)?;

        Ok((input, DiaryLink::new(date, maybe_description, maybe_anchor)))
    }

    context(
        "DiaryLink",
        locate(capture(map_parser(surround_in_line1("[[", "]]"), inner))),
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

        assert_eq!(link.date, NaiveDate::from_ymd(2012, 03, 05));
        assert_eq!(link.description, None);
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn diary_link_should_support_a_description() {
        let input = Span::from("[[diary:2012-03-05|some description]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.date, NaiveDate::from_ymd(2012, 03, 05));
        assert_eq!(
            link.description,
            Some(Description::from("some description"))
        );
        assert_eq!(link.anchor, None);
    }

    #[test]
    fn diary_link_should_support_an_anchor() {
        let input = Span::from("[[diary:2012-03-05#Tomorrow]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.date, NaiveDate::from_ymd(2012, 03, 05));
        assert_eq!(link.description, None,);
        assert_eq!(link.anchor, Some(Anchor::from("Tomorrow")));
    }

    #[test]
    fn diary_link_should_support_an_anchor_and_description() {
        let input =
            Span::from("[[diary:2012-03-05#Tomorrow|Tasks for tomorrow]]");
        let (input, link) =
            diary_link(input).expect("Parser unexpectedly failed");

        // Link should be consumed
        assert!(input.is_empty());

        assert_eq!(link.date, NaiveDate::from_ymd(2012, 03, 05));
        assert_eq!(
            link.description,
            Some(Description::from("Tasks for tomorrow"))
        );
        assert_eq!(link.anchor, Some(Anchor::from("Tomorrow")));
    }
}
