mod tree;
pub use tree::{ElementNode, ElementTree};

/// Represents a collection of `ElementTree` instances and provides an API that
/// executes across all of them. Standardizes the id namespace across trees
/// such that accessing and manipulating `ElementNode` by their id is
/// straightforward.
#[derive(Clone, Debug, Default)]
pub struct ElementForest<'a> {
    trees: Vec<ElementTree<'a>>,
}
