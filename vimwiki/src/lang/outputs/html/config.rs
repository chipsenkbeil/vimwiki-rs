use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents configuration properties for HTML writing that are separate from
/// the running state during HTML conversion
#[derive(Default, Serialize, Deserialize)]
pub struct HtmlConfig {
    #[serde(default)]
    pub list: HtmlListConfig,
    #[serde(default)]
    pub text: HtmlTextConfig,
    #[serde(default)]
    pub header: HtmlHeaderConfig,
    #[serde(default)]
    pub code: HtmlCodeConfig,
}

/// Represents configuration options related to lists
#[derive(Serialize, Deserialize)]
pub struct HtmlListConfig {
    /// If true, newlines are ignored when producing lists, otherwise the
    /// line breaks are respected and <br /> is added for each line break in
    /// a list
    #[serde(default = "HtmlListConfig::default_ignore_newline")]
    pub ignore_newline: bool,
}

impl Default for HtmlListConfig {
    fn default() -> Self {
        Self {
            ignore_newline: Self::default_ignore_newline(),
        }
    }
}

impl HtmlListConfig {
    #[inline]
    pub fn default_ignore_newline() -> bool {
        true
    }
}

/// Represents configuration options related to text
#[derive(Serialize, Deserialize)]
pub struct HtmlTextConfig {
    /// If true, newlines are ignored when producing paragraphs, otherwise the
    /// line breaks are respected and <br /> is added for each line break in
    /// a paragraph
    #[serde(default = "HtmlTextConfig::default_ignore_newline")]
    pub ignore_newline: bool,
}

impl Default for HtmlTextConfig {
    fn default() -> Self {
        Self {
            ignore_newline: Self::default_ignore_newline(),
        }
    }
}

impl HtmlTextConfig {
    #[inline]
    pub fn default_ignore_newline() -> bool {
        true
    }
}

/// Represents configuration options related to headers
#[derive(Serialize, Deserialize)]
pub struct HtmlHeaderConfig {
    /// Represents the text that a header could have to be marked as the ToC
    #[serde(default = "HtmlHeaderConfig::default_table_of_contents")]
    pub table_of_contents: String,
}

impl Default for HtmlHeaderConfig {
    fn default() -> Self {
        Self {
            table_of_contents: Self::default_table_of_contents(),
        }
    }
}

impl HtmlHeaderConfig {
    #[inline]
    pub fn default_table_of_contents() -> String {
        String::from("Contents")
    }
}

/// Represents configuration options related to code
#[derive(Serialize, Deserialize)]
pub struct HtmlCodeConfig {
    /// Represents the built-in theme to be used for syntax highlighting when
    /// being performed server-side instead of client-side
    #[serde(default = "HtmlCodeConfig::default_theme")]
    pub theme: String,

    /// Represents the directory containing `.tmTheme` theme files to be used
    /// for syntax highlighting when being performed server-side instead of
    /// client-side
    #[serde(default = "HtmlCodeConfig::default_theme_dir")]
    pub theme_dir: Option<PathBuf>,

    /// If true, will perform server-side rendering instead of client-side
    /// rendering for syntax highlighting
    #[serde(default = "HtmlCodeConfig::default_server_side")]
    pub server_side: bool,

    /// Represents the directory containing `.tmLanguage` syntax files to be used
    /// for language syntax when being performed server-side instead of client-side
    #[serde(default = "HtmlCodeConfig::default_syntax_dir")]
    pub syntax_dir: Option<PathBuf>,
}

impl Default for HtmlCodeConfig {
    fn default() -> Self {
        Self {
            theme: Self::default_theme(),
            theme_dir: Self::default_theme_dir(),
            server_side: Self::default_server_side(),
            syntax_dir: Self::default_syntax_dir(),
        }
    }
}

impl HtmlCodeConfig {
    #[inline]
    pub fn default_theme() -> String {
        String::from("InspiredGitHub")
    }

    #[inline]
    pub fn default_theme_dir() -> Option<PathBuf> {
        None
    }

    #[inline]
    pub fn default_server_side() -> bool {
        false
    }

    #[inline]
    pub fn default_syntax_dir() -> Option<PathBuf> {
        None
    }
}
