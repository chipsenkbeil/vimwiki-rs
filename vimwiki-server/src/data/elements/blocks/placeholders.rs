use crate::data::{
    Date, Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use entity_async_graphql::*;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(async_graphql::Union, Debug)]
pub enum Placeholder {
    Title(PlaceholderTitle),
    NoHtml(PlaceholderNoHtml),
    Template(PlaceholderTemplate),
    Date(PlaceholderDate),
    Other(PlaceholderOther),
}

impl Placeholder {
    pub fn region(&self) -> &Region {
        match self {
            Self::Title(x) => x.region(),
            Self::NoHtml(x) => x.region(),
            Self::Template(x) => x.region(),
            Self::Date(x) => x.region(),
            Self::Other(x) => x.region(),
        }
    }

    pub fn page_id(&self) -> Id {
        match self {
            Self::Title(x) => x.page_id(),
            Self::NoHtml(x) => x.page_id(),
            Self::Template(x) => x.page_id(),
            Self::Date(x) => x.page_id(),
            Self::Other(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Title(x) => x.parent_id(),
            Self::NoHtml(x) => x.parent_id(),
            Self::Template(x) => x.parent_id(),
            Self::Date(x) => x.parent_id(),
            Self::Other(x) => x.parent_id(),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for Placeholder {
    type Element = Located<v::Placeholder<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        match element.into_inner() {
            v::Placeholder::Title(title) => GraphqlDatabaseError::wrap(
                PlaceholderTitle::build()
                    .region(region)
                    .title(title.to_string())
                    .page(page_id)
                    .parent(parent_id)
                    .finish_and_commit(),
            )
            .map(Self::from),
            v::Placeholder::NoHtml => GraphqlDatabaseError::wrap(
                PlaceholderNoHtml::build()
                    .region(region)
                    .page(page_id)
                    .parent(parent_id)
                    .finish_and_commit(),
            )
            .map(Self::from),
            v::Placeholder::Template(template) => GraphqlDatabaseError::wrap(
                PlaceholderTemplate::build()
                    .region(region)
                    .template(template.to_string())
                    .page(page_id)
                    .parent(parent_id)
                    .finish_and_commit(),
            )
            .map(Self::from),
            v::Placeholder::Date(date) => GraphqlDatabaseError::wrap(
                PlaceholderDate::build()
                    .region(region)
                    .date(Date::from(date))
                    .page(page_id)
                    .parent(parent_id)
                    .finish_and_commit(),
            )
            .map(Self::from),
            v::Placeholder::Other { name, value } => {
                GraphqlDatabaseError::wrap(
                    PlaceholderOther::build()
                        .region(region)
                        .name(name.to_string())
                        .value(value.to_string())
                        .page(page_id)
                        .parent(parent_id)
                        .finish_and_commit(),
                )
                .map(Self::from)
            }
        }
    }
}

/// Represents a single document title placeholder
#[simple_ent]
#[derive(EntObject, EntFilter)]
pub struct PlaceholderTitle {
    /// The segment of the document this placeholder covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The title associated with this placeholder
    title: String,

    /// Page containing the placeholder
    #[ent(edge)]
    page: Page,

    /// Parent element to this placeholder
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

/// Represents a single document nohtml placeholder
#[simple_ent]
#[derive(EntObject, EntFilter)]
pub struct PlaceholderNoHtml {
    /// The segment of the document this placeholder covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// Page containing the placeholder
    #[ent(edge)]
    page: Page,

    /// Parent element to this placeholder
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

/// Represents a single document template placeholder
#[simple_ent]
#[derive(EntObject, EntFilter)]
pub struct PlaceholderTemplate {
    /// The segment of the document this placeholder covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The template associated with this placeholder
    template: String,

    /// Page containing the placeholder
    #[ent(edge)]
    page: Page,

    /// Parent element to this placeholder
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

/// Represents a single document date placeholder
#[simple_ent]
#[derive(EntObject, EntFilter)]
pub struct PlaceholderDate {
    /// The segment of the document this placeholder covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The date associated with this placeholder
    #[ent(field(graphql(filter_untyped)))]
    date: Date,

    /// Page containing the placeholder
    #[ent(edge)]
    page: Page,

    /// Parent element to this placeholder
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

/// Represents a single document other placeholder
#[simple_ent]
#[derive(EntObject, EntFilter)]
pub struct PlaceholderOther {
    /// The segment of the document this placeholder covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The name associated with this placeholder
    name: String,

    /// The value associated with this placeholder
    value: String,

    /// Page containing the placeholder
    #[ent(edge)]
    page: Page,

    /// Parent element to this placeholder
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity_inmemory::InmemoryDatabase;
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_placeholder!(r#"%title some title"#);
            let region = Region::from(element.region());
            let ent =
                Placeholder::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
            assert!(matches!(ent, Placeholder::Title(_)));

            let element = vimwiki_placeholder!(r#"%nohtml"#);
            let region = Region::from(element.region());
            let ent =
                Placeholder::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
            assert!(matches!(ent, Placeholder::NoHtml(_)));

            let element = vimwiki_placeholder!(r#"%template some template"#);
            let region = Region::from(element.region());
            let ent =
                Placeholder::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
            assert!(matches!(ent, Placeholder::Template(_)));

            let element = vimwiki_placeholder!(r#"%date 2017-07-08"#);
            let region = Region::from(element.region());
            let ent =
                Placeholder::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
            assert!(matches!(ent, Placeholder::Date(_)));

            let element = vimwiki_placeholder!(r#"%other text"#);
            let region = Region::from(element.region());
            let ent =
                Placeholder::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
            assert!(matches!(ent, Placeholder::Other(_)));
        });
    }
}
