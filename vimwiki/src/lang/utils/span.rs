use nom::Slice;
use nom_locate::LocatedSpan;
use std::ops::Range;

/// Represents a span that has not been altered with skippable regions
pub type MasterSpan<'a> = LocatedSpan<&'a str>;

/// Represents a span that is spawned from a master span (removing skippable regions)
pub type Span<'a> = LocatedSpan<&'a str, SpanFactory<'a>>;

/// Convenience function to produce a new span that has a span factory with
/// an master span whose fragment is identical to that of the span
pub fn new_span(input: &str) -> Span {
    Span::new_extra(input, SpanFactory::from(input))
}

/// Represents a producer of spans based on a non-altered span (master) and
/// a collection of ranges that are skippable (to be removed in produced spans)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SpanFactory<'a> {
    /// Represents the master span to compare against sub-spans where
    /// skippable regions have been removed
    ///
    /// This span should be at line 1, offset 0
    master: MasterSpan<'a>,

    /// Calculated shortened fragment; as this requires a new allocation, we
    /// store the fragment on the heap and use a reference counter s
    pub shortened_fragment: &'a str,

    /// Represents the offset ranges within the destination span that should
    /// be missing from other spans (skippable)
    ///
    /// NOTE: Assumes that ranges are sorted by starting position and are not
    ///       overlapping
    pub skippable_ranges: &'a [Range<usize>],
}

impl<'a> SpanFactory<'a> {
    pub fn new(
        input: &'a str,
        shortened_fragment: &'a str,
        skippable_ranges: &'a [Range<usize>],
    ) -> Self {
        Self {
            master: MasterSpan::new(input),
            shortened_fragment,
            skippable_ranges,
        }
    }

    /// Produces a new span where skippable ranges have been removed
    ///
    /// NOTE: This will allocate an entirely new internal slice for the span
    pub fn make_span(&'a self) -> Span<'a> {
        Span::new_extra(&self.shortened_fragment, *self)
    }

    /// Produces a copy of the underlying master span
    pub fn as_master(&self) -> &MasterSpan<'a> {
        &self.master
    }

    /// Retrieves the line and column (assuming 1 byte = 1 column) of the
    /// master span based on the given input span
    pub fn master_line_and_column(&self, input: Span) -> (u32, usize) {
        let master = self.to_master_at_input(input);
        (master.location_line(), master.get_column())
    }

    /// Retrieves the line and utf8 column based on the given input span
    pub fn master_line_and_utf8_column(&self, input: Span) -> (u32, usize) {
        let master = self.to_master_at_input(input);
        (master.location_line(), master.get_utf8_column())
    }

    /// Retrieves the line and utf8 column (using naive method that may be
    /// better for sub-100 column lines) based on the given input span
    pub fn master_line_and_naive_utf8_column(
        &self,
        input: Span,
    ) -> (u32, usize) {
        let master = self.to_master_at_input(input);
        (master.location_line(), master.naive_get_utf8_column())
    }

    /// Produces a copy of the underlying master span starting at the same
    /// associated position as the input span (relative to skippable ranges)
    fn to_master_at_input(&self, input: Span) -> MasterSpan {
        let offset = self.master_offset(input);
        self.master.slice(offset..)
    }

    /// Determines the offset of the master span based on the input span
    fn master_offset(&self, input: Span) -> usize {
        let base_offset = input.location_offset();
        let mut offset = base_offset;

        // Assuming that our origin had all skippable regions removed, we can
        // just increment the offset by all of the leading range lengths
        for r in self.skippable_ranges {
            if r.start <= base_offset {
                // Range is not inclusive at end, so [2, 3) == len(1)
                offset += r.end - r.start;
            } else {
                break;
            }
        }

        offset
    }

    /// Produces a new fragment with skippable ranges removed
    ///
    /// TODO: Figure out if there is a way to do this without a new allocation
    pub fn shorten_fragment(
        fragment: &'a str,
        skippable_ranges: &'a [Range<usize>],
    ) -> String {
        // Gather all pieces of a fragment that are not skippable
        let mut fragment_pieces: Vec<&str> = Vec::new();

        let mut start = 0;
        for r in skippable_ranges {
            // Check if our start position of a non-skippable fragment
            // is before a skippable section, if so, grab our start to
            // just before the start of a skippable fragment
            if start < r.start {
                fragment_pieces.push(&fragment[start..r.start]);
            }

            // Update our new start to the end of the skippable range, which
            // should be the start of the next non-skippable range as our
            // ranges are non-inclusive of the end
            start = r.end;

            // If our start is now beyond our master fragment, exit early
            if start >= fragment.len() {
                break;
            }
        }

        // Add remaining fragment
        fragment_pieces.push(&fragment[start..]);

        fragment_pieces.concat()
    }
}

impl<'a> From<&'a str> for SpanFactory<'a> {
    fn from(input: &'a str) -> Self {
        Self::new(input, input, Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorten_fragment_should_return_input_fragment_if_no_ranges_provided() {
        todo!();
    }

    #[test]
    fn shorten_fragment_should_remove_segments_within_input_fragment() {
        todo!();
    }

    #[test]
    fn shorten_fragment_should_be_okay_if_a_range_exceeds_length_of_input_fragment(
    ) {
        todo!();
    }
}