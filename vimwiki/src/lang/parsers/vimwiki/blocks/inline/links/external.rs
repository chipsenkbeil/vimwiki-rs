use crate::lang::{
    elements::{
        Description, ExternalFileLink, ExternalFileLinkScheme, Located,
    },
    parsers::{
        utils::{
            capture, context, cow_path, cow_str, locate,
            take_line_until_one_of_two1,
        },
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::{pair, preceded},
};
use std::{borrow::Cow, path::Path};

#[inline]
pub fn external_file_link(input: Span) -> IResult<Located<ExternalFileLink>> {
    fn inner(input: Span) -> IResult<ExternalFileLink> {
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

    context("External File Link", locate(capture(inner)))(input)
}

#[inline]
fn take_external_file_link(
    scheme: ExternalFileLinkScheme,
) -> impl Fn(Span) -> IResult<ExternalFileLink> {
    move |input: Span| {
        map(take_path_and_description, |(p, d)| {
            ExternalFileLink::new(scheme, p, d)
        })(input)
    }
}

#[inline]
fn take_path_and_description<'a>(
    input: Span<'a>,
) -> IResult<(Cow<'a, Path>, Option<Description<'a>>)> {
    pair(
        cow_path(take_segment),
        opt(preceded(
            tag("|"),
            map(cow_str(take_segment), Description::from),
        )),
    )(input)
}

#[inline]
fn take_segment(input: Span) -> IResult<Span> {
    take_line_until_one_of_two1("|", "]]")(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::Description;
    use std::path::PathBuf;

    #[test]
    fn external_link_should_support_absolute_path_with_no_scheme() {
        let input = Span::from("[[//absolute_path]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Absolute);
        assert_eq!(link.path, PathBuf::from("absolute_path"));
        assert_eq!(link.description, None);

        let input = Span::from("[[///tmp/in_root_tmp]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Absolute);
        assert_eq!(link.path, PathBuf::from("/tmp/in_root_tmp"));
        assert_eq!(link.description, None);

        let input = Span::from("[[//~/in_home_dir]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Absolute);
        assert_eq!(link.path, PathBuf::from("~/in_home_dir"));
        assert_eq!(link.description, None);

        let input = Span::from("[[//$HOME/in_home_dir]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Absolute);
        assert_eq!(link.path, PathBuf::from("$HOME/in_home_dir"));
        assert_eq!(link.description, None);
    }

    #[test]
    fn external_link_should_support_file_scheme() {
        let input = Span::from("[[file:/home/somebody/a/b/c/music.mp3]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(link.path, PathBuf::from("/home/somebody/a/b/c/music.mp3"));
        assert_eq!(link.description, None);

        let input = Span::from("[[file:C:/Users/somebody/d/e/f/music.mp3]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(
            link.path,
            PathBuf::from("C:/Users/somebody/d/e/f/music.mp3")
        );
        assert_eq!(link.description, None);

        let input = Span::from("[[file:~/a/b/c/music.mp3]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(link.path, PathBuf::from("~/a/b/c/music.mp3"));
        assert_eq!(link.description, None);

        let input = Span::from("[[file:../assets/data.csv|Important Data]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(link.path, PathBuf::from("../assets/data.csv"));
        assert_eq!(link.description, Some(Description::from("Important Data")));

        let input =
            Span::from("[[file:/home/user/documents/|Link to a directory]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::File);
        assert_eq!(link.path, PathBuf::from("/home/user/documents/"));
        assert_eq!(
            link.description,
            Some(Description::from("Link to a directory"))
        );
    }

    #[test]
    fn external_link_should_support_local_scheme() {
        let input = Span::from("[[local:C:/Users/somebody/d/e/f/music.mp3]]");
        let (input, link) = external_file_link(input).unwrap();
        assert!(input.is_empty(), "Did not consume link");
        assert_eq!(link.scheme, ExternalFileLinkScheme::Local);
        assert_eq!(
            link.path,
            PathBuf::from("C:/Users/somebody/d/e/f/music.mp3")
        );
        assert_eq!(link.description, None);
    }
}
