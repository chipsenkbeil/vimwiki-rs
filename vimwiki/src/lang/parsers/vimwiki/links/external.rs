use super::{
    components::{Description, ExternalFileLink, ExternalFileLinkScheme},
    utils::{context, lc, take_line_while1},
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, not, opt},
    sequence::{pair, preceded},
};
use std::path::PathBuf;

#[inline]
pub fn external_file_link(input: Span) -> VimwikiIResult<LC<ExternalFileLink>> {
    fn inner(input: Span) -> VimwikiIResult<ExternalFileLink> {
        let (input, _) = tag("[[")(input)?;
        let (input, link) = alt((
            preceded(
                tag("local:"),
                take_external_file_link(ExternalFileLinkScheme::Local),
            ),
            preceded(
                tag("file:"),
                take_external_file_link(ExternalFileLinkScheme::File),
            ),
            preceded(
                tag("//"),
                take_external_file_link(ExternalFileLinkScheme::Absolute),
            ),
        ))(input)?;
        let (input, _) = tag("]]")(input)?;

        Ok((input, link))
    }

    context("External File Link", lc(inner))(input)
}

#[inline]
fn take_external_file_link(
    scheme: ExternalFileLinkScheme,
) -> impl Fn(Span) -> VimwikiIResult<ExternalFileLink> {
    move |input: Span| {
        map(take_path_and_description, |(p, d)| {
            ExternalFileLink::new(scheme, p, d)
        })(input)
    }
}

#[inline]
fn take_path_and_description(
    input: Span,
) -> VimwikiIResult<(PathBuf, Option<Description>)> {
    pair(
        map(take_segment, |s: Span| PathBuf::from(s.fragment())),
        opt(preceded(
            tag("|"),
            map(take_segment, |s: Span| Description::from(*s.fragment())),
        )),
    )(input)
}

#[inline]
fn take_segment(input: Span) -> VimwikiIResult<Span> {
    let not_end = not(alt((tag("|"), tag("]]"))));
    take_line_while1(not_end)(input)
}

#[cfg(test)]
mod tests {
    use super::super::components::Description;
    use super::*;
    use crate::lang::utils::new_span;
    use std::path::PathBuf;

    #[test]
    fn external_link_should_support_absolute_path_with_no_scheme() {
        let input = new_span("[[//absolute_path]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Absolute);
        assert_eq!(link.path, PathBuf::from("absolute_path"));
        assert_eq!(link.description, None);

        let input = new_span("[[///tmp/in_root_tmp]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Absolute);
        assert_eq!(link.path, PathBuf::from("/tmp/in_root_tmp"));
        assert_eq!(link.description, None);

        let input = new_span("[[//~/in_home_dir]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Absolute);
        assert_eq!(link.path, PathBuf::from("~/in_home_dir"));
        assert_eq!(link.description, None);

        let input = new_span("[[//$HOME/in_home_dir]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Absolute);
        assert_eq!(link.path, PathBuf::from("$HOME/in_home_dir"));
        assert_eq!(link.description, None);
    }

    #[test]
    fn external_link_should_support_file_scheme() {
        let input = new_span("[[file:/home/somebody/a/b/c/music.mp3]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(link.path, PathBuf::from("/home/somebody/a/b/c/music.mp3"));
        assert_eq!(link.description, None);

        let input = new_span("[[file:C:/Users/somebody/d/e/f/music.mp3]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(
            link.path,
            PathBuf::from("C:/Users/somebody/d/e/f/music.mp3")
        );
        assert_eq!(link.description, None);

        let input = new_span("[[file:~/a/b/c/music.mp3]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(link.path, PathBuf::from("~/a/b/c/music.mp3"));
        assert_eq!(link.description, None);

        let input = new_span("[[file:../assets/data.csv|Important Data]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(link.path, PathBuf::from("../assets/data.csv"));
        assert_eq!(link.description, Some(Description::from("Important Data")));

        let input =
            new_span("[[file:/home/user/documents/|Link to a directory]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(link.path, PathBuf::from("/home/user/documents/"));
        assert_eq!(
            link.description,
            Some(Description::from("Link to a directory"))
        );
    }

    #[test]
    fn external_link_should_support_local_scheme() {
        let input = new_span("[[local:C:/Users/somebody/d/e/f/music.mp3]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Local);
        assert_eq!(
            link.path,
            PathBuf::from("C:/Users/somebody/d/e/f/music.mp3")
        );
        assert_eq!(link.description, None);
    }
}
