use super::InlineComponent;
use derive_more::From;
use petgraph::{
    graph::{Graph, NodeIndex},
    Directed,
};
use serde::{Deserialize, Serialize};

/// Represents a term in a definition list;
/// A term can have one or more definitions
pub type Term = String;

/// Represents a definition in a definition list;
/// A definition can be associated with one or more terms
pub type Definition = Vec<InlineComponent>;

#[derive(Clone, Debug, From, Serialize, Deserialize)]
enum TermOrDefinition {
    Term(Term),
    Definition(Definition),
}

/// Represents a list of terms and definitions
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DefinitionList {
    graph: Graph<TermOrDefinition, (), Directed>,
}

impl DefinitionList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new, unconnected term to the list, returning an index that
    /// is used when connecting a term with a definition
    pub fn add_term(&mut self, term: Term) -> usize {
        self.graph.add_node(term.into()).index()
    }

    /// Adds a new, unconnected definition to the list, returning an index that
    /// is used when connecting a term with a definition
    pub fn add_definition(&mut self, definition: Definition) -> usize {
        self.graph.add_node(definition.into()).index()
    }

    /// Connects a term with a definition
    pub fn connect_term_to_definition(
        &mut self,
        termIdx: usize,
        definitionIdx: usize,
    ) -> usize {
        self.graph
            .add_edge(
                NodeIndex::new(termIdx),
                NodeIndex::new(definitionIdx),
                (),
            )
            .index()
    }

    /// Searches for all definitions associated with term
    pub fn find_definitions(&self, term: &Term) -> Vec<Definition> {
        let g = &self.graph;
        self.find_term_index(term)
            .map(|idx| {
                g.neighbors(idx)
                    .into_iter()
                    .flat_map(|idx| match g[idx] {
                        TermOrDefinition::Term(_) => None,
                        TermOrDefinition::Definition(x) => Some(x),
                    })
                    .collect::<Vec<Definition>>()
            })
            .unwrap_or_default()
    }

    /// Searches for all alternative terms to the specified term
    pub fn find_alternative_terms(&self, term: &Term) -> Vec<Term> {
        let g = &self.graph;
        self.find_term_index(term)
            .map(|idx| {
                g.neighbors(idx)
                    .into_iter()
                    .flat_map(|idx| match g[idx] {
                        TermOrDefinition::Term(x) => Some(x),
                        TermOrDefinition::Definition(_) => None,
                    })
                    .collect::<Vec<Term>>()
            })
            .unwrap_or_default()
    }

    /// Finds the index for a term through brute force
    fn find_term_index(&self, term: &Term) -> Option<NodeIndex> {
        let g = &self.graph;

        g.node_indices().find(|i| match g[*i] {
            TermOrDefinition::Term(x) => &x == term,
            TermOrDefinition::Definition(_) => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_definitions_should_list_all_definitions_for_term() {
        let mut dl = DefinitionList::new();

        let t1 = dl.add_term("term1".to_string());
        let d0 = dl.add_definition(vec![
            "def".to_string().into(),
            "text".to_string().into(),
        ]);
        let d1 = dl.add_definition(vec!["def1".to_string().into()]);
        dl.connect_term_to_definition(t1, d0);
        dl.connect_term_to_definition(t1, d1);

        let t2 = dl.add_term("term2".to_string());
        let d2 = dl.add_definition(vec!["def2".to_string().into()]);
        dl.connect_term_to_definition(t2, d2);

        let t3 = dl.add_term("term3".to_string());

        // Test looking for defs of term with multiple defs
        let defs = dl.find_definitions(&"term1".to_string());
        assert_eq!(2, defs.len());

        // Test looking for defs of term with one def
        let defs = dl.find_definitions(&"term2".to_string());
        assert_eq!(1, defs.len());

        // Test looking for defs of term with no defs
        let defs = dl.find_definitions(&"term3".to_string());
        assert_eq!(0, defs.len());

        // Test looking for defs of term that does not exist
        let defs = dl.find_definitions(&"term4".to_string());
        assert_eq!(0, defs.len());
    }

    #[test]
    fn find_alternative_terms_should_list_all_terms_with_same_definitions() {
        let mut dl = DefinitionList::new();

        let t0 = dl.add_term("term0".to_string());
        let t1 = dl.add_term("term1".to_string());
        let t2 = dl.add_term("term2".to_string());
        let t3 = dl.add_term("term3".to_string());
        let t4 = dl.add_term("term4".to_string());
        let t5 = dl.add_term("term5".to_string());

        let d0 = dl.add_definition(vec!["def0".to_string().into()]);
        let d1 = dl.add_definition(vec!["def1".to_string().into()]);

        dl.connect_term_to_definition(t0, d0);
        dl.connect_term_to_definition(t1, d0);
        dl.connect_term_to_definition(t2, d0);

        dl.connect_term_to_definition(t3, d1);
        dl.connect_term_to_definition(t4, d1);

        // Test looking for alternate terms for term that has multiple
        let terms = dl.find_alternative_terms(&"term1".to_string());
        assert!(terms.contains(&"term0".to_string()));
        assert!(terms.contains(&"term2".to_string()));

        // Test looking for alternate terms for term that has one
        let terms = dl.find_alternative_terms(&"term3".to_string());
        assert_eq!(terms, vec![&"term4".to_string()]);

        // Test looking for alternate terms for term that has no alternatives
        let terms = dl.find_alternative_terms(&"term5".to_string());
        assert_eq!(terms.len(), 0);

        // Test looking for alternate terms for term that does not exist
        let terms = dl.find_alternative_terms(&"term999".to_string());
        assert_eq!(terms.len(), 0);
    }
}
