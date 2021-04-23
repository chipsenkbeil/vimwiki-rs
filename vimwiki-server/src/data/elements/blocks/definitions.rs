use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, InlineElement, InlineElementQuery, Page, PageQuery,
    Region,
};
use entity::*;
use entity_async_graphql::*;
use std::fmt;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(EntFilter)]
pub struct DefinitionList {
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    #[ent(edge(policy = "deep"))]
    terms: Vec<Term>,

    #[ent(edge(policy = "deep"))]
    definitions: Vec<Definition>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

/// Represents a single list of terms & definitions in a document
#[async_graphql::Object]
impl DefinitionList {
    /// The terms found within the list
    #[graphql(name = "terms")]
    async fn gql_terms(&self) -> async_graphql::Result<Vec<Term>> {
        self.load_terms()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The definitions found within the list
    #[graphql(name = "definitions")]
    async fn gql_definitions(&self) -> async_graphql::Result<Vec<Definition>> {
        self.load_definitions()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The definitions for a specific term
    #[graphql(name = "definitions_for_term")]
    async fn gql_definitions_for_term(
        &self,
        term: String,
    ) -> async_graphql::Result<Vec<Definition>> {
        let terms = self
            .load_terms()
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;
        for t in terms {
            if t.to_string() == term {
                return t
                    .load_definitions()
                    .map_err(|x| async_graphql::Error::new(x.to_string()));
            }
        }
        Ok(Vec::new())
    }

    /// The page containing this definition list
    #[graphql(name = "page")]
    async fn gql_page(&self) -> async_graphql::Result<Page> {
        self.load_page()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The parent element containing this definition list
    #[graphql(name = "parent")]
    async fn gql_parent(&self) -> async_graphql::Result<Option<Element>> {
        self.load_parent()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }
}

impl<'a> FromVimwikiElement<'a> for DefinitionList {
    type Element = Located<v::DefinitionList<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());

        // First, create a definition list that has no terms or definitions
        // so we can get its id to use as the parent for each of those
        let mut definition_list = GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .terms(Vec::new())
                .definitions(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        // Second, create all of the children terms and definitions
        let mut terms: Vec<Id> = Vec::new();
        let mut definitions: Vec<Id> = Vec::new();
        for (term, defs) in element.into_inner() {
            let mut ent_term = Term::from_vimwiki_element(
                page_id,
                Some(definition_list.id()),
                term,
            )?;

            let mut ent_def_ids: Vec<Id> = Vec::new();
            for def in defs {
                ent_def_ids.push(
                    Definition::from_vimwiki_element(
                        page_id,
                        Some(definition_list.id()),
                        def,
                    )?
                    .id(),
                );
            }

            // NOTE: When first created, the ent term won't have any definitions
            //       associated, so we need to make it aware of them and update
            //       it within the database
            ent_term.set_definitions_ids(ent_def_ids.clone());
            ent_term.commit().map_err(GraphqlDatabaseError::Database)?;

            terms.push(ent_term.id());
            definitions.extend(ent_def_ids);
        }

        // Third, update the definition list with the created term and definition ids
        definition_list.set_terms_ids(terms);
        definition_list.set_definitions_ids(definitions);
        definition_list
            .commit()
            .map_err(GraphqlDatabaseError::Database)?;

        Ok(definition_list)
    }
}

#[simple_ent]
#[derive(EntFilter)]
pub struct Term {
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    contents: Vec<InlineElement>,

    #[ent(edge(policy = "deep"))]
    definitions: Vec<Definition>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

#[async_graphql::Object]
impl Term {
    /// The segment of the document this term covers
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        self.region()
    }

    /// The content within the term as individual elements
    #[graphql(name = "contents")]
    async fn gql_contents(&self) -> async_graphql::Result<Vec<InlineElement>> {
        self.load_contents()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The content within the term as it would be read by humans without frills
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.to_string()
    }

    /// The definitions associated with this term
    #[graphql(name = "definitions")]
    async fn gql_definitions(&self) -> async_graphql::Result<Vec<Definition>> {
        self.load_definitions()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The page containing this term
    #[graphql(name = "page")]
    async fn gql_page(&self) -> async_graphql::Result<Page> {
        self.load_page()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The parent element containing this term
    #[graphql(name = "parent")]
    async fn gql_parent(&self) -> async_graphql::Result<Option<Element>> {
        self.load_parent()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.load_contents() {
            Ok(contents) => {
                for content in contents {
                    write!(f, "{}", content.to_string())?;
                }
                Ok(())
            }
            Err(x) => {
                write!(f, "{}", x)?;
                Ok(())
            }
        }
    }
}

impl<'a> FromVimwikiElement<'a> for Term {
    type Element = Located<v::Term<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());

        // NOTE: We are not populating definitions here because the vimwiki
        //       Term does not have a connection by itself
        let mut term = GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .contents(Vec::new())
                .definitions(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        let mut contents = Vec::new();
        for content in element.into_inner().into_inner().elements {
            contents.push(
                InlineElement::from_vimwiki_element(
                    page_id,
                    Some(term.id()),
                    content,
                )?
                .id(),
            );
        }

        term.set_contents_ids(contents);
        term.commit().map_err(GraphqlDatabaseError::Database)?;

        Ok(term)
    }
}

#[simple_ent]
#[derive(EntFilter)]
pub struct Definition {
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    contents: Vec<InlineElement>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

#[async_graphql::Object]
impl Definition {
    /// The segment of the document this definition covers
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        self.region()
    }

    /// The content within the definition as individual elements
    #[graphql(name = "contents")]
    async fn gql_contents(&self) -> async_graphql::Result<Vec<InlineElement>> {
        self.load_contents()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The content within the definition as it would be read by humans
    /// without frills
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.to_string()
    }

    /// The page containing this definition
    #[graphql(name = "page")]
    async fn gql_page(&self) -> async_graphql::Result<Page> {
        self.load_page()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The parent element containing this definition
    #[graphql(name = "parent")]
    async fn gql_parent(&self) -> async_graphql::Result<Option<Element>> {
        self.load_parent()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.load_contents() {
            Ok(contents) => {
                for content in contents {
                    write!(f, "{}", content.to_string())?;
                }
                Ok(())
            }
            Err(x) => {
                write!(f, "{}", x)?;
                Ok(())
            }
        }
    }
}

impl<'a> FromVimwikiElement<'a> for Definition {
    type Element = Located<v::Definition<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());

        let mut definition = GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .contents(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        let mut contents = Vec::new();
        for content in element.into_inner().into_inner().elements {
            contents.push(
                InlineElement::from_vimwiki_element(
                    page_id,
                    Some(definition.id()),
                    content,
                )?
                .id(),
            );
        }

        definition.set_contents_ids(contents);
        definition
            .commit()
            .map_err(GraphqlDatabaseError::Database)?;

        Ok(definition)
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
            let element = vimwiki_definition_list! {r#"
                    term1:: definition 1
                    term2::
                    :: definition 2
                    :: definition 3
                "#};
            let region = Region::from(element.region());

            let ent =
                DefinitionList::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");
            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));

            let mut terms = ent.load_terms().expect("Failed to load terms");
            let mut defs =
                ent.load_definitions().expect("Failed to load definitions");

            // NOTE: Sorting to ensure that term1 comes before term2
            terms.sort_unstable_by_key(|k| k.to_string());
            defs.sort_unstable_by_key(|k| k.to_string());

            // Verify that all children have same page and parent
            for term in terms.iter() {
                assert_eq!(term.page_id(), 999);
                assert_eq!(term.parent_id(), Some(ent.id()));
                for content in
                    term.load_contents().expect("Failed to load term contents")
                {
                    assert_eq!(content.page_id(), 999);
                    assert_eq!(content.parent_id(), Some(term.id()));
                }
            }
            for def in defs.iter() {
                assert_eq!(def.page_id(), 999);
                assert_eq!(def.parent_id(), Some(ent.id()));
                for content in def
                    .load_contents()
                    .expect("Failed to load definition contents")
                {
                    assert_eq!(content.page_id(), 999);
                    assert_eq!(content.parent_id(), Some(def.id()));
                }
            }

            macro_rules! assert_contains_same {
                ($t:ty, $a:expr, $b:expr) => {{
                    use std::collections::HashSet;
                    let aa: HashSet<$t> = ($a).into_iter().collect();
                    let bb: HashSet<$t> = ($b).into_iter().collect();
                    assert_eq!(aa, bb);
                }};
            }

            assert_contains_same!(
                String,
                terms
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>(),
                vec!["term1".to_string(), "term2".to_string()]
            );
            assert_contains_same!(
                Id,
                terms[0].definitions_ids().clone(),
                vec![defs[0].id()]
            );
            assert_contains_same!(
                Id,
                terms[1].definitions_ids().clone(),
                vec![defs[1].id(), defs[2].id()]
            );

            assert_contains_same!(
                String,
                defs.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>(),
                vec![
                    "definition 1".to_string(),
                    "definition 2".to_string(),
                    "definition 3".to_string()
                ]
            );
        });
    }
}
