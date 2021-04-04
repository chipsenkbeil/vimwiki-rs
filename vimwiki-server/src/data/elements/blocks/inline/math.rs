use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use std::fmt;
use vimwiki::{elements as v, Located};

/// Represents a single document inline math formula
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct MathInline {
    /// The segment of the document this inline math covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The raw formula
    formula: String,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for MathInline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formula())
    }
}

impl<'a> FromVimwikiElement<'a> for MathInline {
    type Element = Located<v::MathInline<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(element.region()))
                .formula(element.into_inner().formula.to_string())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_math_inline!(r#"$some math$"#);
            let region = Region::from(element.region());
            let ent = MathInline::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.formula(), "some code");
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
