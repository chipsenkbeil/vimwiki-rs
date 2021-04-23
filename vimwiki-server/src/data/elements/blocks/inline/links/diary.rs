use crate::data::{
    Anchor, Date, Description, Element, ElementQuery, FromVimwikiElement,
    GqlPageFilter, GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use entity_async_graphql::*;
use std::fmt;
use vimwiki::{elements as v, Located};

/// Represents a single document link to a diary entry
#[simple_ent]
#[derive(EntObject, EntFilter)]
pub struct DiaryLink {
    /// The segment of the document this link covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// Date of diary entry
    #[ent(field(graphql(filter_untyped)))]
    date: Date,

    /// Optional description associated with the link
    #[ent(field(graphql(filter_untyped)))]
    description: Option<Description>,

    /// Optional anchor associated with the link
    #[ent(field(graphql(filter_untyped)))]
    anchor: Option<Anchor>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for DiaryLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.date()),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for DiaryLink {
    type Element = Located<v::DiaryLink<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let element = element.into_inner();

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .date(Date::from(element.date))
                .description(element.description.map(Description::from))
                .anchor(element.anchor.map(Anchor::from))
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity_inmemory::InmemoryDatabase;
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_diary_link!(
                r#"[[diary:2021-04-03#one#two|Some description]]"#
            );
            let region = Region::from(element.region());
            let ent = DiaryLink::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.date(), &"2021-04-03".parse::<Date>().unwrap());
            assert_eq!(
                ent.description(),
                &Some(Description::Text(String::from("Some description")))
            );
            assert_eq!(ent.anchor(), &Some(Anchor::new(vec!["one", "two"])));
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
