use super::Description;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents the scheme associated with the external file link
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ExternalFileLinkScheme {
    Local,
    File,
    Absolute,
}

/// Represents an external link to some file or directory on the host system
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct ExternalFileLink {
    pub scheme: ExternalFileLinkScheme,
    pub path: PathBuf,
    pub description: Option<Description>,
}

impl ExternalFileLink {
    /// Creates new external file link with no description
    pub fn using_scheme_and_path(
        scheme: ExternalFileLinkScheme,
        path: PathBuf,
    ) -> Self {
        Self::new(scheme, path, None)
    }
}
