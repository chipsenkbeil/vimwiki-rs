use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Keyword, Link, Page, PageQuery, Region, Text,
};
use derive_more::Display;
use entity::*;
use entity_async_graphql::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(EntFilter)]
pub struct DecoratedText {
    /// The segment of the document this decorated text covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The decoration applied to this decorated text
    #[ent(field(graphql(filter_untyped)))]
    decoration: Decoration,

    /// The contents of this decorated text
    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    contents: Vec<DecoratedTextContent>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for DecoratedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.load_contents() {
            Ok(contents) => {
                for content in contents {
                    write!(f, "{}", content.to_string())?;
                }
                Ok(())
            }
            Err(x) => {
                write!(f, "{}", x)?;
                Ok(())
            }
        }
    }
}

/// Represents some text (or series of inline content) that has a decoration
/// applied to it
#[async_graphql::Object]
impl DecoratedText {
    /// The segment of the document this decorated text is within
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        self.region()
    }

    /// The content within the decoration as individual elements
    #[graphql(name = "contents")]
    async fn gql_contents(
        &self,
    ) -> async_graphql::Result<Vec<DecoratedTextContent>> {
        self.load_contents()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The content within the decoration as it would be read by humans
    /// without frills
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.to_string()
    }

    /// Represents the decoration applied to some text
    #[graphql(name = "decoration")]
    async fn gql_decoration(&self) -> &Decoration {
        self.decoration()
    }

    /// The page containing this decorated text
    #[graphql(name = "page")]
    async fn gql_page(&self) -> async_graphql::Result<Page> {
        self.load_page()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The parent element containing this decorated text
    #[graphql(name = "parent")]
    async fn gql_parent(&self) -> async_graphql::Result<Option<Element>> {
        self.load_parent()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }
}

impl<'a> FromVimwikiElement<'a> for DecoratedText {
    type Element = Located<v::DecoratedText<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());

        // First, figure out the type of decoration
        let decoration = match element.as_inner() {
            v::DecoratedText::Bold(_) => Decoration::Bold,
            v::DecoratedText::Italic(_) => Decoration::Italic,
            v::DecoratedText::Strikeout(_) => Decoration::Strikeout,
            v::DecoratedText::Superscript(_) => Decoration::Superscript,
            v::DecoratedText::Subscript(_) => Decoration::Subscript,
        };

        // Second, we create the decorated text without content since we need
        // this ent's id to pass along as parent
        let mut ent = GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .decoration(decoration)
                .contents(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        // Third, we need to create all of the content contained within the text
        let mut contents = Vec::new();
        for content in element.into_inner().into_contents() {
            contents.push(
                DecoratedTextContent::from_vimwiki_element(
                    page_id,
                    Some(ent.id()),
                    content,
                )?
                .id(),
            );
        }

        ent.set_contents_ids(contents);
        ent.commit().map_err(GraphqlDatabaseError::Database)?;

        Ok(ent)
    }
}

/// Represents content that can be contained within a decoration
#[simple_ent]
#[derive(async_graphql::Union, Debug, Display)]
pub enum DecoratedTextContent {
    Text(Text),
    Keyword(Keyword),
    #[ent(wrap)]
    #[graphql(flatten)]
    Link(Link),
    DecoratedText(DecoratedText),
}

impl DecoratedTextContent {
    pub fn page_id(&self) -> Id {
        match self {
            Self::Text(x) => x.page_id(),
            Self::Keyword(x) => x.page_id(),
            Self::Link(x) => x.page_id(),
            Self::DecoratedText(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Text(x) => x.parent_id(),
            Self::Keyword(x) => x.parent_id(),
            Self::Link(x) => x.parent_id(),
            Self::DecoratedText(x) => x.parent_id(),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for DecoratedTextContent {
    type Element = Located<v::DecoratedTextContent<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = element.region();
        Ok(match element.into_inner() {
            v::DecoratedTextContent::Text(x) => {
                Self::Text(Text::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::DecoratedTextContent::DecoratedText(x) => {
                Self::DecoratedText(DecoratedText::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::DecoratedTextContent::Keyword(x) => {
                Self::Keyword(Keyword::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::DecoratedTextContent::Link(x) => {
                Self::Link(Link::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
        })
    }
}

/// Represents the type of decoration to apply to some text
#[derive(
    async_graphql::Enum,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
pub enum Decoration {
    Bold,
    Italic,
    Strikeout,
    Superscript,
    Subscript,
}

impl ValueLike for Decoration {
    fn into_value(self) -> Value {
        match self {
            Self::Bold => Value::from("bold"),
            Self::Italic => Value::from("italic"),
            Self::Strikeout => Value::from("strikeout"),
            Self::Superscript => Value::from("superscript"),
            Self::Subscript => Value::from("subscript"),
        }
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => match x.as_str() {
                "bold" => Ok(Self::Bold),
                "italic" => Ok(Self::Italic),
                "strikeout" => Ok(Self::Strikeout),
                "superscript" => Ok(Self::Superscript),
                "subscript" => Ok(Self::Subscript),
                _ => Err(Value::Text(x)),
            },
            x => Err(x),
        }
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
            let element = vimwiki_decorated_text!(r#"*some text*"#);
            let region = Region::from(element.region());
            let ent =
                DecoratedText::from_vimwiki_element(999, Some(123), element)
                    .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
            assert_eq!(ent.to_string(), "some text");
            for content in ent.load_contents().expect("Failed to load contents")
            {
                assert_eq!(content.page_id(), 999);
                assert_eq!(content.parent_id(), Some(ent.id()));
            }
        });
    }
}
