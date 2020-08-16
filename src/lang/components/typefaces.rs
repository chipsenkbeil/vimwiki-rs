use derive_more::Constructor;
use serde::{Deserialize, Serialize};

/// Represents a typeface decoration that can be applied to text
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Decoration {
    Bold,
    Italic,
    BoldItalic,
    Strikeout,
    Code,
    Superscript,
    Subscript,
}

/// Represents text with a typeface decoration
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct DecoratedText {
    text: String,
    decoration: Decoration,
}

impl DecoratedText {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn decoration(&self) -> Decoration {
        self.decoration
    }
}

/// Represents special keywords that have unique syntax highlighting
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Keyword {
    TODO,
    DONE,
    STARTED,
    FIXME,
    FIXED,
    XXX,
}
