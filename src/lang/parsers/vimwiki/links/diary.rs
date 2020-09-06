use super::{
    components::DiaryLink,
    utils::{new_nom_error, take_line_while1},
    wiki::wiki_link,
    Span, VimwikiIResult, LC,
};
use chrono::NaiveDate;
use nom::{
    bytes::complete::tag, character::complete::anychar, sequence::preceded,
};

#[inline]
pub fn diary_link(input: Span) -> VimwikiIResult<LC<DiaryLink>> {
    // First, parse as a standard wiki link, which should stash the potential
    // diary as the path
    let (input, link) = wiki_link(input)?;
    let path = link.path.to_str().ok_or_else(|| {
        nom::Err::Error(new_nom_error(input, "Not diary link"))
    })?;

    // Second, check if the link is a diary
    match parse_date_from_path(path) {
        Some((_, date)) => Ok((
            input,
            link.map(|c| DiaryLink::new(date, c.description, c.anchor)),
        )),
        _ => Err(nom::Err::Error(new_nom_error(input, "Not diary link"))),
    }
}

#[inline]
fn parse_date_from_path(path: &str) -> Option<(&str, NaiveDate)> {
    preceded(tag("diary:"), take_line_while1(anychar))(Span::new(path))
        .ok()
        .map(|x| {
            NaiveDate::parse_from_str(x.1.fragment(), "%Y-%m-%d")
                .ok()
                .map(|date| (*x.0.fragment(), date))
        })
        .flatten()
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
