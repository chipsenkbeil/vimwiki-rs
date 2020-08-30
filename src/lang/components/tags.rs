use derive_more::Constructor;
use serde::{Deserialize, Serialize};

/// Represents a sequence of one or more tags
///
/// In vimwiki, :my-tag: would become
///
/// TagSequence([ Tag(my-tag) ])
///
/// Similarly, :my-tag-1:my-tag-2: would become
///
/// TagSequence([ Tag(my-tag-1), Tag(my-tag-2) ])
///
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct TagSequence(pub Vec<Tag>);

/// Represents a single tag
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Tag(pub String);
