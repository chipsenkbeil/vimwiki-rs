use crate::lang::elements::InlineElementContainer;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the type used for a single term
pub type Term<'a> = InlineElementContainer<'a>;

/// Represents the type used for a single definition
pub type Definition<'a> = InlineElementContainer<'a>;

/// Represents a term and associated definitions
#[derive(Constructor, Clone, Debug, Serialize, Deserialize)]
pub struct TermAndDefinitions<'a> {
    pub term: Term<'a>,
    pub definitions: Vec<Definition<'a>>,
}

impl<'a> Eq for TermAndDefinitions<'a> {}

impl<'a> PartialEq for TermAndDefinitions<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.term.to_string() == other.term.to_string()
    }
}

impl<'a, 'b> PartialEq<InlineElementContainer<'b>> for TermAndDefinitions<'a> {
    fn eq(&self, other: &InlineElementContainer<'b>) -> bool {
        self.term.to_string() == other.to_string()
    }
}

impl<'a> PartialEq<String> for TermAndDefinitions<'a> {
    fn eq(&self, other: &String) -> bool {
        &self.term.to_string() == other
    }
}

impl<'a, 'b> PartialEq<&'b str> for TermAndDefinitions<'a> {
    fn eq(&self, other: &&'b str) -> bool {
        &self.term.to_string() == other
    }
}

/// Represents a list of terms and definitions, where a term can have multiple
/// definitions associated with it
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct DefinitionList<'a> {
    terms_and_definitions: HashMap<String, TermAndDefinitions<'a>>,
}

impl<'a> DefinitionList<'a> {
    /// Retrieves a term and its associated definitions
    pub fn get(&self, term: &str) -> Option<&TermAndDefinitions<'a>> {
        self.terms_and_definitions.get(term)
    }

    /// Iterates through all term and definitions instances in list
    pub fn iter(&self) -> impl Iterator<Item = &TermAndDefinitions<'a>> {
        self.terms_and_definitions.values()
    }

    /// Iterates through all terms in the list
    pub fn terms(&self) -> impl Iterator<Item = &Term<'a>> {
        self.terms_and_definitions.values().map(|td| &td.term)
    }

    /// Retrieves the definitions for a term
    pub fn defs_for_term(
        &self,
        term: &str,
    ) -> Option<impl Iterator<Item = &Definition<'a>>> {
        self.get(term).map(|td| td.definitions.iter())
    }
}

impl<'a> From<Vec<TermAndDefinitions<'a>>> for DefinitionList<'a> {
    fn from(mut term_and_definitions: Vec<TermAndDefinitions<'a>>) -> Self {
        let mut dl = Self::default();

        for td in term_and_definitions.drain(..) {
            dl.terms_and_definitions.insert(td.term.to_string(), td);
        }

        dl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{
        DecoratedText, DecoratedTextContent, InlineElement, Located, Text,
    };

    #[test]
    fn term_and_definitions_should_equal_other_instance_if_names_are_same() {
        let td1 =
            TermAndDefinitions::new(Term::from(Located::from("term")), vec![]);
        let td2 = TermAndDefinitions::new(
            Term::from(Located::from("term")),
            vec![Definition::from(Located::from("definition"))],
        );
        assert_eq!(td1, td2);
    }

    #[test]
    fn term_and_definitions_should_equal_le_string_if_name_equals_string() {
        let td =
            TermAndDefinitions::new(Term::from(Located::from("term")), vec![]);
        let other = Term::from(Located::from("term"));
        assert_eq!(td, other);
    }

    #[test]
    fn term_and_definitions_should_equal_string_if_name_equals_string() {
        let td =
            TermAndDefinitions::new(Term::from(Located::from("term")), vec![]);
        let other = String::from("term");
        assert_eq!(td, other);
    }

    #[test]
    fn term_and_definitions_should_equal_str_slice_if_name_equals_str_slice() {
        let td =
            TermAndDefinitions::new(Term::from(Located::from("term")), vec![]);
        let other = "term";
        assert_eq!(td, other);
    }

    #[test]
    fn term_and_definitions_should_hash_using_its_name() {
        let td1 =
            TermAndDefinitions::new(Term::from(Located::from("term")), vec![]);
        let td2 = TermAndDefinitions::new(
            Term::from(Located::from("term")),
            vec![Term::from(Located::from("definition"))],
        );

        let mut hs = HashMap::new();
        hs.insert(td1.term.to_string(), td1);
        assert_eq!(hs.len(), 1);
        assert!(hs.get(&td2.term.to_string()).is_some());
    }

    #[test]
    fn definition_list_should_be_able_to_get_term_and_definitions_by_term_name()
    {
        let dl = DefinitionList::from(vec![TermAndDefinitions::new(
            Term::from(Located::from("term")),
            vec![Definition::from(Located::from("definition"))],
        )]);
        assert!(dl.get("term").is_some());
    }

    #[test]
    fn definition_list_should_be_able_to_iterate_through_terms() {
        let dl = DefinitionList::from(vec![
            TermAndDefinitions::new(
                Term::from(Located::from("term1")),
                vec![Definition::from(Located::from("definition"))],
            ),
            TermAndDefinitions::new(Term::from(Located::from("term2")), vec![]),
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
                Term::from(Located::from("term1")),
                vec![Definition::from(Located::from("definition"))],
            ),
            TermAndDefinitions::new(Term::from(Located::from("term2")), vec![]),
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
                Located::from(InlineElement::DecoratedText(
                    DecoratedText::Bold(vec![Located::from(
                        DecoratedTextContent::from(Text::from("term")),
                    )]),
                )),
                Located::from(InlineElement::Text(Text::from(" 1"))),
            ]),
            vec![Definition::from(Located::from("definition"))],
        )]);
        assert!(dl.get("term 1").is_some());
    }
}
