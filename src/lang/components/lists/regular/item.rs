use super::{InlineComponentContainer, RegularList};
use derive_more::From;
use numerals::roman::Roman;
use serde::{Deserialize, Serialize};

/// Represents a suffix such as . or ) used after beginning of list item
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RegularListItemSuffix {
    Period,
    Paren,
}

impl RegularListItemSuffix {
    pub fn as_char(self) -> char {
        match self {
            Self::Period => '.',
            Self::Paren => ')',
        }
    }
}

/// Represents some content associated with a list item, either being
/// a series of inline components or a new sublist
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum RegularListItemContent {
    InlineContent(InlineComponentContainer),
    RegularList(RegularList),
}

pub type RegularListItemContents = Vec<RegularListItemContent>;

/// Represents supported prefix types for a list item
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RegularListItem {
    /// -
    Hyphen {
        pos: usize,
        contents: RegularListItemContents,
    },
    /// *
    Asterisk {
        pos: usize,
        contents: RegularListItemContents,
    },
    /// 1
    Number {
        pos: usize,
        suffix: RegularListItemSuffix,
        contents: RegularListItemContents,
    },
    /// a
    LowercaseAlphabet {
        pos: usize,
        suffix: RegularListItemSuffix,
        contents: RegularListItemContents,
    },
    /// A
    UppercaseAlphabet {
        pos: usize,
        suffix: RegularListItemSuffix,
        contents: RegularListItemContents,
    },
    /// i
    LowercaseRoman {
        pos: usize,
        suffix: RegularListItemSuffix,
        contents: RegularListItemContents,
    },
    /// I
    UppercaseRoman {
        pos: usize,
        suffix: RegularListItemSuffix,
        contents: RegularListItemContents,
    },
    /// ???
    Other {
        pos: usize,
        value: String,
        contents: RegularListItemContents,
    },
}

impl RegularListItem {
    /// Indicates whether or not this list item represents an unordered item
    pub fn is_unordered(&self) -> bool {
        match self {
            Self::Hyphen { .. }
            | Self::Asterisk { .. }
            | Self::Other { .. } => true,
            _ => false,
        }
    }

    /// Indicates whether or not this list item represents an ordered item
    pub fn is_ordered(&self) -> bool {
        !self.is_unordered()
    }

    /// Represents the position of the item within a list, starting at 0
    pub fn pos(&self) -> usize {
        *match self {
            Self::Hyphen { pos, .. } => pos,
            Self::Asterisk { pos, .. } => pos,
            Self::Number { pos, .. } => pos,
            Self::LowercaseAlphabet { pos, .. } => pos,
            Self::UppercaseAlphabet { pos, .. } => pos,
            Self::LowercaseRoman { pos, .. } => pos,
            Self::UppercaseRoman { pos, .. } => pos,
            Self::Other { pos, .. } => pos,
        }
    }

    /// Represents the contents of the list item
    pub fn contents(&self) -> &[RegularListItemContent] {
        match self {
            Self::Hyphen { contents, .. } => contents,
            Self::Asterisk { contents, .. } => contents,
            Self::Number { contents, .. } => contents,
            Self::LowercaseAlphabet { contents, .. } => contents,
            Self::UppercaseAlphabet { contents, .. } => contents,
            Self::LowercaseRoman { contents, .. } => contents,
            Self::UppercaseRoman { contents, .. } => contents,
            Self::Other { contents, .. } => contents,
        }
    }

    fn suffix(&self) -> Option<RegularListItemSuffix> {
        match self {
            Self::Hyphen { .. } => None,
            Self::Asterisk { .. } => None,
            Self::Number { suffix, .. } => Some(*suffix),
            Self::LowercaseAlphabet { suffix, .. } => Some(*suffix),
            Self::UppercaseAlphabet { suffix, .. } => Some(*suffix),
            Self::LowercaseRoman { suffix, .. } => Some(*suffix),
            Self::UppercaseRoman { suffix, .. } => Some(*suffix),
            Self::Other { .. } => None,
        }
    }

    /// Allocates a new string representing the full prefix of the list item
    /// such as * or 1. or iii)
    pub fn to_prefix(&self) -> String {
        let mut base = match self {
            Self::Hyphen { .. } => String::from("-"),
            Self::Asterisk { .. } => String::from("*"),
            Self::Number { pos, .. } => pos.to_string(),
            Self::LowercaseAlphabet { pos, .. } => {
                pos_to_alphabet(*pos + 1, true)
            }
            Self::UppercaseAlphabet { pos, .. } => {
                pos_to_alphabet(*pos + 1, false)
            }
            Self::LowercaseRoman { pos, .. } => {
                format!("{:x}", Roman::from((pos + 1) as i16))
            }
            Self::UppercaseRoman { pos, .. } => {
                format!("{:X}", Roman::from((pos + 1) as i16))
            }
            Self::Other { value, .. } => value.to_string(),
        };

        if let Some(suffix) = self.suffix() {
            base.push(suffix.as_char());
        }

        base
    }
}

/// Converts a position in a list (base 0) to an alphabetic representation
/// where 0 == a, 25 == z, 26 == aa, and so on
fn pos_to_alphabet(pos: usize, to_lower: bool) -> String {
    let mut s = String::new();
    let mut i = pos as u32;
    let base = if to_lower { 97 } else { 65 };

    loop {
        if let Some(c) = std::char::from_u32(base + (i % 26)) {
            s.push(c);
        }

        if i >= 26 {
            i %= 26;
        } else {
            break;
        }
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_ordered_should_return_true_if_ordered_prefix() {
        panic!("TODO: Implement");
    }

    #[test]
    fn is_ordered_should_return_false_if_unordered_prefix() {
        panic!("TODO: Implement");
    }

    #[test]
    fn is_unordered_should_return_true_if_unordered_prefix() {
        panic!("TODO: Implement");
    }

    #[test]
    fn is_unordered_should_return_false_if_ordered_prefix() {
        panic!("TODO: Implement");
    }

    #[test]
    fn pos_should_return_internal_position() {
        panic!("TODO: Implement");
    }

    #[test]
    fn contents_should_return_internal_contents() {
        panic!("TODO: Implement");
    }

    #[test]
    fn suffix_should_return_some_suffix_if_applicable() {
        panic!("TODO: Implement");
    }

    #[test]
    fn suffix_should_return_none_if_not_applicable() {
        panic!("TODO: Implement");
    }

    #[test]
    fn to_prefix_should_return_hyphen_if_hyphen_type() {
        panic!("TODO: Implement");
    }

    #[test]
    fn to_prefix_should_return_asterisk_if_asterisk_type() {
        panic!("TODO: Implement");
    }

    #[test]
    fn to_prefix_should_return_base_1_position_if_number_type() {
        panic!("TODO: Implement");
    }

    #[test]
    fn to_prefix_should_return_lowercase_alphabetic_string_if_lowercase_alphabetic_type(
    ) {
        panic!("TODO: Implement");
    }

    #[test]
    fn to_prefix_should_return_uppercase_alphabetic_string_if_uppercase_alphabetic_type(
    ) {
        panic!("TODO: Implement");
    }

    #[test]
    fn to_prefix_should_return_lowercase_roman_numeral_string_if_lowercase_roman_numeral_type(
    ) {
        panic!("TODO: Implement");
    }

    #[test]
    fn to_prefix_should_return_uppercase_roman_numeral_string_if_uppercase_roman_numeral_type(
    ) {
        panic!("TODO: Implement");
    }

    #[test]
    fn to_prefix_should_return_internal_value_if_other_type() {
        panic!("TODO: Implement");
    }
}
