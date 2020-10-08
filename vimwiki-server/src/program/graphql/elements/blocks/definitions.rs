use super::InlineElement;
use vimwiki::{elements, LE};

#[derive(Debug)]
pub struct DefinitionList(LE<elements::DefinitionList>);

impl From<LE<elements::DefinitionList>> for DefinitionList {
    fn from(le: LE<elements::DefinitionList>) -> Self {
        Self(le)
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
            .map(|x| Term::new(x.to_owned()))
            .collect()
    }

    /// The definitions for a specific term
    async fn definitions_for_term(&self, term: String) -> Vec<Definition> {
        match self.0.element.defs_for_term(&term) {
            Some(defs) => defs.map(|x| Definition::new(x.to_owned())).collect(),
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
            term: Term::new(td.term),
            definitions: td
                .definitions
                .drain(..)
                .map(Definition::new)
                .collect(),
        }
    }
}

pub struct Term(elements::Term);

impl Term {
    pub fn new(term: elements::Term) -> Self {
        Self(term)
    }
}

/// Represents a term with one or more definitions
#[async_graphql::Object]
impl Term {
    /// The content of the term, aka the term as a string as it would be
    /// read by humans without frills
    async fn content(&self) -> String {
        self.0.to_string()
    }

    /// The content within the term as individual elements
    async fn content_elements(&self) -> Vec<InlineElement> {
        self.0
            .elements
            .iter()
            .map(|e| InlineElement::from(e.clone()))
            .collect()
    }
}

pub struct Definition(elements::Definition);

impl Definition {
    pub fn new(definition: elements::Definition) -> Self {
        Self(definition)
    }
}

/// Represents a definition associated with a term
#[async_graphql::Object]
impl Definition {
    /// The content of the definition, aka the definition as a string as it
    /// would be read by humans without frills
    async fn content(&self) -> String {
        self.0.to_string()
    }

    /// The content within the definition as individual elements
    async fn content_elements(&self) -> Vec<InlineElement> {
        self.0
            .elements
            .iter()
            .map(|e| InlineElement::from(e.clone()))
            .collect()
    }
}
