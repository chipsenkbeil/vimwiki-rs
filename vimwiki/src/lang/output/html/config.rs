use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// Represents configuration properties for HTML writing that are separate from
/// the running state during HTML conversion
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HtmlConfig {
    /// Represents runtime-only configurations such as indicating the path to
    /// the page that is being processed
    ///
    /// [RUNTIME ONLY] Runtime-only config that is not saved/loaded!
    #[serde(skip)]
    pub runtime: HtmlRuntimeConfig,

    /// Maps to vimwiki's wiki config and order matters for use in indexed
    /// wiki links
    #[serde(default)]
    pub wikis: Vec<HtmlWikiConfig>,

    /// Configuration settings that apply specifically to lists
    #[serde(default)]
    pub list: HtmlListConfig,

    /// Configuration settings that apply specifically to text
    #[serde(default)]
    pub text: HtmlTextConfig,

    /// Configuration settings that apply specifically to headers
    #[serde(default)]
    pub header: HtmlHeaderConfig,

    /// Configuration settings that apply specifically to code
    #[serde(default)]
    pub code: HtmlCodeConfig,

    /// Configuration settings that apply specifically to comments
    #[serde(default)]
    pub comment: HtmlCommentConfig,

    /// Configuration settings that apply specifically to templates
    #[serde(default)]
    pub template: HtmlTemplateConfig,
}

impl HtmlConfig {
    /// Returns true if config is for one of many wikis
    #[inline]
    pub fn is_multi_wiki(&self) -> bool {
        !self.wikis.is_empty()
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
    /// let path = config.get_path_to_root().unwrap();
    /// assert_eq!(path, PathBuf::from("../.."));
    /// ```
    pub fn get_path_to_root(&self) -> Option<PathBuf> {
        // Remove the directory from the file path as well as remove the file
        // from the path itself
        self.get_path_within_root().parent().map(|path| {
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
    /// let path = config.get_path_within_root().unwrap();
    /// assert_eq!(path, Path::new("to/a/file.wiki"));
    /// ```
    pub fn get_path_within_root(&self) -> &Path {
        let wiki = self.to_active_wiki_config();
        let root = wiki.get_root_path();
        let page = self.runtime.get_active_page_path();

        // NOTE: This should always succeed as the root found will always have
        //       a path that can be stripped from the page's path
        page.strip_prefix(root)
            .expect("Impossible: matched wiki does not contain page")
    }

    pub fn to_active_wiki_config(&self) -> HtmlWikiConfig {
        self.get_wiki_config_containing_active_page()
            .cloned()
            .unwrap_or_else(|| self.runtime.active_page_to_wiki_config())
    }

    /// Returns a reference to the config of the wiki containing the page that
    /// is actively being processed, or None if no wiki contains the page
    pub fn get_wiki_config_containing_active_page(
        &self,
    ) -> Option<&HtmlWikiConfig> {
        self.runtime
            .get_wiki_index_for_active_page()
            .and_then(|idx| self.find_wiki_by_index(idx))
    }

    /// Finds the wiki config with the given index
    pub fn find_wiki_by_index(&self, idx: usize) -> Option<&HtmlWikiConfig> {
        self.wikis.get(idx)
    }

    /// Finds the first wiki config with an assigned name that matches the
    /// given name
    pub fn find_wiki_by_name<S: AsRef<str>>(
        &self,
        name: S,
    ) -> Option<&HtmlWikiConfig> {
        let name = name.as_ref();
        self.wikis
            .iter()
            .find(|wiki| wiki.name.as_deref() == Some(name))
    }
}

/// Represents a configuration that provides runtime-only configuration settings
/// needed to convert to HTML at a page or wiki-wide level such as the path to
/// the current page that is being processed
#[derive(Clone, Debug, Default)]
pub struct HtmlRuntimeConfig {
    /// Index of wiki that contains the page being processed
    pub wiki_index: Option<usize>,

    /// Path to the page's file that is being processed
    pub page: PathBuf,
}

impl HtmlRuntimeConfig {
    /// Returns index of wiki that contains the page being processed
    pub fn get_wiki_index_for_active_page(&self) -> Option<usize> {
        self.wiki_index
    }

    /// Returns raw file path to current wiki page being processed
    pub fn get_active_page_path(&self) -> &Path {
        self.page.as_path()
    }

    /// Produces a wiki config that treats the page being processed as the
    /// only file within it (for standalone wiki files)
    pub fn active_page_to_wiki_config(&self) -> HtmlWikiConfig {
        HtmlWikiConfig {
            path: self
                .get_active_page_path()
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default(),
            ..Default::default()
        }
    }
}

/// Represents a configuration representing various properties associated with
/// a vimwiki wiki instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HtmlWikiConfig {
    /// Path to the wiki on the local machine
    #[serde(default = "HtmlWikiConfig::default_path")]
    pub path: PathBuf,

    /// Optional name to associate with the wiki for named links and other
    /// use cases
    #[serde(default = "HtmlWikiConfig::default_name")]
    pub name: Option<String>,

    /// Name of css file to use for styling of pages within the wiki
    pub css_name: Option<String>,
}

impl Default for HtmlWikiConfig {
    fn default() -> Self {
        Self {
            path: Self::default_path(),
            name: Self::default_name(),
            css_name: None,
        }
    }
}

impl HtmlWikiConfig {
    /// Returns raw file path to root wiki directory
    #[inline]
    pub fn get_root_path(&self) -> &Path {
        self.path.as_path()
    }

    /// Get name of css file to use, or the default css style
    #[inline]
    pub fn get_css_name_or_default(&self) -> &str {
        self.css_name
            .as_deref()
            .unwrap_or_else(|| Self::default_css_name())
    }

    #[inline]
    pub fn default_path() -> PathBuf {
        PathBuf::new()
    }

    #[inline]
    pub const fn default_name() -> Option<String> {
        None
    }

    #[inline]
    pub const fn default_css_name() -> &'static str {
        "style.css"
    }
}

/// Represents configuration options related to lists
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Represents configuration options related to comments
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HtmlCommentConfig {
    /// If true, will include comments in HTML output as `<!-- {comment} -->`
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
    pub fn default_include() -> bool {
        false
    }
}

/// Represents configuration options related to templates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HtmlTemplateConfig {
    /// Represents the name of the default template to use (e.g. default)
    #[serde(default = "HtmlTemplateConfig::default_name")]
    pub name: String,

    /// Represents the file extension to use for all template files (e.g. tpl)
    #[serde(default = "HtmlTemplateConfig::default_ext")]
    pub ext: String,

    /// Represents the directory containing all vimwiki templates
    /// (e.g. $HOME/vimwiki/templates)
    #[serde(default = "HtmlTemplateConfig::default_dir")]
    pub dir: PathBuf,

    /// Represents the text to use for the template if no explicit template
    /// is specified
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
