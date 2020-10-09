use super::{InlineElementContainer, LE};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// Represents the type used for a single term
pub type Term = InlineElementContainer;

/// Represents the type used for a single definition
pub type Definition = InlineElementContainer;

/// Represents a term and associated definitions
#[derive(Constructor, Clone, Debug, Serialize, Deserialize)]
pub struct TermAndDefinitions {
    pub term: Term,
    pub definitions: Vec<Definition>,
}

impl Eq for TermAndDefinitions {}

impl PartialEq for TermAndDefinitions {
    fn eq(&self, other: &Self) -> bool {
        self.term.to_string() == other.term.to_string()
    }
}

impl PartialEq<InlineElementContainer> for TermAndDefinitions {
    fn eq(&self, other: &InlineElementContainer) -> bool {
        self.term.to_string() == other.to_string()
    }
}

impl PartialEq<String> for TermAndDefinitions {
    fn eq(&self, other: &String) -> bool {
        &self.term.to_string() == other
    }
}

impl PartialEq<&str> for TermAndDefinitions {
    fn eq(&self, other: &&str) -> bool {
        &self.term.to_string() == other
    }
}

impl Hash for TermAndDefinitions {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.term.to_string().hash(state);
    }
}

/// Represents a list of terms and definitions, where a term can have multiple
/// definitions associated with it
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct DefinitionList {
    terms_and_definitions: HashSet<TermAndDefinitions>,
}

impl DefinitionList {
    /// Retrieves a term and its associated definitions
    pub fn get(&self, term: &str) -> Option<&TermAndDefinitions> {
        self.terms_and_definitions.get(&TermAndDefinitions {
            term: Term::from(LE::from(term)),
            definitions: vec![],
        })
    }

    /// Iterates through all term and definitions instances in list
    pub fn iter(&self) -> impl Iterator<Item = &TermAndDefinitions> {
        self.terms_and_definitions.iter()
    }

    /// Iterates through all terms in the list
    pub fn terms(&self) -> impl Iterator<Item = &Term> {
        self.terms_and_definitions.iter().map(|td| &td.term)
    }

    /// Retrieves the definitions for a term
    pub fn defs_for_term(
        &self,
        term: &str,
    ) -> Option<impl Iterator<Item = &Definition>> {
        self.get(term).map(|td| td.definitions.iter())
    }
}

impl From<Vec<TermAndDefinitions>> for DefinitionList {
    fn from(mut term_and_definitions: Vec<TermAndDefinitions>) -> Self {
        let mut dl = Self::default();

        for td in term_and_definitions.drain(..) {
            dl.terms_and_definitions.insert(td);
        }

        dl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{
        DecoratedText, DecoratedTextContent, InlineElement, Text,
    };

    #[test]
    fn term_and_definitions_should_equal_other_instance_if_names_are_same() {
        let td1 = TermAndDefinitions::new(Term::from(LE::from("term")), vec![]);
        let td2 = TermAndDefinitions::new(
            Term::from(LE::from("term")),
            vec![Definition::from(LE::from("definition"))],
        );
        assert_eq!(td1, td2);
    }

    #[test]
    fn term_and_definitions_should_equal_le_string_if_name_equals_string() {
        let td = TermAndDefinitions::new(Term::from(LE::from("term")), vec![]);
        let other = Term::from(LE::from("term"));
        assert_eq!(td, other);
    }

    #[test]
    fn term_and_definitions_should_equal_string_if_name_equals_string() {
        let td = TermAndDefinitions::new(Term::from(LE::from("term")), vec![]);
        let other = String::from("term");
        assert_eq!(td, other);
    }

    #[test]
    fn term_and_definitions_should_equal_str_slice_if_name_equals_str_slice() {
        let td = TermAndDefinitions::new(Term::from(LE::from("term")), vec![]);
        let other = "term";
        assert_eq!(td, other);
    }

    #[test]
    fn term_and_definitions_should_hash_using_its_name() {
        let td1 = TermAndDefinitions::new(Term::from(LE::from("term")), vec![]);
        let td2 = TermAndDefinitions::new(
            Term::from(LE::from("term")),
            vec![Term::from(LE::from("definition"))],
        );

        // Insert first TermAndDefinitions and use second, which has definitions
        // with the same name, to look up the first
        let mut hs = HashSet::new();
        hs.insert(td1);
        assert_eq!(hs.len(), 1);
        assert!(hs.get(&td2).is_some());
    }

    #[test]
    fn definition_list_should_be_able_to_get_term_and_definitions_by_term_name()
    {
        let dl = DefinitionList::from(vec![TermAndDefinitions::new(
            Term::from(LE::from("term")),
            vec![Definition::from(LE::from("definition"))],
        )]);
        assert!(dl.get("term").is_some());
    }

    #[test]
    fn definition_list_should_be_able_to_iterate_through_terms() {
        let dl = DefinitionList::from(vec![
            TermAndDefinitions::new(
                Term::from(LE::from("term1")),
                vec![Definition::from(LE::from("definition"))],
            ),
            TermAndDefinitions::new(Term::from(LE::from("term2")), vec![]),
        ]);

        let term_names =
            dl.terms().map(|t| t.to_string()).collect::<Vec<String>>();
        assert_eq!(term_names.len(), 2);
        assert!(term_names.contains(&"term1".to_string()));
        assert!(term_names.contains(&"term2".to_string()));
    }

    #[test]
    fn definition_list_should_be_able_to_iterate_through_definitions_for_term()
    {
        let dl = DefinitionList::from(vec![
            TermAndDefinitions::new(
                Term::from(LE::from("term1")),
                vec![Definition::from(LE::from("definition"))],
            ),
            TermAndDefinitions::new(Term::from(LE::from("term2")), vec![]),
        ]);

        let defs = dl
            .defs_for_term("term1")
            .expect("Failed to find term")
            .map(|d| d.to_string())
            .collect::<Vec<String>>();
        assert_eq!(defs.len(), 1);
        assert!(defs.contains(&"definition".to_string()));

        let defs = dl
            .defs_for_term("term2")
            .expect("Failed to find term")
            .map(|d| d.to_string())
            .collect::<Vec<String>>();
        assert!(defs.is_empty());

        assert!(dl.defs_for_term("term-unknown").is_none());
    }

    #[test]
    fn definition_list_should_support_lookup_with_terms_containing_other_inline_elements(
    ) {
        let dl = DefinitionList::from(vec![TermAndDefinitions::new(
            Term::new(vec![
                LE::from(InlineElement::DecoratedText(DecoratedText::Bold(
                    vec![LE::from(DecoratedTextContent::Text(Text::from(
                        "term",
                    )))],
                ))),
                LE::from(InlineElement::Text(Text::from(" 1"))),
            ]),
            vec![Definition::from(LE::from("definition"))],
        )]);
        assert!(dl.get("term 1").is_some());
    }
}
