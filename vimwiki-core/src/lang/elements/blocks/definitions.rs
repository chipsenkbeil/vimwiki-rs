use crate::{
    lang::elements::{
        InlineBlockElement, InlineElement, InlineElementContainer,
        IntoChildren, Located,
    },
    StrictEq,
};
use derive_more::{
    AsRef, Constructor, Display, Index, IndexMut, Into, IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    iter::FromIterator,
};

/// Represents the newtype used for terms & definitions
#[derive(
    AsRef,
    Constructor,
    Clone,
    Debug,
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
pub struct DefinitionListValue<'a>(InlineElementContainer<'a>);

impl<'a> DefinitionListValue<'a> {
    pub fn as_inner(&self) -> &InlineElementContainer<'a> {
        &self.0
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

/// Represents a list of terms and definitions, where a term can have multiple
/// definitions associated with it
#[derive(
    Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, IntoIterator,
)]
pub struct DefinitionList<'a> {
    #[into_iterator(owned, ref, ref_mut)]
    #[serde(with = "serde_with::rust::map_as_tuple_list")]
    mapping: HashMap<Located<Term<'a>>, Vec<Located<Definition<'a>>>>,
}

impl DefinitionList<'_> {
    pub fn to_borrowed(&self) -> DefinitionList {
        let mapping = self
            .mapping
            .iter()
            .map(|(key, value)| {
                (
                    key.as_ref().map(DefinitionListValue::to_borrowed),
                    value
                        .iter()
                        .map(|x| {
                            x.as_ref().map(DefinitionListValue::to_borrowed)
                        })
                        .collect(),
                )
            })
            .collect();

        DefinitionList { mapping }
    }

    pub fn into_owned(self) -> DefinitionList<'static> {
        let mapping = self
            .mapping
            .into_iter()
            .map(|(key, value)| {
                (
                    key.map(DefinitionListValue::into_owned),
                    value
                        .into_iter()
                        .map(|x| x.map(DefinitionListValue::into_owned))
                        .collect(),
                )
            })
            .collect();

        DefinitionList { mapping }
    }
}

impl<'a> DefinitionList<'a> {
    pub fn new<
        I: IntoIterator<Item = (Located<Term<'a>>, Vec<Located<Definition<'a>>>)>,
    >(
        iter: I,
    ) -> Self {
        Self {
            mapping: iter.into_iter().collect(),
        }
    }

    /// Retrieves definitions for an specific term
    pub fn get(
        &'a self,
        term: impl Into<Term<'a>>,
    ) -> Option<&[Located<Definition<'a>>]> {
        self.mapping
            .get(&Located::from(term.into()))
            .map(AsRef::as_ref)
    }

    /// Iterates through all terms and their associated definitions in the list
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&Located<Term<'a>>, &[Located<Definition<'a>>])>
    {
        self.mapping.iter().map(|(k, v)| (k, v.as_slice()))
    }

    /// Iterates through all terms in the list
    pub fn terms(&self) -> impl Iterator<Item = &Located<Term<'a>>> {
        self.mapping.keys()
    }

    /// Iterates through all definitions in the list
    pub fn definitions(
        &self,
    ) -> impl Iterator<Item = &Located<Definition<'a>>> {
        self.mapping.values().flatten()
    }
}

impl<'a> IntoChildren for DefinitionList<'a> {
    type Child = Located<InlineBlockElement<'a>>;

    fn into_children(mut self) -> Vec<Self::Child> {
        self.mapping
            .drain()
            .flat_map(|(term, defs)| {
                std::iter::once(term.map(InlineBlockElement::Term)).chain(
                    defs.into_iter()
                        .map(|x| x.map(InlineBlockElement::Definition)),
                )
            })
            .collect()
    }
}

impl<'a, T: IntoIterator<Item = Located<Definition<'a>>>>
    FromIterator<(Located<Term<'a>>, T)> for DefinitionList<'a>
{
    fn from_iter<I: IntoIterator<Item = (Located<Term<'a>>, T)>>(
        iter: I,
    ) -> Self {
        let mut dl = Self::default();

        for (term, definitions) in iter.into_iter() {
            dl.mapping.insert(term, definitions.into_iter().collect());
        }

        dl
    }
}

impl<'a> StrictEq for DefinitionList<'a> {
    /// Performs strict_eq on inner mapping
    fn strict_eq(&self, other: &Self) -> bool {
        self.mapping.len() == other.mapping.len()
            && self.mapping.iter().all(|(key, value)| {
                other.mapping.get_key_value(key).map_or(false, |(k, v)| {
                    key.strict_eq(k) && value.strict_eq(v)
                })
            })
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

        let mut hs = HashMap::new();
        hs.insert(t1, vec![Definition::from("definition")]);
        assert_eq!(hs.len(), 1);
        assert!(hs.get(&t2).is_some());
    }

    #[test]
    fn definition_list_should_be_able_to_iterate_through_terms() {
        let dl = DefinitionList::from(vec![
            (Located::from(Term::from("term1")), vec![]),
            (Located::from(Term::from("term2")), vec![]),
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
            (
                Located::from(Term::from("term1")),
                vec![Located::from(Definition::from("definition"))],
            ),
            (Located::from(Term::from("term2")), vec![]),
        ]);

        let defs = dl
            .get("term1")
            .expect("Failed to find term")
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<String>>();
        assert_eq!(defs.len(), 1);
        assert!(defs.contains(&"definition".to_string()));

        let defs = dl
            .get("term2")
            .expect("Failed to find term")
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<String>>();
        assert!(defs.is_empty());

        assert!(dl.get("term-unknown").is_none());
    }

    #[test]
    fn definition_list_should_support_lookup_with_terms_containing_other_inline_elements(
    ) {
        let dl = DefinitionList::from(vec![
            (
                Located::from(Term::from("term1")),
                vec![
                    Located::from(Definition::from("def1")),
                    Located::from(Definition::from("def2")),
                ],
            ),
            (Located::from(Term::from("term2")), vec![]),
        ]);
        assert!(dl.get("term1").is_some());
    }
}
