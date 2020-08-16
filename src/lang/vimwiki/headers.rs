use super::components::{
    Header, Header1, Header2, Header3, Header4, Header5, Header6,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    combinator::map,
    sequence::delimited,
    IResult,
};

pub fn header(input: &str) -> IResult<&str, Header> {
    alt((
        map(header1, From::from),
        map(header2, From::from),
        map(header3, From::from),
        map(header4, From::from),
        map(header5, From::from),
        map(header6, From::from),
    ))(input)
}

fn header1(input: &str) -> IResult<&str, Header1> {
    map(
        delimited(tag("= "), is_not(" ="), tag(" =")),
        |text: &str| Header1::new(text.to_string()),
    )(input)
}

fn header2(input: &str) -> IResult<&str, Header2> {
    map(
        delimited(tag("== "), is_not(" =="), tag(" ==")),
        |text: &str| Header2::new(text.to_string()),
    )(input)
}

fn header3(input: &str) -> IResult<&str, Header3> {
    map(
        delimited(tag("=== "), is_not(" ==="), tag(" ===")),
        |text: &str| Header3::new(text.to_string()),
    )(input)
}

fn header4(input: &str) -> IResult<&str, Header4> {
    map(
        delimited(tag("==== "), is_not(" ===="), tag(" ====")),
        |text: &str| Header4::new(text.to_string()),
    )(input)
}

fn header5(input: &str) -> IResult<&str, Header5> {
    map(
        delimited(tag("===== "), is_not(" ====="), tag(" =====")),
        |text: &str| Header5::new(text.to_string()),
    )(input)
}

fn header6(input: &str) -> IResult<&str, Header6> {
    map(
        delimited(tag("====== "), is_not(" ======"), tag(" ======")),
        |text: &str| Header6::new(text.to_string()),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_should_parse_level_1_header() {
        assert_eq!(
            header("= test header ="),
            Ok(("", Header::Header1("test header".into())))
        )
    }

    #[test]
    fn header_should_parse_level_2_header() {
        assert_eq!(
            header("== test header =="),
            Ok(("", Header::Header2("test header".into())))
        )
    }

    #[test]
    fn header_should_parse_level_3_header() {
        assert_eq!(
            header("=== test header ==="),
            Ok(("", Header::Header3("test header".into())))
        )
    }

    #[test]
    fn header_should_parse_level_4_header() {
        assert_eq!(
            header("==== test header ===="),
            Ok(("", Header::Header4("test header".into())))
        )
    }

    #[test]
    fn header_should_parse_level_5_header() {
        assert_eq!(
            header("===== test header ====="),
            Ok(("", Header::Header5("test header".into())))
        )
    }

    #[test]
    fn header_should_parse_level_6_header() {
        assert_eq!(
            header("====== test header ======"),
            Ok(("", Header::Header6("test header".into())))
        )
    }
}
