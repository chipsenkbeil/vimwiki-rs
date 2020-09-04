use super::{
    components::{
        Header, Header1, Header2, Header3, Header4, Header5, Header6,
    },
    utils::{end_of_line_or_input, position},
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{anychar, line_ending, space0},
    combinator::{map, not, recognize, verify},
    error::context,
    multi::many1,
    sequence::{delimited, tuple},
};

/// Parses a vimwiki header, returning the associated header if successful
#[inline]
pub fn header(input: Span) -> VimwikiIResult<LC<Header>> {
    // TODO: Custom error type to return error of parser that made the most
    //       progress across all of the below, rather than the last parser
    let (input, pos) = position(input)?;

    // NOTE: We split out the header definitions into standalone functions
    //       as we were hitting a type length limit through a series of
    //       impl Fn(...) -> IResult<...> in combination with alt(...), which
    //       itself is a Fn(...) -> IResult<...>; so, breaking out the headers
    //       enabled us to close off the series of impl Fn(...) -> IResult<...>
    //       earlier, which prevented the odd type length limit comp error
    let (input, header) = alt((
        map(header1, Header::from),
        map(header2, Header::from),
        map(header3, Header::from),
        map(header4, Header::from),
        map(header5, Header::from),
        map(header6, Header::from),
    ))(input)?;

    Ok((input, LC::from((header, pos, input))))
}

#[inline]
pub fn header1(input: Span) -> VimwikiIResult<Header1> {
    context("Header1", inner_header(1, Header1::from))(input)
}

#[inline]
pub fn header2(input: Span) -> VimwikiIResult<Header2> {
    context("Header2", inner_header(2, Header2::from))(input)
}

#[inline]
pub fn header3(input: Span) -> VimwikiIResult<Header3> {
    context("Header3", inner_header(3, Header3::from))(input)
}

#[inline]
pub fn header4(input: Span) -> VimwikiIResult<Header4> {
    context("Header4", inner_header(4, Header4::from))(input)
}

#[inline]
pub fn header5(input: Span) -> VimwikiIResult<Header5> {
    context("Header5", inner_header(5, Header5::from))(input)
}

#[inline]
pub fn header6(input: Span) -> VimwikiIResult<Header6> {
    context("Header6", inner_header(6, Header6::from))(input)
}

/// Builds a parser for a header based on the provided level
#[inline]
fn inner_header<'a, T>(
    level: u8,
    f: impl Fn((&'a str, bool)) -> T,
) -> impl Fn(Span<'a>) -> VimwikiIResult<T> {
    move |input: Span<'a>| {
        // Header itself is in the form of ===text===
        let header =
            delimited(surrounding(level), content(level), surrounding(level));

        // Header can be preceded by zero or more whitespace, which will indicate
        // whether it should be centered (in HTML) and is ended by zero or more
        // whitespace and either an end of line termination or the end of all
        // input if it's on the last line
        let (input, (leading_space, content, _)) =
            context("Header", tuple((space0, header, space0)))(input)?;

        // If we did not receive a line termination and the input is not empty,
        // then this is not actually a header
        let (input, _) = end_of_line_or_input(input)?;

        Ok((
            input,
            f((content.fragment(), !leading_space.fragment().is_empty())),
        ))
    }
}

/// Builds a parser to read content of header, which will not start with
/// an = sign (that would still be part of the surrounding) and is not empty
/// (in case a parser tries to inject in the middle of a surrounding)
#[inline]
fn content<'a>(level: u8) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
    verify(
        recognize(many1(tuple((
            not(surrounding(level)),
            not(line_ending),
            anychar,
        )))),
        |s| !s.fragment().is_empty() && !s.fragment().starts_with('='),
    )
}

/// Builds a parser to find a header boundary (surrounding =)
#[inline]
fn surrounding<'a>(level: u8) -> impl Fn(Span<'a>) -> VimwikiIResult<Span<'a>> {
    verify(take(level), is_header_boundary)
}

#[inline]
fn is_header_boundary<'a>(span: &Span<'a>) -> bool {
    span.fragment().chars().all(|c| c == '=')
}

#[cfg(test)]
mod tests {
    use super::super::super::utils::convert_error;
    use super::*;
    use nom::Err;

    fn parse_and_eval<'a>(input: Span<'a>, f: impl Fn((Span<'a>, LC<Header>))) {
        match header(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!("{}", convert_error(input, e))
            }
            Err(Err::Incomplete(needed)) => panic!("Incomplete: {:?}", needed),
            Ok(result) => f(result),
        }
    }

    fn parse_and_test(
        input_str: &str,
        level: usize,
        text: &str,
        centered: bool,
    ) {
        let input = Span::new(input_str);
        parse_and_eval(input, |result| {
            assert!(
                result.0.fragment().is_empty(),
                "Entire input not consumed! Input: '{}' | Remainder: '{}'",
                input,
                result.0,
            );
            assert_eq!(
                result.1.component.level(),
                level,
                "Wrong header level: Got {}, but wanted {}",
                result.1.component.level(),
                level,
            );
            assert_eq!(
                result.1.component.text(),
                "test header",
                "Wrong header text: Got '{}', but wanted '{}'",
                result.1.component.text(),
                text,
            );
            assert_eq!(
                result.1.component.is_centered(),
                centered,
                "Wrong header centered: Got {}, but wanted {}",
                result.1.component.is_centered(),
                centered,
            );
        });
    }

    #[test]
    fn header_should_parse_level_1_header() {
        let input = "=test header=";
        parse_and_test(input, 1, "test header", false);

        let input = " =test header= ";
        parse_and_test(input, 1, "test header", true);
    }

    #[test]
    fn header_should_parse_level_2_header() {
        let input = "==test header==";
        parse_and_test(input, 2, "test header", false);

        let input = " ==test header== ";
        parse_and_test(input, 2, "test header", true);
    }

    #[test]
    fn header_should_parse_level_3_header() {
        let input = "===test header===";
        parse_and_test(input, 3, "test header", false);

        let input = " ===test header=== ";
        parse_and_test(input, 3, "test header", true);
    }

    #[test]
    fn header_should_parse_level_4_header() {
        let input = "====test header====";
        parse_and_test(input, 4, "test header", false);

        let input = " ====test header==== ";
        parse_and_test(input, 4, "test header", true);
    }

    #[test]
    fn header_should_parse_level_5_header() {
        let input = "=====test header=====";
        parse_and_test(input, 5, "test header", false);

        let input = " =====test header===== ";
        parse_and_test(input, 5, "test header", true);
    }

    #[test]
    fn header_should_parse_level_6_header() {
        let input = "======test header======";
        parse_and_test(input, 6, "test header", false);

        let input = " ======test header====== ";
        parse_and_test(input, 6, "test header", true);
    }
}
