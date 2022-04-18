use crate::{
    lang::elements::{
        InlineBlockElement, InlineElement, InlineElementContainer,
        IntoChildren, Located, Region, Text,
    },
    ElementLike, StrictEq,
};
use derive_more::{
    AsMut, AsRef, Constructor, Deref, DerefMut, Display, Index, IndexMut, Into,
    IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
    iter::FromIterator,
};

/// Represents the newtype used for terms & definitions
#[derive(
    AsRef,
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    Display,
    Index,
    IndexMut,
    IntoIterator,
    Into,
    Serialize,
    Deserialize,
)]
#[as_ref(forward)]
#[display(fmt = "{}", _0)]
#[into_iterator(owned, ref, ref_mut)]
#[serde(transparent)]
pub struct DefinitionListValue<'a>(
    /// Represents the inner type that the definition list value wraps
    InlineElementContainer<'a>,
);

impl ElementLike for DefinitionListValue<'_> {}

impl<'a> DefinitionListValue<'a> {
    /// Returns reference to underlying container
    pub fn as_inner(&self) -> &InlineElementContainer<'a> {
        &self.0
    }

    /// Converts into underlying container
    pub fn into_inner(self) -> InlineElementContainer<'a> {
        self.0
    }
}

impl DefinitionListValue<'_> {
    pub fn to_borrowed(&self) -> DefinitionListValue {
        DefinitionListValue(self.0.to_borrowed())
    }

    pub fn into_owned(self) -> DefinitionListValue<'static> {
        DefinitionListValue(self.0.into_owned())
    }
}

impl<'a> IntoChildren for DefinitionListValue<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.0.into_children()
    }
}

impl<'a> Hash for DefinitionListValue<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_string().hash(state);
    }
}

impl<'a> Eq for DefinitionListValue<'a> {}

impl<'a> PartialEq for DefinitionListValue<'a> {
    #[allow(clippy::cmp_owned)]
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl<'a, 'b> PartialEq<InlineElementContainer<'b>> for DefinitionListValue<'a> {
    #[allow(clippy::cmp_owned)]
    fn eq(&self, other: &InlineElementContainer<'b>) -> bool {
        self.to_string() == other.to_string()
    }
}

impl<'a> PartialEq<String> for DefinitionListValue<'a> {
    fn eq(&self, other: &String) -> bool {
        &self.to_string() == other
    }
}

impl<'a, 'b> PartialEq<&'b str> for DefinitionListValue<'a> {
    fn eq(&self, other: &&'b str) -> bool {
        &self.to_string() == other
    }
}

impl<'a> From<&'a str> for DefinitionListValue<'a> {
    /// Special conversion to support wrapping str as a [`Text`] element,
    /// wrapped as an [`InlineElement`], wrapped as an [`InlineElementContainer`],
    /// and finally wrapped as a [`DefinitionListValue`]
    fn from(s: &'a str) -> Self {
        let element = InlineElement::Text(Text::from(s));
        let container =
            InlineElementContainer::new(vec![Located::from(element)]);
        Self(container)
    }
}

impl<'a> StrictEq for DefinitionListValue<'a> {
    /// Performs strict_eq on inner container
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self.0.strict_eq(&other.0)
    }
}

/// Represents the type alias used for a single term
pub type Term<'a> = DefinitionListValue<'a>;

/// Represents the type alias used for a single definition
pub type Definition<'a> = DefinitionListValue<'a>;

/// Represents a bundle of definitions tied to a singular term
#[derive(
    AsMut,
    AsRef,
    Constructor,
    Clone,
    Debug,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
#[into_iterator(owned, ref, ref_mut)]
pub struct DefinitionBundle<'a>(Vec<Located<Definition<'a>>>);

impl ElementLike for DefinitionBundle<'_> {}

impl<'a> DefinitionBundle<'a> {
    /// Creates a new, empty bundle
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Returns iterator over references to elements
    pub fn iter(&self) -> impl Iterator<Item = &Located<Definition<'a>>> {
        self.into_iter()
    }

    /// Returns iterator over mutable references to elements
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut Located<Definition<'a>>> {
        self.into_iter()
    }

    /// Returns total elements contained within container
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if container has no elements
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns reference to element at specified index, if it exists
    pub fn get(&self, idx: usize) -> Option<&Located<Definition<'a>>> {
        self.0.get(idx)
    }

    /// Returns the definitions contained within the bundle
    pub fn into_definitions(self) -> Vec<Located<Definition<'a>>> {
        self.0
    }
}

impl DefinitionBundle<'_> {
    pub fn to_borrowed(&self) -> DefinitionBundle {
        let elements = self
            .iter()
            .map(|x| x.as_ref().map(Definition::to_borrowed))
            .collect();

        DefinitionBundle::new(elements)
    }

    pub fn into_owned(self) -> DefinitionBundle<'static> {
        let elements = self
            .into_iter()
            .map(|x| x.map(Definition::into_owned))
            .collect();

        DefinitionBundle::new(elements)
    }
}

impl<'a> IntoChildren for DefinitionBundle<'a> {
    type Child = Located<Definition<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.0
    }
}

impl<'a> StrictEq for DefinitionBundle<'a> {
    /// Performs strict_eq on all definitions
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self.0.strict_eq(&other.0)
    }
}

impl<'a> fmt::Display for DefinitionBundle<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for le in self.iter() {
            write!(f, "{}", le.as_inner())?;
        }
        Ok(())
    }
}

/// Represents a term and its associated definitions
#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TermAndDefinitions<'a> {
    pub term: Located<Term<'a>>,
    pub definitions: Located<DefinitionBundle<'a>>,
}

impl ElementLike for TermAndDefinitions<'_> {}

impl TermAndDefinitions<'_> {
    pub fn to_borrowed(&self) -> TermAndDefinitions {
        let term = self.term.as_ref().map(DefinitionListValue::to_borrowed);
        let definitions =
            self.definitions.as_ref().map(DefinitionBundle::to_borrowed);

        TermAndDefinitions { term, definitions }
    }

    pub fn into_owned(self) -> TermAndDefinitions<'static> {
        let term = self.term.map(DefinitionListValue::into_owned);
        let definitions = self.definitions.map(DefinitionBundle::into_owned);

        TermAndDefinitions { term, definitions }
    }
}

impl<'a> StrictEq for TermAndDefinitions<'a> {
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self.term.strict_eq(&other.term)
            && self.definitions.strict_eq(&other.definitions)
    }
}

impl<'a> IntoChildren for TermAndDefinitions<'a> {
    type Child = Located<InlineBlockElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        std::iter::once(self.term.map(InlineBlockElement::Term))
            .chain(
                self.definitions
                    .into_inner()
                    .into_iter()
                    .map(|x| x.map(InlineBlockElement::Definition)),
            )
            .collect()
    }
}

/// Represents a list of terms and definitions, where a term can have multiple
/// definitions associated with it
#[derive(
    Constructor,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    IntoIterator,
)]
pub struct DefinitionList<'a> {
    #[into_iterator(owned, ref, ref_mut)]
    pub items: Vec<Located<TermAndDefinitions<'a>>>,
}

impl ElementLike for DefinitionList<'_> {}

impl DefinitionList<'_> {
    pub fn to_borrowed(&self) -> DefinitionList {
        DefinitionList::new(
            self.iter()
                .map(|x| x.as_ref().map(TermAndDefinitions::to_borrowed))
                .collect(),
        )
    }

    pub fn into_owned(self) -> DefinitionList<'static> {
        DefinitionList::new(
            self.into_iter()
                .map(|x| x.map(TermAndDefinitions::into_owned))
                .collect(),
        )
    }
}

impl<'a> DefinitionList<'a> {
    /// Retrieves definitions for an specific term by name
    pub fn get(&'a self, term: &str) -> Option<&Located<DefinitionBundle<'a>>> {
        self.items
            .iter()
            .find(|x| x.term.as_inner() == &term)
            .map(|x| &x.definitions)
    }

    /// Iterates through all terms and their associated definitions in the list
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &Located<TermAndDefinitions<'a>>> {
        self.items.iter()
    }

    /// Iterates through all terms in the list
    pub fn terms(&self) -> impl Iterator<Item = &Located<Term<'a>>> {
        self.iter().map(|x| &x.term)
    }

    /// Iterates through all definitions in the list
    pub fn definitions(
        &self,
    ) -> impl Iterator<Item = &Located<Definition<'a>>> {
        self.iter().flat_map(|x| x.definitions.iter())
    }
}

impl<'a> IntoChildren for DefinitionList<'a> {
    type Child = Located<InlineBlockElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.items
            .into_iter()
            .flat_map(|x| x.into_inner().into_children())
            .collect()
    }
}

impl<'a> FromIterator<(Located<Term<'a>>, Located<DefinitionBundle<'a>>)>
    for DefinitionList<'a>
{
    fn from_iter<
        I: IntoIterator<Item = (Located<Term<'a>>, Located<DefinitionBundle<'a>>)>,
    >(
        iter: I,
    ) -> Self {
        let mut dl = Self::default();

        for (term, def_bundle) in iter.into_iter() {
            // TODO: Should we perform more checks? Right now, we assume that these
            //       two follow immediately together to calculate the region
            let region = Region::new(
                term.region().offset(),
                term.region().len() + def_bundle.region().len(),
            );

            dl.items.push(Located::new(
                TermAndDefinitions::new(term, def_bundle),
                region,
            ));
        }

        dl
    }
}

impl<'a> StrictEq for DefinitionList<'a> {
    /// Performs strict_eq on inner mapping
    fn strict_eq(&self, other: &Self) -> bool {
        self.items.len() == other.items.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a.strict_eq(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InlineElement, Located};

    #[test]
    fn term_should_equal_other_instance_if_string_representations_are_same() {
        let t1 = Term::from("term");
        let t2 = Term::new(InlineElementContainer::new(vec![
            Located::from(InlineElement::Text("t".into())),
            Located::from(InlineElement::Text("e".into())),
            Located::from(InlineElement::Text("r".into())),
            Located::from(InlineElement::Text("m".into())),
        ]));
        assert_eq!(t1, t2);
    }

    #[test]
    fn term_should_equal_inline_element_container_if_string_representations_are_same(
    ) {
        let term = Term::from("term");
        let other = InlineElementContainer::new(vec![Located::from(
            InlineElement::Text("term".into()),
        )]);
        assert_eq!(term, other);
    }

    #[test]
    fn term_should_equal_string_if_string_representations_are_same() {
        let term = Term::from("term");
        let other = String::from("term");
        assert_eq!(term, other);
    }

    #[test]
    fn term_should_equal_str_slice_if_string_representations_are_same() {
        let term = Term::from("term");
        let other = "term";
        assert_eq!(term, other);
    }

    #[test]
    fn term_should_hash_using_its_string_representation() {
        let t1 = Term::from("term");
        let t2 = Term::new(InlineElementContainer::new(vec![
            Located::from(InlineElement::Text("t".into())),
            Located::from(InlineElement::Text("e".into())),
            Located::from(InlineElement::Text("r".into())),
            Located::from(InlineElement::Text("m".into())),
        ]));

        let mut hs = std::collections::HashMap::new();
        hs.insert(t1, vec![Definition::from("definition")]);
        assert_eq!(hs.len(), 1);
        assert!(hs.get(&t2).is_some());
    }

    #[test]
    fn definition_list_should_be_able_to_iterate_through_terms() {
        let dl: DefinitionList = vec![
            (
                Located::from(Term::from("term1")),
                Located::from(DefinitionBundle::empty()),
            ),
            (
                Located::from(Term::from("term2")),
                Located::from(DefinitionBundle::empty()),
            ),
        ]
        .into_iter()
        .collect();

        let term_names =
            dl.terms().map(|t| t.to_string()).collect::<Vec<String>>();
        assert_eq!(term_names.len(), 2);
        assert!(term_names.contains(&"term1".to_string()));
        assert!(term_names.contains(&"term2".to_string()));
    }

    #[test]
    fn definition_list_should_be_able_to_iterate_through_definitions_for_term()
    {
        let dl: DefinitionList = vec![
            (
                Located::from(Term::from("term1")),
                Located::from(DefinitionBundle::new(vec![Located::from(
                    Definition::from("definition"),
                )])),
            ),
            (
                Located::from(Term::from("term2")),
                Located::from(DefinitionBundle::empty()),
            ),
        ]
        .into_iter()
        .collect();

        let defs = dl
            .get("term1")
            .expect("Failed to find term")
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<String>>();
        assert_eq!(defs.len(), 1);
        assert!(defs.contains(&"definition".to_string()));

        let mut defs = dl.get("term2").expect("Failed to find term").iter();
        assert!(defs.next().is_none(), "Definitions is not empty");

        assert!(dl.get("term-unknown").is_none());
    }

    #[test]
    fn definition_list_should_support_lookup_with_terms_containing_other_inline_elements(
    ) {
        let dl: DefinitionList = vec![
            (
                Located::from(Term::from("term1")),
                Located::from(DefinitionBundle::new(vec![
                    Located::from(Definition::from("def1")),
                    Located::from(Definition::from("def2")),
                ])),
            ),
            (
                Located::from(Term::from("term2")),
                Located::from(DefinitionBundle::empty()),
            ),
        ]
        .into_iter()
        .collect();
        assert!(dl.get("term1").is_some());
    }
}
