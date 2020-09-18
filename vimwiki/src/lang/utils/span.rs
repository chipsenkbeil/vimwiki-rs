use bytes::Bytes;
use memchr::Memchr;
use nom::{
    error::{ErrorKind, ParseError},
    AsBytes, Compare, CompareResult, Err, ExtendInto, FindSubstring, FindToken,
    IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset,
    ParseTo, Slice,
};
use std::{
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
    pub fn new(segments: Vec<Range<usize>>) -> Self {
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
                .map(|seg| bytes.slice(seg.start..seg.end).into_iter())
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
                break;
            }

            // Otherwise, continue and flag the end of the last segment so
            // we can determine the next gap size
            last_end = seg.end;
        }

        global_offset
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
    line: u32,

    /// Cached local utf8 column
    cached_local_utf8_column: Arc<Mutex<Option<usize>>>,

    /// Cached global offset
    cached_global_offset: Arc<Mutex<Option<usize>>>,

    /// Cached global line
    cached_global_line: Arc<Mutex<Option<u32>>>,

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
        line: u32,
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
    pub fn local_line(&self) -> u32 {
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
                let before_local = Self::get_columns_and_bytes_before_offset(
                    self.local.as_ref(),
                    self.local_offset(),
                )
                .1;
                bytecount::num_chars(before_local) + 1
            })
    }

    /// Retrieves the global offset number (base 0)
    pub fn global_offset(&self) -> usize {
        self.segments
            .map_local_to_global_offset(self.local_offset())
    }

    /// Retrieves the global line number (base 1)
    pub fn global_line(&self) -> u32 {
        *self
            .cached_global_line
            .lock()
            .unwrap()
            .get_or_insert_with(|| {
                let offset = self.global_offset();
                self.slice(offset..).local_line()
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
                let before_global = Self::get_columns_and_bytes_before_offset(
                    self.global.as_ref(),
                    self.global_offset(),
                )
                .1;
                bytecount::num_chars(before_global) + 1
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

    /// Retrieves the byte column location (index 1) and series of bytes
    /// prior to the current position within a byte slice
    fn get_columns_and_bytes_before_offset(
        bytes: &[u8],
        offset: usize,
    ) -> (usize, &[u8]) {
        let column = match memchr::memrchr(b'\n', bytes) {
            None => offset + 1,
            Some(pos) => offset - pos,
        };

        (column, &bytes[offset - (column - 1)..])
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
    ( $fragment_type:ty, $range_type:ty, $can_return_self:expr ) => {
        impl Slice<$range_type> for Span {
            fn slice(&self, range: $range_type) -> Self {
                if $can_return_self(&range) {
                    return self.clone();
                }
                let next_local = self.local.slice(range);
                let consumed_len = self.local.offset(&next_local);
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
                let number_of_lines = iter.count() as u32;
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

impl_slice_range! {&[u8], Range<usize>, |_| false}
impl_slice_range! {&[u8], RangeTo<usize>, |_| false}
impl_slice_range! {&[u8], RangeFrom<usize>, |range:&RangeFrom<usize>| range.start == 0}
impl_slice_range! {&[u8], RangeFull, |_| true}

#[cfg(test)]
mod tests {
    use super::*;

    mod span_segments {
        use super::*;

        #[test]
        fn convert_bytes_to_segments_should_empty_if_no_segments_provided() {
            let bytes = Bytes::from_static(b"some fragment");

            assert_eq!(
                SpanSegments::new(vec![])
                    .convert_bytes_to_segments(&bytes)
                    .as_ref(),
                b"some"
            );
        }

        #[test]
        fn convert_bytes_to_segments_should_only_keep_bytes_in_segments() {
            let bytes = Bytes::from_static(b"some fragment");

            assert_eq!(
                SpanSegments::new(vec![0..5])
                    .convert_bytes_to_segments(&bytes)
                    .as_ref(),
                b"some"
            );

            assert_eq!(
                SpanSegments::new(vec![4..5])
                    .convert_bytes_to_segments(&bytes)
                    .as_ref(),
                b" "
            );

            assert_eq!(
                SpanSegments::new(vec![4..13])
                    .convert_bytes_to_segments(&bytes)
                    .as_ref(),
                b"fragment"
            );

            assert_eq!(
                SpanSegments::new(vec![1..2, 2..3, 3..4])
                    .convert_bytes_to_segments(&bytes)
                    .as_ref(),
                b"ome"
            );

            assert_eq!(
                SpanSegments::new(vec![0..2, 4..5, 8..11,])
                    .convert_bytes_to_segments(&bytes)
                    .as_ref(),
                b"so gme"
            );
        }

        #[test]
        fn convert_bytes_to_segments_should_be_okay_if_a_range_exceeds_length_of_input_fragment(
        ) {
            let bytes = Bytes::from_static(b"some fragment");

            assert_eq!(
                SpanSegments::new(vec![0..999])
                    .convert_bytes_to_segments(&bytes)
                    .as_ref(),
                b"some fragment"
            );

            assert_eq!(
                SpanSegments::new(vec![999..1000])
                    .convert_bytes_to_segments(&bytes)
                    .as_ref(),
                b""
            );
        }
    }

    mod span {
        use super::*;

        #[test]
        fn global_line_and_utf8_column_should_properly_translate_across_segments(
        ) {
            let input = Span::from_static("line1\nline2\nline3")
                .into_segments(vec![0..2, 4..8, 13..15, 16..17]);

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
