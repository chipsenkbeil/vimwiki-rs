use super::utils::LC;
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

mod blockquotes;
pub use blockquotes::*;

mod comments;
pub use comments::*;

mod definitions;
pub use definitions::*;

mod dividers;
pub use dividers::*;

mod headers;
pub use headers::*;

mod links;
pub use links::*;

mod lists;
pub use lists::*;

mod math;
pub use math::*;

mod paragraphs;
pub use paragraphs::*;

mod placeholders;
pub use placeholders::*;

mod preformatted;
pub use preformatted::*;

mod tables;
pub use tables::*;

mod tags;
pub use tags::*;

mod typefaces;
pub use typefaces::*;

mod inline;
pub use inline::*;

/// Represents a full page containing different components
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Page {
    /// Comprised of the components within a page
    pub components: Vec<LC<BlockComponent>>,

    /// Comprised of the comments within a page
    pub comments: Vec<LC<Comment>>,
}

/// Represents components that are standalone (metaphorically a block element in CSS)
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockComponent {
    Header(Header),
    Paragraph(Paragraph),
    DefinitionList(DefinitionList),
    List(List),
    Table(Table),
    PreformattedText(PreformattedText),
    Math(MathBlock),
    Blockquote(Blockquote),
    Divider(Divider),
    Tags(Tags),
    Placeholder(Placeholder),
    NonBlankLine(String),
    BlankLine,
}

macro_rules! lc_mapping {
    ($type:ty) => {
        impl From<LC<$type>> for LC<BlockComponent> {
            fn from(component: LC<$type>) -> Self {
                component.map(BlockComponent::from)
            }
        }
    };
}

lc_mapping!(Header);
lc_mapping!(Paragraph);
lc_mapping!(DefinitionList);
lc_mapping!(List);
lc_mapping!(Table);
lc_mapping!(PreformattedText);
lc_mapping!(MathBlock);
lc_mapping!(Blockquote);
lc_mapping!(Divider);
lc_mapping!(Tags);
lc_mapping!(Placeholder);
lc_mapping!(String);
