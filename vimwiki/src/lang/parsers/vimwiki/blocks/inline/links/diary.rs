use super::wiki::wiki_link;
use crate::lang::{
    elements::{DiaryLink, Located},
    parsers::{utils::context, Error, IResult, Span},
};
use chrono::NaiveDate;

#[inline]
pub fn diary_link(input: Span) -> IResult<Located<DiaryLink>> {
    fn inner(input: Span) -> IResult<Located<DiaryLink>> {
        // First, parse as a standard wiki link, which should stash the potential
        // diary as the path
        let (input, link) = wiki_link(input)?;

        let path = link.path.to_str().ok_or_else(|| {
            nom::Err::Error(Error::from_ctx(&input, "Not diary link"))
        })?;

        // Second, check if the link is a diary
        match parse_date_from_path(path) {
            Some(date) => Ok((
                input,
                link.map(|c| DiaryLink::new(date, c.description, c.anchor)),
            )),
            _ => {
                Err(nom::Err::Error(Error::from_ctx(&input, "Not diary link")))
            }
        }
    }

    context("Diary Link", inner)(input)
}

#[inline]
fn parse_date_from_path(path: &str) -> Option<NaiveDate> {
    path.strip_prefix("diary:").and_then(|date_str| {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
    })
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
