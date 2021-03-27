use crate::data::{
    GraphqlDatabaseError, InlineElement, InlineElementQuery, Region,
};
use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEntFilter)]
pub struct DefinitionList {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    #[ent(edge(policy = "deep"))]
    terms: Vec<Term>,

    #[ent(edge(policy = "deep"))]
    definitions: Vec<Definition>,
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
}

impl<'a> TryFrom<Located<v::DefinitionList<'a>>> for DefinitionList {
    type Error = GraphqlDatabaseError;

    fn try_from(
        le: Located<v::DefinitionList<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());

        let mut terms: Vec<Id> = Vec::new();
        let mut definitions: Vec<Id> = Vec::new();
        for (term, defs) in le.into_inner() {
            let mut ent_term = Term::try_from(term)?;

            let mut ent_def_ids: Vec<Id> = Vec::new();
            for def in defs {
                ent_def_ids.push(Definition::try_from(def)?.id());
            }

            // NOTE: When first created, the ent term won't have any definitions
            //       associated, so we need to make it aware of them and update
            //       it within the database
            ent_term.set_definitions_ids(ent_def_ids.clone());
            ent_term.commit().map_err(GraphqlDatabaseError::Database)?;

            terms.push(ent_term.id());
            definitions.extend(ent_def_ids);
        }

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .terms(terms)
                .definitions(definitions)
                .finish_and_commit(),
        )
    }
}

#[simple_ent]
#[derive(AsyncGraphqlEntFilter)]
pub struct Term {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<InlineElement>,

    #[ent(edge(policy = "deep"))]
    definitions: Vec<Definition>,
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

impl<'a> TryFrom<Located<v::Term<'a>>> for Term {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Term<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());

        let mut contents = Vec::new();
        for content in le.into_inner().into_inner().elements {
            contents.push(InlineElement::try_from(content)?.id());
        }

        // NOTE: We are not populating definitions here because the vimwiki
        //       Term does not have a connection by itself
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .contents(contents)
                .definitions(Vec::new())
                .finish_and_commit(),
        )
    }
}

#[simple_ent]
#[derive(AsyncGraphqlEntFilter)]
pub struct Definition {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<InlineElement>,
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

impl<'a> TryFrom<Located<v::Definition<'a>>> for Definition {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Definition<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());

        let mut contents = Vec::new();
        for content in le.into_inner().into_inner().elements {
            contents.push(InlineElement::try_from(content)?.id());
        }

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .contents(contents)
                .finish_and_commit(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vimwiki_macros::*;

    #[test]
    fn definition_list_should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_definition_list! {r#"
                    term1:: definition 1
                    term2::
                    :: definition 2
                    :: definition 3
                "#};
            let region = Region::from(element.region());
            println!("ELEMENT: {:?}", element);

            let ent = DefinitionList::try_from(element)
                .expect("Failed to convert from element");
            assert_eq!(ent.region(), &region);
            println!("ENT: {:?}", ent);

            let terms = ent.load_terms().expect("Failed to load terms");
            let defs =
                ent.load_definitions().expect("Failed to load definitions");

            assert_eq!(
                terms
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>(),
                vec!["term1".to_string(), "term2".to_string()]
            );
            assert_eq!(
                terms
                    .iter()
                    .map(|x| x.definitions_ids().clone())
                    .collect::<Vec<Vec<Id>>>(),
                vec![vec![defs[0].id()], vec![defs[1].id(), defs[2].id()]]
            );

            assert_eq!(
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
