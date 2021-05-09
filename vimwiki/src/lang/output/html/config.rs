use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// Represents a configuration specifically geared towards converting to an
/// HTML page (not element) based on wiki properties
#[derive(Builder, Clone, Debug, Default)]
#[builder(pattern = "owned", build_fn(name = "finish"), setter(into))]
pub struct HtmlWikiPageConfig {
    #[builder(default, setter(strip_option))]
    pub wiki_root: Option<PathBuf>,
    pub page: PathBuf,
    #[builder(default, setter(strip_option))]
    pub css_name: Option<String>,
}

impl HtmlWikiPageConfig {
    #[inline]
    pub fn build() -> HtmlWikiPageConfigBuilder {
        HtmlWikiPageConfigBuilder::default()
    }

    #[inline]
    pub fn get_wiki_root_path(&self) -> Option<&Path> {
        self.wiki_root.as_deref()
    }

    #[inline]
    pub fn get_page_path(&self) -> &Path {
        self.page.as_path()
    }

    /// Returns the relative path of the page to the wiki root if the page is
    /// found within the wiki root
    ///
    /// ### Examples
    ///
    /// ```rust
    /// use vimwiki::HtmlWikiPageConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = HtmlWikiPageConfig {
    ///     wiki_root: Some(PathBuf::from("/some/wiki/dir")),
    ///     page: PathBuf::from("/some/wiki/dir/to/a/file.wiki"),
    ///     css_name: None,
    /// };
    /// let path = config.get_page_relative_path_to_root().unwrap();
    /// assert_eq!(path, PathBuf::from("../.."));
    /// ```
    pub fn get_page_relative_path_to_root(&self) -> Option<PathBuf> {
        // Remove the directory from the file path as well as remove the file
        // from the path itself
        self.get_page_path_within_root()
            .and_then(|p| p.parent())
            .map(|path| {
                // Now, we convert each component to a .. to signify that we have
                // to go back up
                let mut rel_path = PathBuf::new();
                for _ in path.components() {
                    rel_path.push(Component::ParentDir);
                }
                rel_path
            })
    }

    /// Returns the relative path of the page to the wiki root if a wiki root
    /// exists and the page is found within it
    ///
    /// ### Examples
    ///
    /// ```rust
    /// use vimwiki::HtmlWikiPageConfig;
    /// use std::path::{PathBuf, Path};
    ///
    /// let config = HtmlWikiPageConfig {
    ///     wiki_root: Some(PathBuf::from("/some/wiki/dir")),
    ///     page: PathBuf::from("/some/wiki/dir/to/a/file.wiki"),
    ///     css_name: None,
    /// };
    /// let path = config.get_page_path_within_root().unwrap();
    /// assert_eq!(path, Path::new("to/a/file.wiki"));
    /// ```
    pub fn get_page_path_within_root(&self) -> Option<&Path> {
        let root = self.get_wiki_root_path();
        let page = self.get_page_path();

        root.and_then(|r| page.strip_prefix(r).ok())
    }

    #[inline]
    pub fn get_css_name_or_default(&self) -> &str {
        self.css_name
            .as_deref()
            .unwrap_or_else(|| Self::default_css_name())
    }

    #[inline]
    pub const fn default_css_name() -> &'static str {
        "style.css"
    }
}

/// Represents configuration properties for HTML writing that are separate from
/// the running state during HTML conversion
#[derive(Builder, Clone, Debug, Default, Serialize, Deserialize)]
#[builder(pattern = "owned", build_fn(name = "finish"), setter(into))]
pub struct HtmlConfig {
    #[builder(default)]
    #[serde(default)]
    pub list: HtmlListConfig,
    #[builder(default)]
    #[serde(default)]
    pub text: HtmlTextConfig,
    #[builder(default)]
    #[serde(default)]
    pub header: HtmlHeaderConfig,
    #[builder(default)]
    #[serde(default)]
    pub code: HtmlCodeConfig,
    #[builder(default)]
    #[serde(default)]
    pub comment: HtmlCommentConfig,
    #[builder(default)]
    #[serde(default)]
    pub template: HtmlTemplateConfig,
}

impl HtmlConfig {
    #[inline]
    pub fn build() -> HtmlConfigBuilder {
        HtmlConfigBuilder::default()
    }
}

/// Represents configuration options related to lists
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(pattern = "owned", build_fn(name = "finish"), setter(into))]
pub struct HtmlListConfig {
    /// If true, newlines are ignored when producing lists, otherwise the
    /// line breaks are respected and <br /> is added for each line break in
    /// a list
    #[builder(default = "HtmlListConfig::default_ignore_newline()")]
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
    pub fn build() -> HtmlListConfigBuilder {
        HtmlListConfigBuilder::default()
    }

    #[inline]
    pub fn default_ignore_newline() -> bool {
        true
    }
}

/// Represents configuration options related to text
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(pattern = "owned", build_fn(name = "finish"), setter(into))]
pub struct HtmlTextConfig {
    /// If true, newlines are ignored when producing paragraphs, otherwise the
    /// line breaks are respected and <br /> is added for each line break in
    /// a paragraph
    #[builder(default = "HtmlTextConfig::default_ignore_newline()")]
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
    pub fn build() -> HtmlTextConfigBuilder {
        HtmlTextConfigBuilder::default()
    }

    #[inline]
    pub fn default_ignore_newline() -> bool {
        true
    }
}

/// Represents configuration options related to headers
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(pattern = "owned", build_fn(name = "finish"), setter(into))]
pub struct HtmlHeaderConfig {
    /// Represents the text that a header could have to be marked as the ToC
    #[builder(default = "HtmlHeaderConfig::default_table_of_contents()")]
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
    pub fn build() -> HtmlHeaderConfigBuilder {
        HtmlHeaderConfigBuilder::default()
    }

    #[inline]
    pub fn default_table_of_contents() -> String {
        String::from("Contents")
    }
}

/// Represents configuration options related to code
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(pattern = "owned", build_fn(name = "finish"), setter(into))]
pub struct HtmlCodeConfig {
    /// Represents the built-in theme to be used for syntax highlighting when
    /// being performed server-side instead of client-side
    #[builder(default = "HtmlCodeConfig::default_theme()")]
    #[serde(default = "HtmlCodeConfig::default_theme")]
    pub theme: String,

    /// Represents the directory containing `.tmTheme` theme files to be used
    /// for syntax highlighting when being performed server-side instead of
    /// client-side
    #[builder(default = "HtmlCodeConfig::default_theme_dir()")]
    #[serde(default = "HtmlCodeConfig::default_theme_dir")]
    pub theme_dir: Option<PathBuf>,

    /// If true, will perform server-side rendering instead of client-side
    /// rendering for syntax highlighting
    #[builder(default = "HtmlCodeConfig::default_server_side()")]
    #[serde(default = "HtmlCodeConfig::default_server_side")]
    pub server_side: bool,

    /// Represents the directory containing `.tmLanguage` syntax files to be used
    /// for language syntax when being performed server-side instead of client-side
    #[builder(default = "HtmlCodeConfig::default_syntax_dir()")]
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
    pub fn build() -> HtmlCodeConfigBuilder {
        HtmlCodeConfigBuilder::default()
    }

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

/// Represents configuration options related to comments
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(pattern = "owned", build_fn(name = "finish"), setter(into))]
pub struct HtmlCommentConfig {
    /// If true, will include comments in HTML output as `<!-- {comment} -->`
    #[builder(default = "HtmlCommentConfig::default_include()")]
    #[serde(default = "HtmlCommentConfig::default_include")]
    pub include: bool,
}

impl Default for HtmlCommentConfig {
    fn default() -> Self {
        Self {
            include: Self::default_include(),
        }
    }
}

impl HtmlCommentConfig {
    #[inline]
    pub fn build() -> HtmlCommentConfigBuilder {
        HtmlCommentConfigBuilder::default()
    }

    #[inline]
    pub fn default_include() -> bool {
        false
    }
}

/// Represents configuration options related to templates
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[builder(pattern = "owned", build_fn(name = "finish"), setter(into))]
pub struct HtmlTemplateConfig {
    /// Represents the name of the default template to use (e.g. default)
    #[builder(default = "HtmlTemplateConfig::default_name()")]
    #[serde(default = "HtmlTemplateConfig::default_name")]
    pub name: String,

    /// Represents the file extension to use for all template files (e.g. tpl)
    #[builder(default = "HtmlTemplateConfig::default_ext()")]
    #[serde(default = "HtmlTemplateConfig::default_ext")]
    pub ext: String,

    /// Represents the directory containing all vimwiki templates
    /// (e.g. $HOME/vimwiki/templates)
    #[builder(default = "HtmlTemplateConfig::default_dir()")]
    #[serde(default = "HtmlTemplateConfig::default_dir")]
    pub dir: PathBuf,

    /// Represents the text to use for the template if no explicit template
    /// is specified
    #[builder(default = "HtmlTemplateConfig::default_text()")]
    #[serde(default = "HtmlTemplateConfig::default_text")]
    pub text: String,
}

impl Default for HtmlTemplateConfig {
    fn default() -> Self {
        Self {
            name: Self::default_name(),
            ext: Self::default_ext(),
            dir: Self::default_dir(),
            text: Self::default_text(),
        }
    }
}

impl HtmlTemplateConfig {
    #[inline]
    pub fn build() -> HtmlTemplateConfigBuilder {
        HtmlTemplateConfigBuilder::default()
    }

    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    #[inline]
    pub fn default_name() -> String {
        String::from("default")
    }

    #[inline]
    pub fn default_ext() -> String {
        String::from("tpl")
    }

    #[inline]
    pub fn default_dir() -> PathBuf {
        let mut path = PathBuf::new();
        if let Some(dir) = dirs::home_dir() {
            path.push(dir);
        }
        path.push("vimwiki");
        path.push("templates");
        path
    }

    #[inline]
    pub fn default_text() -> String {
        static DEFAULT_TEMPLATE_STR: &str = r#"<!DOCTYPE html>
<html>
<head>
<link rel="Stylesheet" type="text/css" href="%root_path%%css%">
<title>%title%</title>
<meta http-equiv="Content-Type" content="text/html; charset=%encoding%">
<meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
%content%
</body>
</html>
"#;

        DEFAULT_TEMPLATE_STR.to_string()
    }
}
