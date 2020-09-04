use super::{
    components::PreformattedText,
    utils::{any_line, beginning_of_line, position},
    Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::{tag, take_till, take_till1},
    character::complete::char,
    combinator::{map, not},
    multi::many1,
    sequence::{delimited, preceded, separated_pair},
};
use std::collections::HashMap;

#[inline]
pub fn preformatted_text(input: Span) -> VimwikiIResult<LC<PreformattedText>> {
    let (input, pos) = position(input)?;

    let (input, metadata) = preformatted_text_start(input)?;
    let (input, lines) =
        many1(preceded(not(preformatted_text_end), any_line))(input)?;
    let (input, _) = preformatted_text_end(input)?;

    Ok((
        input,
        LC::from((PreformattedText::new(metadata, lines), pos, input)),
    ))
}

#[inline]
fn preformatted_text_start(
    input: Span,
) -> VimwikiIResult<HashMap<String, String>> {
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = tag("{{{")(input)?;
    let (input, (k, v)) = map(
        separated_pair(
            take_till1(|c| c == '='),
            char('='),
            delimited(char('"'), take_till(|c| c == '='), char('"')),
        ),
        |(k, v): (Span, Span)| {
            (k.fragment().to_string(), v.fragment().to_string())
        },
    )(input)?;

    let mut metadata = HashMap::new();
    metadata.insert(k, v);

    Ok((input, metadata))
}

#[inline]
fn preformatted_text_end(input: Span) -> VimwikiIResult<()> {
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = tag("}}}")(input)?;

    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preformatted_text_should_fail_if_does_not_have_starting_line() {
        todo!();
    }

    #[test]
    fn preformatted_text_should_fail_if_does_not_have_ending_line() {
        todo!();
    }

    #[test]
    fn preformatted_text_should_fail_if_does_not_have_lines_inbetween() {
        todo!();
    }

    #[test]
    fn preformatted_text_should_parse_all_lines_between() {
        todo!();
    }

    #[test]
    fn preformatted_text_should_support_metadata() {
        todo!();
    }
}
