use super::Description;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents the scheme associated with the external link
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ExternalLinkScheme {
    Local,
    File,
    Absolute,
}

/// Represents an external link to some file or directory on the host system
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct ExternalLink {
    pub scheme: ExternalLinkScheme,
    pub path: PathBuf,
    pub description: Option<Description>,
}

impl ExternalLink {
    /// Creates new external link with no description
    pub fn using_scheme_and_path(
        scheme: ExternalLinkScheme,
        path: PathBuf,
    ) -> Self {
        Self::new(scheme, path, None)
    }
}
