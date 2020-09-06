use super::WikiLink;
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

/// Represents a link to a file or directory in another wiki
#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum InterWikiLink {
    Indexed(IndexedInterWikiLink),
    Named(NamedInterWikiLink),
}

impl InterWikiLink {
    pub fn link(&self) -> &WikiLink {
        match self {
            Self::Indexed(x) => &x.link,
            Self::Named(x) => &x.link,
        }
    }
}

/// Represents a link to a file or directory in another wiki specified by
/// an index that maps to the g:vimwiki_list
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct IndexedInterWikiLink {
    pub index: u32,
    pub link: WikiLink,
}

impl From<(u32, WikiLink)> for IndexedInterWikiLink {
    fn from((index, link): (u32, WikiLink)) -> Self {
        Self::new(index, link)
    }
}

/// Represents a link to a file or directory in another wiki specified by
/// a name that maps to the name key in g:vimwiki_list
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct NamedInterWikiLink {
    pub name: String,
    pub link: WikiLink,
}

impl From<(String, WikiLink)> for NamedInterWikiLink {
    fn from((name, link): (String, WikiLink)) -> Self {
        Self::new(name, link)
    }
}
