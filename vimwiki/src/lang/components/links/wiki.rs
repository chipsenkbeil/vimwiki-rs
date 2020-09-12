use super::{Anchor, Description};
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
    pub path: PathBuf,
    pub description: Option<Description>,
    pub anchor: Option<Anchor>,
}

impl WikiLink {
    /// Whether or not the link is representing an anchor to the current page
    pub fn is_local_anchor(&self) -> bool {
        self.path.as_os_str().is_empty() && self.anchor.is_some()
    }

    /// Checks if the link's path is to a directory without actually evaluating
    /// in the filesystem. Only checks if the path appears as that of a
    /// directory
    pub fn is_path_dir(&self) -> bool {
        self.path
            .to_string_lossy()
            .chars()
            .last()
            .map(std::path::is_separator)
            .unwrap_or_default()
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