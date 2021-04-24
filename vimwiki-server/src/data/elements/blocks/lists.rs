use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, InlineElement, InlineElementQuery, Page, PageQuery,
    Region,
};
use entity::*;
use entity_async_graphql::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use vimwiki::{elements as v, Located};

/// Represents a single document list
#[gql_ent]
pub struct List {
    /// The segment of the document this list covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The items contained in the list
    #[ent(edge(policy = "deep"))]
    items: Vec<ListItem>,

    /// Page containing this list
    #[ent(edge)]
    page: Page,

    /// Parent element to this list
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl<'a> FromVimwikiElement<'a> for List {
    type Element = Located<v::List<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());

        let mut ent = GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .items(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        let mut items = Vec::new();
        for item in element.into_inner().items {
            items.push(
                ListItem::from_vimwiki_element(page_id, Some(ent.id()), item)?
                    .id(),
            );
        }

        ent.set_items_ids(items);
        ent.commit().map_err(GraphqlDatabaseError::Database)?;

        Ok(ent)
    }
}

/// Represents a single item within a list in a document
#[gql_ent]
pub struct ListItem {
    /// The segment of the document this list item covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The type of the list item
    #[ent(field(graphql(filter_untyped)))]
    item_type: ListItemType,

    /// The suffix to a list item
    #[ent(field(graphql(filter_untyped)))]
    suffix: ListItemSuffix,

    /// The position of this list item among all items in a list
    position: i32,

    /// The contents contained within the list item
    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    contents: Vec<ListItemContent>,

    /// Additional attributes associated with the list item
    #[ent(edge(policy = "deep"))]
    attributes: ListItemAttributes,

    /// Page containing this list item
    #[ent(edge)]
    page: Page,

    /// Parent element to this list item
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl<'a> FromVimwikiElement<'a> for ListItem {
    type Element = Located<v::ListItem<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let item = element.into_inner();

        let item_type = ListItemType::from(item.item_type);
        let suffix = ListItemSuffix::from(item.suffix);
        let position = item.pos as i32;

        let mut ent = GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .item_type(item_type)
                .suffix(suffix)
                .position(position)
                .contents(Vec::new())
                .attributes(0)
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        let mut contents = Vec::new();
        for content in item.contents.contents {
            contents.push(
                ListItemContent::from_vimwiki_element(
                    page_id,
                    Some(ent.id()),
                    content,
                )?
                .id(),
            );
        }

        let attributes = ListItemAttributes::from_vimwiki_element(
            page_id,
            Some(ent.id()),
            item.attributes,
        )?
        .id();

        ent.set_contents_ids(contents);
        ent.set_attributes_id(attributes);
        ent.commit().map_err(GraphqlDatabaseError::Database)?;

        Ok(ent)
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

#[gql_ent]
pub enum ListItemContent {
    InlineContent(InlineContent),
    List(List),
}

impl ListItemContent {
    pub fn page_id(&self) -> Id {
        match self {
            Self::InlineContent(x) => x.page_id(),
            Self::List(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::InlineContent(x) => x.parent_id(),
            Self::List(x) => x.parent_id(),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for ListItemContent {
    type Element = Located<v::ListItemContent<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = element.region();
        Ok(match element.into_inner() {
            v::ListItemContent::InlineContent(x) => {
                Self::InlineContent(InlineContent::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::ListItemContent::List(x) => {
                Self::List(List::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
        })
    }
}

#[gql_ent]
pub struct InlineContent {
    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    contents: Vec<InlineElement>,

    /// Page containing this inline content
    #[ent(edge)]
    page: Page,

    /// Parent element to this inline content
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl<'a> FromVimwikiElement<'a> for InlineContent {
    type Element = Located<v::InlineElementContainer<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let mut ent = GraphqlDatabaseError::wrap(
            Self::build()
                .contents(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        let mut contents = Vec::new();
        for content in element.into_inner().elements {
            contents.push(
                InlineElement::from_vimwiki_element(
                    page_id,
                    Some(ent.id()),
                    content,
                )?
                .id(),
            );
        }

        ent.set_contents_ids(contents);
        ent.commit().map_err(GraphqlDatabaseError::Database)?;

        Ok(ent)
    }
}

#[gql_ent]
pub struct ListItemAttributes {
    #[ent(field(graphql(filter_untyped)))]
    todo_status: Option<ListItemTodoStatus>,

    /// Page containing this list item attribute set
    #[ent(edge)]
    page: Page,

    /// Parent element to this list item attribute set
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl<'a> FromVimwikiElement<'a> for ListItemAttributes {
    type Element = v::ListItemAttributes;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let todo_status = element.todo_status.map(ListItemTodoStatus::from);

        GraphqlDatabaseError::wrap(
            Self::build()
                .todo_status(todo_status)
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use entity_inmemory::InmemoryDatabase;
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_list! {r#"
            - item 1
            - item 2
                - sub item 1
                - sub item 2
            - [ ] item 3
            "#};
            let region = Region::from(element.region());

            let ent = List::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");
            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));

            for item in ent.load_items().expect("Failed to load items") {
                assert_eq!(item.page_id(), 999);
                assert_eq!(item.parent_id(), Some(ent.id()));
            }

            // TODO: Validate rest of content for proper edge trickling and
            //       other info like fields
        });
    }
}
