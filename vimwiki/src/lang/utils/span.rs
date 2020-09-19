use bytes::Bytes;
use memchr::Memchr;
use nom::{
    error::{ErrorKind, ParseError},
    AsBytes, Compare, CompareResult, Err, ExtendInto, FindSubstring, FindToken,
    IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset,
    ParseTo, Slice,
};
use std::{
    cmp::{self, Ordering},
    fmt::{Display, Formatter, Result as FmtResult},
    iter::{Enumerate, FromIterator},
    ops::{Range, RangeFrom, RangeFull, RangeTo},
    str::FromStr,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SpanSegments {
    segments: Arc<Vec<Range<usize>>>,
}

impl SpanSegments {
    pub fn new(mut segments: Vec<Range<usize>>) -> Self {
        // Ensures that our segments are in order by start and then by end
        // TODO: Is this needed, or can we assume it's sorted?
        segments.sort_unstable_by(|a, b| {
            if a.start < b.start {
                Ordering::Less
            } else if a.start > b.start {
                Ordering::Greater
            } else if a.end < b.end {
                Ordering::Less
            } else if a.end > b.end {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        Self {
            segments: Arc::new(segments),
        }
    }

    /// Converts a collection of bytes into a collection using only the
    /// segments contained by this `SpanSegments`
    pub fn convert_bytes_to_segments(&self, bytes: &Bytes) -> Bytes {
        Bytes::from_iter(
            self.segments
                .iter()
                .filter_map(|seg| {
                    // Only provide a slice if we are within bounds
                    if seg.start < bytes.len() {
                        // Ensure that the end of the range doesn't exceed
                        // the length of the byte sequence
                        let end = cmp::min(seg.end, bytes.len());
                        Some(bytes.slice(seg.start..end).into_iter())
                    } else {
                        None
                    }
                })
                .flatten(),
        )
    }

    /// Converts a local offset to a global offset
    pub fn map_local_to_global_offset(&self, offset: usize) -> usize {
        // |xxx|ooo|xxx|
        //  012 345 678
        //      012
        //
        //  Segments: 3..6
        //
        //  |x|oooo|x|oo|x|
        //   0 1234 5 67 8
        //     0123   45
        //
        //  Segments: 1..5, 6..8

        // While we are not in a segment, keep shifting towards the next
        // segment's starting point until we find ourselves inside a segment
        let mut last_end = 0;
        let mut global_offset = offset;
        for seg in self.segments.iter() {
            // Check if our baseline global offset, shifted over by the last
            // gap, is within the current segment
            let gap = seg.start - last_end;
            global_offset += gap;
            if seg.contains(&global_offset) {
                return global_offset;
            }

            // Otherwise, continue and flag the end of the last segment so
            // we can determine the next gap size
            last_end = seg.end;
        }

        // If we reach here, no segment matched, so just return the input
        offset
    }
}

/// Represents an input into our parsing constructs that keeps track of both
/// a global byte sequence and a local byte sequence. These can be the same,
/// but it's also possible to segment the global byte sequence into a local
/// one where only specific segments are included.
#[derive(Clone, Debug)]
pub struct Span {
    /// Represents a pointer to the global byte input (non-altered)
    global: Bytes,

    /// Represents a pointer to the local byte input (altered subset of global)
    local: Bytes,

    /// Represents segments from master that were kept in the fragment
    segments: SpanSegments,

    /// Offset and line location for fragment
    offset: usize,
    line: usize,

    /// Cached local utf8 column
    cached_local_utf8_column: Arc<Mutex<Option<usize>>>,

    /// Cached global offset
    cached_global_offset: Arc<Mutex<Option<usize>>>,

    /// Cached global line
    cached_global_line: Arc<Mutex<Option<usize>>>,

    /// Cached global utf8 column
    cached_global_utf8_column: Arc<Mutex<Option<usize>>>,
}

impl Span {
    fn new(global: Bytes, local: Bytes, segments: SpanSegments) -> Self {
        Self::new_at_pos(global, local, segments, 0, 1)
    }

    fn new_at_pos(
        global: Bytes,
        local: Bytes,
        segments: SpanSegments,
        offset: usize,
        line: usize,
    ) -> Self {
        Self {
            global,
            local,
            segments,
            offset,
            line,
            cached_local_utf8_column: Arc::new(Mutex::new(None)),
            cached_global_offset: Arc::new(Mutex::new(None)),
            cached_global_line: Arc::new(Mutex::new(None)),
            cached_global_utf8_column: Arc::new(Mutex::new(None)),
        }
    }

    /// Creates a new span from a static string; no new allocation is made
    /// to represent the string internally
    pub fn from_static(s: &'static str) -> Self {
        let global = Bytes::from_static(s.as_bytes());
        let local = global.clone();
        Self::new(global, local, Default::default())
    }

    /// Converts span into global span, translating the span's local offset
    /// into the global span's offset
    pub fn into_global(self) -> Self {
        let offset = self.global_offset();
        let line = self.global_line();
        let global = self.global;
        let local = global.slice(offset..);
        Self::new_at_pos(global, local, Default::default(), offset, line)
    }

    /// Retrieves the local offset number (base 0)
    pub fn local_offset(&self) -> usize {
        self.offset
    }

    /// Retrieves the local line number (base 1)
    pub fn local_line(&self) -> usize {
        self.line
    }

    /// Retrieves the local column number using code pointers since a UTF8
    /// character may span multiple bytes (base 1)
    pub fn local_utf8_column(&self) -> usize {
        *self
            .cached_local_utf8_column
            .lock()
            .unwrap()
            .get_or_insert_with(|| {
                // TODO: This won't work as is because local is shifting
                //       around and doesn't have a contiguous memory pointer
                //       (or does it?)
                Self::find_column(self.local.as_ref(), self.local_offset())
            })
    }

    /// Retrieves the global offset number (base 0)
    pub fn global_offset(&self) -> usize {
        self.segments
            .map_local_to_global_offset(self.local_offset())
    }

    /// Retrieves the global line number (base 1)
    pub fn global_line(&self) -> usize {
        *self
            .cached_global_line
            .lock()
            .unwrap()
            .get_or_insert_with(|| {
                Self::find_line(self.global.as_ref(), self.global_offset())
            })
    }

    /// Retrieves the global column number using code pointers since a UTF8
    /// character may span multiple bytes (base 1)
    pub fn global_utf8_column(&self) -> usize {
        *self
            .cached_global_utf8_column
            .lock()
            .unwrap()
            .get_or_insert_with(|| {
                Self::find_column(self.global.as_ref(), self.global_offset())
            })
    }

    /// Retrieves a reference to the current, local fragment
    pub fn fragment(&self) -> &[u8] {
        self.local.as_ref()
    }

    /// Assumes that the span has UTF-8 compliant bytes and converts to a str
    pub fn fragment_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.fragment()) }
    }

    /// Returns the length (in bytes) of the internal fragment contained by
    /// the span, which can be less than the original input if the fragment
    /// has been reduced to specific segments
    pub fn fragment_len(&self) -> usize {
        self.local.len()
    }

    /// Produces a new span comprised only of the provided segments, which
    /// will keep track of line/column positioning based on its
    /// original sequence
    ///
    /// NOTE: This will copy all segments into a new byte sequence, so this
    ///       is an expensive operation
    pub fn into_segments(self, segments: Vec<Range<usize>>) -> Self {
        let global = self.local;
        let segments = SpanSegments::new(segments);

        // Chain segments provided together, which will result in copying
        // the byte sequence into a new collection
        let local = segments.convert_bytes_to_segments(&global);

        Self::new(global, local, segments)
    }

    /// Produces a new span comprised of all bytes except those in the
    /// provided segments, which will keep track of line/column positioning
    /// based on its original sequence
    ///
    /// NOTE: This will copy all segments into a new byte sequence, so this
    ///       is an expensive operation
    pub fn without_segments(self, segments: Vec<Range<usize>>) -> Self {
        let mut segments_to_keep = Vec::new();

        // Build up segments to keep by identifying the range in front of
        // each segment to remove
        let mut start = 0;
        for segment in segments.iter() {
            if start < segment.start {
                segments_to_keep.push(start..segment.start);
            }
            start = segment.end;
        }

        // Finally, add one last segment at the end of the final segment to
        // remove, as long as we didn't remove the end of the byte sequence
        if start < self.global.len() {
            segments_to_keep.push(start..self.global.len());
        }

        self.into_segments(segments_to_keep)
    }

    /// Determines the line position (base index of 1) based on a series of
    /// bytes and an offset
    fn find_line(bytes: &[u8], offset: usize) -> usize {
        let before_offset = bytes.slice(..offset);
        let cnt = Memchr::new(b'\n', before_offset).count();
        cnt + 1
    }

    /// Determines the column position (base index of 1) based on a series of
    /// bytes and an offset
    fn find_column(bytes: &[u8], offset: usize) -> usize {
        let bytes = &bytes[..offset];
        let column = match memchr::memrchr(b'\n', bytes) {
            None => offset + 1,
            Some(pos) => offset - pos,
        };

        bytecount::num_chars(&bytes[offset - (column - 1)..]) + 1
    }
}

impl From<String> for Span {
    /// Converts to a Vec<u8> that gets translated into Bytes internally
    fn from(s: String) -> Self {
        let global = Bytes::from(s);
        let local = global.clone();
        Self::new(global, local, Default::default())
    }
}

impl From<&str> for Span {
    /// Allocates a new string based on the provided slice and stores it
    /// internally for usage with combinators
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl PartialEq for Span {
    /// Checks equality by comparing lines, offsets, and local fragments
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line
            && self.offset == other.offset
            && self.local == other.local
    }
}

impl Eq for Span {}

impl AsBytes for Span {
    fn as_bytes(&self) -> &[u8] {
        self.local.as_ref()
    }
}

impl Compare<Span> for Span {
    fn compare(&self, other: Span) -> CompareResult {
        self.fragment().compare(other.fragment())
    }

    fn compare_no_case(&self, other: Span) -> CompareResult {
        self.fragment().compare_no_case(other.fragment())
    }
}

impl Compare<&str> for Span {
    fn compare(&self, other: &str) -> CompareResult {
        self.fragment().compare(other)
    }

    fn compare_no_case(&self, other: &str) -> CompareResult {
        self.fragment().compare_no_case(other)
    }
}

impl Compare<&[u8]> for Span {
    fn compare(&self, other: &[u8]) -> CompareResult {
        self.fragment().compare(other)
    }

    fn compare_no_case(&self, other: &[u8]) -> CompareResult {
        self.fragment().compare_no_case(other)
    }
}

impl ExtendInto for Span {
    type Item = u8;
    type Extender = Vec<u8>;

    #[inline]
    fn new_builder(&self) -> Self::Extender {
        self.fragment().new_builder()
    }

    #[inline]
    fn extend_into(&self, acc: &mut Self::Extender) {
        self.fragment().extend_into(acc)
    }
}

impl<'a> FindSubstring<&'a [u8]> for Span {
    #[inline]
    fn find_substring(&self, substr: &'a [u8]) -> Option<usize> {
        self.fragment().find_substring(substr)
    }
}

impl<'a> FindSubstring<&'a str> for Span {
    #[inline]
    fn find_substring(&self, substr: &'a str) -> Option<usize> {
        self.fragment().find_substring(substr)
    }
}

impl FindToken<u8> for Span {
    fn find_token(&self, token: u8) -> bool {
        self.fragment().find_token(token)
    }
}

impl<'a> FindToken<&'a u8> for Span {
    fn find_token(&self, token: &'a u8) -> bool {
        self.fragment().find_token(token)
    }
}

impl FindToken<char> for Span {
    fn find_token(&self, token: char) -> bool {
        self.fragment().find_token(token)
    }
}

impl InputIter for Span {
    type Item = u8;
    type Iter = Enumerate<Self::IterElem>;
    type IterElem = bytes::buf::IntoIter<Bytes>;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.local.clone().into_iter()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.fragment().position(predicate)
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Option<usize> {
        self.fragment().slice_index(count)
    }
}

impl InputLength for Span {
    fn input_len(&self) -> usize {
        self.fragment().input_len()
    }
}

impl InputTake for Span
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

impl InputTakeAtPosition for Span
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
        match self.fragment().position(predicate) {
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
        match self.fragment().position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
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
        match self.fragment().position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            Some(n) => Ok(self.take_split(n)),
            None => {
                if self.fragment().input_len() == 0 {
                    Err(Err::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}

impl<R> ParseTo<R> for Span
where
    R: FromStr,
{
    #[inline]
    fn parse_to(&self) -> Option<R> {
        self.fragment().parse_to()
    }
}

impl Offset for Span {
    fn offset(&self, second: &Self) -> usize {
        let fst = self.offset;
        let snd = second.offset;

        snd - fst
    }
}

impl Display for Span {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str(unsafe { std::str::from_utf8_unchecked(self.fragment()) })
    }
}

macro_rules! impl_slice_range {
    ( $fragment_type:ty, $range_type:ty, $can_return_self:expr, $calc_consumed_len:expr ) => {
        impl Slice<$range_type> for Span {
            fn slice(&self, range: $range_type) -> Self {
                if $can_return_self(&range) {
                    return self.clone();
                }
                // CHIP CHIP CHIP
                // offset using nom trait impl that is getting and doing pointer
                // comparisons, which don't seem to work right with Bytes
                //
                // Thought is to provide another function to macro that takes
                // in the next_local and range to determine the consumed len
                //
                // Note that this only cares about consumed bytes from the
                // front as the offset and line would need to be recalculated
                let consumed_len = $calc_consumed_len(&range);
                let next_local = self.local.slice(range);
                if consumed_len == 0 {
                    return Span::new_at_pos(
                        self.global.clone(),
                        next_local,
                        self.segments.clone(),
                        self.offset,
                        self.line,
                    );
                }

                let consumed = self.local.slice(..consumed_len);
                let next_offset = self.offset + consumed_len;

                let consumed_as_bytes = consumed.as_bytes();
                let iter = Memchr::new(b'\n', consumed_as_bytes);
                let number_of_lines = iter.count();
                let next_line = self.line + number_of_lines;

                Span::new_at_pos(
                    self.global.clone(),
                    next_local,
                    self.segments.clone(),
                    next_offset,
                    next_line,
                )
            }
        }
    };
}

impl_slice_range! {
    &[u8],
    Range<usize>,
    |_| false,
    |r: &Range<usize>| r.start
}
impl_slice_range! {
    &[u8],
    RangeTo<usize>,
    |_| false,
    |_| 0
}
impl_slice_range! {
    &[u8],
    RangeFrom<usize>,
    |r: &RangeFrom<usize>| r.start == 0,
    |r: &RangeFrom<usize>| r.start
}
impl_slice_range! {
    &[u8],
    RangeFull,
    |_| true,
    |_| 0
}

#[cfg(test)]
mod tests {
    use super::*;

    mod span_segments {
        use super::*;

        fn btos(b: &[u8]) -> &str {
            unsafe { std::str::from_utf8_unchecked(b) }
        }

        #[test]
        fn convert_bytes_to_segments_should_empty_if_no_segments_provided() {
            let bytes = Bytes::from_static(b"some fragment");

            assert_eq!(
                btos(
                    SpanSegments::new(vec![])
                        .convert_bytes_to_segments(&bytes)
                        .as_ref()
                ),
                ""
            );
        }

        #[test]
        fn convert_bytes_to_segments_should_only_keep_bytes_in_segments() {
            let bytes = Bytes::from_static(b"some fragment");

            assert_eq!(
                btos(
                    SpanSegments::new(vec![0..4])
                        .convert_bytes_to_segments(&bytes)
                        .as_ref()
                ),
                "some"
            );

            assert_eq!(
                btos(
                    SpanSegments::new(vec![4..5])
                        .convert_bytes_to_segments(&bytes)
                        .as_ref()
                ),
                " "
            );

            assert_eq!(
                btos(
                    SpanSegments::new(vec![5..13])
                        .convert_bytes_to_segments(&bytes)
                        .as_ref()
                ),
                "fragment"
            );

            assert_eq!(
                btos(
                    SpanSegments::new(vec![1..2, 2..3, 3..4])
                        .convert_bytes_to_segments(&bytes)
                        .as_ref()
                ),
                "ome"
            );

            assert_eq!(
                btos(
                    SpanSegments::new(vec![0..2, 4..5, 8..11,])
                        .convert_bytes_to_segments(&bytes)
                        .as_ref()
                ),
                "so gme"
            );
        }

        #[test]
        fn convert_bytes_to_segments_should_be_okay_if_a_range_exceeds_length_of_input_fragment(
        ) {
            let bytes = Bytes::from_static(b"some fragment");

            assert_eq!(
                btos(
                    SpanSegments::new(vec![0..999])
                        .convert_bytes_to_segments(&bytes)
                        .as_ref()
                ),
                "some fragment"
            );

            assert_eq!(
                btos(
                    SpanSegments::new(vec![999..1000])
                        .convert_bytes_to_segments(&bytes)
                        .as_ref()
                ),
                ""
            );
        }

        #[test]
        fn map_local_to_global_offset_should_be_identity_if_one_comprehensive_segment(
        ) {
            let segments = SpanSegments::new(vec![0..5]);
            let mut actual = Vec::new();
            for i in 0..5 {
                actual.push(segments.map_local_to_global_offset(i));
            }

            assert_eq!(actual, vec![0, 1, 2, 3, 4]);
        }

        #[test]
        fn map_local_to_global_offset_should_support_gaps_in_middle() {
            let segments = SpanSegments::new(vec![0..2, 3..5]);
            let mut actual = Vec::new();
            for i in 0..4 {
                actual.push(segments.map_local_to_global_offset(i));
            }

            assert_eq!(actual, vec![0, 1, 3, 4]);
        }

        #[test]
        fn map_local_to_global_offset_should_support_starting_gap() {
            let segments = SpanSegments::new(vec![3..5]);
            let mut actual = Vec::new();
            for i in 0..2 {
                actual.push(segments.map_local_to_global_offset(i));
            }

            assert_eq!(actual, vec![3, 4]);
        }

        #[test]
        fn map_local_to_global_offset_should_support_contiguous_segments() {
            let segments = SpanSegments::new(vec![0..2, 2..4, 4..5]);
            let mut actual = Vec::new();
            for i in 0..5 {
                actual.push(segments.map_local_to_global_offset(i));
            }

            assert_eq!(actual, vec![0, 1, 2, 3, 4]);
        }
    }

    mod span {
        use super::*;

        mod nom_traits {
            use super::*;

            #[test]
            fn as_bytes_should_return_local_bytes_as_slice() {
                let span = Span::new(
                    Bytes::from_static(b"global"),
                    Bytes::from_static(b"local"),
                    Default::default(),
                );
                assert_eq!(span.fragment(), b"local");
            }

            #[test]
            fn compare_should_yield_ok_if_local_bytes_are_equal() {
                let span1 = Span::from_static("abcdef");
                let span2 = Span::from_static("abcdef");
                assert_eq!(span1.compare(span2), CompareResult::Ok);
                assert_eq!(span1.compare("abcdef"), CompareResult::Ok);
                assert_eq!(span1.compare(&b"abcdef"[..]), CompareResult::Ok);
            }

            #[test]
            fn compare_should_yield_error_if_local_bytes_are_not_equal() {
                let span1 = Span::from_static("abcdef");
                let span2 = Span::from_static("defabc");
                assert_eq!(span1.compare(span2), CompareResult::Error);
                assert_eq!(span1.compare("defabc"), CompareResult::Error);
                assert_eq!(span1.compare(&b"defabc"[..]), CompareResult::Error);
            }

            #[test]
            fn compare_should_yield_incomplete_if_local_bytes_length_is_smaller_than_other(
            ) {
                let span1 = Span::from_static("abc");
                let span2 = Span::from_static("abcdef");
                assert_eq!(span1.compare(span2), CompareResult::Incomplete);
                assert_eq!(span1.compare("abcdef"), CompareResult::Incomplete);
                assert_eq!(
                    span1.compare(&b"abcdef"[..]),
                    CompareResult::Incomplete
                );
            }

            #[test]
            fn compare_no_case_should_yield_ok_if_local_bytes_are_equal() {
                let span1 = Span::from_static("abcdef");
                let span2 = Span::from_static("AbCdEf");
                assert_eq!(span1.compare_no_case(span2), CompareResult::Ok);
                assert_eq!(span1.compare_no_case("AbCdEf"), CompareResult::Ok);
                assert_eq!(
                    span1.compare_no_case(&b"AbCdEf"[..]),
                    CompareResult::Ok
                );
            }

            #[test]
            fn compare_no_case_should_yield_error_if_local_bytes_are_not_equal()
            {
                let span1 = Span::from_static("abcdef");
                let span2 = Span::from_static("DeFaBc");
                assert_eq!(span1.compare_no_case(span2), CompareResult::Error);
                assert_eq!(
                    span1.compare_no_case("DeFaBc"),
                    CompareResult::Error
                );
                assert_eq!(
                    span1.compare_no_case(&b"DeFaBc"[..]),
                    CompareResult::Error
                );
            }

            #[test]
            fn compare_no_case_should_yield_incomplete_if_local_bytes_length_is_smaller_than_other(
            ) {
                let span1 = Span::from_static("abc");
                let span2 = Span::from_static("AbCdEf");
                assert_eq!(
                    span1.compare_no_case(span2),
                    CompareResult::Incomplete
                );
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
                let span1 = Span::from_static("abc");
                assert_eq!(span1.new_builder(), vec![]);
            }

            #[test]
            fn extend_into_should_copy_local_bytes_to_end_of_provided_byte_vec()
            {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc"),
                    Default::default(),
                );
                let mut acc = b"123".to_vec();
                span.extend_into(&mut acc);
                assert_eq!(acc, b"123abc");
                assert_eq!(span.fragment_str(), "abc");
            }

            #[test]
            fn find_substring_should_yield_none_if_unable_to_find_byte_string_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_substring(&b"cc"[..]), None);
            }

            #[test]
            fn find_substring_should_yield_position_of_first_byte_string_match_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_substring(&b"c1"[..]), Some(2));
            }

            #[test]
            fn find_substring_should_yield_none_if_unable_to_find_string_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_substring("cc"), None);
            }

            #[test]
            fn find_substring_should_yield_some_position_of_first_string_match()
            {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_substring("c1"), Some(2));
            }

            #[test]
            fn find_token_should_yield_true_if_byte_exists_in_local_bytes() {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_token(b'c'), true);
            }

            #[test]
            fn find_token_should_yield_false_if_byte_missing_in_local_bytes() {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_token(b'z'), false);
            }

            #[test]
            fn find_token_should_yield_true_if_byte_ref_exists_in_local_bytes()
            {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_token(&b'c'), true);
            }

            #[test]
            fn find_token_should_yield_false_if_byte_ref_missing_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_token(&b'z'), false);
            }

            #[test]
            fn find_token_should_yield_true_if_char_exists_in_local_bytes() {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_token('c'), true);
            }

            #[test]
            fn find_token_should_yield_false_if_char_missing_in_local_bytes() {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.find_token('z'), false);
            }

            #[test]
            fn iter_indicies_should_yield_an_iterator_of_local_index_and_byte_tuples(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(
                    span.iter_indices().collect::<Vec<_>>(),
                    vec![
                        (0, b'a'),
                        (1, b'b'),
                        (2, b'c'),
                        (3, b'1'),
                        (4, b'2'),
                        (5, b'3'),
                    ]
                );
            }

            #[test]
            fn iter_elements_should_yield_an_iterator_of_local_bytes() {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(
                    span.iter_elements().collect::<Vec<_>>(),
                    vec![b'a', b'b', b'c', b'1', b'2', b'3']
                );
            }

            #[test]
            fn position_should_yield_an_none_if_the_predicate_does_not_match_a_local_byte(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.position(|_| false), None);
            }

            #[test]
            fn position_should_yield_an_index_if_the_predicate_matches_a_local_byte(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.position(|b| b == b'c'), Some(2));
            }

            #[test]
            fn slice_index_should_yield_the_index_if_available_in_local_bytes()
            {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.slice_index(3), Some(3));
            }

            #[test]
            fn slice_index_should_yield_none_if_unavailable_in_local_bytes() {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.slice_index(7), None);
            }

            #[test]
            fn input_len_should_yield_the_byte_length_of_local_bytes() {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(span.input_len(), 6);
            }

            #[test]
            fn take_should_yield_a_span_that_has_the_first_n_local_bytes() {
                let span = Span::from_static("abc123");
                let span = span.take(3);
                assert_eq!(span.fragment_str(), "abc");
                assert_eq!(span.global.as_ref(), b"abc123");
            }

            #[test]
            fn take_split_should_yield_two_spans_the_first_is_local_bytes_after_n_and_second_is_local_bytes_up_to_n(
            ) {
                let span = Span::from_static("abc123");
                let (suffix, prefix) = span.take_split(2);

                assert_eq!(prefix.fragment_str(), "ab");
                assert_eq!(prefix.global.as_ref(), b"abc123");

                assert_eq!(suffix.fragment_str(), "c123");
                assert_eq!(suffix.global.as_ref(), b"abc123");
            }

            #[test]
            fn take_split_should_support_producing_an_empty_prefix_span() {
                let span = Span::from_static("abc123");
                let (suffix, prefix) = span.take_split(0);

                assert_eq!(prefix.fragment_str(), "");
                assert_eq!(prefix.global.as_ref(), b"abc123");

                assert_eq!(suffix.fragment_str(), "abc123");
                assert_eq!(suffix.global.as_ref(), b"abc123");
            }

            #[test]
            fn take_split_should_support_producing_an_empty_suffix_span() {
                let span = Span::from_static("abc123");
                let (suffix, prefix) = span.take_split(6);

                assert_eq!(prefix.fragment_str(), "abc123");
                assert_eq!(prefix.global.as_ref(), b"abc123");

                assert_eq!(suffix.fragment_str(), "");
                assert_eq!(suffix.global.as_ref(), b"abc123");
            }

            #[test]
            fn split_at_position_should_yield_incomplete_if_no_match_found_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(
                    span.split_at_position::<_, ()>(|_| false),
                    Err(nom::Err::Incomplete(nom::Needed::Size(1)))
                );
            }

            #[test]
            fn split_at_position_should_yield_local_bytes_up_to_the_first_match_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::from("abc123456def"),
                    Bytes::from("abc123"),
                    Default::default(),
                );

                let (suffix, prefix) =
                    span.split_at_position::<_, ()>(|b| b == b'c').unwrap();
                assert_eq!(prefix.fragment_str(), "ab");
                assert_eq!(prefix.global.as_ref(), b"abc123456def");

                assert_eq!(suffix.fragment_str(), "c123");
                assert_eq!(suffix.global.as_ref(), b"abc123456def");
            }

            #[test]
            fn split_at_position_should_support_an_empty_span_being_produced_from_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::from("abc123456def"),
                    Bytes::from("abc123"),
                    Default::default(),
                );

                let (suffix, prefix) =
                    span.split_at_position::<_, ()>(|b| b == b'a').unwrap();
                assert_eq!(prefix.fragment_str(), "");
                assert_eq!(prefix.global.as_ref(), b"abc123456def");

                assert_eq!(suffix.fragment_str(), "abc123");
                assert_eq!(suffix.global.as_ref(), b"abc123456def");
            }

            #[test]
            fn split_at_position1_should_yield_incomplete_if_no_match_found_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(
                    span.split_at_position1::<_, ()>(
                        |_| false,
                        ErrorKind::Alpha
                    ),
                    Err(nom::Err::Incomplete(nom::Needed::Size(1)))
                );
            }

            #[test]
            fn split_at_position1_should_yield_local_bytes_up_to_the_first_match_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::from("abc123456def"),
                    Bytes::from("abc123"),
                    Default::default(),
                );

                let (suffix, prefix) = span
                    .split_at_position1::<_, ()>(
                        |b| b == b'c',
                        ErrorKind::Alpha,
                    )
                    .unwrap();
                assert_eq!(prefix.fragment_str(), "ab");
                assert_eq!(prefix.global.as_ref(), b"abc123456def");

                assert_eq!(suffix.fragment_str(), "c123");
                assert_eq!(suffix.global.as_ref(), b"abc123456def");
            }

            #[test]
            fn split_at_position1_fail_if_an_empty_span_would_be_produced_from_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::from("abc123456def"),
                    Bytes::from("abc123"),
                    Default::default(),
                );

                assert_eq!(
                    span.split_at_position1::<_, (Span, ErrorKind)>(
                        |b| b == b'a',
                        ErrorKind::Alpha,
                    ),
                    Err(nom::Err::Error((span, ErrorKind::Alpha)))
                );
            }

            #[test]
            fn split_at_position_complete_should_yield_all_input_if_no_match_found_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(
                    span.split_at_position_complete::<_, ()>(|_| false),
                    Ok((
                        Span::new_at_pos(
                            Bytes::new(),
                            Bytes::new(),
                            Default::default(),
                            6,
                            1,
                        ),
                        span
                    ))
                );
            }

            #[test]
            fn split_at_position_complete_should_yield_local_bytes_up_to_the_first_match_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::from("abc123456def"),
                    Bytes::from("abc123"),
                    Default::default(),
                );

                let (suffix, prefix) = span
                    .split_at_position_complete::<_, ()>(|b| b == b'c')
                    .unwrap();
                assert_eq!(prefix.fragment_str(), "ab");
                assert_eq!(prefix.global.as_ref(), b"abc123456def");

                assert_eq!(suffix.fragment_str(), "c123");
                assert_eq!(suffix.global.as_ref(), b"abc123456def");
            }

            #[test]
            fn split_at_position_complete_should_support_an_empty_span_being_produced_from_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::from("abc123456def"),
                    Bytes::from("abc123"),
                    Default::default(),
                );

                let (suffix, prefix) = span
                    .split_at_position_complete::<_, ()>(|b| b == b'a')
                    .unwrap();
                assert_eq!(prefix.fragment_str(), "");
                assert_eq!(prefix.global.as_ref(), b"abc123456def");

                assert_eq!(suffix.fragment_str(), "abc123");
                assert_eq!(suffix.global.as_ref(), b"abc123456def");
            }

            #[test]
            fn split_at_position1_complete_should_yield_all_input_if_no_match_found_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("abc123"),
                    Default::default(),
                );
                assert_eq!(
                    span.split_at_position1_complete::<_, ()>(
                        |_| false,
                        ErrorKind::Alpha
                    ),
                    Ok((
                        Span::new_at_pos(
                            Bytes::new(),
                            Bytes::new(),
                            Default::default(),
                            6,
                            1,
                        ),
                        span
                    ))
                );
            }

            #[test]
            fn split_at_position1_complete_should_yield_local_bytes_up_to_the_first_match_in_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::from("abc123456def"),
                    Bytes::from("abc123"),
                    Default::default(),
                );

                let (suffix, prefix) = span
                    .split_at_position1_complete::<_, ()>(
                        |b| b == b'c',
                        ErrorKind::Alpha,
                    )
                    .unwrap();
                assert_eq!(prefix.fragment_str(), "ab");
                assert_eq!(prefix.global.as_ref(), b"abc123456def");

                assert_eq!(suffix.fragment_str(), "c123");
                assert_eq!(suffix.global.as_ref(), b"abc123456def");
            }

            #[test]
            fn split_at_position1_complete_fail_if_an_empty_span_would_be_produced_from_local_bytes(
            ) {
                let span = Span::new(
                    Bytes::from("abc123456def"),
                    Bytes::from("abc123"),
                    Default::default(),
                );

                assert_eq!(
                    span.split_at_position1_complete::<_, (Span, ErrorKind)>(
                        |b| b == b'a',
                        ErrorKind::Alpha,
                    ),
                    Err(nom::Err::Error((span, ErrorKind::Alpha)))
                );
            }

            #[test]
            fn offset_should_yield_zero_if_at_same_offset() {
                let span1 = Span::from_static("abc123");
                let mut span2 = span1.clone();
                span2.offset = 3;

                assert_eq!(span1.offset(&span2), 3);
            }

            #[test]
            fn offset_should_yield_offset_between_first_local_byte_of_self_with_local_byte_of_other(
            ) {
                let span1 = Span::from_static("abc123");
                let span2 = span1.clone();

                assert_eq!(span1.offset(&span2), 0);
            }

            #[test]
            #[should_panic]
            fn offset_should_panic_if_would_yield_negative_value() {
                let span1 = Span::from_static("abc123");
                let mut span2 = span1.clone();
                span2.offset = 3;

                span2.offset(&span1);
            }

            #[test]
            fn parse_to_should_convert_local_bytes_to_str_and_then_apply_parse()
            {
                let span = Span::new(
                    Bytes::new(),
                    Bytes::from("123"),
                    Default::default(),
                );
                let result: u32 = span.parse_to().unwrap();
                assert_eq!(result, 123);
            }

            #[test]
            fn parse_to_should_yield_none_if_failing_to_parse() {
                let span = Span::new(
                    Bytes::from("123"),
                    Bytes::from("abc"),
                    Default::default(),
                );
                let result: Option<u32> = span.parse_to();
                assert_eq!(result, None);
            }

            #[test]
            fn slice_should_yield_a_clone_of_span_if_given_full_range() {
                let span1 = Span::new_at_pos(
                    Bytes::from("abc\ndef\nghi"),
                    Bytes::from("123\n456\n789"),
                    SpanSegments::new(vec![1..3]),
                    5,
                    2,
                );
                let span2 = span1.slice(..);
                assert_eq!(span2.global, span1.global);
                assert_eq!(span2.local, span1.local);
                assert_eq!(span2.segments, span1.segments);
                assert_eq!(span2.offset, span1.offset);
                assert_eq!(span2.line, span1.line);
            }

            #[test]
            fn slice_should_support_yielding_an_empty_span() {
                let span1 = Span::new_at_pos(
                    Bytes::from("abc\ndef\nghi"),
                    Bytes::from("123\n456\n789"),
                    SpanSegments::new(vec![1..3]),
                    5,
                    2,
                );

                let span2 = span1.slice(0..0);
                assert_eq!(span2.global, span1.global);
                assert_eq!(span2.local, Bytes::new());
                assert_eq!(span2.segments, span1.segments);
                assert_eq!(span2.offset, span1.offset);
                assert_eq!(span2.line, span1.line);

                let span2 = span1.slice(11..);
                assert_eq!(span2.global, span1.global);
                assert_eq!(span2.local, Bytes::new());
                assert_eq!(span2.segments, span1.segments);
                assert_eq!(span2.offset, span1.offset + 11);
                assert_eq!(span2.line, span1.line + 2);

                let span2 = span1.slice(..0);
                assert_eq!(span2.global, span1.global);
                assert_eq!(span2.local, Bytes::new());
                assert_eq!(span2.segments, span1.segments);
                assert_eq!(span2.offset, span1.offset);
                assert_eq!(span2.line, span1.line);
            }

            #[test]
            fn slice_should_yield_same_offset_and_line_with_new_local_bytes_if_offset_equal_given_range(
            ) {
                let span1 = Span::new_at_pos(
                    Bytes::from("abc\ndef\nghi"),
                    Bytes::from("123\n456\n789"),
                    SpanSegments::new(vec![1..3]),
                    5,
                    2,
                );
                let span2 = span1.slice(0..2);
                assert_eq!(span2.global, span1.global);
                assert_eq!(span2.local, Bytes::from("12"));
                assert_eq!(span2.segments, span1.segments);
                assert_eq!(span2.offset, span1.offset);
                assert_eq!(span2.line, span1.line);
            }

            #[test]
            fn slice_should_yield_new_line_and_offset_alongside_new_local_bytes_if_offset_different_given_range(
            ) {
                let span1 = Span::new_at_pos(
                    Bytes::from("abc\ndef\nghi"),
                    Bytes::from("123\n456\n789"),
                    SpanSegments::new(vec![1..3]),
                    5,
                    2,
                );
                let span2 = span1.slice(5..10);
                assert_eq!(span2.global, span1.global);
                assert_eq!(span2.local, Bytes::from("56\n78"));
                assert_eq!(span2.segments, span1.segments);
                assert_eq!(span2.offset, span1.offset + 5);
                assert_eq!(span2.line, span1.line + 1);
            }

            #[test]
            fn slice_should_yield_same_offset_and_line_with_new_local_bytes_if_given_range_to(
            ) {
                let span1 = Span::new_at_pos(
                    Bytes::from("abc\ndef\nghi"),
                    Bytes::from("123\n456\n789"),
                    SpanSegments::new(vec![1..3]),
                    5,
                    2,
                );
                let span2 = span1.slice(..10);
                assert_eq!(span2.global, span1.global);
                assert_eq!(span2.local, Bytes::from("123\n456\n78"));
                assert_eq!(span2.segments, span1.segments);
                assert_eq!(span2.offset, span1.offset);
                assert_eq!(span2.line, span1.line);
            }

            #[test]
            fn slice_should_yield_same_offset_and_line_with_new_local_bytes_if_offset_equal_given_range_from(
            ) {
                let span1 = Span::new_at_pos(
                    Bytes::from("abc\ndef\nghi"),
                    Bytes::from("123\n456\n789"),
                    SpanSegments::new(vec![1..3]),
                    5,
                    2,
                );
                let span2 = span1.slice(0..);
                assert_eq!(span2.global, span1.global);
                assert_eq!(span2.local, Bytes::from("123\n456\n789"));
                assert_eq!(span2.segments, span1.segments);
                assert_eq!(span2.offset, span1.offset);
                assert_eq!(span2.line, span1.line);
            }

            #[test]
            fn slice_should_yield_new_line_and_offset_alongside_new_local_bytes_if_offset_different_given_range_from(
            ) {
                let span1 = Span::new_at_pos(
                    Bytes::from("abc\ndef\nghi"),
                    Bytes::from("123\n456\n789"),
                    SpanSegments::new(vec![1..3]),
                    5,
                    2,
                );
                let span2 = span1.slice(5..);
                assert_eq!(span2.global, span1.global);
                assert_eq!(span2.local, Bytes::from("56\n789"));
                assert_eq!(span2.segments, span1.segments);
                assert_eq!(span2.offset, span1.offset + 5);
                assert_eq!(span2.line, span1.line + 1);
            }
        }

        #[test]
        fn global_line_and_utf8_column_should_properly_translate_across_segments(
        ) {
            let input = Span::from_static("line1\nline2\nline3")
                .into_segments(vec![2..4, 8..13, 15..16]);

            // line1|line2|line3
            // xxooxxxxoooooxxox
            // 0 2 4   8    13
            //                15
            //                 16
            assert_eq!(input.fragment_str(), "nene2\nle");

            // Calculate the line and UTF8 column of each byte position
            let mut lines_and_columns = Vec::new();
            for i in 0..input.fragment_len() {
                let pos = input.slice(i..);
                lines_and_columns
                    .push((pos.global_line(), pos.global_utf8_column()));
            }

            // Lines & columns are using base of 1 and 1
            assert_eq!(
                lines_and_columns,
                vec![
                    (1, 3), // n    master offset: 2
                    (1, 4), // e    master offset: 2
                    (2, 3), // n    master offset: 6
                    (2, 4), // e    master offset: 6
                    (2, 5), // 2    master offset: 6
                    (2, 6), // \n   master offset: 6
                    (3, 1), // l    master offset: 6
                    (3, 4), // e    master offset: 8
                ]
            );
        }
    }
}
