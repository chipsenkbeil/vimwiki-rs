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
    character::complete::line_ending,
    combinator::{map, map_parser, rest, rest_len},
    multi::many0,
    sequence::terminated,
};

pub fn comment(input: Span) -> IResult<Located<Comment>> {
    context(
        "Comment",
        alt((
            map(multi_line_comment, |c| c.map(Comment::from)),
            map(line_comment, |c| c.map(Comment::from)),
        )),
    )(input)
}

pub fn line_comment(input: Span) -> IResult<Located<LineComment>> {
    fn inner(input: Span) -> IResult<LineComment> {
        let (input, _) = tag("%%")(input)?;
        let (input, text) =
            map_parser(take_until_end_of_line_or_input, cow_str)(input)?;

        Ok((input, LineComment::new(text)))
    }

    context("Line Comment", locate(capture(inner)))(input)
}

pub fn multi_line_comment(input: Span) -> IResult<Located<MultiLineComment>> {
    fn inner(input: Span) -> IResult<MultiLineComment> {
        let (input, _) = tag("%%+")(input)?;

        // Capture all content between comments as individual lines
        let (input, lines) = map_parser(take_until("+%%"), |input| {
            // Get all lines but potentially the last one
            let (input, mut lines) = many0(terminated(
                map_parser(take_until_end_of_line_or_input, cow_str),
                line_ending,
            ))(input)?;

            // Get last line if there is anything in it and append it
            let (input, remaining) = rest_len(input)?;
            if remaining > 0 {
                let (input, last_line) = map_parser(rest, cow_str)(input)?;
                lines.push(last_line);
                Ok((input, lines))
            } else {
                Ok((input, lines))
            }
        })(input)?;

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
            Comment::Line(x) => assert_eq!(x.as_str(), " comment"),
            x => panic!("Unexpected element: {:?}", x),
        }
    }

    #[test]
    fn comment_should_parse_line_comment() {
        let input = Span::from("%% comment");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Did not consume comment");
        match c.into_inner() {
            Comment::Line(x) => assert_eq!(x.as_str(), " comment"),
            x => panic!("Unexpected element: {:?}", x),
        }

        // Support \n line termination
        let input = Span::from("%% comment\nnext line");
        let (input, c) = comment(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "\nnext line",
            "Unexpected input consumed"
        );
        match c.into_inner() {
            Comment::Line(x) => assert_eq!(x.as_str(), " comment"),
            x => panic!("Unexpected element: {:?}", x),
        }

        // Support \r\n line termination
        let input = Span::from("%% comment\r\nnext line");
        let (input, c) = comment(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "\r\nnext line",
            "Unexpected input consumed"
        );
        match c.into_inner() {
            Comment::Line(x) => assert_eq!(x.as_str(), " comment"),
            x => panic!("Unexpected element: {:?}", x),
        }
    }

    #[test]
    fn comment_should_parse_multi_line_comment() {
        let input = Span::from("%%+ comment +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Did not consume comment");
        match c.into_inner() {
            Comment::MultiLine(x) => {
                assert_eq!(
                    x.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
                    vec![" comment "]
                )
            }
            x => panic!("Unexpected element: {:?}", x),
        }

        // Support \n line termination
        let input = Span::from("%%+ comment\nnext line +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Did not consume comment");
        match c.into_inner() {
            Comment::MultiLine(x) => {
                assert_eq!(
                    x.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
                    vec![" comment", "next line "]
                )
            }
            x => panic!("Unexpected element: {:?}", x),
        }

        // Support \r\n line termination
        let input = Span::from("%%+ comment\r\nnext line +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Did not consume comment");
        match c.into_inner() {
            Comment::MultiLine(x) => {
                assert_eq!(
                    x.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
                    vec![" comment", "next line "]
                )
            }
            x => panic!("Unexpected element: {:?}", x),
        }

        // Support \n line termination
        let input = Span::from("%%+ comment\nnext line +%%after");
        let (input, c) = comment(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "after",
            "Unexpected input consumed"
        );
        match c.into_inner() {
            Comment::MultiLine(x) => {
                assert_eq!(
                    x.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
                    vec![" comment", "next line "]
                )
            }
            x => panic!("Unexpected element: {:?}", x),
        }

        // Support \r\n line termination
        let input = Span::from("%%+ comment\r\nnext line +%%after");
        let (input, c) = comment(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "after",
            "Unexpected input consumed"
        );
        match c.into_inner() {
            Comment::MultiLine(x) => {
                assert_eq!(
                    x.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
                    vec![" comment", "next line "]
                )
            }
            x => panic!("Unexpected element: {:?}", x),
        }

        // Support \n line termination
        let input = Span::from("%%+ comment\n+%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Input not fully consumed");
        match c.into_inner() {
            Comment::MultiLine(x) => {
                assert_eq!(
                    x.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
                    vec![" comment"]
                )
            }
            x => panic!("Unexpected element: {:?}", x),
        }

        // Support \r\n line termination
        let input = Span::from("%%+ comment\r\n+%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Input not fully consumed");
        match c.into_inner() {
            Comment::MultiLine(x) => {
                assert_eq!(
                    x.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
                    vec![" comment"]
                )
            }
            x => panic!("Unexpected element: {:?}", x),
        }
    }
}
