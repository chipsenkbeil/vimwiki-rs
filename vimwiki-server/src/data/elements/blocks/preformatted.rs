use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use entity_async_graphql::*;
use std::collections::HashMap;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(EntFilter)]
pub struct PreformattedText {
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    language: Option<String>,
    lines: Vec<String>,

    // TODO: Support a typed filter once predicate available:
    //       https://github.com/chipsenkbeil/entity-rs/issues/53
    #[ent(field(graphql(filter_untyped)))]
    metadata: HashMap<String, String>,

    /// Page containing the preformatted text
    #[ent(edge)]
    page: Page,

    /// Parent element to this preformatted text
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

/// Represents a single document block of preformatted text (aka code block)
#[async_graphql::Object]
impl PreformattedText {
    /// The segment of the document this preformatted text covers
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        self.region()
    }

    /// The lines of content contained within this preformatted text
    #[graphql(name = "lines")]
    async fn gql_lines(&self) -> &[String] {
        self.lines()
    }

    /// The lines joined with " " inbetween
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.lines().join(" ")
    }

    /// The language associated with this preformatted text
    #[graphql(name = "language")]
    async fn gql_language(&self) -> Option<String> {
        self.language()
            .as_deref()
            .or_else(|| {
                self.metadata
                    .get("class")
                    .and_then(|x| x.strip_prefix("brush:"))
            })
            .map(|x| x.trim().to_string())
    }

    /// The metadata associated with some key
    #[graphql(name = "metadata_for_key")]
    async fn gql_metadata_for_key(&self, key: String) -> Option<&String> {
        self.metadata().get(&key)
    }

    /// All metadata associated with the preformatted text
    #[graphql(name = "metadata")]
    async fn gql_metadata(&self) -> &HashMap<String, String> {
        self.metadata()
    }

    /// The page containing this preformatted text
    #[graphql(name = "page")]
    async fn gql_page(&self) -> async_graphql::Result<Page> {
        self.load_page()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The parent element containing this preformatted text
    #[graphql(name = "parent")]
    async fn gql_parent(&self) -> async_graphql::Result<Option<Element>> {
        self.load_parent()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }
}

impl<'a> FromVimwikiElement<'a> for PreformattedText {
    type Element = Located<v::PreformattedText<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let language =
            element.as_inner().lang.as_ref().map(ToString::to_string);
        let lines = element
            .as_inner()
            .lines
            .iter()
            .map(ToString::to_string)
            .collect();
        let metadata = element
            .into_inner()
            .metadata
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .language(language)
                .lines(lines)
                .metadata(metadata)
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )
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
            let element = vimwiki_preformatted_text! {r#"
                {{{c++ prop="text"
                First line of text
                Second line of text
                }}}
            "#};
            let region = Region::from(element.region());
            let ent =
                PreformattedText::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");

            assert_eq!(
                ent.lines(),
                &[
                    "First line of text".to_string(),
                    "Second line of text".to_string()
                ],
            );
            assert_eq!(ent.language(), &Some("c++".to_string()));

            let mut metadata = HashMap::new();
            metadata.insert("prop".to_string(), "text".to_string());
            assert_eq!(ent.metadata(), &metadata);

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
