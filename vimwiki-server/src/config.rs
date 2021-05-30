use crate::{utils, Opt};
use serde::{Deserialize, Serialize};
use std::{
    io,
    path::{Component, PathBuf},
};

/// Represents a config file that can be loaded and used by the server
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Contains configs for individual wikis
    #[serde(default)]
    pub wikis: Vec<WikiConfig>,
}

impl Config {
    /// Loads config using provided options
    pub fn load(opt: &Opt) -> io::Result<Config> {
        utils::load_config(opt.config.as_deref(), opt.merge)
    }
}

/// Represents a config associated with a singular wiki
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiConfig {
    /// Path to the wiki on the local machine (must be absolute path)
    #[serde(
        default = "WikiConfig::default_path",
        deserialize_with = "utils::deserialize_absolute_path"
    )]
    pub path: PathBuf,

    /// Optional name to associate with the wiki for named links and other
    /// use cases
    #[serde(default = "WikiConfig::default_name")]
    pub name: Option<String>,

    /// Path for diary directory relative to this wiki's path
    #[serde(default = "WikiConfig::default_diary_rel_path")]
    pub diary_rel_path: PathBuf,

    /// File extension for files within a wiki to load and parse
    #[serde(default = "WikiConfig::default_ext")]
    pub ext: String,
}

impl Default for WikiConfig {
    fn default() -> Self {
        Self {
            path: Self::default_path(),
            name: Self::default_name(),
            diary_rel_path: Self::default_diary_rel_path(),
            ext: Self::default_ext(),
        }
    }
}

impl WikiConfig {
    #[inline]
    pub fn default_path() -> PathBuf {
        directories::UserDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .unwrap_or_else(|| {
                let mut path = PathBuf::new();
                path.push(Component::RootDir);
                path
            })
            .join("vimwiki")
    }

    #[inline]
    pub const fn default_name() -> Option<String> {
        None
    }

    #[inline]
    pub fn default_diary_rel_path() -> PathBuf {
        PathBuf::from("diary")
    }

    #[inline]
    pub fn default_ext() -> String {
        String::from("wiki")
    }
}
