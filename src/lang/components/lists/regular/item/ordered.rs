use super::ListItemContents;
use derive_more::Constructor;
use numerals::roman::Roman;
use serde::{Deserialize, Serialize};

/// Represents a suffix such as . or ) used after beginning of list item
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItemSuffix {
    Period,
    Paren,
}

impl ListItemSuffix {
    pub fn as_char(self) -> char {
        match self {
            Self::Period => '.',
            Self::Paren => ')',
        }
    }
}

/// Represents the type associated with an ordered item
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItemType {
    /// 1
    Number,
    /// a
    LowercaseAlphabet,
    /// A
    UppercaseAlphabet,
    /// i
    LowercaseRoman,
    /// I
    UppercaseRoman,
}

/// Represents an ordered item in a list
#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    pub item_type: ListItemType,
    pub suffix: ListItemSuffix,
    pub pos: usize,
    pub contents: ListItemContents,
}

impl ListItem {
    /// Allocates a new string representing the full prefix of the list item
    /// such as 1. or iii)
    pub fn to_prefix(&self) -> String {
        let mut base = match self.item_type {
            // NOTE: Numbers start at 1, not 0, so use base 1
            ListItemType::Number => (self.pos + 1).to_string(),
            ListItemType::LowercaseAlphabet => pos_to_alphabet(self.pos, true),
            ListItemType::UppercaseAlphabet => pos_to_alphabet(self.pos, false),
            // NOTE: Roman numerals start at 1, not 0, so use base 1
            ListItemType::LowercaseRoman => {
                format!("{:x}", Roman::from((self.pos + 1) as i16))
            }
            // NOTE: Roman numerals start at 1, not 0, so use base 1
            ListItemType::UppercaseRoman => {
                format!("{:X}", Roman::from((self.pos + 1) as i16))
            }
        };

        base.push(self.suffix.as_char());

        base
    }
}

impl Default for ListItem {
    fn default() -> Self {
        Self {
            item_type: ListItemType::Number,
            suffix: ListItemSuffix::Paren,
            pos: 0,
            contents: vec![],
        }
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
    use super::super::{ListItemContent, LC};
    use super::*;

    macro_rules! ordered_item {
        ($type:ident, $suffix:ident, $pos:expr, $contents:expr) => {
            ListItem::new(
                ListItemType::$type,
                ListItemSuffix::$suffix,
                $pos,
                $contents,
            )
        };
        ($type:ident, $suffix:ident, $pos:expr) => {
            ListItem::new(
                ListItemType::$type,
                ListItemSuffix::$suffix,
                $pos,
                vec![],
            )
        };
        ($type:ident, $suffix:ident) => {
            ListItem::new(
                ListItemType::$type,
                ListItemSuffix::$suffix,
                0,
                vec![],
            )
        };
        ($type:ident) => {
            ListItem::new(ListItemType::$type, ListItemSuffix::Paren, 0, vec![])
        };
    }

    fn make_content(text: &str) -> ListItemContents {
        vec![LC::from(ListItemContent::InlineContent(text.into()))]
    }

    #[test]
    fn pos_should_return_internal_position() {
        assert_eq!(ordered_item!(Number, Paren, 999).pos, 999);
        assert_eq!(ordered_item!(LowercaseAlphabet, Paren, 999).pos, 999);
        assert_eq!(ordered_item!(UppercaseAlphabet, Paren, 999).pos, 999);
        assert_eq!(ordered_item!(LowercaseRoman, Paren, 999).pos, 999);
        assert_eq!(ordered_item!(UppercaseRoman, Paren, 999).pos, 999);
    }

    #[test]
    fn contents_should_return_internal_contents() {
        assert_eq!(
            ordered_item!(Number, Paren, 0, make_content("test")).contents,
            &make_content("test")[..],
        );

        assert_eq!(
            ordered_item!(LowercaseAlphabet, Paren, 0, make_content("test"))
                .contents,
            &make_content("test")[..],
        );

        assert_eq!(
            ordered_item!(UppercaseAlphabet, Paren, 0, make_content("test"))
                .contents,
            &make_content("test")[..],
        );

        assert_eq!(
            ordered_item!(LowercaseRoman, Paren, 0, make_content("test"))
                .contents,
            &make_content("test")[..],
        );

        assert_eq!(
            ordered_item!(UppercaseRoman, Paren, 0, make_content("test"))
                .contents,
            &make_content("test")[..],
        );
    }

    #[test]
    fn suffix_should_return_the_associated_suffix() {
        assert_eq!(
            ordered_item!(Number, Period).suffix,
            ListItemSuffix::Period,
        );
        assert_eq!(ordered_item!(Number, Paren).suffix, ListItemSuffix::Paren,);
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Period).suffix,
            ListItemSuffix::Period,
        );
        assert_eq!(
            ordered_item!(LowercaseAlphabet, Paren).suffix,
            ListItemSuffix::Paren,
        );
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Period).suffix,
            ListItemSuffix::Period,
        );
        assert_eq!(
            ordered_item!(UppercaseAlphabet, Paren).suffix,
            ListItemSuffix::Paren,
        );
        assert_eq!(
            ordered_item!(LowercaseRoman, Period).suffix,
            ListItemSuffix::Period,
        );
        assert_eq!(
            ordered_item!(LowercaseRoman, Paren).suffix,
            ListItemSuffix::Paren,
        );
        assert_eq!(
            ordered_item!(UppercaseRoman, Period).suffix,
            ListItemSuffix::Period,
        );
        assert_eq!(
            ordered_item!(UppercaseRoman, Paren).suffix,
            ListItemSuffix::Paren,
        );
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
}
