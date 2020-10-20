use super::InlineElement;
use vimwiki::elements::{self, Located};

#[derive(Debug)]
pub struct DefinitionList(Located<elements::DefinitionList<'static>>);

impl<'a> From<Located<elements::DefinitionList<'a>>> for DefinitionList {
    fn from(located: Located<elements::DefinitionList<'a>>) -> Self {
        Self(located.map(elements::DefinitionList::into_owned))
    }
}

/// Represents a single list of terms & definitions in a document
#[async_graphql::Object]
impl DefinitionList {
    /// The terms found within the list
    async fn terms(&self) -> Vec<Term> {
        self.0
            .as_inner()
            .terms()
            .map(|x| Term::from(x.as_ref().map(elements::Term::to_borrowed)))
            .collect()
    }

    /// The definitions for a specific term
    async fn definitions_for_term(&self, term: String) -> Vec<Definition> {
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
    async fn terms_and_definitions(&self) -> Vec<TermAndDefinitions> {
        self.0
            .as_inner()
            .iter()
            .map(|(term, defs)| {
                TermAndDefinitions::from((term.to_owned(), defs.to_owned()))
            })
            .collect()
    }
}

/// Represents a term and its associated definitions
#[derive(async_graphql::SimpleObject, Debug)]
pub struct TermAndDefinitions {
    term: Term,
    definitions: Vec<Definition>,
}

impl<'a>
    From<(
        Located<elements::Term<'a>>,
        Vec<Located<elements::Definition<'a>>>,
    )> for TermAndDefinitions
{
    fn from(
        (term, defs): (
            Located<elements::Term<'a>>,
            Vec<Located<elements::Definition<'a>>>,
        ),
    ) -> Self {
        Self {
            term: Term::from(term),
            definitions: defs.into_iter().map(Definition::from).collect(),
        }
    }
}

#[derive(Debug)]
pub struct Term(Located<elements::Term<'static>>);

impl<'a> From<Located<elements::Term<'a>>> for Term {
    fn from(located: Located<elements::Term<'a>>) -> Self {
        Self(located.map(elements::Term::into_owned))
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
            .as_inner()
            .to_borrowed()
            .into_children()
            .into_iter()
            .map(InlineElement::from)
            .collect()
    }
}

#[derive(Debug)]
pub struct Definition(Located<elements::Definition<'static>>);

impl<'a> From<Located<elements::Definition<'a>>> for Definition {
    fn from(located: Located<elements::Definition<'a>>) -> Self {
        Self(located.map(elements::Definition::into_owned))
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
            .as_inner()
            .to_borrowed()
            .into_children()
            .into_iter()
            .map(InlineElement::from)
            .collect()
    }
}
