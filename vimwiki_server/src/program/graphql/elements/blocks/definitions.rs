use super::Region;
use vimwiki::{elements, LC};

#[derive(Debug)]
pub struct DefinitionList(LC<elements::DefinitionList>);

impl From<LC<elements::DefinitionList>> for DefinitionList {
    fn from(lc: LC<elements::DefinitionList>) -> Self {
        Self(lc)
    }
}

/// Represents a single list of terms & definitions in a document
#[async_graphql::Object]
impl DefinitionList {
    /// The terms found within the list
    async fn terms(&self) -> Vec<Term> {
        self.0
            .element
            .terms()
            .map(|x| Term {
                region: Region::from(x.region),
                text: x.element.to_owned(),
            })
            .collect()
    }

    /// The definitions for a specific term
    async fn definitions_for_term(&self, term: String) -> Vec<Definition> {
        match self.0.element.defs_for_term(&term) {
            Some(defs) => defs
                .map(|x| Definition {
                    region: Region::from(x.region),
                    text: x.element.to_owned(),
                })
                .collect(),
            None => vec![],
        }
    }

    /// The terms and their respective definitions
    async fn terms_and_definitions(&self) -> Vec<TermAndDefinitions> {
        self.0
            .element
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

impl From<elements::TermAndDefinitions> for TermAndDefinitions {
    fn from(mut td: elements::TermAndDefinitions) -> Self {
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

impl From<elements::Term> for Term {
    fn from(term: elements::Term) -> Self {
        Self {
            region: Region::from(term.region),
            text: term.element,
        }
    }
}

/// Represents a definition associated with a term
#[derive(async_graphql::SimpleObject)]
pub struct Definition {
    region: Region,
    text: String,
}

impl From<elements::Definition> for Definition {
    fn from(definition: elements::Definition) -> Self {
        Self {
            region: Region::from(definition.region),
            text: definition.element,
        }
    }
}
