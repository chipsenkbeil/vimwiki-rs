use crate::{
    lang::elements::{
        Element, IntoChildren, ListItemContent, ListItemContents, Located,
    },
    StrictEq,
};
use derive_more::{Constructor, From};
use numerals::roman::Roman;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Represents an item in a list
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct ListItem<'a> {
    pub item_type: ListItemType<'a>,
    pub suffix: ListItemSuffix,
    pub pos: usize,
    pub contents: ListItemContents<'a>,
    pub attributes: ListItemAttributes,
}

impl ListItem<'_> {
    pub fn to_borrowed(&self) -> ListItem {
        ListItem {
            item_type: self.item_type.as_borrowed(),
            suffix: self.suffix,
            pos: self.pos,
            contents: self.contents.to_borrowed(),
            attributes: self.attributes,
        }
    }

    pub fn into_owned(self) -> ListItem<'static> {
        ListItem {
            item_type: self.item_type.into_owned(),
            suffix: self.suffix,
            pos: self.pos,
            contents: self.contents.into_owned(),
            attributes: self.attributes,
        }
    }
}

impl<'a> IntoChildren for ListItem<'a> {
    type Child = Located<Element<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.contents.into_children()
    }
}

impl<'a> StrictEq for ListItem<'a> {
    /// Performs a strict_eq check against eqivalent variants
    fn strict_eq(&self, other: &Self) -> bool {
        self.item_type.strict_eq(&other.item_type)
            && self.suffix.strict_eq(&other.suffix)
            && self.pos == other.pos
            && self.contents.strict_eq(&other.contents)
            && self.attributes.strict_eq(&other.attributes)
    }
}

impl<'a> ListItem<'a> {
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

    /// Whether or not this list item has TODO information
    pub fn is_todo(&self) -> bool {
        self.attributes.todo_status.is_some()
    }

    /// Returns percent complete in form of 0.0 == 0% and 1.0 == 100%. This
    /// is a calculated percentage based on the sublist items (if there are
    /// any) or the item itself.
    ///
    /// This will search through all sub list items, check if they have
    /// todo properties, and calculate a sum. If none of the sublists or
    /// any series of nested sublists contains todo items that are NOT
    /// rejected and this item does also not have todo progress,
    /// None will be returned.
    pub fn compute_todo_progress(&self) -> Option<f32> {
        self.contents
            .iter()
            .fold(None, |acc, c| match c.as_inner() {
                ListItemContent::InlineContent(_) => acc,
                ListItemContent::List(list) => {
                    let (mut sum, mut count) =
                        list.items.iter().fold((0.0, 0), |acc, item| {
                            // NOTE: This is a recursive call that is NOT
                            //       tail recursive, but I do not want to
                            //       spend the time needed to translate it
                            //       into an interative approach given we
                            //       need to calculate the leaf todos before
                            //       determining the progress of the current
                            //       todo list item
                            if let Some(p) =
                                item.as_inner().compute_todo_progress()
                            {
                                (acc.0 + p, acc.1 + 1)
                            } else {
                                acc
                            }
                        });

                    if let Some((acc_sum, acc_count)) = acc {
                        sum += acc_sum;
                        count += acc_count;
                    }

                    if count > 0 {
                        Some((sum, count))
                    } else {
                        None
                    }
                }
            })
            .map(|(sum, count)| sum / count as f32)
            .or_else(|| self.to_todo_progress())
    }

    /// Returns progress based on current todo status, or yields None if
    /// not a todo or is a rejected todo.
    ///
    /// Incomplete              == 0%
    /// Partially Complete 1    == 25%
    /// Partially Complete 2    == 50%
    /// Partially Complete 3    == 75%
    /// Completed               == 100%
    #[inline]
    fn to_todo_progress(&self) -> Option<f32> {
        if self.is_todo() && !self.is_todo_rejected() {
            Some(if self.is_todo_partially_complete_1() {
                0.25
            } else if self.is_todo_partially_complete_2() {
                0.5
            } else if self.is_todo_partially_complete_3() {
                0.75
            } else if self.is_todo_complete() {
                1.0
            } else {
                0.0
            })
        } else {
            None
        }
    }

    /// Indicates whether or not this element is incomplete, meaning no progress
    pub fn is_todo_incomplete(&self) -> bool {
        matches!(
            self.attributes.todo_status,
            Some(ListItemTodoStatus::Incomplete)
        )
    }

    /// Indicates whether or not this element is partially complete (any range)
    pub fn is_todo_partially_complete(&self) -> bool {
        self.is_todo_partially_complete_1()
            || self.is_todo_partially_complete_2()
            || self.is_todo_partially_complete_3()
    }

    /// Indicates whether or not this element is partially complete (1-33%)
    pub fn is_todo_partially_complete_1(&self) -> bool {
        matches!(
            self.attributes.todo_status,
            Some(ListItemTodoStatus::PartiallyComplete1)
        )
    }

    /// Indicates whether or not this element is partially complete (34-66%)
    pub fn is_todo_partially_complete_2(&self) -> bool {
        matches!(
            self.attributes.todo_status,
            Some(ListItemTodoStatus::PartiallyComplete2)
        )
    }

    /// Indicates whether or not this element is partially complete (67-99%)
    pub fn is_todo_partially_complete_3(&self) -> bool {
        matches!(
            self.attributes.todo_status,
            Some(ListItemTodoStatus::PartiallyComplete3)
        )
    }

    /// Indicates whether or not this element is complete
    pub fn is_todo_complete(&self) -> bool {
        matches!(
            self.attributes.todo_status,
            Some(ListItemTodoStatus::Complete)
        )
    }

    /// Indicates whether or not this element is rejected
    pub fn is_todo_rejected(&self) -> bool {
        matches!(
            self.attributes.todo_status,
            Some(ListItemTodoStatus::Rejected)
        )
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

impl StrictEq for ListItemSuffix {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItemType<'a> {
    Ordered(OrderedListItemType),
    Unordered(UnorderedListItemType<'a>),
}

impl ListItemType<'_> {
    pub fn as_borrowed(&self) -> ListItemType {
        match self {
            Self::Unordered(ref x) => ListItemType::from(x.as_borrowed()),
            Self::Ordered(ref x) => ListItemType::from(*x),
        }
    }

    pub fn into_owned(self) -> ListItemType<'static> {
        match self {
            Self::Unordered(x) => ListItemType::from(x.into_owned()),
            Self::Ordered(x) => ListItemType::from(x),
        }
    }
}

impl<'a> ListItemType<'a> {
    pub fn is_ordered(&self) -> bool {
        matches!(self, Self::Ordered(_))
    }

    pub fn is_unordered(&self) -> bool {
        matches!(self, Self::Unordered(_))
    }

    pub fn to_prefix(&self, pos: usize, suffix: ListItemSuffix) -> String {
        match self {
            Self::Ordered(x) => x.to_prefix(pos, suffix),
            Self::Unordered(x) => x.to_prefix(suffix),
        }
    }
}

impl<'a> Default for ListItemType<'a> {
    fn default() -> Self {
        Self::Unordered(UnorderedListItemType::default())
    }
}

impl<'a> StrictEq for ListItemType<'a> {
    /// Performs strict_eq on matching inner variants
    fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ordered(x), Self::Ordered(y)) => x.strict_eq(y),
            (Self::Unordered(x), Self::Unordered(y)) => x.strict_eq(y),
            _ => false,
        }
    }
}

/// Represents the type associated with an unordered item
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnorderedListItemType<'a> {
    /// -
    Hyphen,
    /// *
    Asterisk,
    /// Catchall
    Other(Cow<'a, str>),
}

impl UnorderedListItemType<'_> {
    pub fn as_borrowed(&self) -> UnorderedListItemType {
        use self::Cow::*;

        match self {
            Self::Other(ref x) => {
                UnorderedListItemType::Other(Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                }))
            }
            Self::Hyphen => UnorderedListItemType::Hyphen,
            Self::Asterisk => UnorderedListItemType::Asterisk,
        }
    }

    pub fn into_owned(self) -> UnorderedListItemType<'static> {
        match self {
            Self::Other(x) => {
                UnorderedListItemType::Other(Cow::from(x.into_owned()))
            }
            Self::Hyphen => UnorderedListItemType::Hyphen,
            Self::Asterisk => UnorderedListItemType::Asterisk,
        }
    }
}

impl<'a> StrictEq for UnorderedListItemType<'a> {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> UnorderedListItemType<'a> {
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

impl<'a> Default for UnorderedListItemType<'a> {
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

impl StrictEq for OrderedListItemType {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// Represents the todo status for a list item
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItemTodoStatus {
    /// Flags list item as a TODO item that has not been completed
    Incomplete,

    /// Flags list item as a TODO item that is partially complete (1-33%)
    PartiallyComplete1,

    /// Flags list item as a TODO item that is partially complete (34-66%)
    PartiallyComplete2,

    /// Flags list item as a TODO item that is partially complete (67-99%)
    PartiallyComplete3,

    /// Flags list item as a TODO item that is complete
    Complete,

    /// Flags list item as a TODO item that has been rejected
    Rejected,
}

impl StrictEq for ListItemTodoStatus {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// Represents additional attributes associated with a list item
#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct ListItemAttributes {
    /// The TODO status for a list item, if it has been associated with TODO
    pub todo_status: Option<ListItemTodoStatus>,
}

impl StrictEq for ListItemAttributes {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{List, ListItemContent, Located, Text};

    macro_rules! unordered_item {
        ($type:ident, $pos:expr, $content:expr) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::$type),
                ListItemSuffix::default(),
                $pos,
                $content.into(),
                Default::default(),
            )
        };
        ($type:ident, $pos:expr) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::$type),
                ListItemSuffix::default(),
                $pos,
                vec![].into(),
                Default::default(),
            )
        };
        ($type:ident) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::$type),
                ListItemSuffix::default(),
                0,
                vec![].into(),
                Default::default(),
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
                Default::default(),
            )
        };
        ($type:ident, $suffix:ident, $pos:expr) => {
            ListItem::new(
                ListItemType::from(OrderedListItemType::$type),
                ListItemSuffix::$suffix,
                $pos,
                vec![].into(),
                Default::default(),
            )
        };
        ($type:ident, $suffix:ident) => {
            ListItem::new(
                ListItemType::from(OrderedListItemType::$type),
                ListItemSuffix::$suffix,
                0,
                vec![].into(),
                Default::default(),
            )
        };
        ($type:ident) => {
            ListItem::new(
                ListItemType::from(OrderedListItemType::$type),
                ListItemSuffix::Paren,
                0,
                vec![].into(),
                Default::default(),
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
                Default::default(),
            )
        };
        ($value:expr, $suffix:ident, $pos:expr) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::Other($value)),
                ListItemSuffix::$suffix,
                $pos,
                vec![].into(),
                Default::default(),
            )
        };
        ($value:expr, $suffix:ident) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::Other($value)),
                ListItemSuffix::$suffix,
                0,
                vec![].into(),
                Default::default(),
            )
        };
        ($value:expr) => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::Other($value)),
                ListItemSuffix::default(),
                0,
                vec![].into(),
                Default::default(),
            )
        };
        () => {
            ListItem::new(
                ListItemType::from(UnorderedListItemType::Other(Cow::from(""))),
                ListItemSuffix::default(),
                0,
                vec![].into(),
                Default::default(),
            )
        };
    }

    fn todo_list_item(todo_status: ListItemTodoStatus) -> ListItem<'static> {
        let mut item = ListItem::default();

        item.attributes.todo_status = Some(todo_status);

        item
    }

    macro_rules! todo_list_item {
        ($type:ident) => {
            todo_list_item(ListItemTodoStatus::$type)
        };
        ($type:ident, $($child:expr),+) => {
            ListItem::new(
                Default::default(),
                Default::default(),
                0,
                vec![From::from(ListItemContent::List(
                    List::new(vec![$($child),+])
                ))].into(),
                ListItemAttributes {
                    todo_status: Some(ListItemTodoStatus::$type),
                }
            )
        };
    }

    fn make_content(text: &str) -> ListItemContents {
        let le_text = Located::from(Text::from(text));
        vec![Located::from(ListItemContent::InlineContent(
            le_text.into(),
        ))]
        .into()
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
        assert_eq!(other_item!(Cow::from(""), None, 999).pos, 999);
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
            other_item!(Cow::from(""), None, 0, make_content("test")).contents,
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
            other_item!(Cow::from("prefix"), None, 0).to_prefix(),
            "prefix"
        );
        assert_eq!(
            other_item!(Cow::from("prefix"), None, 27).to_prefix(),
            "prefix"
        );
        assert_eq!(
            other_item!(Cow::from("prefix"), None, 99).to_prefix(),
            "prefix"
        );

        assert_eq!(
            other_item!(Cow::from("prefix"), Period, 0).to_prefix(),
            "prefix."
        );
        assert_eq!(
            other_item!(Cow::from("prefix"), Period, 27).to_prefix(),
            "prefix."
        );
        assert_eq!(
            other_item!(Cow::from("prefix"), Period, 99).to_prefix(),
            "prefix."
        );

        assert_eq!(
            other_item!(Cow::from("prefix"), Paren, 0).to_prefix(),
            "prefix)"
        );
        assert_eq!(
            other_item!(Cow::from("prefix"), Paren, 27).to_prefix(),
            "prefix)"
        );
        assert_eq!(
            other_item!(Cow::from("prefix"), Paren, 99).to_prefix(),
            "prefix)"
        );
    }

    #[test]
    fn is_todo_should_return_true_if_contains_any_todo_attribute() {
        assert!(todo_list_item!(Incomplete).is_todo());
        assert!(todo_list_item!(PartiallyComplete1).is_todo());
        assert!(todo_list_item!(PartiallyComplete2).is_todo());
        assert!(todo_list_item!(PartiallyComplete3).is_todo());
        assert!(todo_list_item!(Complete).is_todo());
        assert!(todo_list_item!(Rejected).is_todo());
    }

    #[test]
    fn is_todo_should_return_false_if_does_not_contain_any_todo_attribute() {
        assert!(!ListItem::default().is_todo());
    }

    #[test]
    fn compute_todo_progress_should_use_own_progress_if_no_children() {
        assert_eq!(
            Some(0.0),
            todo_list_item!(Incomplete).compute_todo_progress()
        );
        assert_eq!(
            Some(0.25),
            todo_list_item!(PartiallyComplete1).compute_todo_progress()
        );
        assert_eq!(
            Some(0.5),
            todo_list_item!(PartiallyComplete2).compute_todo_progress()
        );
        assert_eq!(
            Some(0.75),
            todo_list_item!(PartiallyComplete3).compute_todo_progress()
        );
        assert_eq!(
            Some(1.0),
            todo_list_item!(Complete).compute_todo_progress()
        );
        assert_eq!(None, todo_list_item!(Rejected).compute_todo_progress());
        assert_eq!(None, ListItem::default().compute_todo_progress());
    }

    #[test]
    fn compute_todo_progress_should_use_children_progress_if_has_children() {
        // - [ ] <CALEULATING>
        //     - [-] N/A
        //     - [X] 100%
        //     - [.] 25%
        //     - [o] 50%
        //     - [O] 75%
        //     - [ ] 0%
        assert_eq!(
            todo_list_item!(
                Incomplete,
                Located::from(todo_list_item!(Rejected)),
                Located::from(todo_list_item!(Complete)),
                Located::from(todo_list_item!(PartiallyComplete1)),
                Located::from(todo_list_item!(PartiallyComplete2)),
                Located::from(todo_list_item!(PartiallyComplete3)),
                Located::from(todo_list_item!(Incomplete))
            )
            .compute_todo_progress(),
            Some((1.0 + 0.25 + 0.5 + 0.75 + 0.0) / 5.0)
        );
    }

    #[test]
    fn compute_todo_progress_should_support_deeper_children() {
        assert_eq!(
            todo_list_item!(
                Incomplete,
                Located::from(todo_list_item!(
                    Rejected,
                    Located::from(todo_list_item!(Rejected)),
                    Located::from(todo_list_item!(Complete)),
                    Located::from(todo_list_item!(PartiallyComplete1)),
                    Located::from(todo_list_item!(PartiallyComplete2)),
                    Located::from(todo_list_item!(PartiallyComplete3)),
                    Located::from(todo_list_item!(Incomplete))
                ))
            )
            .compute_todo_progress(),
            Some((1.0 + 0.25 + 0.5 + 0.75 + 0.0) / 5.0)
        );
    }

    #[test]
    fn is_todo_incomplete_should_return_true_if_is_incomplete() {
        assert!(todo_list_item!(Incomplete).is_todo_incomplete());
    }

    #[test]
    fn is_todo_incomplete_should_return_false_if_not_incomplete() {
        assert!(!todo_list_item!(PartiallyComplete1).is_todo_incomplete());
        assert!(!todo_list_item!(PartiallyComplete2).is_todo_incomplete());
        assert!(!todo_list_item!(PartiallyComplete3).is_todo_incomplete());
        assert!(!todo_list_item!(Complete).is_todo_incomplete());
        assert!(!todo_list_item!(Rejected).is_todo_incomplete());
        assert!(!ListItem::default().is_todo_incomplete());
    }

    #[test]
    fn is_todo_partially_complete_should_return_true_if_is_any_partially_complete(
    ) {
        assert!(
            todo_list_item!(PartiallyComplete1).is_todo_partially_complete()
        );
        assert!(
            todo_list_item!(PartiallyComplete2).is_todo_partially_complete()
        );
        assert!(
            todo_list_item!(PartiallyComplete3).is_todo_partially_complete()
        );
    }

    #[test]
    fn is_todo_partially_complete_should_return_false_if_not_all_partially_completes(
    ) {
        assert!(!todo_list_item!(Incomplete).is_todo_partially_complete());
        assert!(!todo_list_item!(Complete).is_todo_partially_complete());
        assert!(!todo_list_item!(Rejected).is_todo_partially_complete());
        assert!(!ListItem::default().is_todo_partially_complete());
    }

    #[test]
    fn is_todo_partially_complete_1_should_return_true_if_is_partially_complete_1(
    ) {
        assert!(
            todo_list_item!(PartiallyComplete1).is_todo_partially_complete_1()
        );
    }

    #[test]
    fn is_todo_partially_complete_1_should_return_false_if_not_partially_complete_1(
    ) {
        assert!(!todo_list_item!(Incomplete).is_todo_partially_complete_1());
        assert!(
            !todo_list_item!(PartiallyComplete2).is_todo_partially_complete_1()
        );
        assert!(
            !todo_list_item!(PartiallyComplete3).is_todo_partially_complete_1()
        );
        assert!(!todo_list_item!(Complete).is_todo_partially_complete_1());
        assert!(!todo_list_item!(Rejected).is_todo_partially_complete_1());
        assert!(!ListItem::default().is_todo_partially_complete_1());
    }

    #[test]
    fn is_todo_partially_complete_2_should_return_true_if_is_partially_complete_2(
    ) {
        assert!(
            todo_list_item!(PartiallyComplete2).is_todo_partially_complete_2()
        );
    }

    #[test]
    fn is_todo_partially_complete_2_should_return_false_if_not_partially_complete_2(
    ) {
        assert!(!todo_list_item!(Incomplete).is_todo_partially_complete_2());
        assert!(
            !todo_list_item!(PartiallyComplete1).is_todo_partially_complete_2()
        );
        assert!(
            !todo_list_item!(PartiallyComplete3).is_todo_partially_complete_2()
        );
        assert!(!todo_list_item!(Complete).is_todo_partially_complete_2());
        assert!(!todo_list_item!(Rejected).is_todo_partially_complete_2());
        assert!(!ListItem::default().is_todo_partially_complete_2());
    }

    #[test]
    fn is_todo_partially_complete_3_should_return_true_if_is_partially_complete_3(
    ) {
        assert!(
            todo_list_item!(PartiallyComplete3).is_todo_partially_complete_3()
        );
    }

    #[test]
    fn is_todo_partially_complete_3_should_return_false_if_not_partially_complete_3(
    ) {
        assert!(!todo_list_item!(Incomplete).is_todo_partially_complete_3());
        assert!(
            !todo_list_item!(PartiallyComplete1).is_todo_partially_complete_3()
        );
        assert!(
            !todo_list_item!(PartiallyComplete2).is_todo_partially_complete_3()
        );
        assert!(!todo_list_item!(Complete).is_todo_partially_complete_3());
        assert!(!todo_list_item!(Rejected).is_todo_partially_complete_3());
        assert!(!ListItem::default().is_todo_partially_complete_3());
    }

    #[test]
    fn is_todo_complete_should_return_true_if_is_complete() {
        assert!(todo_list_item!(Complete).is_todo_complete());
    }

    #[test]
    fn is_todo_complete_should_return_false_if_not_complete() {
        assert!(!todo_list_item!(Incomplete).is_todo_complete());
        assert!(!todo_list_item!(PartiallyComplete1).is_todo_complete());
        assert!(!todo_list_item!(PartiallyComplete2).is_todo_complete());
        assert!(!todo_list_item!(PartiallyComplete3).is_todo_complete());
        assert!(!todo_list_item!(Rejected).is_todo_complete());
        assert!(!ListItem::default().is_todo_complete());
    }

    #[test]
    fn is_todo_rejected_should_return_true_if_is_rejected() {
        assert!(todo_list_item!(Rejected).is_todo_rejected());
    }

    #[test]
    fn is_todo_rejected_should_return_false_if_not_rejected() {
        assert!(!todo_list_item!(Incomplete).is_todo_rejected());
        assert!(!todo_list_item!(PartiallyComplete1).is_todo_rejected());
        assert!(!todo_list_item!(PartiallyComplete2).is_todo_rejected());
        assert!(!todo_list_item!(PartiallyComplete3).is_todo_rejected());
        assert!(!todo_list_item!(Complete).is_todo_rejected());
        assert!(!ListItem::default().is_todo_rejected());
    }
}
