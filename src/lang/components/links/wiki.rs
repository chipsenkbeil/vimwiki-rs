use super::{WithAnchor, WithDescription};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a link to a file or directory in the active wiki
#[derive(
    Constructor,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct WikiLink {
    path: PathBuf,
    description: Option<String>,
    anchor: Option<String>,
}

impl WikiLink {
    /// Whether or not the link is representing an anchor to the current page
    pub fn is_local_anchor(&self) -> bool {
        self.path.as_os_str().is_empty() && self.anchor().is_some()
    }
}

impl From<PathBuf> for WikiLink {
    fn from(path: PathBuf) -> Self {
        Self::new(path, None, None)
    }
}

impl From<String> for WikiLink {
    fn from(str_path: String) -> Self {
        Self::from(PathBuf::from(str_path))
    }
}

impl WithDescription for WikiLink {
    fn with_description(&mut self, description: String) -> &mut Self {
        self.description = Some(description);
        self
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl WithAnchor for WikiLink {
    fn with_anchor(&mut self, anchor: String) -> &mut Self {
        self.anchor = Some(anchor);
        self
    }

    fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }
}
