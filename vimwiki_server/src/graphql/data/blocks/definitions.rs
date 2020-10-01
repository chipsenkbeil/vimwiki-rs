use super::Region;
use vimwiki::{components, LC};

pub struct DefinitionList(LC<components::DefinitionList>);

impl From<LC<components::DefinitionList>> for DefinitionList {
    fn from(lc: LC<components::DefinitionList>) -> Self {
        Self(lc)
    }
}

/// Represents a single list of terms & definitions in a document
#[async_graphql::Object]
impl DefinitionList {
    /// The terms found within the list
    async fn terms(&self) -> Vec<Term> {
        self.0
            .component
            .terms()
            .map(|x| Term {
                region: Region::from(x.region),
                text: x.component.to_owned(),
            })
            .collect()
    }

    /// The definitions for a specific term
    async fn definitions_for_term(&self, term: String) -> Vec<Definition> {
        match self.0.component.defs_for_term(&term) {
            Some(defs) => defs
                .map(|x| Definition {
                    region: Region::from(x.region),
                    text: x.component.to_owned(),
                })
                .collect(),
            None => vec![],
        }
    }

    /// The terms and their respective definitions
    async fn terms_and_definitions(&self) -> Vec<TermAndDefinitions> {
        self.0
            .component
            .iter()
            .map(|x| TermAndDefinitions::from(x.to_owned()))
            .collect()
    }
}

/// Represents a term and its associated definitions
#[derive(async_graphql::SimpleObject)]
pub struct TermAndDefinitions {
    term: Term,
    definitions: Vec<Definition>,
}

impl From<components::TermAndDefinitions> for TermAndDefinitions {
    fn from(mut td: components::TermAndDefinitions) -> Self {
        Self {
            term: Term::from(td.term),
            definitions: td
                .definitions
                .drain(..)
                .map(Definition::from)
                .collect(),
        }
    }
}

/// Represents a term with one or more definitions
#[derive(async_graphql::SimpleObject)]
pub struct Term {
    region: Region,
    text: String,
}

impl From<components::Term> for Term {
    fn from(term: components::Term) -> Self {
        Self {
            region: Region::from(term.region),
            text: term.component,
        }
    }
}

/// Represents a definition associated with a term
#[derive(async_graphql::SimpleObject)]
pub struct Definition {
    region: Region,
    text: String,
}

impl From<components::Definition> for Definition {
    fn from(definition: components::Definition) -> Self {
        Self {
            region: Region::from(definition.region),
            text: definition.component,
        }
    }
}
