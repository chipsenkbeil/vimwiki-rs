use super::{ListItemContent, ListItemContents};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

/// Represents the type associated with an unordered item
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItemType {
    /// -
    Hyphen,
    /// *
    Asterisk,
    /// Catchall
    Other(String),
}

/// Represents an unordered item in a list
#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    _type: ListItemType,
    pos: usize,
    contents: ListItemContents,
}

impl ListItem {
    /// Represents the type of unordered list item
    pub fn item_type(&self) -> &ListItemType {
        &self._type
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn contents(&self) -> &[ListItemContent] {
        &self.contents
    }

    /// Allocates a new string representing the full prefix of the list item
    /// such as - or *
    pub fn to_prefix(&self) -> String {
        match &self._type {
            ListItemType::Hyphen => String::from("-"),
            ListItemType::Asterisk => String::from("*"),
            ListItemType::Other(prefix) => prefix.to_string(),
        }
    }
}

impl Default for ListItem {
    fn default() -> Self {
        Self {
            _type: ListItemType::Hyphen,
            pos: 0,
            contents: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! unordered_item {
        ($type:ident, $pos:expr, $contents:expr) => {
            ListItem::new(ListItemType::$type, $pos, $contents)
        };
        ($type:ident, $pos:expr) => {
            ListItem::new(ListItemType::$type, $pos, vec![])
        };
        ($type:ident) => {
            ListItem::new(ListItemType::$type, 0, vec![])
        };
    }

    macro_rules! other_item {
        ($value:expr, $pos:expr, $contents:expr) => {
            ListItem::new(ListItemType::Other($value), $pos, $contents)
        };
        ($value:expr, $pos:expr) => {
            ListItem::new(ListItemType::Other($value), $pos, vec![])
        };
        ($value:expr) => {
            ListItem::new(ListItemType::Other($value), 0, vec![])
        };
        () => {
            ListItem::new(ListItemType::Other(String::new()), 0, vec![])
        };
    }

    fn make_content(text: &str) -> ListItemContents {
        vec![ListItemContent::InlineContent(text.into())]
    }

    #[test]
    fn pos_should_return_internal_position() {
        assert_eq!(unordered_item!(Hyphen, 999).pos(), 999);
        assert_eq!(unordered_item!(Asterisk, 999).pos(), 999);
        assert_eq!(other_item!(String::new(), 999).pos(), 999);
    }

    #[test]
    fn contents_should_return_internal_contents() {
        assert_eq!(
            unordered_item!(Hyphen, 0, make_content("test")).contents(),
            &make_content("test")[..],
        );

        assert_eq!(
            unordered_item!(Asterisk, 0, make_content("test")).contents(),
            &make_content("test")[..],
        );

        assert_eq!(
            other_item!(String::new(), 0, make_content("test")).contents(),
            &make_content("test")[..],
        );
    }

    #[test]
    fn to_prefix_should_return_hyphen_if_hyphen_type() {
        assert_eq!(unordered_item!(Hyphen, 0).to_prefix(), "-");
        assert_eq!(unordered_item!(Hyphen, 27).to_prefix(), "-");
        assert_eq!(unordered_item!(Hyphen, 99).to_prefix(), "-");
    }

    #[test]
    fn to_prefix_should_return_asterisk_if_asterisk_type() {
        assert_eq!(unordered_item!(Asterisk, 0).to_prefix(), "*");
        assert_eq!(unordered_item!(Asterisk, 27).to_prefix(), "*");
        assert_eq!(unordered_item!(Asterisk, 99).to_prefix(), "*");
    }

    #[test]
    fn to_prefix_should_return_internal_value_if_other_type() {
        assert_eq!(other_item!("prefix".to_string(), 0).to_prefix(), "prefix");
        assert_eq!(other_item!("prefix".to_string(), 27).to_prefix(), "prefix");
        assert_eq!(other_item!("prefix".to_string(), 99).to_prefix(), "prefix");
    }
}
