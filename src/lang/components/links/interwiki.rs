use super::{WikiLink, WithAnchor, WithDescription};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

/// Represents a link to a file or directory in another wiki
#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum InterWikiLink {
    Indexed(IndexedInterWikiLink),
    Named(NamedInterWikiLink),
}

/// Represents a link to a file or directory in another wiki specified by
/// an index that maps to the g:vimwiki_list
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct IndexedInterWikiLink {
    index: u32,
    link: WikiLink,
}

impl IndexedInterWikiLink {
    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn link(&self) -> &WikiLink {
        &self.link
    }
}

impl WithDescription for IndexedInterWikiLink {
    fn with_description(&mut self, description: String) -> &mut Self {
        self.link.with_description(description);
        self
    }

    fn description(&self) -> Option<&str> {
        self.link.description()
    }
}

impl WithAnchor for IndexedInterWikiLink {
    fn with_anchor(&mut self, anchor: String) -> &mut Self {
        self.link.with_anchor(anchor);
        self
    }

    fn anchor(&self) -> Option<&str> {
        self.link.anchor()
    }
}

/// Represents a link to a file or directory in another wiki specified by
/// a name that maps to the name key in g:vimwiki_list
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct NamedInterWikiLink {
    name: String,
    link: WikiLink,
}

impl NamedInterWikiLink {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn link(&self) -> &WikiLink {
        &self.link
    }
}

impl WithDescription for NamedInterWikiLink {
    fn with_description(&mut self, description: String) -> &mut Self {
        self.link.with_description(description);
        self
    }

    fn description(&self) -> Option<&str> {
        self.link.description()
    }
}

impl WithAnchor for NamedInterWikiLink {
    fn with_anchor(&mut self, anchor: String) -> &mut Self {
        self.link.with_anchor(anchor);
        self
    }

    fn anchor(&self) -> Option<&str> {
        self.link.anchor()
    }
}
