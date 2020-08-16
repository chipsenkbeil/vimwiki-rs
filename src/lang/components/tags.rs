use derive_more::From;
use serde::{Deserialize, Serialize};

/// Represents a sequence of one or more tags
///
/// In vimwiki, :my-tag: would become
///
///     TagSequence([ Tag(my-tag) ])
///
/// Similarly, :my-tag-1:my-tag-2: would become
///
///     TagSequence([ Tag(my-tag-1), Tag(my-tag-2) ])
///
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub struct TagSequence(Vec<Tag>);

/// Represents a single tag
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tag(String);
