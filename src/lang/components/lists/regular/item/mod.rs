use super::{ListItemContent, ListItemContents, LC};
use derive_more::From;
use serde::{Deserialize, Serialize};

mod enhanced;
pub use enhanced::{EnhancedListItem, EnhancedListItemAttribute};

mod ordered;
pub use ordered::{
    ListItem as OrderedListItem, ListItemSuffix as OrderedListItemSuffix,
    ListItemType as OrderedListItemType,
};

mod unordered;
pub use unordered::{
    ListItem as UnorderedListItem, ListItemType as UnorderedListItemType,
};

/// Represents supported prefix types for a list item
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItem {
    Ordered(OrderedListItem),
    Unordered(UnorderedListItem),
}

impl ListItem {
    /// Indicates whether or not this list item represents an unordered item
    pub fn is_unordered(&self) -> bool {
        match self {
            Self::Ordered(_) => false,
            Self::Unordered(_) => true,
        }
    }

    /// Indicates whether or not this list item represents an ordered item
    pub fn is_ordered(&self) -> bool {
        match self {
            Self::Ordered(_) => true,
            Self::Unordered(_) => false,
        }
    }

    pub fn pos(&self) -> usize {
        match self {
            Self::Ordered(item) => item.pos,
            Self::Unordered(item) => item.pos,
        }
    }

    pub fn contents(&self) -> &[LC<ListItemContent>] {
        match self {
            Self::Ordered(item) => &item.contents[..],
            Self::Unordered(item) => &item.contents[..],
        }
    }

    pub fn to_prefix(&self) -> String {
        match self {
            Self::Ordered(item) => item.to_prefix(),
            Self::Unordered(item) => item.to_prefix(),
        }
    }
}

impl Default for ListItem {
    fn default() -> Self {
        Self::Unordered(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! unordered_item {
        ($type:ident, $pos:expr, $content:expr) => {
            ListItem::Unordered(UnorderedListItem::new(
                UnorderedListItemType::$type,
                $pos,
                $content,
            ))
        };
        ($type:ident, $pos:expr) => {
            ListItem::Unordered(UnorderedListItem::new(
                UnorderedListItemType::$type,
                $pos,
                vec![],
            ))
        };
        ($type:ident) => {
            ListItem::Unordered(UnorderedListItem::new(
                UnorderedListItemType::$type,
                0,
                vec![],
            ))
        };
    }

    macro_rules! ordered_item {
        ($type:ident, $suffix:ident, $pos:expr, $content:expr) => {
            ListItem::Ordered(OrderedListItem::new(
                OrderedListItemType::$type,
                OrderedListItemSuffix::$suffix,
                $pos,
                $content,
            ))
        };
        ($type:ident, $suffix:ident, $pos:expr) => {
            ListItem::Ordered(OrderedListItem::new(
                OrderedListItemType::$type,
                OrderedListItemSuffix::$suffix,
                $pos,
                vec![],
            ))
        };
        ($type:ident, $suffix:ident) => {
            ListItem::Ordered(OrderedListItem::new(
                OrderedListItemType::$type,
                OrderedListItemSuffix::$suffix,
                0,
                vec![],
            ))
        };
        ($type:ident) => {
            ListItem::Ordered(OrderedListItem::new(
                OrderedListItemType::$type,
                OrderedListItemSuffix::Paren,
                0,
                vec![],
            ))
        };
    }

    macro_rules! other_item {
        ($value:expr, $pos:expr, $content:expr) => {
            ListItem::Unordered(UnorderedListItem::new(
                UnorderedListItemType::Other($value),
                $pos,
                $content,
            ))
        };
        ($value:expr, $pos:expr) => {
            ListItem::Unordered(UnorderedListItem::new(
                UnorderedListItemType::Other($value),
                $pos,
                vec![],
            ))
        };
        ($value:expr) => {
            ListItem::Unordered(UnorderedListItem::new(
                UnorderedListItemType::Other($value),
                0,
                vec![],
            ))
        };
        () => {
            ListItem::Unordered(UnorderedListItem::new(
                UnorderedListItemType::Other(String::new()),
                0,
                vec![],
            ))
        };
    }

    fn make_content(text: &str) -> ListItemContents {
        vec![ListItemContent::InlineContent(text.into())]
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
        assert_eq!(unordered_item!(Hyphen, 999).pos(), 999);
        assert_eq!(unordered_item!(Asterisk, 999).pos(), 999);
        assert_eq!(ordered_item!(Number, Paren, 999).pos(), 999);
        assert_eq!(ordered_item!(LowercaseAlphabet, Paren, 999).pos(), 999);
        assert_eq!(ordered_item!(UppercaseAlphabet, Paren, 999).pos(), 999);
        assert_eq!(ordered_item!(LowercaseRoman, Paren, 999).pos(), 999);
        assert_eq!(ordered_item!(UppercaseRoman, Paren, 999).pos(), 999);
        assert_eq!(other_item!(String::new(), 999).pos(), 999);
    }

    #[test]
    fn contents_should_return_internal_contents() {
        assert_eq!(
            unordered_item!(Hyphen, 0, make_content("test")).contents(),
            &make_content("test")[..]
        );

        assert_eq!(
            unordered_item!(Asterisk, 0, make_content("test")).contents(),
            &make_content("test")[..]
        );

        assert_eq!(
            ordered_item!(Number, Paren, 0, make_content("test")).contents(),
            &make_content("test")[..]
        );

        assert_eq!(
            ordered_item!(LowercaseAlphabet, Paren, 0, make_content("test"))
                .contents(),
            &make_content("test")[..],
        );

        assert_eq!(
            ordered_item!(UppercaseAlphabet, Paren, 0, make_content("test"))
                .contents(),
            &make_content("test")[..],
        );

        assert_eq!(
            ordered_item!(LowercaseRoman, Paren, 0, make_content("test"))
                .contents(),
            &make_content("test")[..],
        );

        assert_eq!(
            ordered_item!(UppercaseRoman, Paren, 0, make_content("test"))
                .contents(),
            &make_content("test")[..],
        );

        assert_eq!(
            other_item!(String::new(), 0, make_content("test")).contents(),
            &make_content("test")[..]
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
    fn to_prefix_should_return_internal_value_if_other_type() {
        assert_eq!(other_item!("prefix".to_string(), 0).to_prefix(), "prefix");
        assert_eq!(other_item!("prefix".to_string(), 27).to_prefix(), "prefix");
        assert_eq!(other_item!("prefix".to_string(), 99).to_prefix(), "prefix");
    }
}
