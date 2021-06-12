use serde::{Deserialize, Serialize};

/// Represents configuration properties for HTML vimwiki that are separate from
/// the running state during vimwiki conversion
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VimwikiConfig {
    /// Configuration settings that apply specifically to blockquotes
    #[serde(default)]
    pub blockquote: VimwikiBlockquoteConfig,

    /// Configuration settings that apply specifically to definition lists
    #[serde(default)]
    pub definition_list: VimwikiDefinitionListConfig,

    /// Configuration settings that apply specifically to headers
    #[serde(default)]
    pub header: VimwikiHeaderConfig,
}

/// Represents configuration options related to blockquotes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimwikiBlockquoteConfig {
    /// If true, will use an indented blockquote instead of the traditional
    /// blockquote when possible, rather than defaulting to the traditional
    /// blockquote of `> ...`
    #[serde(
        default = "VimwikiBlockquoteConfig::default_prefer_indented_blockquote"
    )]
    pub prefer_indented_blockquote: bool,

    /// If true, will trim all leading and trailing whitespace from each line
    #[serde(default = "VimwikiBlockquoteConfig::default_trim_lines")]
    pub trim_lines: bool,
}

impl Default for VimwikiBlockquoteConfig {
    fn default() -> Self {
        Self {
            prefer_indented_blockquote:
                Self::default_prefer_indented_blockquote(),
            trim_lines: Self::default_trim_lines(),
        }
    }
}

impl VimwikiBlockquoteConfig {
    #[inline]
    pub fn default_prefer_indented_blockquote() -> bool {
        false
    }

    #[inline]
    pub fn default_trim_lines() -> bool {
        true
    }
}

/// Represents configuration options related to definition lists
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimwikiDefinitionListConfig {
    /// If true, will not place the first definition for a term on the same line
    #[serde(
        default = "VimwikiDefinitionListConfig::default_term_on_line_by_itself"
    )]
    pub term_on_line_by_itself: bool,

    /// If true, will trim all leading and trailing whitespace from each term
    #[serde(default = "VimwikiDefinitionListConfig::default_trim_terms")]
    pub trim_terms: bool,

    /// If true, will trim all leading and trailing whitespace from each definition
    #[serde(default = "VimwikiDefinitionListConfig::default_trim_definitions")]
    pub trim_definitions: bool,
}

impl Default for VimwikiDefinitionListConfig {
    fn default() -> Self {
        Self {
            term_on_line_by_itself: Self::default_term_on_line_by_itself(),
            trim_terms: Self::default_trim_terms(),
            trim_definitions: Self::default_trim_definitions(),
        }
    }
}

impl VimwikiDefinitionListConfig {
    #[inline]
    pub fn default_term_on_line_by_itself() -> bool {
        false
    }

    #[inline]
    pub fn default_trim_terms() -> bool {
        true
    }

    #[inline]
    pub fn default_trim_definitions() -> bool {
        true
    }
}

/// Represents configuration options related to headers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimwikiHeaderConfig {
    /// If true, will not pad a header's content
    #[serde(default = "VimwikiHeaderConfig::default_no_padding")]
    pub no_padding: bool,

    /// If true, will trim all leading and trailing whitespace from header's content
    #[serde(default = "VimwikiHeaderConfig::default_trim_content")]
    pub trim_content: bool,
}

impl Default for VimwikiHeaderConfig {
    fn default() -> Self {
        Self {
            no_padding: Self::default_no_padding(),
            trim_content: Self::default_trim_content(),
        }
    }
}

impl VimwikiHeaderConfig {
    #[inline]
    pub fn default_no_padding() -> bool {
        false
    }

    #[inline]
    pub fn default_trim_content() -> bool {
        true
    }
}
