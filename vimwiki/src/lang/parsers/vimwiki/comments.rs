use super::{
    components::{Comment, LineComment, MultiLineComment},
    utils::{beginning_of_line, lc, pstring, take_until_end_of_line_or_input},
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::map,
    error::context,
};

#[inline]
pub fn comment(input: Span) -> VimwikiIResult<LC<Comment>> {
    context(
        "Comment",
        alt((
            map(multi_line_comment, |c| c.map(Comment::from)),
            map(line_comment, |c| c.map(Comment::from)),
        )),
    )(input)
}

#[inline]
pub(crate) fn line_comment(input: Span) -> VimwikiIResult<LC<LineComment>> {
    fn inner(input: Span) -> VimwikiIResult<LineComment> {
        let (input, _) = beginning_of_line(input)?;
        let (input, _) = tag("%%")(input)?;
        let (input, text) = pstring(take_until_end_of_line_or_input)(input)?;

        Ok((input, LineComment(text)))
    }

    context("Line Comment", lc(inner))(input)
}

#[inline]
pub(crate) fn multi_line_comment(
    input: Span,
) -> VimwikiIResult<LC<MultiLineComment>> {
    fn inner(input: Span) -> VimwikiIResult<MultiLineComment> {
        let (input, _) = tag("%%+")(input)?;

        // Capture all content between comments as individual lines
        let (input, lines) = map(take_until("+%%"), |s: Span| {
            s.fragment().lines().map(String::from).collect()
        })(input)?;

        let (input, _) = tag("+%%")(input)?;

        Ok((input, MultiLineComment(lines)))
    }

    context("Multi Line Comment", lc(inner))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::{new_span, Region};
    use nom::bytes::complete::take;

    #[test]
    fn comment_should_fail_if_no_input() {
        let input = new_span("");
        assert!(comment(input).is_err());
    }

    #[test]
    fn comment_should_fail_if_only_one_percent_sign() {
        let input = new_span("% comment");
        assert!(comment(input).is_err());

        let input = new_span("%+ comment +%");
        assert!(comment(input).is_err());
    }

    #[test]
    fn comment_should_fail_if_line_comment_not_at_start_of_line() {
        let input = new_span("abc%% comment");
        fn advance(input: Span) -> VimwikiIResult<()> {
            let (input, _) = take(3usize)(input)?;
            Ok((input, ()))
        }
        let (input, _) = advance(input).unwrap();
        assert!(comment(input).is_err());
    }

    #[test]
    fn comment_should_parse_line_comment() {
        let input = new_span("%% comment");
        let (input, c) = comment(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume comment");
        assert_eq!(
            c.component,
            Comment::from(LineComment(" comment".to_string()))
        );

        // NOTE: Line comment doesn't consume the newline; it leaves a blank line
        let input = new_span("%% comment\nnext line");
        let (input, c) = comment(input).unwrap();
        assert_eq!(
            *input.fragment(),
            "\nnext line",
            "Unexpected input consumed"
        );
        assert_eq!(
            c.component,
            Comment::from(LineComment(" comment".to_string()))
        );
        assert_eq!(c.region, Region::from((0, 0, 0, 9)));
    }

    #[test]
    fn comment_should_parse_multi_line_comment() {
        let input = new_span("%%+ comment +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume comment");
        assert_eq!(
            c.component,
            Comment::from(MultiLineComment(vec![" comment ".to_string()]))
        );
        assert_eq!(c.region, Region::from((0, 0, 0, 14)));

        let input = new_span("%%+ comment\nnext line +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume comment");
        assert_eq!(
            c.component,
            Comment::from(MultiLineComment(vec![
                " comment".to_string(),
                "next line ".to_string(),
            ]))
        );
        assert_eq!(c.region, Region::from((0, 0, 1, 12)));

        let input = new_span("%%+ comment\nnext line +%%after");
        let (input, c) = comment(input).unwrap();
        assert_eq!(*input.fragment(), "after", "Unexpected input consumed");
        assert_eq!(
            c.component,
            Comment::from(MultiLineComment(vec![
                " comment".to_string(),
                "next line ".to_string(),
            ]))
        );
        assert_eq!(c.region, Region::from((0, 0, 1, 12)));
    }
}
