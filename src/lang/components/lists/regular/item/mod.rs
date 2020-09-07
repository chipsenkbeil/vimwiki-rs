use super::{ListItemContent, ListItemContents};
use derive_more::{Constructor, From};
use numerals::roman::Roman;
use serde::{Deserialize, Serialize};

mod enhanced;
pub use enhanced::*;

/// Represents an item in a list
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct ListItem {
    pub item_type: ListItemType,
    pub suffix: ListItemSuffix,
    pub pos: usize,
    pub contents: ListItemContents,
}

impl ListItem {
    /// Indicates whether or not this list item represents an unordered item
    pub fn is_unordered(&self) -> bool {
        self.item_type.is_unordered()
    }

    /// Indicates whether or not this list item represents an ordered item
    pub fn is_ordered(&self) -> bool {
        self.item_type.is_ordered()
    }

    /// Allocates a new string to represent the prefix of this list item
    pub fn to_prefix(&self) -> String {
        self.item_type.to_prefix(self.pos, self.suffix)
    }
}

/// Represents a suffix such as . or ) used after beginning of list item
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItemSuffix {
    None,
    Period,
    Paren,
}

impl ListItemSuffix {
    pub fn as_char(self) -> Option<char> {
        match self {
            Self::Period => Some('.'),
            Self::Paren => Some(')'),
            Self::None => None,
        }
    }
}

impl Default for ListItemSuffix {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItemType {
    Ordered(OrderedListItemType),
    Unordered(UnorderedListItemType),
}

impl ListItemType {
    pub fn is_ordered(&self) -> bool {
        match self {
            Self::Ordered(_) => true,
            _ => false,
        }
    }

    pub fn is_unordered(&self) -> bool {
        match self {
            Self::Unordered(_) => true,
            _ => false,
        }
    }

    pub fn to_prefix(&self, pos: usize, suffix: ListItemSuffix) -> String {
        match self {
            Self::Ordered(x) => x.to_prefix(pos, suffix),
            Self::Unordered(x) => x.to_prefix(suffix),
        }
    }
}

impl Default for ListItemType {
    fn default() -> Self {
        Self::Unordered(UnorderedListItemType::default())
    }
}

/// Represents the type associated with an unordered item
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnorderedListItemType {
    /// -
    Hyphen,
    /// *
    Asterisk,
    /// Catchall
    Other(String),
}

impl UnorderedListItemType {
    /// Allocates a new string representing the full prefix of the list item
    /// such as - or *
    pub fn to_prefix(&self, suffix: ListItemSuffix) -> String {
        match &self {
            Self::Hyphen => String::from("-"),
            Self::Asterisk => String::from("*"),
            Self::Other(prefix) => {
                let mut base = prefix.to_string();

                if let Some(c) = suffix.as_char() {
                    base.push(c);
                }

                base
            }
        }
    }
}

impl Default for UnorderedListItemType {
    fn default() -> Self {
        Self::Hyphen
    }
}

/// Represents the type associated with an ordered item
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum OrderedListItemType {
    /// 1
    Number,
    /// #
    Pound,
    /// a
    LowercaseAlphabet,
    /// A
    UppercaseAlphabet,
    /// i
    LowercaseRoman,
    /// I
    UppercaseRoman,
}

impl OrderedListItemType {
    /// Allocates a new string representing the full prefix of the list item
    /// such as 1. or iii)
    pub fn to_prefix(&self, pos: usize, suffix: ListItemSuffix) -> String {
        let mut base = match self {
            // NOTE: Numbers start at 1, not 0, so use base 1
            Self::Number => (pos + 1).to_string(),
            Self::Pound => String::from("#"),
            Self::LowercaseAlphabet => pos_to_alphabet(pos, true),
            Self::UppercaseAlphabet => pos_to_alphabet(pos, false),
            // NOTE: Roman numerals start at 1, not 0, so use base 1
            Self::LowercaseRoman => {
                format!("{:x}", Roman::from((pos + 1) as i16))
            }
            // NOTE: Roman numerals start at 1, not 0, so use base 1
            Self::UppercaseRoman => {
                format!("{:X}", Roman::from((pos + 1) as i16))
            }
        };

        if let Some(c) = suffix.as_char() {
            base.push(c);
        }

        base
    }
}

impl Default for OrderedListItemType {
    fn default() -> Self {
        Self::Number
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
    use super::super::LC;
    use super::*;

    macro_rules! unordered_item {
        ($type:ident, $pos:expr, $content:expr) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::$type),
                ListItemSuffix::default(),
                $pos,
                $content.into(),
            )
        };
        ($type:ident, $pos:expr) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::$type),
                ListItemSuffix::default(),
                $pos,
                vec![].into(),
            )
        };
        ($type:ident) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::$type),
                ListItemSuffix::default(),
                0,
                vec![].into(),
            )
        };
    }

    macro_rules! ordered_item {
        ($type:ident, $suffix:ident, $pos:expr, $content:expr) => {
            ListItem::new(
                ListItemType::from(OrderedListItemType::$type),
                ListItemSuffix::$suffix,
                $pos,
                $content.into(),
            )
        };
        ($type:ident, $suffix:ident, $pos:expr) => {
            ListItem::new(
                ListItemType::from(OrderedListItemType::$type),
                ListItemSuffix::$suffix,
                $pos,
                vec![].into(),
            )
        };
        ($type:ident, $suffix:ident) => {
            ListItem::new(
                ListItemType::from(OrderedListItemType::$type),
                ListItemSuffix::$suffix,
                0,
                vec![].into(),
            )
        };
        ($type:ident) => {
            ListItem::new(
                ListItemType::from(OrderedListItemType::$type),
                ListItemSuffix::Paren,
                0,
                vec![].into(),
            )
        };
    }

    macro_rules! other_item {
        ($value:expr, $suffix:ident, $pos:expr, $content:expr) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::Other($value)),
                ListItemSuffix::$suffix,
                $pos,
                $content.into(),
            )
        };
        ($value:expr, $suffix:ident, $pos:expr) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::Other($value)),
                ListItemSuffix::$suffix,
                $pos,
                vec![].into(),
            )
        };
        ($value:expr, $suffix:ident) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::Other($value)),
                ListItemSuffix::$suffix,
                0,
                vec![].into(),
            )
        };
        ($value:expr) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::Other($value)),
                ListItemSuffix::default(),
                0,
                vec![].into(),
            )
        };
        () => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::Other(String::new())),
                ListItemSuffix::default(),
                0,
                vec![].into(),
            )
        };
    }

    fn make_content(text: &str) -> ListItemContents {
        vec![LC::from(ListItemContent::InlineContent(text.into()))].into()
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
        assert_eq!(unordered_item!(Hyphen, 999).pos, 999);
        assert_eq!(unordered_item!(Asterisk, 999).pos, 999);
        assert_eq!(ordered_item!(Number, Paren, 999).pos, 999);
        assert_eq!(ordered_item!(LowercaseAlphabet, Paren, 999).pos, 999);
        assert_eq!(ordered_item!(UppercaseAlphabet, Paren, 999).pos, 999);
        assert_eq!(ordered_item!(LowercaseRoman, Paren, 999).pos, 999);
        assert_eq!(ordered_item!(UppercaseRoman, Paren, 999).pos, 999);
        assert_eq!(other_item!(String::new(), None, 999).pos, 999);
    }

    #[test]
    fn contents_should_return_internal_contents() {
        assert_eq!(
            unordered_item!(Hyphen, 0, make_content("test")).contents,
            make_content("test"),
        );

        assert_eq!(
            unordered_item!(Asterisk, 0, make_content("test")).contents,
            make_content("test"),
        );

        assert_eq!(
            ordered_item!(Number, Paren, 0, make_content("test")).contents,
            make_content("test"),
        );

        assert_eq!(
            ordered_item!(LowercaseAlphabet, Paren, 0, make_content("test"))
                .contents,
            make_content("test"),
        );

        assert_eq!(
            ordered_item!(UppercaseAlphabet, Paren, 0, make_content("test"))
                .contents,
            make_content("test"),
        );

        assert_eq!(
            ordered_item!(LowercaseRoman, Paren, 0, make_content("test"))
                .contents,
            make_content("test"),
        );

        assert_eq!(
            ordered_item!(UppercaseRoman, Paren, 0, make_content("test"))
                .contents,
            make_content("test"),
        );

        assert_eq!(
            other_item!(String::new(), None, 0, make_content("test")).contents,
            make_content("test"),
        );
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
    fn to_prefix_should_return_internal_value_with_suffix_if_other_type() {
        assert_eq!(
            other_item!("prefix".to_string(), None, 0).to_prefix(),
            "prefix"
        );
        assert_eq!(
            other_item!("prefix".to_string(), None, 27).to_prefix(),
            "prefix"
        );
        assert_eq!(
            other_item!("prefix".to_string(), None, 99).to_prefix(),
            "prefix"
        );

        assert_eq!(
            other_item!("prefix".to_string(), Period, 0).to_prefix(),
            "prefix."
        );
        assert_eq!(
            other_item!("prefix".to_string(), Period, 27).to_prefix(),
            "prefix."
        );
        assert_eq!(
            other_item!("prefix".to_string(), Period, 99).to_prefix(),
            "prefix."
        );

        assert_eq!(
            other_item!("prefix".to_string(), Paren, 0).to_prefix(),
            "prefix)"
        );
        assert_eq!(
            other_item!("prefix".to_string(), Paren, 27).to_prefix(),
            "prefix)"
        );
        assert_eq!(
            other_item!("prefix".to_string(), Paren, 99).to_prefix(),
            "prefix)"
        );
    }
}
