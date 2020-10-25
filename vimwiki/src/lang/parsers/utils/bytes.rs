use super::{context, end_of_line_or_input};
use crate::lang::parsers::{Error, IResult, Span};
use memchr::{memchr, memchr2_iter, memchr3_iter, memchr_iter};
use nom::{
    bytes::complete::tag,
    character::complete::anychar,
    combinator::{not, recognize, rest, verify},
    multi::many0,
    AsBytes, InputLength, InputTake,
};

/// Parser that runs child parser and succeeds if the parser yields output
/// that does not contain the given pattern
pub fn not_contains<'a>(
    pattern: &'static str,
    parser: impl Fn(Span<'a>) -> IResult<Span<'a>>,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    move |input: Span| {
        let (input, result) = parser(input)?;
        let (result, _) = not(offset(pattern))(result)?;
        Ok((input, result))
    }
}

/// Parser that finds with byte offset of the given pattern within the provided
/// input, or fails if not found; does not consume input
pub fn offset(pattern: &'static str) -> impl Fn(Span) -> IResult<usize> {
    move |input: Span| {
        let bytes = input.as_bytes();
        for pos in memchr_iter(pattern.as_bytes()[0], bytes) {
            let end = pos + pattern.len();
            if end >= bytes.len() {
                break;
            }

            if &bytes[pos..end] == pattern.as_bytes() {
                return Ok((input, pos));
            }
        }

        Err(nom::Err::Error(Error::from_ctx(
            &input,
            "Input does not contain pattern",
        )))
    }
}

/// Parser that consumes input inside the surrounding left and right sides,
/// failing if not starting with the left or if the right is not found prior
/// to the end of a line. The result is the content WITHIN the surroundings.
/// Will not match right side if it follows immediately from the left.
///
/// Note that the left and right must be non-empty.
pub fn surround_in_line1<'a>(
    left: &'static str,
    right: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    fn inner<'a>(
        left: &'static str,
        right: &'static str,
    ) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
        move |input: Span| {
            let (input, _) = tag(left)(input)?;
            let input_bytes = input.as_bytes();

            // First, figure out where our next line will be
            let maybe_newline_pos = memchr(b'\n', input_bytes);

            // Second, look for the starting byte of the right side of our
            // surround wrapper
            for pos in memchr_iter(right.as_bytes()[0], input_bytes) {
                // If we've reached the end of the line, return an error
                if let Some(newline_pos) = maybe_newline_pos {
                    if pos >= newline_pos {
                        return Err(nom::Err::Error(Error::from_ctx(
                            &input,
                            "end of line reached before right side",
                        )));
                    }
                }

                // If there would be nothing in the surroundings, continue
                if pos == 0 {
                    continue;
                }

                // Grab everything but the possible start of the right
                let (input, content) = input.take_split(pos);
                if input.input_len() < right.len() {
                    break;
                }

                // Verify that the right would be next, and if so return our
                // result having consumed it, otherwise continue
                let (input, right_span) = input.take_split(right.len());
                if right_span.as_bytes() == right.as_bytes() {
                    return Ok((input, content));
                } else {
                    continue;
                }
            }

            // There was no match of the right side
            Err(nom::Err::Error(Error::from_ctx(
                &input,
                "right side not found",
            )))
        }
    }

    context("Surround in Line", inner(left, right))
}

/// Parser that consumes input while the pattern succeeds or we reach the
/// end of the line. Note that this does NOT consume the line termination.
pub fn take_line_while<'a, T>(
    parser: impl Fn(Span<'a>) -> IResult<T>,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    fn single_char<'a, T>(
        parser: impl Fn(Span<'a>) -> IResult<T>,
    ) -> impl Fn(Span<'a>) -> IResult<char> {
        move |input: Span| {
            let (input, _) = not(end_of_line_or_input)(input)?;

            // NOTE: This is the same as peek(parser), but avoids the issue
            //       of variable being moved out of captured Fn(...)
            let (_, _) = parser(input)?;

            anychar(input)
        }
    }

    context("Take Line While", recognize(many0(single_char(parser))))
}

/// Parser that consumes input while the pattern succeeds or we reach the
/// end of the line. Note that this does NOT consume the line termination.
pub fn take_line_while1<'a, T>(
    parser: impl Fn(Span<'a>) -> IResult<T>,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    context(
        "Take Line While 1",
        verify(take_line_while(parser), |s| !s.is_empty()),
    )
}

/// Parser that consumes input until the pattern succeeds or we reach the end
/// of the line. Note that this does NOT consume the pattern or the line
/// termination.
pub fn take_line_until<'a>(
    pattern: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    context("Take Line Until", move |input: Span| {
        let bytes = input.as_bytes();
        for pos in memchr2_iter(b'\n', pattern.as_bytes()[0], bytes) {
            // If we have reached the end of line, return with everything
            // but the end of line
            if bytes[pos] == b'\n' {
                return Ok(input.take_split(pos));
            }

            // Grab everything but the possible pattern
            let (input, content) = input.take_split(pos);
            let end = pos + pattern.len();
            if end >= bytes.len() {
                break;
            }

            // Verify that the pattern would be next, and if so return our
            // result, otherwise continue
            if &bytes[pos..end] == pattern.as_bytes() {
                return Ok((input, content));
            } else {
                continue;
            }
        }

        Ok(input.take_split(input.input_len()))
    })
}

/// Parser that consumes input until the pattern succeeds or we reach the end
/// of the line. Note that this does NOT consume the pattern or the line
/// termination.
pub fn take_line_until1<'a>(
    pattern: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    context(
        "Take Line Until 1",
        verify(take_line_until(pattern), |s| !s.is_empty()),
    )
}

/// Parser that consumes input until one of the two patterns succeed or we
/// reach the end of the line. Note that this does NOT consume the pattern or
/// the line termination.
pub fn take_line_until_one_of_two<'a>(
    pattern1: &'static str,
    pattern2: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    context("Take Line Until One of Two", move |input: Span| {
        let bytes = input.as_bytes();

        let p1_bytes = pattern1.as_bytes();
        let p2_bytes = pattern2.as_bytes();

        for pos in memchr3_iter(b'\n', p1_bytes[0], p2_bytes[0], bytes) {
            // If we have reached the end of line, return everything
            // before the line position
            if bytes[pos] == b'\n' {
                return Ok(input.take_split(pos));
            }

            // Grab everything but the possible pattern
            let (input, content) = input.take_split(pos);
            let end1 = pos + p1_bytes.len();
            let end2 = pos + p2_bytes.len();

            // Verify that the pattern would be next, and if so return our
            // result, otherwise continue
            if (end1 < bytes.len() && &bytes[pos..end1] == p1_bytes.as_bytes())
                || (end2 < bytes.len()
                    && &bytes[pos..end2] == p2_bytes.as_bytes())
            {
                return Ok((input, content));
            } else {
                continue;
            }
        }

        Ok(input.take_split(input.input_len()))
    })
}

/// Parser that consumes input until one of the two patterns succeed or we
/// reach the end of the line. Note that this does NOT consume the pattern or
/// the line termination.
pub fn take_line_until_one_of_two1<'a>(
    pattern1: &'static str,
    pattern2: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    context(
        "Take Line Until One of Two 1",
        verify(take_line_until_one_of_two(pattern1, pattern2), |s| {
            !s.is_empty()
        }),
    )
}

/// Parser that consumes input until one of the three patterns succeed or we
/// reach the end of the line. Note that this does NOT consume the pattern or
/// the line termination.
pub fn take_line_until_one_of_three<'a>(
    pattern1: &'static str,
    pattern2: &'static str,
    pattern3: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    context("Take Line Until One of Three", move |input: Span| {
        let bytes = input.as_bytes();

        let maybe_line_pos = memchr(b'\n', bytes);
        let p1_bytes = pattern1.as_bytes();
        let p1_start = p1_bytes[0];
        let p2_bytes = pattern2.as_bytes();
        let p2_start = p2_bytes[0];
        let p3_bytes = pattern3.as_bytes();
        let p3_start = p3_bytes[0];

        for pos in memchr3_iter(p1_start, p2_start, p3_start, bytes) {
            // If we have reached or passed the end of line, return everything
            // before the line position
            match maybe_line_pos {
                Some(line_pos) if line_pos >= pos => {
                    return Ok(input.take_split(line_pos));
                }
                _ => {}
            }

            // Grab everything but the possible pattern
            let (input, content) = input.take_split(pos);
            let end1 = pos + p1_bytes.len();
            let end2 = pos + p2_bytes.len();
            let end3 = pos + p3_bytes.len();

            // Verify that the pattern would be next, and if so return our
            // result, otherwise continue
            if (end1 < bytes.len() && &bytes[pos..end1] == p1_bytes.as_bytes())
                || (end2 < bytes.len()
                    && &bytes[pos..end2] == p2_bytes.as_bytes())
                || (end3 < bytes.len()
                    && &bytes[pos..end3] == p3_bytes.as_bytes())
            {
                return Ok((input, content));
            } else {
                continue;
            }
        }

        // No match found for any of three patterns, so we return either
        // all of the input or up to the found newline
        Ok(input
            .take_split(maybe_line_pos.unwrap_or_else(|| input.input_len())))
    })
}

/// Parser that consumes input until one of the three patterns succeed or we
/// reach the end of the line. Note that this does NOT consume the pattern or
/// the line termination.
pub fn take_line_until_one_of_three1<'a>(
    pattern1: &'static str,
    pattern2: &'static str,
    pattern3: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    context(
        "Take Line Until One of Three 1",
        verify(
            take_line_until_one_of_three(pattern1, pattern2, pattern3),
            |s| !s.is_empty(),
        ),
    )
}

/// Parser that will consume the remainder of a line (or end of input)
pub fn take_until_end_of_line_or_input(input: Span) -> IResult<Span> {
    fn inner(input: Span) -> IResult<Span> {
        match memchr(b'\n', input.as_bytes()) {
            Some(pos) => Ok(input.take_split(pos)),
            _ => rest(input),
        }
    }

    context("Take Until End of Line or Input", inner)(input)
}

/// Parser that will consume input until the specified pattern is found,
/// failing if the pattern is never found
pub fn take_until<'a>(
    pattern: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>> {
    move |input: Span| {
        let bytes = input.as_bytes();
        for pos in memchr_iter(pattern.as_bytes()[0], bytes) {
            let (input, content) = input.take_split(pos);
            let end = pos + pattern.len();
            if end >= bytes.len() {
                break;
            }

            if &bytes[pos..end] == pattern.as_bytes() {
                return Ok((input, content));
            } else {
                continue;
            }
        }

        Err(nom::Err::Error(Error::from_ctx(
            &input,
            "Unable to find pattern",
        )))
    }
}

/// Takes from the end instead of the beginning
pub fn take_end<'a, C>(count: C) -> impl Fn(Span<'a>) -> IResult<Span<'a>>
where
    C: nom::ToUsize,
{
    use nom::{
        error::{ErrorKind, ParseError},
        Err,
    };
    let cnt = count.to_usize();
    context("Take End", move |input: Span| {
        let len = input.input_len();
        if cnt > len {
            Err(Err::Error(Error::from_error_kind(input, ErrorKind::Eof)))
        } else {
            let (end, input) = input.take_split(len - cnt);
            Ok((input, end))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::character::complete::char;

    #[test]
    fn not_contains_should_fail_if_wrapped_parser_output_does_contain_pattern()
    {
        let input = Span::from("aabbcc");
        assert!(not_contains("bb", tag("aabbcc"))(input).is_err());
    }

    #[test]
    fn not_contains_should_succeed_if_wrapped_parser_output_does_contain_entire_pattern_at_end(
    ) {
        let input = Span::from("aabbcc");
        let (input, result) =
            not_contains("ccc", tag("aabbcc"))(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn not_contains_should_succeed_if_wrapped_parser_output_does_not_contain_pattern(
    ) {
        let input = Span::from("aabbcc");
        let (input, result) = not_contains("dd", tag("aabbcc"))(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn offset_should_return_byte_offset_of_first_pattern_match_in_input() {
        let input = Span::from("aabbbbcc");
        let (input, x) = offset("bb")(input).unwrap();
        assert_eq!(input, "aabbbbcc");
        assert_eq!(x, 2);
    }

    #[test]
    fn offset_should_fail_if_no_pattern_match_in_input() {
        let input = Span::from("aabbcc");
        assert!(offset("dd")(input).is_err());
    }

    #[test]
    fn offset_should_fail_if_only_start_of_right_pattern_found_at_end_of_input()
    {
        let input = Span::from("aabbcc");
        assert!(offset("ccc")(input).is_err());
    }

    #[test]
    fn surround_in_line1_should_fail_if_input_not_starting_with_left_pattern() {
        let input = Span::from("aabbcc");
        assert!(surround_in_line1("dd", "cc")(input).is_err());
    }

    #[test]
    fn surround_in_line1_should_fail_if_input_does_not_contain_right_pattern() {
        let input = Span::from("aabbcc");
        assert!(surround_in_line1("aa", "dd")(input).is_err());
    }

    #[test]
    fn surround_in_line1_should_fail_if_right_pattern_found_after_newline() {
        let input = Span::from("aabb\ncc");
        assert!(surround_in_line1("aa", "cc")(input).is_err());
    }

    #[test]
    fn surround_in_line1_should_fail_if_only_start_of_right_pattern_found_at_end_of_input(
    ) {
        let input = Span::from("aabbcc");
        assert!(surround_in_line1("aa", "ccc")(input).is_err());
    }

    #[test]
    fn surround_in_line1_should_fail_if_no_input_found_between_left_and_right()
    {
        let input = Span::from("aacc");
        assert!(surround_in_line1("aa", "cc")(input).is_err());
    }

    #[test]
    fn surround_in_line1_should_return_input_between_left_and_right() {
        let input = Span::from("aabbcc");
        let (input, result) = surround_in_line1("aa", "cc")(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "bb");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_empty_input() {
        let input = Span::from("");
        let (_, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(taken.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_line_termination_next() {
        let input = Span::from("\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "");

        let input = Span::from("\r\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\r\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn take_line_while_should_yield_empty_if_stops_without_ever_succeeding() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(char('c'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "aabb\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "");
    }

    #[test]
    fn take_line_while_should_take_until_provided_parser_fails() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(char('a'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "bb\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aa");
    }

    #[test]
    fn take_line_while_should_take_until_line_termination_reached() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aabb");
    }

    #[test]
    fn take_line_while_should_count_condition_parser_towards_consumption() {
        // NOTE: Using an ODD number of characters as otherwise we wouldn't
        //       catch the error which was happening where we would use the
        //       parser, char('-'), which would consume a character since it
        //       was not a not(...) and then try to use an anychar, so we
        //       would end up consuming TWO parsers instead of one
        let input = Span::from("-----");
        let (input, taken) = take_line_while(char('-'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "");
        assert_eq!(taken.as_unsafe_remaining_str(), "-----");
    }

    #[test]
    fn take_line_while1_should_fail_if_empty_input() {
        let input = Span::from("");
        assert!(take_line_while1(anychar)(input).is_err());
    }

    #[test]
    fn take_line_while1_should_fail_if_line_termination_next() {
        let input = Span::from("\nabcd");
        assert!(take_line_while1(anychar)(input).is_err());

        let input = Span::from("\r\nabcd");
        assert!(take_line_while1(anychar)(input).is_err());
    }

    #[test]
    fn take_line_while1_should_fail_if_stops_without_ever_succeeding() {
        let input = Span::from("aabb\nabcd");
        assert!(take_line_while1(char('c'))(input).is_err());
    }

    #[test]
    fn take_line_while1_should_take_until_provided_parser_fails() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while1(char('a'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "bb\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aa");
    }

    #[test]
    fn take_line_while1_should_take_until_line_termination_reached() {
        let input = Span::from("aabb\nabcd");
        let (input, taken) = take_line_while1(anychar)(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "\nabcd");
        assert_eq!(taken.as_unsafe_remaining_str(), "aabb");
    }

    #[test]
    fn take_line_while1_should_count_condition_parser_towards_consumption() {
        // NOTE: Using an ODD number of characters as otherwise we wouldn't
        //       catch the error which was happening where we would use the
        //       parser, char('-'), which would consume a character since it
        //       was not a not(...) and then try to use an anychar, so we
        //       would end up consuming TWO parsers instead of one
        let input = Span::from("-----");
        let (input, taken) = take_line_while1(char('-'))(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "");
        assert_eq!(taken.as_unsafe_remaining_str(), "-----");
    }

    #[test]
    fn take_line_until_should_consume_entire_line_except_newline_if_pattern_not_found(
    ) {
        let input = Span::from("aabbcc\nddeeff");
        let (input, result) = take_line_until("zz")(input).unwrap();
        assert_eq!(input, "\nddeeff");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_line_until_should_consume_input_if_no_newline_or_pattern() {
        let input = Span::from("aabbcc");
        let (input, result) = take_line_until("zz")(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_line_until_should_consume_input_if_only_part_of_pattern_found_at_end_of_input(
    ) {
        let input = Span::from("aabbcc");
        let (input, result) = take_line_until("ccc")(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_line_until_should_consume_until_pattern_found() {
        let input = Span::from("aabbcc");
        let (input, result) = take_line_until("bc")(input).unwrap();
        assert_eq!(input, "bcc");
        assert_eq!(result, "aab");
    }

    #[test]
    fn take_line_until_one_of_two_should_consume_entire_line_except_newline_if_both_patterns_not_found(
    ) {
        let input = Span::from("aabbcc\nddeeff");
        let (input, result) =
            take_line_until_one_of_two("yy", "zz")(input).unwrap();
        assert_eq!(input, "\nddeeff");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_line_until_one_of_two_should_consume_input_if_no_newline_or_either_pattern(
    ) {
        let input = Span::from("aabbcc");
        let (input, result) =
            take_line_until_one_of_two("yy", "zz")(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_line_until_one_of_two_should_consume_input_if_only_part_of_either_pattern_found_at_end_of_input(
    ) {
        let input = Span::from("aabbcc");
        let (input, result) =
            take_line_until_one_of_two("ccc", "cccc")(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_line_until_one_of_two_should_consume_until_first_of_either_pattern_found(
    ) {
        let input = Span::from("aabbcc");
        let (input, result) =
            take_line_until_one_of_two("bc", "bb")(input).unwrap();
        assert_eq!(input, "bbcc");
        assert_eq!(result, "aa");
    }

    #[test]
    fn take_line_until_one_of_three_should_consume_entire_line_except_newline_if_all_patterns_not_found(
    ) {
        let input = Span::from("aabbcc\nddeeff");
        let (input, result) =
            take_line_until_one_of_three("xx", "yy", "zz")(input).unwrap();
        assert_eq!(input, "\nddeeff");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_line_until_one_of_three_should_consume_input_if_no_newline_or_any_pattern(
    ) {
        let input = Span::from("aabbcc");
        let (input, result) =
            take_line_until_one_of_three("xx", "yy", "zz")(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_line_until_one_of_three_should_consume_input_if_only_part_of_any_pattern_found_at_end_of_input(
    ) {
        let input = Span::from("aabbcc");
        let (input, result) =
            take_line_until_one_of_three("ccc", "cccc", "ccccc")(input)
                .unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_line_until_one_of_three_should_consume_until_any_pattern_found() {
        let input = Span::from("aabbcc");
        let (input, result) =
            take_line_until_one_of_three("bc", "bb", "cc")(input).unwrap();
        assert_eq!(input, "bbcc");
        assert_eq!(result, "aa");
    }

    #[test]
    fn take_until_end_of_line_or_input_should_return_all_input_up_to_newline() {
        let input = Span::from("aabbcc\nddeeff");
        let (input, result) = take_until_end_of_line_or_input(input).unwrap();
        assert_eq!(input, "\nddeeff");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_until_end_of_line_or_input_should_return_rest_of_input_if_no_newline(
    ) {
        let input = Span::from("aabbcc");
        let (input, result) = take_until_end_of_line_or_input(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(result, "aabbcc");
    }

    #[test]
    fn take_until_end_of_line_or_input_should_return_empty_if_start_is_newline()
    {
        let input = Span::from("\naabbcc");
        let (input, result) = take_until_end_of_line_or_input(input).unwrap();
        assert_eq!(input, "\naabbcc");
        assert_eq!(result, "");
    }

    #[test]
    fn take_until_should_consume_input_until_pattern_found() {
        let input = Span::from("aabbcc");
        let (input, result) = take_until("bb")(input).unwrap();
        assert_eq!(input, "bbcc");
        assert_eq!(result, "aa");
    }

    #[test]
    fn take_until_should_consume_input_across_newlines_if_pattern_found() {
        let input = Span::from("aabbcc\nddeeff");
        let (input, result) = take_until("ee")(input).unwrap();
        assert_eq!(input, "eeff");
        assert_eq!(result, "aabbcc\ndd");
    }

    #[test]
    fn take_until_should_fail_if_pattern_not_found() {
        let input = Span::from("aabbcc");
        assert!(take_until("zz")(input).is_err());
    }

    #[test]
    fn take_until_should_fail_if_only_start_of_pattern_found_at_end_of_input() {
        let input = Span::from("aabbcc");
        assert!(take_until("ccc")(input).is_err());
    }

    #[test]
    fn take_end_should_return_last_n_bytes_from_input() {
        let input = Span::from("aabbcc");
        let (input, result) = take_end(3usize)(input).unwrap();
        assert_eq!(input, "aab");
        assert_eq!(result, "bcc");
    }

    #[test]
    fn take_end_should_fail_if_not_enough_bytes_in_input() {
        let input = Span::from("aabbcc");
        assert!(take_end(7usize)(input).is_err());
    }
}
