use super::LE;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// Represents the type used for a single term
pub type Term = LE<String>;

/// Represents the type used for a single definition
pub type Definition = LE<String>;

/// Represents a term and associated definitions
#[derive(Constructor, Clone, Debug, Serialize, Deserialize)]
pub struct TermAndDefinitions {
    pub term: LE<String>,
    pub definitions: Vec<Definition>,
}

impl Eq for TermAndDefinitions {}

impl PartialEq for TermAndDefinitions {
    fn eq(&self, other: &Self) -> bool {
        self.term == other.term
    }
}

impl PartialEq<LE<String>> for TermAndDefinitions {
    fn eq(&self, other: &LE<String>) -> bool {
        &self.term == other
    }
}

impl PartialEq<String> for TermAndDefinitions {
    fn eq(&self, other: &String) -> bool {
        &self.term.element == other
    }
}

impl PartialEq<&str> for TermAndDefinitions {
    fn eq(&self, other: &&str) -> bool {
        &self.term.element == other
    }
}

impl Hash for TermAndDefinitions {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.term.hash(state);
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
            term: LE::from(term.to_string()),
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

    #[test]
    fn term_and_definitions_should_equal_other_instance_if_names_are_same() {
        let td1 =
            TermAndDefinitions::new(LE::from(String::from("term")), vec![]);
        let td2 = TermAndDefinitions::new(
            LE::from(String::from("term")),
            vec![LE::from(String::from("definition"))],
        );
        assert_eq!(td1, td2);
    }

    #[test]
    fn term_and_definitions_should_equal_lc_string_if_name_equals_string() {
        let td =
            TermAndDefinitions::new(LE::from(String::from("term")), vec![]);
        let other = LE::from(String::from("term"));
        assert_eq!(td, other);
    }

    #[test]
    fn term_and_definitions_should_equal_string_if_name_equals_string() {
        let td =
            TermAndDefinitions::new(LE::from(String::from("term")), vec![]);
        let other = String::from("term");
        assert_eq!(td, other);
    }

    #[test]
    fn term_and_definitions_should_equal_str_slice_if_name_equals_str_slice() {
        let td =
            TermAndDefinitions::new(LE::from(String::from("term")), vec![]);
        let other = "term";
        assert_eq!(td, other);
    }

    #[test]
    fn term_and_definitions_should_hash_using_its_name() {
        let td1 =
            TermAndDefinitions::new(LE::from(String::from("term")), vec![]);
        let td2 = TermAndDefinitions::new(
            LE::from(String::from("term")),
            vec![LE::from(String::from("definition"))],
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
            LE::from(String::from("term")),
            vec![LE::from(String::from("definition"))],
        )]);
        assert!(dl.get("term").is_some());
    }

    #[test]
    fn definition_list_should_be_able_to_iterate_through_terms() {
        let dl = DefinitionList::from(vec![
            TermAndDefinitions::new(
                LE::from(String::from("term1")),
                vec![LE::from(String::from("definition"))],
            ),
            TermAndDefinitions::new(LE::from(String::from("term2")), vec![]),
        ]);

        let term_names =
            dl.terms().map(|t| &t.element[..]).collect::<Vec<&str>>();
        assert_eq!(term_names.len(), 2);
        assert!(term_names.contains(&"term1"));
        assert!(term_names.contains(&"term2"));
    }

    #[test]
    fn definition_list_should_be_able_to_iterate_through_definitions_for_term()
    {
        let dl = DefinitionList::from(vec![
            TermAndDefinitions::new(
                LE::from(String::from("term1")),
                vec![LE::from(String::from("definition"))],
            ),
            TermAndDefinitions::new(LE::from(String::from("term2")), vec![]),
        ]);

        let defs = dl
            .defs_for_term("term1")
            .expect("Failed to find term")
            .map(|d| &d.element[..])
            .collect::<Vec<&str>>();
        assert_eq!(defs.len(), 1);
        assert!(defs.contains(&"definition"));

        let defs = dl
            .defs_for_term("term2")
            .expect("Failed to find term")
            .map(|d| &d.element[..])
            .collect::<Vec<&str>>();
        assert!(defs.is_empty());

        assert!(dl.defs_for_term("term-unknown").is_none());
    }
}
