use super::utils::{deserialize_absolute_path, make_path_relative};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    path::{Component, Path, PathBuf},
};
use uriparse::URI;

/// Represents some data with an associated index
#[derive(Copy, Clone, Debug, PartialEq, Eq, AsRef, AsMut, Deref, DerefMut)]
pub struct Indexed<T> {
    index: usize,

    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    inner: T,
}

impl<T> Indexed<T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

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

    /// Configuration settings that apply specifically to paragraphs
    #[serde(default)]
    pub paragraph: HtmlParagraphConfig,

    /// Configuration settings that apply specifically to links
    #[serde(default)]
    pub link: HtmlLinkConfig,

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
    /// Returns the relative path of the actively-processed page to the root
    /// of its wiki
    ///
    /// ### Examples
    ///
    /// ```rust
    /// use vimwiki::{HtmlConfig, HtmlWikiConfig, HtmlRuntimeConfig};
    /// use std::path::{PathBuf, Path};
    ///
    /// // When the active page does has a wiki associated with it, the path
    /// // of the page is traversed upwards until the wiki root is reached and
    /// // is reflected as a relative series of ..
    /// let config = HtmlConfig {
    ///     wikis: vec![
    ///         HtmlWikiConfig {
    ///             path: PathBuf::from("some/wiki"),
    ///             ..Default::default()
    ///         },
    ///     ],
    ///     runtime: HtmlRuntimeConfig {
    ///         wiki_index: Some(0),
    ///         page: PathBuf::from("some/wiki/path/to/file.wiki"),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    /// let path = config.to_active_page_path_to_wiki_root();
    /// assert_eq!(path, PathBuf::from("../.."));
    /// ```
    ///
    /// ```rust
    /// use vimwiki::{HtmlConfig, HtmlWikiConfig, HtmlRuntimeConfig};
    /// use std::path::{PathBuf, Path};
    ///
    /// // When the active page does not have a wiki associated with it, a
    /// // temporary wiki is used where the page is at the root of the wiki
    /// let config = HtmlConfig {
    ///     runtime: HtmlRuntimeConfig {
    ///         page: PathBuf::from("some/wiki/path/to/file.wiki"),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    /// let path = config.to_active_page_path_to_wiki_root();
    /// assert_eq!(path, PathBuf::new());
    /// ```
    pub fn to_active_page_path_to_wiki_root(&self) -> PathBuf {
        self.find_active_wiki()
            .and_then(|wiki| wiki.path_to_root(self.active_page()))
            .unwrap_or_else(PathBuf::new)
    }

    /// Returns the path of the actively-processed page relative to the wiki
    /// containing it
    ///
    /// ### Examples
    ///
    /// ```rust
    /// use vimwiki::{HtmlConfig, HtmlWikiConfig, HtmlRuntimeConfig};
    /// use std::path::{PathBuf, Path};
    ///
    /// let config = HtmlConfig {
    ///     wikis: vec![
    ///         HtmlWikiConfig {
    ///             path: PathBuf::from("some/wiki"),
    ///             ..Default::default()
    ///         },
    ///     ],
    ///     runtime: HtmlRuntimeConfig {
    ///         wiki_index: Some(0),
    ///         page: PathBuf::from("some/wiki/path/to/file.wiki"),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    /// let path = config.as_active_page_path_within_wiki();
    /// assert_eq!(path, Path::new("path/to/file.wiki"));
    /// ```
    pub fn as_active_page_path_within_wiki(&self) -> &Path {
        // NOTE: This should always succeed as the root found will always have
        //       a path that can be stripped from the page's path
        self.to_current_wiki()
            .path_within(self.active_page())
            .expect("Impossible: matched wiki does not contain page")
    }

    /// Produces a wiki config containing the active page either by finding it
    /// in the wiki list or producing a new config representing a temporary
    /// wiki
    pub fn to_current_wiki(&self) -> HtmlWikiConfig {
        self.find_active_wiki()
            .cloned()
            .unwrap_or_else(|| self.runtime.to_tmp_wiki())
    }

    /// Returns the path to the page referenced in the runtime
    pub fn active_page(&self) -> &Path {
        self.runtime.page.as_path()
    }

    /// Returns a reference to the config of the wiki containing the page that
    /// is actively being processed, or None if no wiki contains the page
    pub fn find_active_wiki(&self) -> Option<&HtmlWikiConfig> {
        self.runtime
            .wiki_index
            .as_ref()
            .copied()
            .and_then(|idx| self.find_wiki_by_index(idx))
    }

    /// Finds the wiki config with the given index
    pub fn find_wiki_by_index(&self, idx: usize) -> Option<&HtmlWikiConfig> {
        self.wikis.get(idx)
    }

    /// Finds the index of the first wiki config with an assigned name that
    /// matches the given name
    pub fn find_wiki_index_by_name<S: AsRef<str>>(
        &self,
        name: S,
    ) -> Option<usize> {
        let name = name.as_ref();
        self.wikis.iter().enumerate().find_map(|(idx, wiki)| {
            if wiki.name.as_deref() == Some(name) {
                Some(idx)
            } else {
                None
            }
        })
    }

    /// Finds the first wiki config with an assigned name that matches the
    /// given name
    pub fn find_wiki_by_name<S: AsRef<str>>(
        &self,
        name: S,
    ) -> Option<&HtmlWikiConfig> {
        self.find_wiki_index_by_name(name)
            .and_then(|idx| self.find_wiki_by_index(idx))
    }

    /// Finds the index of the wiki that contains the file at the specified
    /// path; if more than one wiki would match, then the wiki at the deepest
    /// level will be returned (e.g. /my/wiki over /my)
    ///
    /// Provided path must be absolute & canonicalized in order for matching
    /// wiki to be identified
    pub fn find_wiki_index_by_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Option<usize> {
        // NOTE: We can use components().count() because all wikis should have
        //       absolute, canonicalized paths so no concerns about .. or .
        //       misleading the counts
        self.wikis
            .iter()
            .enumerate()
            .filter(|(_, wiki)| path.as_ref().starts_with(wiki.path.as_path()))
            .max_by_key(|(_, wiki)| wiki.path.components().count())
            .map(|(idx, _)| idx)
    }

    /// Finds the wiki that contains the file at the specified path; if more
    /// than one wiki would match, then the wiki at the deepest level will be
    /// returned (e.g. /my/wiki over /my)
    ///
    /// Provided path must be absolute & canonicalized in order for matching
    /// wiki to be identified
    pub fn find_wiki_by_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Option<&HtmlWikiConfig> {
        self.find_wiki_index_by_path(path)
            .and_then(|idx| self.find_wiki_by_index(idx))
    }

    /// Transforms the runtime of the config
    pub fn map_runtime<F: FnOnce(HtmlRuntimeConfig) -> HtmlRuntimeConfig>(
        &mut self,
        f: F,
    ) {
        self.runtime = f(self.runtime.clone());
    }
}

/// Represents a configuration that provides runtime-only configuration settings
/// needed to convert to HTML at a page or wiki-wide level such as the path to
/// the current page that is being processed
#[derive(Clone, Debug)]
pub struct HtmlRuntimeConfig {
    /// Index of wiki that contains the page being processed
    pub wiki_index: Option<usize>,

    /// Path to the page's file that is being processed
    pub page: PathBuf,
}

impl HtmlRuntimeConfig {
    /// Produces a temporary wiki config that treats the page being processed
    /// as the only file within it (for standalone wiki files)
    pub fn to_tmp_wiki(&self) -> HtmlWikiConfig {
        HtmlWikiConfig {
            path: self
                .page
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default(),
            ..Default::default()
        }
    }
}

impl Default for HtmlRuntimeConfig {
    fn default() -> Self {
        Self {
            wiki_index: None,

            // NOTE: This needs to line up with the default wiki config path
            //       being included, otherwise trying to map the runtime
            //       page (default) to a tmp wiki (default) will fail
            page: HtmlWikiConfig::default_path().join("index.wiki"),
        }
    }
}

/// Represents a configuration representing various properties associated with
/// a vimwiki wiki instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HtmlWikiConfig {
    /// Path to the wiki on the local machine (must be absolute path)
    #[serde(
        default = "HtmlWikiConfig::default_path",
        deserialize_with = "deserialize_absolute_path"
    )]
    pub path: PathBuf,

    /// Path to the html output of the wiki on the local machine (must be absolute path)
    #[serde(
        default = "HtmlWikiConfig::default_path_html",
        deserialize_with = "deserialize_absolute_path"
    )]
    pub path_html: PathBuf,

    /// Optional name to associate with the wiki for named links and other
    /// use cases
    #[serde(default = "HtmlWikiConfig::default_name")]
    pub name: Option<String>,

    /// Name of css file to use for styling of pages within the wiki
    #[serde(default = "HtmlWikiConfig::default_css_name")]
    pub css_name: String,

    /// File extension to use when searching for wiki files within a wiki
    #[serde(default = "HtmlWikiConfig::default_ext")]
    pub ext: String,

    /// Path for diary directory relative to this wiki's path
    #[serde(default = "HtmlWikiConfig::default_diary_rel_path")]
    pub diary_rel_path: PathBuf,
}

impl Default for HtmlWikiConfig {
    fn default() -> Self {
        Self {
            path: Self::default_path(),
            path_html: Self::default_path_html(),
            name: Self::default_name(),
            css_name: Self::default_css_name(),
            ext: Self::default_ext(),
            diary_rel_path: Self::default_diary_rel_path(),
        }
    }
}

impl HtmlWikiConfig {
    /// Returns raw file path to root wiki directory
    #[inline]
    pub fn get_root_path(&self) -> &Path {
        self.path.as_path()
    }

    /// Given an input path, will return a relative path from the input path
    /// to get back to the root of the wiki in the form of `../..`, or None
    /// if the input path does not fall within the wiki
    pub fn path_to_root<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        self.path_within(path.as_ref()).and_then(|path| {
            // Determine how many hops back we need to make, with a path
            // like path/to/file.wiki yielding 2 to get back to root
            // of the wiki
            let hops_back = path.components().count();

            // Our actual total hops is 1 less than what is calculated as the
            // above includes the file itself. We only want to process if we
            // have at least one hop otherwise the above indicates there are
            // no path components
            if hops_back > 0 {
                let mut rel_path = PathBuf::new();
                for _ in 0..(hops_back - 1) {
                    rel_path.push(Component::ParentDir);
                }
                Some(rel_path)
            } else {
                None
            }
        })
    }

    /// Given an input path, will return the path relative to the wiki's root,
    /// or None if the path does not fall within the wiki
    pub fn path_within<'a>(&self, path: &'a Path) -> Option<&'a Path> {
        path.strip_prefix(self.get_root_path()).ok()
    }

    /// Produce an absolute path to the html output destination for the given
    /// input file, replacing the extension of the input path with the provided
    /// extension.
    ///
    /// Determining the true input path and output path will be handled in a
    /// few different ways:
    ///
    /// ### Figuring out the input path
    ///
    /// 1. The input path is relative and is thereby appended to the wiki's
    ///    absolute path to identify the full input path of the file
    /// 2. The input path is absolute and starts with the wiki's path, meaning
    ///    that the absolute input path will be stripped to find the relative
    ///    input path
    /// 3. The input path is absolute and does not start with the wiki's path,
    ///    meaning that we treat the absolute path as relative to the root of
    ///    the wiki's path and thereby follow step 1 above
    ///
    pub fn make_output_path(&self, input: &Path, ext: &str) -> PathBuf {
        println!("make_output_path(input = {:?}), ext = {:?})", input, ext);
        println!("is_relative = {}", input.is_relative());
        println!(
            "input starts_with {:?} = {}",
            self.path,
            input.starts_with(self.path.as_path())
        );
        // First, make an absolute path by either using the input if it is
        // absolute and contained in the wiki or treating the input path as
        // relative to the wiki's root path
        let input =
            if input.is_relative() || !input.starts_with(self.path.as_path()) {
                println!("relative mode -- {:?}", make_path_relative(input));
                self.path.join(make_path_relative(input))
            } else {
                println!("other mode -- {:?}", input);
                input.to_path_buf()
            };
        println!(
            "input.strip_prefix({:?}) = {:?}",
            self.path,
            input.strip_prefix(self.path.as_path()).unwrap()
        );

        // Second, figure out the relative path of the input
        let input = input
            .strip_prefix(self.path.as_path())
            .expect("Impossible: file should always be within wiki");
        println!(
            "self.path_html.join({:?}) = {:?}",
            input,
            self.path_html.join(input)
        );

        // Third, take the relative path of the input and append it to the base
        // html output path of the wiki, replacing the extension with html
        self.path_html.join(input).with_extension(ext)
    }

    #[inline]
    pub fn default_path() -> PathBuf {
        // NOTE: For wasm, home directory will always return None, but we don't
        //       expect the default value to be used in wasm
        dirs::home_dir()
            .unwrap_or_else(|| {
                let mut path = PathBuf::new();
                path.push(Component::RootDir);
                path
            })
            .join("vimwiki")
    }

    #[inline]
    pub fn default_path_html() -> PathBuf {
        // NOTE: For wasm, home directory will always return None, but we don't
        //       expect the default value to be used in wasm
        dirs::home_dir()
            .unwrap_or_else(|| {
                let mut path = PathBuf::new();
                path.push(Component::RootDir);
                path
            })
            .join("vimwiki_html")
    }

    #[inline]
    pub const fn default_name() -> Option<String> {
        None
    }

    #[inline]
    pub fn default_css_name() -> String {
        String::from("style.css")
    }

    #[inline]
    pub fn default_ext() -> String {
        String::from("wiki")
    }

    #[inline]
    pub fn default_diary_rel_path() -> PathBuf {
        PathBuf::from("diary")
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

/// Represents configuration options related to paragraphs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HtmlParagraphConfig {
    /// If true, newlines are ignored when producing paragraphs, otherwise the
    /// line breaks are respected and <br /> is added for each line break in
    /// a paragraph
    #[serde(default = "HtmlParagraphConfig::default_ignore_newline")]
    pub ignore_newline: bool,
}

impl Default for HtmlParagraphConfig {
    fn default() -> Self {
        Self {
            ignore_newline: Self::default_ignore_newline(),
        }
    }
}

impl HtmlParagraphConfig {
    #[inline]
    pub fn default_ignore_newline() -> bool {
        true
    }
}

/// Represents configuration options related to links
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HtmlLinkConfig {
    /// Represents the base url used when forming absolute links
    #[serde(default = "HtmlLinkConfig::default_base_url", with = "uri")]
    pub base_url: URI<'static>,

    /// If true, all relative links (path/to/file.html or even /wiki/path/to/file.html)
    /// will be canonicalized using the base_url, otherwise they are kept as
    /// they are provided
    #[serde(default = "HtmlLinkConfig::default_canonicalize")]
    pub canonicalize: bool,

    /// If true, wiki pages are generated as "ugly URLs" such as `example.com/urls.html`
    /// instead of the pretty form of `example.com/urls/`
    #[serde(default = "HtmlLinkConfig::default_use_ugly_urls")]
    pub use_ugly_urls: bool,
}

/// Module that provides serialize/deserialize of URI to a string type
mod uri {
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    use std::convert::TryFrom;
    use uriparse::URI;

    pub fn serialize<S>(
        data: &URI<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        String::serialize(&data.to_string(), serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<URI<'static>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        URI::try_from(s.as_str()).map(URI::into_owned).map_err(|x| {
            de::Error::invalid_value(
                de::Unexpected::Str(&s),
                &x.to_string().as_str(),
            )
        })
    }
}

impl Default for HtmlLinkConfig {
    fn default() -> Self {
        Self {
            base_url: Self::default_base_url(),
            canonicalize: Self::default_canonicalize(),
            use_ugly_urls: Self::default_use_ugly_urls(),
        }
    }
}

impl HtmlLinkConfig {
    #[inline]
    pub fn default_base_url() -> URI<'static> {
        URI::try_from("https://localhost").unwrap().into_owned()
    }

    #[inline]
    pub const fn default_canonicalize() -> bool {
        false
    }

    #[inline]
    pub const fn default_use_ugly_urls() -> bool {
        false
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
