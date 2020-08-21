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
            // NOTE: Numbers start at 1, not 0, so use base 1
            Self::Number { pos, .. } => (pos + 1).to_string(),
            Self::LowercaseAlphabet { pos, .. } => pos_to_alphabet(*pos, true),
            Self::UppercaseAlphabet { pos, .. } => pos_to_alphabet(*pos, false),
            // NOTE: Roman numerals start at 1, not 0, so use base 1
            Self::LowercaseRoman { pos, .. } => {
                format!("{:x}", Roman::from((pos + 1) as i16))
            }
            // NOTE: Roman numerals start at 1, not 0, so use base 1
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

    // Work our way backwards, starting with the last character and moving left
    loop {
        // Get closest character offset
        let offset = i % 26;
        if let Some(c) = std::char::from_u32(base + offset) {
            s.push(c);
        }

        // Remove closest character from position
        i -= offset;

        // If we have more to process, shift left one
        if i > 0 {
            i = (i / 26) - 1;
        } else {
            break;
        }
    }

    s.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! unordered_item {
        ($type:ident) => {
            RegularListItem::$type {
                pos: 999,
                contents: vec![RegularListItemContent::InlineContent(
                    "test".into(),
                )],
            }
        };
    }

    macro_rules! ordered_item {
        ($type:ident, $suffix:ident, $pos:expr) => {
            RegularListItem::$type {
                pos: $pos,
                suffix: RegularListItemSuffix::$suffix,
                contents: vec![RegularListItemContent::InlineContent(
                    "test".into(),
                )],
            }
        };
        ($type:ident, $suffix:ident) => {
            RegularListItem::$type {
                pos: 999,
                suffix: RegularListItemSuffix::$suffix,
                contents: vec![RegularListItemContent::InlineContent(
                    "test".into(),
                )],
            }
        };
        ($type:ident) => {
            RegularListItem::$type {
                pos: 999,
                suffix: RegularListItemSuffix::Period,
                contents: vec![RegularListItemContent::InlineContent(
                    "test".into(),
                )],
            }
        };
    }

    macro_rules! other_item {
        ($value:expr, $pos:expr) => {
            RegularListItem::Other {
                pos: $pos,
                value: String::from($value),
                contents: vec![RegularListItemContent::InlineContent(
                    "test".into(),
                )],
            }
        };
        ($value:expr) => {
            RegularListItem::Other {
                pos: 999,
                value: String::from($value),
                contents: vec![RegularListItemContent::InlineContent(
                    "test".into(),
                )],
            }
        };
        () => {
            RegularListItem::Other {
                pos: 999,
                value: String::new(),
                contents: vec![RegularListItemContent::InlineContent(
                    "test".into(),
                )],
            }
        };
    }

    #[test]
    fn is_ordered_should_return_true_if_ordered_prefix() {
        assert!(
            ordered_item!(Number).is_ordered(),
            "Number should be ordered"
        );

        assert!(
            ordered_item!(LowercaseAlphabet).is_ordered(),
            "Lowercase alphabet should be ordered"
        );

        assert!(
            ordered_item!(UppercaseAlphabet).is_ordered(),
            "Uppercase alphabet should be ordered"
        );

        assert!(
            ordered_item!(LowercaseRoman).is_ordered(),
            "Lowercase roman numerals should be ordered"
        );

        assert!(
            ordered_item!(UppercaseRoman).is_ordered(),
            "Uppercase roman numerals should be ordered"
        );
    }

    #[test]
    fn is_ordered_should_return_false_if_unordered_prefix() {
        assert!(
            !unordered_item!(Hyphen).is_ordered(),
            "Hyphen should not be ordered"
        );

        assert!(
            !unordered_item!(Asterisk).is_ordered(),
            "Asterisk should not be ordered"
        );

        assert!(!other_item!().is_ordered(), "Other should not be ordered");
    }

    #[test]
    fn is_unordered_should_return_true_if_unordered_prefix() {
        assert!(
            unordered_item!(Hyphen).is_unordered(),
            "Hyphen should be unordered"
        );

        assert!(
            unordered_item!(Asterisk).is_unordered(),
            "Asterisk should be unordered"
        );

        assert!(other_item!().is_unordered(), "Other should be unordered");
    }

    #[test]
    fn is_unordered_should_return_false_if_ordered_prefix() {
        assert!(
            !ordered_item!(Number).is_unordered(),
            "Number should not be unordered"
        );

        assert!(
            !ordered_item!(LowercaseAlphabet).is_unordered(),
            "Lowercase alphabet should not be unordered"
        );

        assert!(
            !ordered_item!(UppercaseAlphabet).is_unordered(),
            "Uppercase alphabet should not be unordered"
        );

        assert!(
            !ordered_item!(LowercaseRoman).is_unordered(),
            "Lowercase roman numerals should not be unordered"
        );

        assert!(
            !ordered_item!(UppercaseRoman).is_unordered(),
            "Uppercase roman numerals should not be unordered"
        );
    }

    #[test]
    fn pos_should_return_internal_position() {
        assert_eq!(unordered_item!(Hyphen).pos(), 999);
        assert_eq!(unordered_item!(Asterisk).pos(), 999);
        assert_eq!(ordered_item!(Number).pos(), 999);
        assert_eq!(ordered_item!(LowercaseAlphabet).pos(), 999);
        assert_eq!(ordered_item!(UppercaseAlphabet).pos(), 999);
        assert_eq!(ordered_item!(LowercaseRoman).pos(), 999);
        assert_eq!(ordered_item!(UppercaseRoman).pos(), 999);
        assert_eq!(other_item!().pos(), 999);
    }

    #[test]
    fn contents_should_return_internal_contents() {
        assert_eq!(
            unordered_item!(Hyphen).contents(),
            &[RegularListItemContent::InlineContent("test".into())],
        );

        assert_eq!(
            unordered_item!(Asterisk).contents(),
            &[RegularListItemContent::InlineContent("test".into())],
        );

        assert_eq!(
            ordered_item!(Number).contents(),
            &[RegularListItemContent::InlineContent("test".into())],
        );

        assert_eq!(
            ordered_item!(LowercaseAlphabet).contents(),
            &[RegularListItemContent::InlineContent("test".into())],
        );

        assert_eq!(
            ordered_item!(UppercaseAlphabet).contents(),
            &[RegularListItemContent::InlineContent("test".into())],
        );

        assert_eq!(
            ordered_item!(LowercaseRoman).contents(),
            &[RegularListItemContent::InlineContent("test".into())],
        );

        assert_eq!(
            ordered_item!(UppercaseRoman).contents(),
            &[RegularListItemContent::InlineContent("test".into())],
        );

        assert_eq!(
            other_item!().contents(),
            &[RegularListItemContent::InlineContent("test".into())],
        );
    }

    #[test]
    fn suffix_should_return_some_suffix_if_applicable() {
        assert_eq!(
            ordered_item!(Number, Period).suffix(),
            Some(RegularListItemSuffix::Period),
        );
        assert_eq!(
            ordered_item!(Number, Paren).suffix(),
            Some(RegularListItemSuffix::Paren),
        );
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Period).suffix(),
            Some(RegularListItemSuffix::Period),
        );
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Paren).suffix(),
            Some(RegularListItemSuffix::Paren),
        );
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Period).suffix(),
            Some(RegularListItemSuffix::Period),
        );
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Paren).suffix(),
            Some(RegularListItemSuffix::Paren),
        );
        assert_eq!(
            ordered_item!(LowercaseRoman, Period).suffix(),
            Some(RegularListItemSuffix::Period),
        );
        assert_eq!(
            ordered_item!(LowercaseRoman, Paren).suffix(),
            Some(RegularListItemSuffix::Paren),
        );
        assert_eq!(
            ordered_item!(UppercaseRoman, Period).suffix(),
            Some(RegularListItemSuffix::Period),
        );
        assert_eq!(
            ordered_item!(UppercaseRoman, Paren).suffix(),
            Some(RegularListItemSuffix::Paren),
        );
    }

    #[test]
    fn suffix_should_return_none_if_not_applicable() {
        assert_eq!(unordered_item!(Hyphen).suffix(), None);
        assert_eq!(unordered_item!(Asterisk).suffix(), None);
        assert_eq!(other_item!().suffix(), None);
    }

    #[test]
    fn to_prefix_should_return_hyphen_if_hyphen_type() {
        assert_eq!(unordered_item!(Hyphen).to_prefix(), "-");
    }

    #[test]
    fn to_prefix_should_return_asterisk_if_asterisk_type() {
        assert_eq!(unordered_item!(Asterisk).to_prefix(), "*");
    }

    #[test]
    fn to_prefix_should_return_base_1_position_if_number_type() {
        assert_eq!(ordered_item!(Number, Paren, 0).to_prefix(), "1)");
        assert_eq!(ordered_item!(Number, Period, 0).to_prefix(), "1.");
        assert_eq!(ordered_item!(Number, Paren, 27).to_prefix(), "28)");
        assert_eq!(ordered_item!(Number, Period, 27).to_prefix(), "28.");
        assert_eq!(ordered_item!(Number, Paren, 704).to_prefix(), "705)");
        assert_eq!(ordered_item!(Number, Period, 704).to_prefix(), "705.");
    }

    #[test]
    fn to_prefix_should_return_lowercase_alphabetic_string_if_lowercase_alphabetic_type(
    ) {
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Paren, 0).to_prefix(),
            "a)"
        );
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Period, 0).to_prefix(),
            "a."
        );
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Paren, 27).to_prefix(),
            "ab)"
        );
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Period, 27).to_prefix(),
            "ab."
        );
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Paren, 730).to_prefix(),
            "abc)"
        );
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Period, 730).to_prefix(),
            "abc."
        );
    }

    #[test]
    fn to_prefix_should_return_uppercase_alphabetic_string_if_uppercase_alphabetic_type(
    ) {
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Paren, 0).to_prefix(),
            "A)"
        );
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Period, 0).to_prefix(),
            "A."
        );
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Paren, 27).to_prefix(),
            "AB)"
        );
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Period, 27).to_prefix(),
            "AB."
        );
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Paren, 730).to_prefix(),
            "ABC)"
        );
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Period, 730).to_prefix(),
            "ABC."
        );
    }

    #[test]
    fn to_prefix_should_return_lowercase_roman_numeral_string_if_lowercase_roman_numeral_type(
    ) {
        assert_eq!(ordered_item!(LowercaseRoman, Paren, 0).to_prefix(), "i)");
        assert_eq!(ordered_item!(LowercaseRoman, Period, 0).to_prefix(), "i.");
        assert_eq!(
            ordered_item!(LowercaseRoman, Paren, 24).to_prefix(),
            "xxv)"
        );
        assert_eq!(
            ordered_item!(LowercaseRoman, Period, 24).to_prefix(),
            "xxv."
        );
        assert_eq!(
            ordered_item!(LowercaseRoman, Paren, 704).to_prefix(),
            "dccv)"
        );
        assert_eq!(
            ordered_item!(LowercaseRoman, Period, 704).to_prefix(),
            "dccv."
        );
    }

    #[test]
    fn to_prefix_should_return_uppercase_roman_numeral_string_if_uppercase_roman_numeral_type(
    ) {
        assert_eq!(ordered_item!(UppercaseRoman, Paren, 0).to_prefix(), "I)");
        assert_eq!(ordered_item!(UppercaseRoman, Period, 0).to_prefix(), "I.");
        assert_eq!(
            ordered_item!(UppercaseRoman, Paren, 24).to_prefix(),
            "XXV)"
        );
        assert_eq!(
            ordered_item!(UppercaseRoman, Period, 24).to_prefix(),
            "XXV."
        );
        assert_eq!(
            ordered_item!(UppercaseRoman, Paren, 704).to_prefix(),
            "DCCV)"
        );
        assert_eq!(
            ordered_item!(UppercaseRoman, Period, 704).to_prefix(),
            "DCCV."
        );
    }

    #[test]
    fn to_prefix_should_return_internal_value_if_other_type() {
        assert_eq!(other_item!("prefix", 0).to_prefix(), "prefix");
        assert_eq!(other_item!("prefix", 27).to_prefix(), "prefix");
        assert_eq!(other_item!("prefix", 704).to_prefix(), "prefix");
    }
}
