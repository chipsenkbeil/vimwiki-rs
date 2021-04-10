use crate::data::{
    Description, Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region, Uri,
};
use entity::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use vimwiki::{elements as v, Located};

/// Represents a single document transclusion link
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct TransclusionLink {
    /// The segment of the document this link covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The URI representing the link's content to pull in
    #[ent(field, ext(async_graphql(filter_untyped)))]
    uri: Uri,

    /// Optional description associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    description: Option<Description>,

    /// Additional properties associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    properties: Vec<Property>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for TransclusionLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.uri()),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for TransclusionLink {
    type Element = Located<v::TransclusionLink<'a>>;

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
                .uri(Uri::from(element.uri))
                .description(element.description.map(Description::from))
                .properties(
                    element
                        .properties
                        .into_iter()
                        .map(|(key, value)| Property {
                            key: key.to_string(),
                            value: value.to_string(),
                        })
                        .collect(),
                )
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )
    }
}

#[derive(
    async_graphql::SimpleObject,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    ValueLike,
)]
pub struct Property {
    key: String,
    value: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_transclusion_link!(
                r#"{{https://example.com/pic.png|Some description|class="some class"}}"#
            );
            let region = Region::from(element.region());
            let ent =
                TransclusionLink::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(
                ent.uri(),
                &"https://example.com/pic.png".parse::<Uri>().unwrap()
            );
            assert_eq!(
                ent.description(),
                &Some(Description::Text(String::from("Some description")))
            );
            assert_eq!(
                ent.properties(),
                &vec![Property {
                    key: "class".to_string(),
                    value: "some class".to_string(),
                }]
            );
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
