use crate::lang::{
    elements::{Comment, LineComment, Located, MultiLineComment},
    parsers::{
        utils::{
            capture, context, cow_str, locate, take_until_end_of_line_or_input,
        },
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::{map, map_parser},
    multi::many1,
    sequence::terminated,
};

pub fn comment<'a>(input: Span<'a>) -> IResult<'a, Located<Comment<'a>>> {
    context(
        "Comment",
        alt((
            map(line_comment, |c| c.map(Comment::from)),
            map(multi_line_comment, |c| c.map(Comment::from)),
        )),
    )(input)
}

pub fn line_comment<'a>(
    input: Span<'a>,
) -> IResult<'a, Located<LineComment<'a>>> {
    fn inner<'a>(input: Span<'a>) -> IResult<'a, LineComment<'a>> {
        let (input, _) = tag("%%")(input)?;
        let (input, text) = cow_str(take_until_end_of_line_or_input)(input)?;

        Ok((input, LineComment::new(text)))
    }

    context("Line Comment", locate(capture(inner)))(input)
}

pub fn multi_line_comment<'a>(
    input: Span<'a>,
) -> IResult<'a, Located<MultiLineComment<'a>>> {
    fn inner<'a>(input: Span<'a>) -> IResult<'a, MultiLineComment<'a>> {
        let (input, _) = tag("%%+")(input)?;

        // Capture all content between comments as individual lines
        let (input, lines) = map_parser(
            take_until("+%%"),
            many1(cow_str(terminated(take_until("\n"), tag("\n")))),
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
            Comment::Line(x) => assert_eq!(x, " comment"),
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

        // NOTE: Line comment doesn't consume the newline; it leaves a blank line
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
            Comment::MultiLine(x) => assert_eq!(x.0, " comment "),
            x => panic!("Unexpected element: {:?}", x),
        }

        let input = Span::from("%%+ comment\nnext line +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.is_empty(), "Did not consume comment");
        match c.into_inner() {
            Comment::MultiLine(x) => assert_eq!(x.0, " comment\nnext line "),
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
            Comment::MultiLine(x) => assert_eq!(x.0, " comment\nnext line "),
            x => panic!("Unexpected element: {:?}", x),
        }
    }
}
