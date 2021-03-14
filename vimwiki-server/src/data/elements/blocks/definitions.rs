use crate::data::{ConvertToDatabaseError, Region};

use entity::*;
use std::convert::TryFrom;
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
    async fn gql_terms(&self) -> Vec<Term> {
        self.terms()
    }

    /// The definitions for a specific term
    async fn gql_definitions_for_term(&self, term: String) -> Vec<Definition> {
        match self.0.as_inner().get(term.as_str()) {
            Some(defs) => defs
                .iter()
                .map(|x| {
                    Definition::from(
                        x.as_ref().map(elements::Definition::to_borrowed),
                    )
                })
                .collect(),
            None => vec![],
        }
    }

    /// The terms and their respective definitions
    async fn gql_terms_and_definitions(&self) -> Vec<TermAndDefinitions> {
        self.0
            .as_inner()
            .iter()
            .map(|(term, defs)| {
                TermAndDefinitions::from((term.to_owned(), defs.to_owned()))
            })
            .collect()
    }
}

impl<'a> TryFrom<Located<v::DefinitionList<'a>>> for DefinitionList {
    type Error = ConvertToDatabaseError;

    fn try_from(
        le: Located<v::DefinitionList<'a>>,
    ) -> Result<Self, Self::Error> {
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .lines(
                    le.into_inner()
                        .lines
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
                .finish_and_commit(),
        )
    }
}

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Term {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    #[ent(edge(policy = "deep"))]
    definitions: Vec<Definition>,
}

impl<'a> TryFrom<Located<v::Term<'a>>> for Term {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Term<'a>>) -> Result<Self, Self::Error> {
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .lines(
                    le.into_inner()
                        .lines
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
                .finish_and_commit(),
        )
    }
}

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Definition {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,
}

impl<'a> TryFrom<Located<v::Definition<'a>>> for Definition {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Definition<'a>>) -> Result<Self, Self::Error> {
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .lines(
                    le.into_inner()
                        .lines
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
                .finish_and_commit(),
        )
    }
}
