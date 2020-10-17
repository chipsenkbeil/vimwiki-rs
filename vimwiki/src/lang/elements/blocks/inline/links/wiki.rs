use super::{Anchor, Description};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fmt,
    path::{Path, PathBuf},
};

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
pub struct WikiLink<'a> {
    pub path: Cow<'a, Path>,
    pub description: Option<Description<'a>>,
    pub anchor: Option<Anchor<'a>>,
}

impl WikiLink<'_> {
    pub fn to_borrowed(&self) -> WikiLink {
        use self::Cow::*;

        let path = Cow::Borrowed(match &self.path {
            Borrowed(x) => *x,
            Owned(x) => x.as_path(),
        });
        let description = self.description.map(|x| x.to_borrowed());
        let anchor = self.anchor.map(|x| x.to_borrowed());

        WikiLink {
            path,
            description,
            anchor,
        }
    }

    pub fn into_owned(self) -> WikiLink<'static> {
        let path = Cow::from(self.path.into_owned());
        let description = self.description.map(|x| x.into_owned());
        let anchor = self.anchor.map(|x| x.into_owned());

        WikiLink {
            path,
            description,
            anchor,
        }
    }
}

impl<'a> WikiLink<'a> {
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

impl<'a> fmt::Display for WikiLink<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = self.description.as_ref() {
            write!(f, "{}", desc)
        } else {
            write!(f, "{}", self.path.to_string_lossy())?;
            if let Some(anchor) = self.anchor.as_ref() {
                write!(f, "{}", anchor)?;
            }
            Ok(())
        }
    }
}

impl From<PathBuf> for WikiLink<'static> {
    fn from(path: PathBuf) -> Self {
        Self::new(Cow::from(path), None, None)
    }
}

impl<'a> From<&'a Path> for WikiLink<'a> {
    fn from(path: &'a Path) -> Self {
        Self::new(Cow::from(path), None, None)
    }
}

impl From<String> for WikiLink<'static> {
    fn from(str_path: String) -> Self {
        Self::from(PathBuf::from(str_path))
    }
}

impl<'a> From<&'a str> for WikiLink<'a> {
    fn from(str_path: &'a str) -> Self {
        Self::from(Path::new(str_path))
    }
}
