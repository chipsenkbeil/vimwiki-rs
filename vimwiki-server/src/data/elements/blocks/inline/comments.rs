use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};
use derive_more::Display;
use entity::*;
use std::fmt;
use vimwiki::{elements as v, Located};

/// Represents a single document comment
#[simple_ent]
#[derive(async_graphql::Union, Debug, Display)]
pub enum Comment {
    Line(LineComment),
    MultiLine(MultiLineComment),
}

impl Comment {
    pub fn region(&self) -> &Region {
        match self {
            Self::Line(x) => x.region(),
            Self::MultiLine(x) => x.region(),
        }
    }

    pub fn page_id(&self) -> Id {
        match self {
            Self::Line(x) => x.page_id(),
            Self::MultiLine(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Line(x) => x.parent_id(),
            Self::MultiLine(x) => x.parent_id(),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for Comment {
    type Element = Located<v::Comment<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = element.region();
        Ok(match element.into_inner() {
            v::Comment::Line(x) => {
                Self::Line(LineComment::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::Comment::MultiLine(x) => {
                Self::MultiLine(MultiLineComment::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
        })
    }
}

/// Represents a comment on a single line of a document
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct LineComment {
    /// The segment of the document this comment covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The line of content contained within this comment
    line: String,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for LineComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.line())
    }
}

impl<'a> FromVimwikiElement<'a> for LineComment {
    type Element = Located<v::LineComment<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(element.region()))
                .line(element.into_inner().0.to_string())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )
    }
}

/// Represents a comment that can potentially cross multiple lines of a document
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct MultiLineComment {
    /// The segment of the document this comment covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The lines of content contained within this comment
    lines: Vec<String>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for MultiLineComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.lines().iter() {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl<'a> FromVimwikiElement<'a> for MultiLineComment {
    type Element = Located<v::MultiLineComment<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(element.region()))
                .lines(
                    element
                        .into_inner()
                        .0
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
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
            let element = vimwiki_comment!(r#"%%some comment"#);
            let region = Region::from(element.region());
            let ent = Comment::from_vimwiki_element(999, Some(123), element)
                .expect("failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.to_string(), "some comment");
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));

            let element = vimwiki_comment!(r#"%%+some comment+%%"#);
            let region = Region::from(element.region());
            let ent = Comment::from_vimwiki_element(999, Some(123), element)
                .expect("failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.to_string(), "some comment");
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
