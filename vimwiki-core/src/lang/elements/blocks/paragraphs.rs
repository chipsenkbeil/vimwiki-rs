use crate::{
    lang::elements::{
        InlineElement, InlineElementContainer, IntoChildren, Located,
    },
    StrictEq,
};
use derive_more::{Constructor, Index, IndexMut, IntoIterator};
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;

#[derive(
    Constructor,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Index,
    IndexMut,
    IntoIterator,
    Serialize,
    Deserialize,
)]
pub struct Paragraph<'a> {
    /// Represents the lines of content contained within the paragraph
    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    pub lines: Vec<InlineElementContainer<'a>>,
}

impl<'a> Paragraph<'a> {
    /// Returns true if the paragraph only contains blank lines (or has no
    /// lines at all)
    pub fn is_blank(&self) -> bool {
        self.nonblank_lines().count() == 0
    }

    /// Returns an iterator over all lines that are not blank, meaning that
    /// they contain non-comment inline elements
    pub fn nonblank_lines(
        &self,
    ) -> impl Iterator<Item = &InlineElementContainer<'a>> {
        self.into_iter().filter(|line| {
            line.into_iter()
                .any(|e| !matches!(e.as_inner(), InlineElement::Comment(_)))
        })
    }
}

impl Paragraph<'_> {
    pub fn to_borrowed(&self) -> Paragraph {
        Paragraph::new(self.into_iter().map(|x| x.to_borrowed()).collect())
    }

    pub fn into_owned(self) -> Paragraph<'static> {
        Paragraph::new(self.into_iter().map(|x| x.into_owned()).collect())
    }
}

impl<'a> IntoChildren for Paragraph<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.into_iter().flat_map(|x| x.into_children()).collect()
    }
}

impl<'a> FromIterator<InlineElementContainer<'a>> for Paragraph<'a> {
    fn from_iter<I: IntoIterator<Item = InlineElementContainer<'a>>>(
        iter: I,
    ) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<'a> StrictEq for Paragraph<'a> {
    /// Performs strict_eq on content
    fn strict_eq(&self, other: &Self) -> bool {
        self.lines.strict_eq(&other.lines)
    }
}
