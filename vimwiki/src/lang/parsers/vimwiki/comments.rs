use super::{
    elements::{Comment, LineComment, MultiLineComment},
    utils::{context, lc, pstring, take_until_end_of_line_or_input},
    Span, VimwikiIResult, LE,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::map,
};

#[inline]
pub fn comment(input: Span) -> VimwikiIResult<LE<Comment>> {
    context(
        "Comment",
        alt((
            map(multi_line_comment, |c| c.map(Comment::from)),
            map(line_comment, |c| c.map(Comment::from)),
        )),
    )(input)
}

#[inline]
pub(crate) fn line_comment(input: Span) -> VimwikiIResult<LE<LineComment>> {
    fn inner(input: Span) -> VimwikiIResult<LineComment> {
        let (input, _) = tag("%%")(input)?;
        let (input, text) = pstring(take_until_end_of_line_or_input)(input)?;

        Ok((input, LineComment(text)))
    }

    context("Line Comment", lc(inner))(input)
}

#[inline]
pub(crate) fn multi_line_comment(
    input: Span,
) -> VimwikiIResult<LE<MultiLineComment>> {
    fn inner(input: Span) -> VimwikiIResult<MultiLineComment> {
        let (input, _) = tag("%%+")(input)?;

        // Capture all content between comments as individual lines
        let (input, lines) = map(take_until("+%%"), |s: Span| {
            s.fragment_str().lines().map(String::from).collect()
        })(input)?;

        let (input, _) = tag("+%%")(input)?;

        Ok((input, MultiLineComment(lines)))
    }

    context("Multi Line Comment", lc(inner))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::{Region, Span};
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
        fn advance(input: Span) -> VimwikiIResult<()> {
            let (input, _) = take(3usize)(input)?;
            Ok((input, ()))
        }
        let (input, _) = advance(input).unwrap();
        let (input, c) = comment(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume comment");
        assert_eq!(
            c.element,
            Comment::from(LineComment(" comment".to_string()))
        );
    }

    #[test]
    fn comment_should_parse_line_comment() {
        let input = Span::from("%% comment");
        let (input, c) = comment(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume comment");
        assert_eq!(
            c.element,
            Comment::from(LineComment(" comment".to_string()))
        );

        // NOTE: Line comment doesn't consume the newline; it leaves a blank line
        let input = Span::from("%% comment\nnext line");
        let (input, c) = comment(input).unwrap();
        assert_eq!(
            input.fragment_str(),
            "\nnext line",
            "Unexpected input consumed"
        );
        assert_eq!(
            c.element,
            Comment::from(LineComment(" comment".to_string()))
        );
        assert_eq!(c.region, Region::from((1, 1, 1, 10)));
    }

    #[test]
    fn comment_should_parse_multi_line_comment() {
        let input = Span::from("%%+ comment +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume comment");
        assert_eq!(
            c.element,
            Comment::from(MultiLineComment(vec![" comment ".to_string()]))
        );
        assert_eq!(c.region, Region::from((1, 1, 1, 15)));

        let input = Span::from("%%+ comment\nnext line +%%");
        let (input, c) = comment(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume comment");
        assert_eq!(
            c.element,
            Comment::from(MultiLineComment(vec![
                " comment".to_string(),
                "next line ".to_string(),
            ]))
        );
        assert_eq!(c.region, Region::from((1, 1, 2, 13)));

        let input = Span::from("%%+ comment\nnext line +%%after");
        let (input, c) = comment(input).unwrap();
        assert_eq!(input.fragment_str(), "after", "Unexpected input consumed");
        assert_eq!(
            c.element,
            Comment::from(MultiLineComment(vec![
                " comment".to_string(),
                "next line ".to_string(),
            ]))
        );
        assert_eq!(c.region, Region::from((1, 1, 2, 13)));
    }
}
