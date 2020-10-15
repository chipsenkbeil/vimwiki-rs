use memchr::{memchr2, memchr_iter, memrchr};
use nom::{
    error::{ErrorKind, ParseError},
    AsBytes, Compare, CompareResult, Err, ExtendInto, FindSubstring, FindToken,
    IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset,
    ParseTo, Slice,
};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    iter::Enumerate,
    ops::{Range, RangeFrom, RangeFull, RangeTo},
    str::FromStr,
};

/// Represents a span across some input, which is passed around to various
/// parser combinators to examine and process
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span<'a> {
    inner: &'a [u8],
    start: usize,
    end: usize,
}

impl<'a> Span<'a> {
    /// Creates a new span with the provided byte slice, start offset relative
    /// to the provided byte slice, and end offset (exclusive) relative to the
    /// provided byte slice
    pub fn new(inner: &'a [u8], start: usize, end: usize) -> Self {
        Self { inner, start, end }
    }

    /// Creates a copy of the span starting at the new offset relative to
    /// its existing offset. If start exceeds end, then start will be set
    /// to end.
    ///
    /// e.g. start = 2, end = 4, starting_at(1) yields start = 3
    pub fn starting_at(&self, start: usize) -> Self {
        let start = self.start + start;
        let end = self.end;
        Self::new(self.inner, if start > end { end } else { start }, end)
    }

    /// Creates a copy of the span ending at the new offset (exclusive)
    /// relative to its existing offset.
    ///
    /// e.g. start = 2, end = 4, ending_at(1) yields end = 3
    pub fn ending_at(&self, end: usize) -> Self {
        Self::new(self.inner, self.start, self.start + end)
    }

    /// Creates a copy of the span whose ending offset is adjusted to fit
    /// the desired length.
    ///
    /// The span cannot grow (even if the inner has space); therefore, a length
    /// greater than the current remaining len will do nothing.
    pub fn with_length(&self, len: usize) -> Self {
        if len < self.remaining_len() {
            Self::new(self.inner, self.start, self.start + len)
        } else {
            *self
        }
    }

    /// Represents the inner byte slice starting from the original span
    /// (offset not applied)
    pub fn as_inner(&self) -> &[u8] {
        self.inner
    }

    /// Represents the inner byte slice as a str
    ///
    /// This will have undefined behavior if the inner bytes are not UTF-8
    pub fn as_unsafe_inner_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_inner()) }
    }

    /// Represents the starting offset of the span relative to the
    /// inner byte slice (inclusive)
    pub fn start_offset(&self) -> usize {
        self.start
    }

    /// Represents the ending offset of the span relative to the
    /// inner byte slice (exclusive)
    pub fn end_offset(&self) -> usize {
        self.end
    }

    /// Represents the consumed bytes from the start of the input
    /// (everything up to but not including the offset)
    pub fn as_consumed(&self) -> &[u8] {
        &self.inner[..self.start]
    }

    /// Represents the consumed input as a str
    ///
    /// This will have undefined behavior if the consumed bytes are not UTF-8
    pub fn as_unsafe_consumed_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_consumed()) }
    }

    /// Represents the total number of bytes consumed from the input thus far;
    /// this is equivalent to the start offset
    pub fn consumed_len(&self) -> usize {
        self.start
    }

    /// Represents the remaining bytes of the input, starting at the offset
    pub fn as_remaining(&self) -> &[u8] {
        &self.inner[self.start..self.end]
    }

    /// Represents the remaining input as a str
    ///
    /// This will have undefined behavior if the remaining bytes are not UTF-8
    pub fn as_unsafe_remaining_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_remaining()) }
    }

    /// Represents the total number of bytes remaining from the input
    pub fn remaining_len(&self) -> usize {
        if self.start <= self.end {
            self.end - self.start
        } else {
            0
        }
    }

    /// Whether or not there is more input remaining
    pub fn is_empty(&self) -> bool {
        self.remaining_len() == 0
    }

    /// Whether or not the remaining bytes are comprised of only spaces
    /// or tabs
    pub fn is_only_whitespace(&self) -> bool {
        memchr2(b' ', b'\t', self.as_remaining()).is_some()
    }

    /// Calculates the line position of this span using newline (\n) chars
    pub fn line(&self) -> usize {
        // Count the number of newline (\n) characters that take place before
        // our current position; increment by 1 since our first line is 1, not 0
        // memchr_iter(b'\n', self.inner)
        //     .take_while(|pos| pos < &self.start)
        //     .count()
        //     + 1
        1
    }

    /// Calculates the column position of this span by looking backwards from
    /// the offset for the last newline and then counting code points
    pub fn column(&self) -> usize {
        // Determine the offset position that represents the start of the line,
        // which is just after a newline or the beginning of the entire inner
        // slice if we are on the first line
        // let start_of_line = memrchr(b'\n', self.as_consumed())
        //     .map(|pos| pos + 1)
        //     .unwrap_or_default();

        // Get a slice for the line starting at the beginning
        // let line_up_to_offset = &self.inner[start_of_line..self.start];

        // Count the codepoints thus far and increment by 1 since our first
        // column is 1, not 0 (meaning if we are within the first code point,
        // we are in column 1)
        // bytecount::num_chars(line_up_to_offset) + 1
        1
    }
}

impl<'a> Display for Span<'a> {
    /// Displays the span's inner byte slice as a UTF-8 str starting from the
    /// span's offset, or if the byte slice is not a UTF-8 str will display
    /// nothing
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        if let Ok(s) = std::str::from_utf8(self.as_remaining()) {
            fmt.write_str(s)?;
        }

        Ok(())
    }
}

/*****************************************************************************/
/* BEGIN EQUALITY HELPERS                                                    */
/*****************************************************************************/

impl<'a> PartialEq<&'a str> for Span<'a> {
    /// Tests whether the bytes represented by this span equal the given str
    fn eq(&self, other: &&'a str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<'a> PartialEq<&'a [u8]> for Span<'a> {
    /// Tests whether the bytes represented by this span equal the given bytes
    fn eq(&self, other: &&'a [u8]) -> bool {
        self.as_bytes() == *other
    }
}

/*****************************************************************************/
/* BEGIN ITERATOR IMPL                                                       */
/*****************************************************************************/

pub struct SpanIterator<'a> {
    span: Span<'a>,
}

impl<'a> IntoIterator for Span<'a> {
    type Item = &'a u8;
    type IntoIter = SpanIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SpanIterator { span: self }
    }
}

impl<'a> Iterator for SpanIterator<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        let span = &mut self.span;
        if span.start < span.end && span.start < span.inner.len() {
            let item = &span.inner[span.start];
            span.start += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for SpanIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let span = &mut self.span;
        if span.start < span.end && span.end < span.inner.len() {
            let item = &span.inner[span.end];
            span.end -= 1;
            Some(item)
        } else {
            None
        }
    }
}

/*****************************************************************************/
/* BEGIN CONVERSION HELPERS                                                  */
/*****************************************************************************/

impl<'a> From<&'a [u8]> for Span<'a> {
    fn from(inner: &'a [u8]) -> Self {
        Self::new(inner, 0, inner.len())
    }
}

impl<'a> From<&'a str> for Span<'a> {
    fn from(inner: &'a str) -> Self {
        Self::from(inner.as_bytes())
    }
}

macro_rules! impl_from_fixed_size_byte_array {
    ($len:expr) => {
        impl<'a> From<&'a [u8; $len]> for Span<'a> {
            fn from(inner: &'a [u8; $len]) -> Self {
                let inner_slice: &[u8] = inner;
                Self::from(inner_slice)
            }
        }
    };
}

impl_from_fixed_size_byte_array!(1);
impl_from_fixed_size_byte_array!(2);
impl_from_fixed_size_byte_array!(3);
impl_from_fixed_size_byte_array!(4);
impl_from_fixed_size_byte_array!(5);
impl_from_fixed_size_byte_array!(6);
impl_from_fixed_size_byte_array!(7);
impl_from_fixed_size_byte_array!(8);
impl_from_fixed_size_byte_array!(9);
impl_from_fixed_size_byte_array!(10);
impl_from_fixed_size_byte_array!(11);
impl_from_fixed_size_byte_array!(12);
impl_from_fixed_size_byte_array!(13);
impl_from_fixed_size_byte_array!(14);
impl_from_fixed_size_byte_array!(15);
impl_from_fixed_size_byte_array!(16);
impl_from_fixed_size_byte_array!(17);
impl_from_fixed_size_byte_array!(18);
impl_from_fixed_size_byte_array!(19);
impl_from_fixed_size_byte_array!(20);
impl_from_fixed_size_byte_array!(21);
impl_from_fixed_size_byte_array!(22);
impl_from_fixed_size_byte_array!(23);
impl_from_fixed_size_byte_array!(24);
impl_from_fixed_size_byte_array!(25);
impl_from_fixed_size_byte_array!(26);

/*****************************************************************************/
/* BEGIN NOM TRAITS IMPLEMENTATION                                           */
/*****************************************************************************/

impl<'a> AsBytes for Span<'a> {
    fn as_bytes(&self) -> &[u8] {
        self.as_remaining()
    }
}

impl<'a, 'b> Compare<Span<'b>> for Span<'a> {
    fn compare(&self, other: Span<'b>) -> CompareResult {
        self.as_bytes().compare(other.as_bytes())
    }

    fn compare_no_case(&self, other: Span<'b>) -> CompareResult {
        self.as_bytes().compare_no_case(other.as_bytes())
    }
}

impl<'a, 'b> Compare<&'b str> for Span<'a> {
    fn compare(&self, other: &'b str) -> CompareResult {
        self.as_bytes().compare(other.as_bytes())
    }

    fn compare_no_case(&self, other: &'b str) -> CompareResult {
        self.as_bytes().compare_no_case(other.as_bytes())
    }
}

impl<'a, 'b> Compare<&'b [u8]> for Span<'a> {
    fn compare(&self, other: &'b [u8]) -> CompareResult {
        self.as_bytes().compare(other)
    }

    fn compare_no_case(&self, other: &'b [u8]) -> CompareResult {
        self.as_bytes().compare_no_case(other)
    }
}

impl<'a> ExtendInto for Span<'a> {
    type Item = u8;
    type Extender = Vec<u8>;

    #[inline]
    fn new_builder(&self) -> Self::Extender {
        self.as_bytes().new_builder()
    }

    #[inline]
    fn extend_into(&self, acc: &mut Self::Extender) {
        self.as_bytes().extend_into(acc)
    }
}

impl<'a, 'b> FindSubstring<&'b [u8]> for Span<'a> {
    #[inline]
    fn find_substring(&self, substr: &'b [u8]) -> Option<usize> {
        self.as_bytes().find_substring(substr)
    }
}

impl<'a, 'b> FindSubstring<&'b str> for Span<'a> {
    #[inline]
    fn find_substring(&self, substr: &'b str) -> Option<usize> {
        self.as_bytes().find_substring(substr)
    }
}

impl<'a> FindToken<u8> for Span<'a> {
    fn find_token(&self, token: u8) -> bool {
        self.as_bytes().find_token(token)
    }
}

impl<'a, 'b> FindToken<&'b u8> for Span<'a> {
    fn find_token(&self, token: &'b u8) -> bool {
        self.as_bytes().find_token(token)
    }
}

impl<'a> FindToken<char> for Span<'a> {
    fn find_token(&self, token: char) -> bool {
        self.as_bytes().find_token(token)
    }
}

impl<'a> InputIter for Span<'a> {
    type Item = u8;
    type Iter = Enumerate<Self::IterElem>;
    type IterElem = std::iter::Copied<SpanIterator<'a>>;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.clone().into_iter().copied()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.as_bytes().position(predicate)
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Option<usize> {
        self.as_bytes().slice_index(count)
    }
}

impl<'a> InputLength for Span<'a> {
    fn input_len(&self) -> usize {
        self.as_bytes().input_len()
    }
}

impl<'a> InputTake for Span<'a>
where
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
{
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl<'a> InputTakeAtPosition for Span<'a>
where
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>> + Clone,
{
    type Item = <Self as InputIter>::Item;

    fn split_at_position_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.split_at_position(predicate) {
            Err(Err::Incomplete(_)) => Ok(self.take_split(self.input_len())),
            res => res,
        }
    }

    fn split_at_position<P, E: ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.as_bytes().position(predicate) {
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::Size(1))),
        }
    }

    fn split_at_position1<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.as_bytes().position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(*self, e))),
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::Size(1))),
        }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.as_bytes().position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(*self, e))),
            Some(n) => Ok(self.take_split(n)),
            None => {
                if self.as_bytes().input_len() == 0 {
                    Err(Err::Error(E::from_error_kind(*self, e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}

impl<'a, R> ParseTo<R> for Span<'a>
where
    R: FromStr,
{
    #[inline]
    fn parse_to(&self) -> Option<R> {
        self.as_bytes().parse_to()
    }
}

impl<'a> Offset for Span<'a> {
    fn offset(&self, second: &Self) -> usize {
        let fst = self.start;
        let snd = second.start;

        snd - fst
    }
}

impl<'a> Slice<Range<usize>> for Span<'a> {
    fn slice(&self, range: Range<usize>) -> Self {
        self.ending_at(range.end).starting_at(range.start)
    }
}

impl<'a> Slice<RangeTo<usize>> for Span<'a> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        self.ending_at(range.end)
    }
}

impl<'a> Slice<RangeFrom<usize>> for Span<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.starting_at(range.start)
    }
}

impl<'a> Slice<RangeFull> for Span<'a> {
    fn slice(&self, _range: RangeFull) -> Self {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod nom_traits {
        use super::*;

        #[test]
        fn as_bytes_should_return_remaining_bytes_as_slice() {
            let span = Span::from(b"all bytes").starting_at(4);
            assert_eq!(span.as_bytes(), b"bytes");
        }

        #[test]
        fn compare_should_yield_ok_if_remaining_bytes_are_equal() {
            let span1 = Span::from(b"abcdef").starting_at(4);
            let span2 = Span::from(b"abcdef").starting_at(4);
            assert_eq!(span1.compare(span2), CompareResult::Ok);
            assert_eq!(span1.compare("ef"), CompareResult::Ok);
            assert_eq!(span1.compare(&b"ef"[..]), CompareResult::Ok);
        }

        #[test]
        fn compare_should_yield_error_if_remaining_bytes_are_not_equal() {
            let span1 = Span::from(b"abcdef").starting_at(4);
            let span2 = Span::from(b"defabc").starting_at(4);
            assert_eq!(span1.compare(span2), CompareResult::Error);
            assert_eq!(span1.compare("defabc"), CompareResult::Error);
            assert_eq!(span1.compare(&b"defabc"[..]), CompareResult::Error);
        }

        #[test]
        fn compare_should_yield_incomplete_if_remaining_bytes_length_is_smaller_than_other(
        ) {
            let span1 = Span::from(b"abcdef").with_length(2);
            let span2 = Span::from(b"abcdef").with_length(4);
            assert_eq!(span1.compare(span2), CompareResult::Incomplete);
            assert_eq!(span1.compare("abcdef"), CompareResult::Incomplete);
            assert_eq!(
                span1.compare(&b"abcdef"[..]),
                CompareResult::Incomplete
            );
        }

        #[test]
        fn compare_no_case_should_yield_ok_if_remaining_bytes_are_equal() {
            let span1 = Span::from("abcdef").starting_at(4);
            let span2 = Span::from("AbCdEf").starting_at(4);
            assert_eq!(span1.compare_no_case(span2), CompareResult::Ok);
            assert_eq!(span1.compare_no_case("Ef"), CompareResult::Ok);
            assert_eq!(span1.compare_no_case(&b"Ef"[..]), CompareResult::Ok);
        }

        #[test]
        fn compare_no_case_should_yield_error_if_remaining_bytes_are_not_equal()
        {
            let span1 = Span::from("abcdef").starting_at(4);
            let span2 = Span::from("DeFaBc").starting_at(4);
            assert_eq!(span1.compare_no_case(span2), CompareResult::Error);
            assert_eq!(span1.compare_no_case("Bc"), CompareResult::Error);
            assert_eq!(span1.compare_no_case(&b"Bc"[..]), CompareResult::Error);
        }

        #[test]
        fn compare_no_case_should_yield_incomplete_if_remaining_bytes_length_is_smaller_than_other(
        ) {
            let span1 = Span::from("abcdef").with_length(2);
            let span2 = Span::from("AbCdEf").with_length(4);
            assert_eq!(span1.compare_no_case(span2), CompareResult::Incomplete);
            assert_eq!(
                span1.compare_no_case("AbCdEf"),
                CompareResult::Incomplete
            );
            assert_eq!(
                span1.compare_no_case(&b"AbCdEf"[..]),
                CompareResult::Incomplete
            );
        }

        #[test]
        fn new_builder_should_create_an_empty_byte_vec() {
            let span1 = Span::from("abc");
            assert_eq!(span1.new_builder(), vec![]);
        }

        #[test]
        fn extend_into_should_copy_remaining_bytes_to_end_of_provided_byte_vec()
        {
            let span = Span::from(b"abcdef").starting_at(3);
            let mut acc = b"123".to_vec();
            span.extend_into(&mut acc);
            assert_eq!(acc, b"123def");
            assert_eq!(span.as_bytes(), b"def");
        }

        #[test]
        fn find_substring_should_yield_none_if_unable_to_find_byte_string_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(4);
            assert_eq!(span.find_substring(&b"c1"[..]), None);
        }

        #[test]
        fn find_substring_should_yield_position_of_first_byte_string_match_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(span.find_substring(&b"c1"[..]), Some(0));
        }

        #[test]
        fn find_substring_should_yield_none_if_unable_to_find_string_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(4);
            assert_eq!(span.find_substring("c1"), None);
        }

        #[test]
        fn find_substring_should_yield_some_position_of_first_string_match() {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(span.find_substring("c1"), Some(0));
        }

        #[test]
        fn find_token_should_yield_true_if_byte_exists_in_remaining_bytes() {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(span.find_token(b'c'), true);
        }

        #[test]
        fn find_token_should_yield_false_if_byte_missing_in_remaining_bytes() {
            let span = Span::from(b"abc123").starting_at(4);
            assert_eq!(span.find_token(b'c'), false);
        }

        #[test]
        fn find_token_should_yield_true_if_byte_ref_exists_in_remaining_bytes()
        {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(span.find_token(&b'c'), true);
        }

        #[test]
        fn find_token_should_yield_false_if_byte_ref_missing_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(4);
            assert_eq!(span.find_token(&b'c'), false);
        }

        #[test]
        fn find_token_should_yield_true_if_char_exists_in_remaining_bytes() {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(span.find_token('c'), true);
        }

        #[test]
        fn find_token_should_yield_false_if_char_missing_in_remaining_bytes() {
            let span = Span::from(b"abc123").starting_at(4);
            assert_eq!(span.find_token('c'), false);
        }

        #[test]
        fn iter_indicies_should_yield_an_iterator_of_remaining_index_and_byte_tuples(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(
                span.iter_indices().collect::<Vec<_>>(),
                vec![(0, b'c'), (1, b'1'), (2, b'2'), (3, b'3')]
            );
        }

        #[test]
        fn iter_elements_should_yield_an_iterator_of_remaining_bytes() {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(
                span.iter_elements().collect::<Vec<_>>(),
                vec![b'c', b'1', b'2', b'3']
            );
        }

        #[test]
        fn position_should_yield_an_none_if_the_predicate_does_not_match_a_remaining_byte(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(span.position(|_| false), None);
        }

        #[test]
        fn position_should_yield_an_index_if_the_predicate_matches_a_remaining_byte(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(span.position(|b| b == b'c'), Some(0));
        }

        #[test]
        fn slice_index_should_yield_the_index_if_available_in_remaining_bytes()
        {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(span.slice_index(3), Some(3));
        }

        #[test]
        fn slice_index_should_yield_none_if_unavailable_in_remaining_bytes() {
            let span = Span::from(b"abc123").starting_at(3);
            assert_eq!(span.slice_index(4), None);
        }

        #[test]
        fn input_len_should_yield_the_byte_length_of_remaining_bytes() {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(span.input_len(), 4);
        }

        #[test]
        fn take_should_yield_a_span_that_has_the_first_n_remaining_bytes() {
            let span = Span::from(b"abc123").starting_at(2);
            let span = span.take(3);
            assert_eq!(span.as_bytes(), b"c12");
        }

        #[test]
        fn take_split_should_yield_two_spans_the_first_is_remaining_bytes_after_n_and_second_is_remaining_bytes_up_to_n(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            let (suffix, prefix) = span.take_split(2);
            assert_eq!(prefix.as_bytes(), b"c1");
            assert_eq!(suffix.as_bytes(), b"23");
        }

        #[test]
        fn take_split_should_support_producing_an_empty_prefix_span() {
            let span = Span::from(b"abc123").starting_at(2);
            let (suffix, prefix) = span.take_split(0);
            assert_eq!(prefix.as_bytes(), b"");
            assert_eq!(suffix.as_bytes(), b"c123");
        }

        #[test]
        fn take_split_should_support_producing_an_empty_suffix_span() {
            let span = Span::from(b"abc123").starting_at(2);
            let (suffix, prefix) = span.take_split(4);
            assert_eq!(prefix.as_bytes(), b"c123");
            assert_eq!(suffix.as_bytes(), b"");
        }

        #[test]
        fn split_at_position_should_yield_incomplete_if_no_match_found_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(
                span.split_at_position::<_, ()>(|_| false),
                Err(nom::Err::Incomplete(nom::Needed::Size(1)))
            );
        }

        #[test]
        fn split_at_position_should_yield_remaining_bytes_up_to_the_first_match_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            let (suffix, prefix) =
                span.split_at_position::<_, ()>(|b| b == b'2').unwrap();
            assert_eq!(prefix.as_bytes(), b"c1");
            assert_eq!(suffix.as_bytes(), b"23");
        }

        #[test]
        fn split_at_position_should_support_an_empty_span_being_produced_from_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            let (suffix, prefix) =
                span.split_at_position::<_, ()>(|b| b == b'c').unwrap();
            assert_eq!(prefix.as_bytes(), b"");
            assert_eq!(suffix.as_bytes(), b"c123");
        }

        #[test]
        fn split_at_position1_should_yield_incomplete_if_no_match_found_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(
                span.split_at_position1::<_, ()>(|_| false, ErrorKind::Alpha),
                Err(nom::Err::Incomplete(nom::Needed::Size(1)))
            );
        }

        #[test]
        fn split_at_position1_should_yield_remaining_bytes_up_to_the_first_match_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            let (suffix, prefix) = span
                .split_at_position1::<_, ()>(|b| b == b'2', ErrorKind::Alpha)
                .unwrap();
            assert_eq!(prefix.as_bytes(), b"c1");
            assert_eq!(suffix.as_bytes(), b"23");
        }

        #[test]
        fn split_at_position1_fail_if_an_empty_span_would_be_produced_from_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(
                span.split_at_position1::<_, (Span, ErrorKind)>(
                    |b| b == b'c',
                    ErrorKind::Alpha,
                ),
                Err(nom::Err::Error((span, ErrorKind::Alpha)))
            );
        }

        #[test]
        fn split_at_position_complete_should_yield_all_input_if_no_match_found_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(
                span.split_at_position_complete::<_, ()>(|_| false),
                Ok((Span::from(b"abc123").starting_at(6), span))
            );
        }

        #[test]
        fn split_at_position_complete_should_yield_remaining_bytes_up_to_the_first_match_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            let (suffix, prefix) = span
                .split_at_position_complete::<_, ()>(|b| b == b'2')
                .unwrap();
            assert_eq!(prefix.as_bytes(), b"c1");
            assert_eq!(suffix.as_bytes(), b"23");
        }

        #[test]
        fn split_at_position_complete_should_support_an_empty_span_being_produced_from_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            let (suffix, prefix) = span
                .split_at_position_complete::<_, ()>(|b| b == b'c')
                .unwrap();
            assert_eq!(prefix.as_bytes(), b"");
            assert_eq!(suffix.as_bytes(), b"c123");
        }

        #[test]
        fn split_at_position1_complete_should_yield_all_input_if_no_match_found_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(
                span.split_at_position1_complete::<_, ()>(
                    |_| false,
                    ErrorKind::Alpha
                ),
                Ok((Span::from(b"abc123").starting_at(6), span))
            );
        }

        #[test]
        fn split_at_position1_complete_should_yield_remaining_bytes_up_to_the_first_match_in_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            let (suffix, prefix) = span
                .split_at_position1_complete::<_, ()>(
                    |b| b == b'2',
                    ErrorKind::Alpha,
                )
                .unwrap();
            assert_eq!(prefix.as_bytes(), b"c1");
            assert_eq!(suffix.as_bytes(), b"23");
        }

        #[test]
        fn split_at_position1_complete_fail_if_an_empty_span_would_be_produced_from_remaining_bytes(
        ) {
            let span = Span::from(b"abc123").starting_at(2);
            assert_eq!(
                span.split_at_position1_complete::<_, (Span, ErrorKind)>(
                    |b| b == b'c',
                    ErrorKind::Alpha,
                ),
                Err(nom::Err::Error((span, ErrorKind::Alpha)))
            );
        }

        #[test]
        fn offset_should_yield_offset_between_first_remaining_byte_of_self_with_remaining_byte_of_other(
        ) {
            let span1 = Span::from(b"abc123").starting_at(2);
            let span2 = Span::from(b"abc123").starting_at(3);

            assert_eq!(span1.offset(&span2), 1);
        }

        #[test]
        fn offset_should_yield_zero_if_at_same_offset() {
            let span1 = Span::from("abc123").starting_at(2);
            let span2 = Span::from("abc123").starting_at(2);

            assert_eq!(span1.offset(&span2), 0);
        }

        #[test]
        #[should_panic]
        fn offset_should_panic_if_would_yield_negative_value() {
            let span1 = Span::from("abc123").starting_at(2);
            let span2 = Span::from(b"abc123").starting_at(3);

            span2.offset(&span1);
        }

        #[test]
        fn parse_to_should_convert_remaining_bytes_to_str_and_then_apply_parse()
        {
            let span = Span::from(b"abc123").starting_at(3);
            let result: u32 = span.parse_to().unwrap();
            assert_eq!(result, 123);
        }

        #[test]
        fn parse_to_should_yield_none_if_failing_to_parse() {
            let span = Span::from(b"abc123").starting_at(2);
            let result: Option<u32> = span.parse_to();
            assert_eq!(result, None);
        }

        #[test]
        fn slice_should_yield_a_clone_of_span_if_given_full_range() {
            let span1 = Span::from(b"abc123").starting_at(2);
            let span2 = span1.slice(..);
            assert_eq!(span1, span2);
        }

        #[test]
        fn slice_should_support_yielding_an_empty_span() {
            let span1 = Span::from(b"abc123").starting_at(2);

            let span2 = span1.slice(0..0);
            assert_eq!(span2.start_offset(), 2);
            assert_eq!(span2.end_offset(), 2);
            assert_eq!(span2.as_bytes(), b"");

            let span2 = span1.slice(11..);
            assert_eq!(span2.start_offset(), 6);
            assert_eq!(span2.end_offset(), 6);
            assert_eq!(span2.as_bytes(), b"");

            let span2 = span1.slice(..0);
            assert_eq!(span2.start_offset(), 2);
            assert_eq!(span2.end_offset(), 2);
            assert_eq!(span2.as_bytes(), b"");
        }

        #[test]
        fn slice_with_range_should_adjust_start_and_end_offsets_accordingly() {
            let span1 = Span::from(b"abc123").starting_at(2);
            let span2 = span1.slice(0..2);
        }
    }
}
