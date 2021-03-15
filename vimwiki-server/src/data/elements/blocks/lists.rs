use crate::data::{
    ConvertToDatabaseError, InlineElement, InlineElementQuery, Region,
};
use entity::*;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::{Display, EnumString};
use vimwiki::{elements as v, Located};

/// Represents a single document list
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct List {
    /// The segment of the document this list covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The items contained in the list
    #[ent(edge(policy = "deep"))]
    items: Vec<ListItem>,
}

impl<'a> TryFrom<Located<v::List<'a>>> for List {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::List<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());

        let mut items = Vec::new();
        for item in le.into_inner().items {
            items.push(ListItem::try_from(item)?.id());
        }

        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .items(items)
                .finish_and_commit(),
        )
    }
}

/// Represents a single item within a list in a document
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct ListItem {
    /// The segment of the document this list item covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The type of the list item
    #[ent(field, ext(async_graphql(filter_untyped)))]
    item_type: ListItemType,

    /// The suffix to a list item
    #[ent(field, ext(async_graphql(filter_untyped)))]
    suffix: ListItemSuffix,

    /// The position of this list item among all items in a list
    position: i32,

    /// The contents contained within the list item
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<ListItemContent>,

    /// Additional attributes associated with the list item
    #[ent(edge(policy = "deep"))]
    attributes: ListItemAttributes,
}

impl<'a> TryFrom<Located<v::ListItem<'a>>> for ListItem {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::ListItem<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let item = le.into_inner();

        let item_type = ListItemType::from(item.item_type);
        let suffix = ListItemSuffix::from(item.suffix);
        let position = item.pos as i32;

        let mut contents = Vec::new();
        for content in item.contents.contents {
            contents.push(ListItemContent::try_from(content)?.id());
        }

        let attributes = ListItemAttributes::try_from(item.attributes)?.id();

        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .item_type(item_type)
                .suffix(suffix)
                .position(position)
                .contents(contents)
                .attributes(attributes)
                .finish_and_commit(),
        )
    }
}

/// Represents the type of prefix used with a list item
#[derive(
    async_graphql::Enum,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Display,
    EnumString,
    Serialize,
    Deserialize,
)]
pub enum ListItemType {
    Number,
    Pound,
    LowercaseAlphabet,
    UppercaseAlphabet,
    LowercaseRoman,
    UppercaseRoman,
    Hyphen,
    Asterisk,
    Other,
}

impl ValueLike for ListItemType {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => x.as_str().parse().map_err(|_| Value::Text(x)),
            x => Err(x),
        }
    }
}

impl<'a> From<v::ListItemType<'a>> for ListItemType {
    fn from(t: v::ListItemType<'a>) -> Self {
        match t {
            v::ListItemType::Ordered(x) => match x {
                v::OrderedListItemType::LowercaseAlphabet => {
                    Self::LowercaseAlphabet
                }
                v::OrderedListItemType::LowercaseRoman => Self::LowercaseRoman,
                v::OrderedListItemType::Number => Self::Number,
                v::OrderedListItemType::Pound => Self::Pound,
                v::OrderedListItemType::UppercaseAlphabet => {
                    Self::UppercaseAlphabet
                }
                v::OrderedListItemType::UppercaseRoman => Self::UppercaseRoman,
            },
            v::ListItemType::Unordered(x) => match x {
                v::UnorderedListItemType::Asterisk => Self::Asterisk,
                v::UnorderedListItemType::Hyphen => Self::Hyphen,
                v::UnorderedListItemType::Other(_) => Self::Other,
            },
        }
    }
}

#[derive(
    async_graphql::Enum,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Display,
    EnumString,
    Serialize,
    Deserialize,
)]
#[graphql(remote = "vimwiki::elements::ListItemSuffix")]
#[strum(serialize_all = "snake_case")]
pub enum ListItemSuffix {
    None,
    Period,
    Paren,
}

impl ValueLike for ListItemSuffix {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => x.as_str().parse().map_err(|_| Value::Text(x)),
            x => Err(x),
        }
    }
}

#[simple_ent]
#[derive(async_graphql::Union, Debug)]
pub enum ListItemContent {
    InlineContent(InlineContent),
    List(List),
}

impl<'a> TryFrom<Located<v::ListItemContent<'a>>> for ListItemContent {
    type Error = ConvertToDatabaseError;

    fn try_from(
        le: Located<v::ListItemContent<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = le.region();
        Ok(match le.into_inner() {
            v::ListItemContent::InlineContent(x) => {
                Self::InlineContent(InlineContent::try_from(x)?)
            }
            v::ListItemContent::List(x) => {
                Self::List(List::try_from(Located::new(x, region))?)
            }
        })
    }
}

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct InlineContent {
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<InlineElement>,
}

impl<'a> TryFrom<v::InlineElementContainer<'a>> for InlineContent {
    type Error = ConvertToDatabaseError;

    fn try_from(x: v::InlineElementContainer<'a>) -> Result<Self, Self::Error> {
        let mut contents = Vec::new();
        for content in x.elements {
            contents.push(InlineElement::try_from(content)?.id());
        }

        ConvertToDatabaseError::wrap(
            Self::build().contents(contents).finish_and_commit(),
        )
    }
}

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct ListItemAttributes {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    todo_status: Option<ListItemTodoStatus>,
}

impl TryFrom<v::ListItemAttributes> for ListItemAttributes {
    type Error = ConvertToDatabaseError;

    fn try_from(x: v::ListItemAttributes) -> Result<Self, Self::Error> {
        let todo_status = x.todo_status.map(ListItemTodoStatus::from);

        ConvertToDatabaseError::wrap(
            Self::build().todo_status(todo_status).finish_and_commit(),
        )
    }
}

#[derive(
    async_graphql::Enum,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Display,
    EnumString,
    Serialize,
    Deserialize,
)]
#[graphql(remote = "vimwiki::elements::ListItemTodoStatus")]
#[strum(serialize_all = "snake_case")]
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

impl ValueLike for ListItemTodoStatus {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => x.as_str().parse().map_err(|_| Value::Text(x)),
            x => Err(x),
        }
    }
}
