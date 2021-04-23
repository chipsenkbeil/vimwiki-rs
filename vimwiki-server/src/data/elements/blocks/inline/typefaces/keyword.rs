use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use entity_async_graphql::*;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use vimwiki::{elements as v, Located};

/// Represents special keywords that have unique syntax highlighting
#[simple_ent]
#[derive(EntObject, EntFilter)]
pub struct Keyword {
    /// The segment of the document this keyword covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The type of keyword
    #[ent(field(graphql(filter_untyped)))]
    ty: KeywordType,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ty)
    }
}

impl<'a> FromVimwikiElement<'a> for Keyword {
    type Element = Located<v::Keyword>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .ty(KeywordType::from(element.into_inner()))
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )
    }
}

/// Represents type of special keywords that have unique syntax highlighting
#[derive(
    async_graphql::Enum,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
pub enum KeywordType {
    Todo,
    Done,
    Started,
    Fixme,
    Fixed,
    Xxx,
}

impl fmt::Display for KeywordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Todo => "todo",
                Self::Done => "done",
                Self::Started => "started",
                Self::Fixme => "fixme",
                Self::Fixed => "fixed",
                Self::Xxx => "xxx",
            }
        )
    }
}

impl FromStr for KeywordType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "todo" => Ok(Self::Todo),
            "done" => Ok(Self::Done),
            "started" => Ok(Self::Started),
            "fixme" => Ok(Self::Fixme),
            "fixed" => Ok(Self::Fixed),
            "xxx" => Ok(Self::Xxx),
            _ => Err(()),
        }
    }
}

impl From<v::Keyword> for KeywordType {
    fn from(k: v::Keyword) -> Self {
        match k {
            v::Keyword::Todo => KeywordType::Todo,
            v::Keyword::Done => KeywordType::Done,
            v::Keyword::Started => KeywordType::Started,
            v::Keyword::Fixme => KeywordType::Fixme,
            v::Keyword::Fixed => KeywordType::Fixed,
            v::Keyword::Xxx => KeywordType::Xxx,
        }
    }
}

impl ValueLike for KeywordType {
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
            let element = vimwiki_keyword!(r#"TODO"#);
            let region = Region::from(element.region());
            let ent = Keyword::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(*ent.ty(), KeywordType::Todo);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
