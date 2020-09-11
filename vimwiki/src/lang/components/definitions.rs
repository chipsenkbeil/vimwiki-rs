use super::LC;
use petgraph::{
    graph::{Graph, NodeIndex},
    Undirected,
};
use serde::{Deserialize, Serialize};

/// Represents the type of terms stored in a definition list
pub type Term = LC<String>;

/// Represents the type of definitions stored in a definition list
pub type Definition = LC<String>;

/// Represents a node in our graph used for definitions
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum GraphNode {
    Term(Term),
    Definition(Definition),
}

/// Represents a list of terms and definitions, where a term can have multiple
/// definitions associated with it
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DefinitionList {
    graph: Graph<GraphNode, (), Undirected>,
}

impl DefinitionList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new, unconnected term to the list, returning an index that
    /// is used when connecting a term with a definition
    pub fn add_term(&mut self, term: Term) -> usize {
        self.graph.add_node(GraphNode::Term(term)).index()
    }

    /// Whether or not the list has the specified term
    pub fn has_term(&self, term: &Term) -> bool {
        self.find_term_index(term).is_some()
    }

    /// Adds a new, unconnected definition to the list, returning an index that
    /// is used when connecting a term with a definition
    pub fn add_definition(&mut self, def: Definition) -> usize {
        self.graph.add_node(GraphNode::Definition(def)).index()
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
    pub fn find_definitions(&self, term: &Term) -> Vec<&Definition> {
        let g = &self.graph;
        self.find_term_index(term)
            .map(|(_, idx)| {
                g.neighbors(idx)
                    .flat_map(|idx| match &g[idx] {
                        GraphNode::Definition(x) => Some(x),
                        _ => None,
                    })
                    .collect::<Vec<&Definition>>()
            })
            .unwrap_or_default()
    }

    /// Searches for all alternative terms to the specified term
    pub fn find_alternative_terms(&self, term: &Term) -> Vec<&Term> {
        let g = &self.graph;
        self.find_term_index(term)
            .map(|(term, idx)| {
                g.neighbors(idx)
                    .flat_map(|idx| match &g[idx] {
                        GraphNode::Definition(_) => Some(idx),
                        _ => None,
                    })
                    .flat_map(|idx| {
                        g.neighbors(idx)
                            .flat_map(|idx| match &g[idx] {
                                GraphNode::Term(x) if x != term => Some(x),
                                _ => None,
                            })
                            .collect::<Vec<&Term>>()
                    })
                    .collect::<Vec<&Term>>()
            })
            .unwrap_or_default()
    }

    /// Finds the index for a term through brute force
    fn find_term_index<'a>(
        &'a self,
        term: &'a Term,
    ) -> Option<(&'a Term, NodeIndex)> {
        let g = &self.graph;

        g.node_indices()
            .find(|i| match &g[*i] {
                GraphNode::Term(x) => x == term,
                _ => false,
            })
            .map(|idx| (term, idx))
    }
}

impl Eq for DefinitionList {}

impl PartialEq for DefinitionList {
    /// Compares two definition lists, ensuring that their terms and
    /// definitions are equal as well as the Term <-> Definition associations
    fn eq(&self, other: &Self) -> bool {
        // Implementation from Github comment:
        // https://github.com/petgraph/petgraph/issues/199#issuecomment-484077775
        let s_ns = self.graph.raw_nodes().iter().map(|n| &n.weight);
        let o_ns = other.graph.raw_nodes().iter().map(|n| &n.weight);
        let s_es = self
            .graph
            .raw_edges()
            .iter()
            .map(|e| (e.source(), e.target(), &e.weight));
        let o_es = other
            .graph
            .raw_edges()
            .iter()
            .map(|e| (e.source(), e.target(), &e.weight));
        s_ns.eq(o_ns) && s_es.eq(o_es)
    }
}

#[cfg(test)]
mod tests {
    use super::super::LC;
    use super::*;

    impl From<&str> for LC<String> {
        fn from(text: &str) -> Self {
            Self::from(text.to_string())
        }
    }

    #[test]
    fn find_definitions_should_list_all_definitions_for_term() {
        let mut dl = DefinitionList::new();

        let t1 = dl.add_term("term1".into());
        let d0 = dl.add_definition("def text".into());
        let d1 = dl.add_definition("def1".into());
        dl.connect_term_to_definition(t1, d0);
        dl.connect_term_to_definition(t1, d1);

        let t2 = dl.add_term("term2".into());
        let d2 = dl.add_definition("def2".into());
        dl.connect_term_to_definition(t2, d2);

        dl.add_term("term3".into());

        // Test looking for defs of term with multiple defs
        let defs = dl.find_definitions(&"term1".into());
        assert_eq!(2, defs.len());

        // Test looking for defs of term with one def
        let defs = dl.find_definitions(&"term2".into());
        assert_eq!(1, defs.len());

        // Test looking for defs of term with no defs
        let defs = dl.find_definitions(&"term3".into());
        assert_eq!(0, defs.len());

        // Test looking for defs of term that does not exist
        let defs = dl.find_definitions(&"term4".into());
        assert_eq!(0, defs.len());
    }

    #[test]
    fn find_alternative_terms_should_list_all_terms_with_same_definitions() {
        let mut dl = DefinitionList::new();

        let t0 = dl.add_term("term0".into());
        let t1 = dl.add_term("term1".into());
        let t2 = dl.add_term("term2".into());
        let t3 = dl.add_term("term3".into());
        let t4 = dl.add_term("term4".into());
        dl.add_term("term5".into());

        let d0 = dl.add_definition("def0".into());
        let d1 = dl.add_definition("def1".into());

        dl.connect_term_to_definition(t0, d0);
        dl.connect_term_to_definition(t1, d0);
        dl.connect_term_to_definition(t2, d0);

        dl.connect_term_to_definition(t3, d1);
        dl.connect_term_to_definition(t4, d1);

        // Test looking for alternate terms for term that has multiple
        let terms = dl.find_alternative_terms(&"term1".into());
        assert_eq!(terms.len(), 2);
        assert!(terms.contains(&(&LC::from("term0"))));
        assert!(terms.contains(&(&LC::from("term2"))));

        // Test looking for alternate terms for term that has one
        let terms = dl.find_alternative_terms(&"term3".into());
        assert_eq!(terms.len(), 1);
        assert_eq!(terms, vec![&LC::from("term4")]);

        // Test looking for alternate terms for term that has no alternatives
        let terms = dl.find_alternative_terms(&"term5".into());
        assert_eq!(terms.len(), 0);

        // Test looking for alternate terms for term that does not exist
        let terms = dl.find_alternative_terms(&"term999".into());
        assert_eq!(terms.len(), 0);
    }
}
