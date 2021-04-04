use crate::data::{
    Description, Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use vimwiki::{elements as v, Located};

/// Represents a single document link to an external file
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct ExternalFileLink {
    /// The segment of the document this link covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// Scheme associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    scheme: ExternalFileLinkScheme,

    /// Path to the local file
    path: String,

    /// Optional description associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    description: Option<Description>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for ExternalFileLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.path()),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for ExternalFileLink {
    type Element = Located<v::ExternalFileLink<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let element = element.into_inner();

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .scheme(ExternalFileLinkScheme::from(element.scheme))
                .path(element.path.to_string_lossy().to_string())
                .description(element.description.map(Description::from))
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )
    }
}

/// Represents the scheme associated with an external file link
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
pub enum ExternalFileLinkScheme {
    Local,
    File,
    Absolute,
}

impl From<v::ExternalFileLinkScheme> for ExternalFileLinkScheme {
    fn from(s: v::ExternalFileLinkScheme) -> Self {
        match s {
            v::ExternalFileLinkScheme::Local => Self::Local,
            v::ExternalFileLinkScheme::File => Self::File,
            v::ExternalFileLinkScheme::Absolute => Self::Absolute,
        }
    }
}

impl ValueLike for ExternalFileLinkScheme {
    fn into_value(self) -> Value {
        match self {
            Self::Local => Value::from("local"),
            Self::File => Value::from("file"),
            Self::Absolute => Value::from("absolute"),
        }
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => match x.as_str() {
                "local" => Ok(Self::Local),
                "file" => Ok(Self::File),
                "absolute" => Ok(Self::Absolute),
                _ => Err(Value::Text(x)),
            },
            x => Err(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_external_file_link!(
                r#"[[file:/some/file/path.txt|Some description]]"#
            );
            let region = Region::from(element.region());
            let ent =
                ExternalFileLink::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.scheme(), ExternalFileLinkScheme::File);
            assert_eq!(ent.path(), "/some/file/path.txt");
            assert_eq!(
                ent.descripton(),
                Some(Description::Text(String::from("Some description")))
            );
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
