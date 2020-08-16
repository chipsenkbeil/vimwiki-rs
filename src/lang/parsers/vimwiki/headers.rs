use super::components::{
    Header, Header1, Header2, Header3, Header4, Header5, Header6,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    combinator::map,
    error::ParseError,
    multi::fold_many0,
    sequence::delimited,
    IResult,
};

const HEADER1_START: &'static str = "= ";
const HEADER1_END: &'static str = " =";
const HEADER2_START: &'static str = "== ";
const HEADER2_END: &'static str = " ==";
const HEADER3_START: &'static str = "=== ";
const HEADER3_END: &'static str = " ===";
const HEADER4_START: &'static str = "==== ";
const HEADER4_END: &'static str = " ====";
const HEADER5_START: &'static str = "===== ";
const HEADER5_END: &'static str = " =====";
const HEADER6_START: &'static str = "====== ";
const HEADER6_END: &'static str = " ======";

pub fn header<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Header, E> {
    alt((
        map(
            make_header_parser(HEADER1_START, HEADER1_END, Header1::new),
            From::from,
        ),
        map(
            make_header_parser(HEADER2_START, HEADER2_END, Header2::new),
            From::from,
        ),
        map(
            make_header_parser(HEADER3_START, HEADER3_END, Header3::new),
            From::from,
        ),
        map(
            make_header_parser(HEADER4_START, HEADER4_END, Header4::new),
            From::from,
        ),
        map(
            make_header_parser(HEADER5_START, HEADER5_END, Header5::new),
            From::from,
        ),
        map(
            make_header_parser(HEADER6_START, HEADER6_END, Header6::new),
            From::from,
        ),
    ))(input)
}

fn make_header_parser<'a, T, E: ParseError<&'a str>>(
    start: &'static str,
    end: &'static str,
    f: impl Fn(String) -> T,
) -> impl Fn(&'a str) -> IResult<&'a str, T, E> {
    map(
        delimited(
            tag(start),
            fold_many0(is_not(end), String::new(), |mut s, item| {
                s.push_str(item);
                s
            }),
            tag(end),
        ),
        f,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{convert_error, VerboseError},
        Err,
    };

    #[test]
    fn header_should_parse_level_1_header() {
        let input = "= test header =";
        match header::<VerboseError<&str>>(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!("{}", convert_error(input, e))
            }
            Err(Err::Incomplete(needed)) => panic!("Incomplete: {:?}", needed),
            Ok(result) => {
                assert_eq!(result, ("", Header::Header1("test header".into())));
            }
        }
    }

    #[test]
    fn header_should_parse_level_2_header() {
        let input = "== test header ==";
        match header::<VerboseError<&str>>(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!("{}", convert_error(input, e))
            }
            Err(Err::Incomplete(needed)) => panic!("Incomplete: {:?}", needed),
            Ok(result) => {
                assert_eq!(result, ("", Header::Header2("test header".into())));
            }
        }
    }

    #[test]
    fn header_should_parse_level_3_header() {
        let input = "=== test header ===";
        match header::<VerboseError<&str>>(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!("{}", convert_error(input, e))
            }
            Err(Err::Incomplete(needed)) => panic!("Incomplete: {:?}", needed),
            Ok(result) => {
                assert_eq!(result, ("", Header::Header3("test header".into())));
            }
        }
    }

    #[test]
    fn header_should_parse_level_4_header() {
        let input = "==== test header ====";
        match header::<VerboseError<&str>>(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!("{}", convert_error(input, e))
            }
            Err(Err::Incomplete(needed)) => panic!("Incomplete: {:?}", needed),
            Ok(result) => {
                assert_eq!(result, ("", Header::Header4("test header".into())));
            }
        }
    }

    #[test]
    fn header_should_parse_level_5_header() {
        let input = "===== test header =====";
        match header::<VerboseError<&str>>(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!("{}", convert_error(input, e))
            }
            Err(Err::Incomplete(needed)) => panic!("Incomplete: {:?}", needed),
            Ok(result) => {
                assert_eq!(result, ("", Header::Header5("test header".into())));
            }
        }
    }

    #[test]
    fn header_should_parse_level_6_header() {
        let input = "====== test header ======";
        match header::<VerboseError<&str>>(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!("{}", convert_error(input, e))
            }
            Err(Err::Incomplete(needed)) => panic!("Incomplete: {:?}", needed),
            Ok(result) => {
                assert_eq!(result, ("", Header::Header6("test header".into())));
            }
        }
    }
}
