use derive_more::{
    Constructor, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator,
};
use serde::{Deserialize, Serialize};

/// Represents a sequence of one or more tags
///
/// In vimwiki, :my-tag: would become
///
/// Tags([ Tag(my-tag) ])
///
/// Similarly, :my-tag-1:my-tag-2: would become
///
/// Tags([ Tag(my-tag-1), Tag(my-tag-2) ])
///
#[derive(
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    From,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct Tags(pub Vec<Tag>);

impl From<Tag> for Tags {
    fn from(tag: Tag) -> Self {
        Self::new(vec![tag])
    }
}

impl From<String> for Tags {
    fn from(s: String) -> Self {
        Self::from(Tag::new(s))
    }
}

impl From<&str> for Tags {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

/// Represents a single tag
#[derive(
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    From,
    Into,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct Tag(pub String);

impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}
