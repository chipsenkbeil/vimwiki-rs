use crate::lang::{
    elements::{Comment, LineComment, Located, MultiLineComment},
    parsers::{
        utils::{
            capture, context, cow_str, locate, take_until,
            take_until_end_of_line_or_input,
        },
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, map_parser, rest},
    multi::separated_list,
};

pub fn comment<'a>(input: Span<'a>) -> IResult<'a, Located<Comment<'a>>> {
    context(
        "Comment",
        alt((
            map(multi_line_comment, |c| c.map(Comment::from)),
            map(line_comment, |c| c.map(Comment::from)),
        )),
    )(input)
}

pub fn line_comment<'a>(
    input: Span<'a>,
) -> IResult<'a, Located<LineComment<'a>>> {
    fn inner(input: Span) -> IResult<LineComment> {
        let (input, _) = tag("%%")(input)?;
        let (input, text) =
            map_parser(take_until_end_of_line_or_input, cow_str)(input)?;

        Ok((input, LineComment::new(text)))
    }

    context("Line Comment", locate(capture(inner)))(input)
}

pub fn multi_line_comment<'a>(
    input: Span<'a>,
) -> IResult<'a, Located<MultiLineComment<'a>>> {
    fn inner(input: Span) -> IResult<MultiLineComment> {
        let (input, _) = tag("%%+")(input)?;

        // Capture all content between comments as individual lines
        let (input, lines) = map_parser(
            take_until("+%%"),
            separated_list(
                tag("\n"),
                map_parser(alt((take_until("\n"), rest)), cow_str),
            ),
        )(input)?;

        let (input, _) = tag("+%%")(input)?;

        Ok((input, MultiLineComment::new(lines)))
    }

    context("Multi Line Comment", locate(capture(inner)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::bytes::complete::take;

    #[test]
    fn comment_should_fail_if_no_input() {
        let input = Span::from("");
        assert!(comment(input).is_err());
    }

    #[test]
    fn comment_should_fail_if_only_one_percent_sign() {
        let input = Span::from("% comment");
        assert!(comment(input).is_err());

        let input = Span::from("%+ comment +%");
        assert!(comment(input).is_err());
    }

    #[test]
    fn comment_should_support_line_comment_not_at_beginning_of_line() {
        let input = Span::from("abc%% comment");
        fn advance(input: Span) -> IResult<()> {
            let (input, _) = take(3usize)(input)?;
            Ok((input, ()))
        }
        let (input, _) = advance(input).unwrap();
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Did not consume comment");

        match c.into_inner() {
            Comment::Line(x) => assert_eq!(x.0, " comment"),
            x => panic!("Unexpected element: {:?}", x),
        }
    }

    #[test]
    fn comment_should_parse_line_comment() {
        let input = Span::from("%% comment");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Did not consume comment");
        match c.into_inner() {
            Comment::Line(x) => assert_eq!(x.0, " comment"),
            x => panic!("Unexpected element: {:?}", x),
        }

        let input = Span::from("%% comment\nnext line");
        let (input, c) = comment(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "\nnext line",
            "Unexpected input consumed"
        );
        match c.into_inner() {
            Comment::Line(x) => assert_eq!(x.0, " comment"),
            x => panic!("Unexpected element: {:?}", x),
        }
    }

    #[test]
    fn comment_should_parse_multi_line_comment() {
        let input = Span::from("%%+ comment +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Did not consume comment");
        match c.into_inner() {
            Comment::MultiLine(x) => assert_eq!(x.0, vec![" comment "]),
            x => panic!("Unexpected element: {:?}", x),
        }

        let input = Span::from("%%+ comment\nnext line +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Did not consume comment");
        match c.into_inner() {
            Comment::MultiLine(x) => {
                assert_eq!(x.0, vec![" comment", "next line "])
            }
            x => panic!("Unexpected element: {:?}", x),
        }

        let input = Span::from("%%+ comment\nnext line +%%after");
        let (input, c) = comment(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "after",
            "Unexpected input consumed"
        );
        match c.into_inner() {
            Comment::MultiLine(x) => {
                assert_eq!(x.0, vec![" comment", "next line "])
            }
            x => panic!("Unexpected element: {:?}", x),
        }
    }
}
