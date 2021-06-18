use serde::{Deserialize, Serialize};

/// Represents configuration properties for HTML vimwiki that are separate from
/// the running state during vimwiki conversion
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VimwikiConfig {
    /// Configuration settings that apply broadly to general formatting
    #[serde(default)]
    pub format: VimwikiFormatConfig,

    /// Configuration settings that apply specifically to blockquotes
    #[serde(default)]
    pub blockquote: VimwikiBlockquoteConfig,

    /// Configuration settings that apply specifically to definition lists
    #[serde(default)]
    pub definition_list: VimwikiDefinitionListConfig,

    /// Configuration settings that apply specifically to headers
    #[serde(default)]
    pub header: VimwikiHeaderConfig,

    /// Configuration settings that apply specifically to lists
    #[serde(default)]
    pub list: VimwikiListConfig,

    /// Configuration settings that apply specifically to paragraphs
    #[serde(default)]
    pub paragraph: VimwikiParagraphConfig,

    /// Configuration settings that apply specifically to tables
    #[serde(default)]
    pub table: VimwikiTableConfig,
}

/// Represents configuration options related to general formatting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimwikiFormatConfig {
    /// Represents the string to use when indenting (e.g. four spaces or a tab)
    #[serde(default = "VimwikiFormatConfig::default_indent_str")]
    pub indent_str: String,
}

impl Default for VimwikiFormatConfig {
    fn default() -> Self {
        Self {
            indent_str: Self::default_indent_str(),
        }
    }
}

impl VimwikiFormatConfig {
    #[inline]
    pub fn default_indent_str() -> String {
        String::from("    ")
    }
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

/// Represents configuration options related to lists
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimwikiListConfig {
    /// If true, will trim all leading and trailing whitespace from each line
    #[serde(default = "VimwikiListConfig::default_trim_lines")]
    pub trim_lines: bool,

    /// Configuration settings that apply specifically to todo list items
    #[serde(default)]
    pub todo: VimwikiTodoListItemConfig,
}

impl Default for VimwikiListConfig {
    fn default() -> Self {
        Self {
            trim_lines: Self::default_trim_lines(),
            todo: Default::default(),
        }
    }
}

impl VimwikiListConfig {
    #[inline]
    pub fn default_trim_lines() -> bool {
        true
    }
}

/// Represents configuration options related to todo list items
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct VimwikiTodoListItemConfig {
    /// Text to use within [ ] to signify incomplete status
    #[serde(default = "VimwikiTodoListItemConfig::default_incomplete_char")]
    pub incomplete_char: char,

    /// Text to use within [ ] to signify partially complete 1 status
    #[serde(
        default = "VimwikiTodoListItemConfig::default_partially_complete_1_char"
    )]
    pub partially_complete_1_char: char,

    /// Text to use within [ ] to signify partially complete 2 status
    #[serde(
        default = "VimwikiTodoListItemConfig::default_partially_complete_2_char"
    )]
    pub partially_complete_2_char: char,

    /// Text to use within [ ] to signify partially complete 3 status
    #[serde(
        default = "VimwikiTodoListItemConfig::default_partially_complete_3_char"
    )]
    pub partially_complete_3_char: char,

    /// Text to use within [ ] to signify complete status
    #[serde(default = "VimwikiTodoListItemConfig::default_complete_char")]
    pub complete_char: char,

    /// Text to use within [ ] to signify rejected status
    #[serde(default = "VimwikiTodoListItemConfig::default_rejected_char")]
    pub rejected_char: char,
}

impl Default for VimwikiTodoListItemConfig {
    fn default() -> Self {
        Self {
            incomplete_char: Self::default_incomplete_char(),
            partially_complete_1_char: Self::default_partially_complete_1_char(
            ),
            partially_complete_2_char: Self::default_partially_complete_2_char(
            ),
            partially_complete_3_char: Self::default_partially_complete_3_char(
            ),
            complete_char: Self::default_complete_char(),
            rejected_char: Self::default_rejected_char(),
        }
    }
}

impl VimwikiTodoListItemConfig {
    #[inline]
    pub fn default_incomplete_char() -> char {
        ' '
    }

    #[inline]
    pub fn default_partially_complete_1_char() -> char {
        '.'
    }

    #[inline]
    pub fn default_partially_complete_2_char() -> char {
        'o'
    }

    #[inline]
    pub fn default_partially_complete_3_char() -> char {
        'O'
    }

    #[inline]
    pub fn default_complete_char() -> char {
        'X'
    }

    #[inline]
    pub fn default_rejected_char() -> char {
        '-'
    }
}

/// Represents configuration options related to paragraphs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimwikiParagraphConfig {
    /// The column to enforce a line wrap at word boundaries where possible
    #[serde(default = "VimwikiParagraphConfig::default_line_wrap_column")]
    pub line_wrap_column: usize,

    /// If true, will leave lines as they are and not wrap
    #[serde(default = "VimwikiParagraphConfig::default_no_line_wrap")]
    pub no_line_wrap: bool,

    /// If true, will trim all leading and trailing whitespace from each line
    #[serde(default = "VimwikiParagraphConfig::default_trim_lines")]
    pub trim_lines: bool,
}

impl Default for VimwikiParagraphConfig {
    fn default() -> Self {
        Self {
            line_wrap_column: Self::default_line_wrap_column(),
            no_line_wrap: Self::default_no_line_wrap(),
            trim_lines: Self::default_trim_lines(),
        }
    }
}

impl VimwikiParagraphConfig {
    #[inline]
    pub fn default_line_wrap_column() -> usize {
        80
    }

    #[inline]
    pub fn default_no_line_wrap() -> bool {
        false
    }

    #[inline]
    pub fn default_trim_lines() -> bool {
        true
    }
}

/// Represents configuration options related to tables
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VimwikiTableConfig {
    /// If true, will not pad a cell's content within a table
    #[serde(default = "VimwikiHeaderConfig::default_no_padding")]
    pub no_padding: bool,
}

impl Default for VimwikiTableConfig {
    fn default() -> Self {
        Self {
            no_padding: Self::default_no_padding(),
        }
    }
}

impl VimwikiTableConfig {
    #[inline]
    pub fn default_no_padding() -> bool {
        false
    }
}
