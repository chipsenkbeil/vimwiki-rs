use super::components::{
    Header, Header1, Header2, Header3, Header4, Header5, Header6,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::space0,
    combinator::{map, verify},
    error::{context, ParseError},
    sequence::{delimited, tuple},
    IResult,
};

/// Parses a vimwiki header, returning the associated header if successful
pub fn header<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Header, E> {
    // TODO: Custom error type to return error of parser that made the most
    //       progress across all of the below, rather than the last parser
    alt((
        context(
            "Header1",
            map(make_header_parser("=", Header1::from), From::from),
        ),
        context(
            "Header2",
            map(make_header_parser("==", Header2::from), From::from),
        ),
        context(
            "Header3",
            map(make_header_parser("===", Header3::from), From::from),
        ),
        context(
            "Header4",
            map(make_header_parser("====", Header4::from), From::from),
        ),
        context(
            "Header5",
            map(make_header_parser("=====", Header5::from), From::from),
        ),
        context(
            "Header6",
            map(make_header_parser("======", Header6::from), From::from),
        ),
    ))(input)
}

fn make_header_parser<'a, T, E: ParseError<&'a str>>(
    pattern: &'a str,
    f: impl Fn((&'a str, bool)) -> T,
) -> impl Fn(&'a str) -> IResult<&'a str, T, E> {
    // TODO: Handle newline; ensure that
    map(
        tuple((
            space0,
            delimited(
                tag(pattern),
                verify(take_until(pattern), |s: &str| {
                    !s.is_empty() && !s.starts_with('=')
                }),
                tag(pattern),
            ),
            space0,
        )),
        move |x| f((x.1, !x.0.is_empty())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{convert_error, VerboseError},
        Err,
    };

    fn parse_and_eval(input: &str, f: impl Fn((&str, Header))) {
        match header::<VerboseError<&str>>(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!("{}", convert_error(input, e))
            }
            Err(Err::Incomplete(needed)) => panic!("Incomplete: {:?}", needed),
            Ok(result) => f(result),
        }
    }

    fn parse_and_test(input: &str, level: usize, text: &str, centered: bool) {
        parse_and_eval(input, |result| {
            assert!(
                result.0.is_empty(),
                "Entire input not consumed: '{}'",
                result.0,
            );
            assert_eq!(
                result.1.level(),
                level,
                "Wrong header level: Got {}, but wanted {}",
                result.1.level(),
                level,
            );
            assert_eq!(
                result.1.text(),
                "test header",
                "Wrong header text: Got '{}', but wanted '{}'",
                result.1.text(),
                text,
            );
            assert_eq!(
                result.1.is_centered(),
                centered,
                "Wrong header centered: Got {}, but wanted {}",
                result.1.is_centered(),
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
