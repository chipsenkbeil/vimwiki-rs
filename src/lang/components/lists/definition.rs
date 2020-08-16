use super::InlineComponentContainer;
use derive_more::From;
use petgraph::{
    graph::{Graph, NodeIndex},
    Undirected,
};
use serde::{Deserialize, Serialize};

/// Represents a term in a definition list;
/// A term can have one or more definitions
pub type Term = String;

/// Represents a definition in a definition list;
/// A definition can be associated with one or more terms
pub type Definition = InlineComponentContainer;

#[derive(Clone, Debug, From, Serialize, Deserialize)]
enum TermOrDefinition {
    Term(Term),
    Definition(Definition),
}

/// Represents a list of terms and definitions
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DefinitionList {
    graph: Graph<TermOrDefinition, (), Undirected>,
}

impl DefinitionList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new, unconnected term to the list, returning an index that
    /// is used when connecting a term with a definition
    pub fn add_term(&mut self, into_term: impl Into<Term>) -> usize {
        let term: Term = into_term.into();
        self.graph.add_node(term.into()).index()
    }

    /// Adds a new, unconnected definition to the list, returning an index that
    /// is used when connecting a term with a definition
    pub fn add_definition(
        &mut self,
        into_definition: impl Into<Definition>,
    ) -> usize {
        let definition: Definition = into_definition.into();
        self.graph.add_node(definition.into()).index()
    }

    /// Connects a term with a definition
    pub fn connect_term_to_definition(
        &mut self,
        term_idx: usize,
        def_idx: usize,
    ) -> usize {
        self.graph
            .add_edge(NodeIndex::new(term_idx), NodeIndex::new(def_idx), ())
            .index()
    }

    /// Searches for all definitions associated with term
    pub fn find_definitions(
        &self,
        into_term: impl Into<Term>,
    ) -> Vec<&Definition> {
        let g = &self.graph;
        self.find_term_index(into_term)
            .map(|(_, idx)| {
                g.neighbors(idx)
                    .into_iter()
                    .flat_map(|idx| match &g[idx] {
                        TermOrDefinition::Term(_) => None,
                        TermOrDefinition::Definition(x) => Some(x),
                    })
                    .collect::<Vec<&Definition>>()
            })
            .unwrap_or_default()
    }

    /// Searches for all alternative terms to the specified term
    pub fn find_alternative_terms(
        &self,
        into_term: impl Into<Term>,
    ) -> Vec<&Term> {
        let g = &self.graph;
        self.find_term_index(into_term)
            .map(|(term, idx)| {
                g.neighbors(idx)
                    .into_iter()
                    .flat_map(|idx| match &g[idx] {
                        TermOrDefinition::Term(_) => None,
                        TermOrDefinition::Definition(_) => Some(idx),
                    })
                    .flat_map(|idx| {
                        g.neighbors(idx)
                            .into_iter()
                            .flat_map(|idx| match &g[idx] {
                                TermOrDefinition::Term(x) if x != &term => {
                                    Some(x)
                                }
                                TermOrDefinition::Term(_) => None,
                                TermOrDefinition::Definition(_) => None,
                            })
                            .collect::<Vec<&Term>>()
                    })
                    .collect::<Vec<&Term>>()
            })
            .unwrap_or_default()
    }

    /// Finds the index for a term through brute force
    fn find_term_index(
        &self,
        into_term: impl Into<Term>,
    ) -> Option<(Term, NodeIndex)> {
        let term: Term = into_term.into();
        let g = &self.graph;

        g.node_indices()
            .find(|i| match &g[*i] {
                TermOrDefinition::Term(x) => x == &term,
                TermOrDefinition::Definition(_) => false,
            })
            .map(|idx| (term, idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_definitions_should_list_all_definitions_for_term() {
        let mut dl = DefinitionList::new();

        let t1 = dl.add_term("term1");
        let d0 = dl.add_definition("def text");
        let d1 = dl.add_definition("def1");
        dl.connect_term_to_definition(t1, d0);
        dl.connect_term_to_definition(t1, d1);

        let t2 = dl.add_term("term2");
        let d2 = dl.add_definition("def2");
        dl.connect_term_to_definition(t2, d2);

        dl.add_term("term3");

        // Test looking for defs of term with multiple defs
        let defs = dl.find_definitions("term1");
        assert_eq!(2, defs.len());

        // Test looking for defs of term with one def
        let defs = dl.find_definitions("term2");
        assert_eq!(1, defs.len());

        // Test looking for defs of term with no defs
        let defs = dl.find_definitions("term3");
        assert_eq!(0, defs.len());

        // Test looking for defs of term that does not exist
        let defs = dl.find_definitions("term4");
        assert_eq!(0, defs.len());
    }

    #[test]
    fn find_alternative_terms_should_list_all_terms_with_same_definitions() {
        let mut dl = DefinitionList::new();

        let t0 = dl.add_term("term0");
        let t1 = dl.add_term("term1");
        let t2 = dl.add_term("term2");
        let t3 = dl.add_term("term3");
        let t4 = dl.add_term("term4");
        dl.add_term("term5");

        let d0 = dl.add_definition("def0");
        let d1 = dl.add_definition("def1");

        dl.connect_term_to_definition(t0, d0);
        dl.connect_term_to_definition(t1, d0);
        dl.connect_term_to_definition(t2, d0);

        dl.connect_term_to_definition(t3, d1);
        dl.connect_term_to_definition(t4, d1);

        // Test looking for alternate terms for term that has multiple
        let terms = dl.find_alternative_terms("term1");
        assert_eq!(terms.len(), 2);
        assert!(terms.contains(&(&"term0".to_string())));
        assert!(terms.contains(&(&"term2".to_string())));

        // Test looking for alternate terms for term that has one
        let terms = dl.find_alternative_terms("term3");
        assert_eq!(terms.len(), 1);
        assert_eq!(terms, vec!["term4"]);

        // Test looking for alternate terms for term that has no alternatives
        let terms = dl.find_alternative_terms("term5");
        assert_eq!(terms.len(), 0);

        // Test looking for alternate terms for term that does not exist
        let terms = dl.find_alternative_terms("term999");
        assert_eq!(terms.len(), 0);
    }
}
