use crate::utils;
use std::{borrow::Cow, collections::HashMap, convert::TryFrom};
use vimwiki::{
    self as v,
    vendor::{chrono, uriparse},
    ToHtmlString,
};
use wasm_bindgen::prelude::*;

/// Represents a wrapper around a vimwiki page
#[wasm_bindgen]
pub struct Page(v::Page<'static>);

#[wasm_bindgen]
impl Page {
    /// Returns top-level element at the given index if it exists
    pub fn element_at(&self, idx: usize) -> Option<BlockElement> {
        self.0
            .elements
            .get(idx)
            .map(|x| BlockElement(x.to_borrowed().into_owned()))
    }

    /// Represents total number of top-level elements within the page
    #[wasm_bindgen(getter)]
    pub fn element_cnt(&self) -> usize {
        self.0.elements.len()
    }
}

/// Represents a wrapper around a vimwiki element
#[wasm_bindgen]
pub struct Element(v::Element<'static>);

#[wasm_bindgen]
impl Element {
    /// Returns true if element is block element
    pub fn is_block(&self) -> bool {
        matches!(self.0, v::Element::Block(_))
    }

    /// Casts to block element if it is one
    pub fn as_block(&self) -> Option<BlockElement> {
        match self.0 {
            v::Element::Block(ref x) => {
                Some(BlockElement(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is inline element
    pub fn is_inline(&self) -> bool {
        matches!(self.0, v::Element::Inline(_))
    }

    /// Casts to inline element if it is one
    pub fn as_inline(&self) -> Option<InlineElement> {
        match self.0 {
            v::Element::Inline(ref x) => {
                Some(InlineElement(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is inline block element
    pub fn is_inline_block(&self) -> bool {
        matches!(self.0, v::Element::InlineBlock(_))
    }

    /// Casts to inline block element if it is one
    pub fn as_inline_block(&self) -> Option<InlineBlockElement> {
        match self.0 {
            v::Element::InlineBlock(ref x) => {
                Some(InlineBlockElement(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }
}

/// Represents a wrapper around a vimwiki block element
#[wasm_bindgen]
pub struct BlockElement(v::BlockElement<'static>);

#[wasm_bindgen]
impl BlockElement {
    /// Returns true if element is blockquote
    pub fn is_blockquote(&self) -> bool {
        matches!(self.0, v::BlockElement::Blockquote(_))
    }

    /// Casts to blockquote if it is one
    pub fn as_blockquote(&self) -> Option<Blockquote> {
        match self.0 {
            v::BlockElement::Blockquote(ref x) => {
                Some(Blockquote(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is code block
    pub fn is_code(&self) -> bool {
        matches!(self.0, v::BlockElement::CodeBlock(_))
    }

    /// Casts to code block if it is one
    pub fn as_code(&self) -> Option<CodeBlock> {
        match self.0 {
            v::BlockElement::CodeBlock(ref x) => {
                Some(CodeBlock(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is math block
    pub fn is_math(&self) -> bool {
        matches!(self.0, v::BlockElement::Math(_))
    }

    /// Casts to math block if it is one
    pub fn as_math(&self) -> Option<MathBlock> {
        match self.0 {
            v::BlockElement::Math(ref x) => {
                Some(MathBlock(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is definition list
    pub fn is_definition_list(&self) -> bool {
        matches!(self.0, v::BlockElement::DefinitionList(_))
    }

    /// Casts to definition list if it is one
    pub fn as_definition_list(&self) -> Option<DefinitionList> {
        match self.0 {
            v::BlockElement::DefinitionList(ref x) => {
                Some(DefinitionList(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is divider
    pub fn is_divider(&self) -> bool {
        matches!(self.0, v::BlockElement::Divider(_))
    }

    /// Casts to divider if it is one
    pub fn as_divider(&self) -> Option<Divider> {
        match self.0 {
            v::BlockElement::Divider(ref x) => Some(Divider(*x)),
            _ => None,
        }
    }

    /// Returns true if element is header
    pub fn is_header(&self) -> bool {
        matches!(self.0, v::BlockElement::Header(_))
    }

    /// Casts to definition list if it is one
    pub fn as_header(&self) -> Option<Header> {
        match self.0 {
            v::BlockElement::Header(ref x) => {
                Some(Header(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is list
    pub fn is_list(&self) -> bool {
        matches!(self.0, v::BlockElement::List(_))
    }

    /// Casts to list if it is one
    pub fn as_list(&self) -> Option<List> {
        match self.0 {
            v::BlockElement::List(ref x) => {
                Some(List(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is paragraph
    pub fn is_paragraph(&self) -> bool {
        matches!(self.0, v::BlockElement::Paragraph(_))
    }

    /// Casts to paragraph if it is one
    pub fn as_paragraph(&self) -> Option<Paragraph> {
        match self.0 {
            v::BlockElement::Paragraph(ref x) => {
                Some(Paragraph(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is placeholder
    pub fn is_placeholder(&self) -> bool {
        matches!(self.0, v::BlockElement::Placeholder(_))
    }

    /// Casts to placeholder if it is one
    pub fn as_placeholder(&self) -> Option<Placeholder> {
        match self.0 {
            v::BlockElement::Placeholder(ref x) => {
                Some(Placeholder(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is table
    pub fn is_table(&self) -> bool {
        matches!(self.0, v::BlockElement::Table(_))
    }

    /// Casts to table if it is one
    pub fn as_table(&self) -> Option<Table> {
        match self.0 {
            v::BlockElement::Table(ref x) => {
                Some(Table(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }
}

/// Represents a wrapper around a vimwiki inline block element
#[wasm_bindgen]
pub struct InlineBlockElement(v::InlineBlockElement<'static>);

#[wasm_bindgen]
impl InlineBlockElement {
    /// Returns true if element is list item
    pub fn is_list_item(&self) -> bool {
        matches!(self.0, v::InlineBlockElement::ListItem(_))
    }

    /// Casts to list item if it is one
    pub fn as_list_item(&self) -> Option<ListItem> {
        match self.0 {
            v::InlineBlockElement::ListItem(ref x) => {
                Some(ListItem(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is term
    pub fn is_term(&self) -> bool {
        matches!(self.0, v::InlineBlockElement::Term(_))
    }

    /// Casts to term if it is one
    pub fn as_term(&self) -> Option<InlineElementContainer> {
        match self.0 {
            v::InlineBlockElement::Term(ref x) => Some(InlineElementContainer(
                x.as_inner().to_borrowed().into_owned(),
            )),
            _ => None,
        }
    }

    /// Returns true if element is definition
    pub fn is_definition(&self) -> bool {
        matches!(self.0, v::InlineBlockElement::Definition(_))
    }

    /// Casts to definition if it is one
    pub fn as_definition(&self) -> Option<InlineElementContainer> {
        match self.0 {
            v::InlineBlockElement::Definition(ref x) => Some(
                InlineElementContainer(x.as_inner().to_borrowed().into_owned()),
            ),
            _ => None,
        }
    }
}

/// Represents a wrapper around a vimwiki inline element
#[wasm_bindgen]
pub struct InlineElement(v::InlineElement<'static>);

#[wasm_bindgen]
impl InlineElement {
    /// Returns true if element is text
    pub fn is_text(&self) -> bool {
        matches!(self.0, v::InlineElement::Text(_))
    }

    /// Casts to text if it is one
    pub fn as_text(&self) -> Option<Text> {
        match self.0 {
            v::InlineElement::Text(ref x) => {
                Some(Text(x.as_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is decorated text
    pub fn is_decorated_text(&self) -> bool {
        matches!(self.0, v::InlineElement::DecoratedText(_))
    }

    /// Casts to decorated text if it is one
    pub fn as_decorated_text(&self) -> Option<DecoratedText> {
        match self.0 {
            v::InlineElement::DecoratedText(ref x) => {
                Some(DecoratedText(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is keyword
    pub fn is_keyword(&self) -> bool {
        matches!(self.0, v::InlineElement::Keyword(_))
    }

    /// Casts to keyword if it is one
    pub fn as_keyword(&self) -> Option<Keyword> {
        match self.0 {
            v::InlineElement::Keyword(ref x) => Some(Keyword::from(*x)),
            _ => None,
        }
    }

    /// Returns true if element is link
    pub fn is_link(&self) -> bool {
        matches!(self.0, v::InlineElement::Link(_))
    }

    /// Casts to link if it is one
    pub fn as_link(&self) -> Option<Link> {
        match self.0 {
            v::InlineElement::Link(ref x) => {
                Some(Link(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is tag set
    pub fn is_tags(&self) -> bool {
        matches!(self.0, v::InlineElement::Tags(_))
    }

    /// Casts to tag set if it is one
    pub fn as_tags(&self) -> Option<Tags> {
        match self.0 {
            v::InlineElement::Tags(ref x) => {
                Some(Tags(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is inline code
    pub fn is_code(&self) -> bool {
        matches!(self.0, v::InlineElement::Code(_))
    }

    /// Casts to inline code if it is one
    pub fn as_code(&self) -> Option<CodeInline> {
        match self.0 {
            v::InlineElement::Code(ref x) => {
                Some(CodeInline(x.as_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is inline math
    pub fn is_math(&self) -> bool {
        matches!(self.0, v::InlineElement::Math(_))
    }

    /// Casts to inline math if it is one
    pub fn as_math(&self) -> Option<MathInline> {
        match self.0 {
            v::InlineElement::Math(ref x) => {
                Some(MathInline(x.as_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if element is comment
    pub fn is_comment(&self) -> bool {
        matches!(self.0, v::InlineElement::Comment(_))
    }

    /// Casts to comment if it is one
    pub fn as_comment(&self) -> Option<Comment> {
        match self.0 {
            v::InlineElement::Comment(ref x) => {
                Some(Comment(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }
}

/// Represents a wrapper around a vimwiki inline element container
#[wasm_bindgen]
pub struct InlineElementContainer(v::InlineElementContainer<'static>);

#[wasm_bindgen]
impl InlineElementContainer {
    /// Returns element at the given index if it exists
    pub fn element_at(&self, idx: usize) -> Option<InlineElement> {
        self.0
            .get(idx)
            .map(|x| InlineElement(x.to_borrowed().into_owned()))
    }

    /// Represents total number of elements within container
    #[wasm_bindgen(getter)]
    pub fn element_cnt(&self) -> usize {
        self.0.len()
    }

    /// Converts container to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a wrapper around a vimwiki blockquote
#[wasm_bindgen]
pub struct Blockquote(v::Blockquote<'static>);

#[wasm_bindgen]
impl Blockquote {
    /// Returns line at the given index if it exists
    pub fn line_at(&self, idx: usize) -> Option<String> {
        self.0.lines.get(idx).map(ToString::to_string)
    }

    /// Represents total number of lines within the blockquote
    #[wasm_bindgen(getter)]
    pub fn line_cnt(&self) -> usize {
        self.0.lines.len()
    }
}

/// Represents a wrapper around a vimwiki code block
#[wasm_bindgen]
pub struct CodeBlock(v::CodeBlock<'static>);

#[wasm_bindgen]
impl CodeBlock {
    /// Represents the language associated with the code block
    #[wasm_bindgen(getter)]
    pub fn lang(&self) -> Option<String> {
        self.0.lang.as_ref().map(ToString::to_string)
    }

    /// Returns object containing metadata of code block
    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> Option<js_sys::Object> {
        let arr = js_sys::Array::new();

        for (key, value) in self.0.metadata.iter() {
            let tuple = js_sys::Array::new();
            tuple.push(&JsValue::from_str(key.as_ref()));
            tuple.push(&JsValue::from_str(value.as_ref()));

            arr.push(&tuple);
        }

        js_sys::Object::from_entries(&arr).ok()
    }

    /// Returns line at the given index if it exists
    pub fn line_at(&self, idx: usize) -> Option<String> {
        self.0.lines.get(idx).map(ToString::to_string)
    }

    /// Represents total number of lines within the block
    #[wasm_bindgen(getter)]
    pub fn line_cnt(&self) -> usize {
        self.0.lines.len()
    }
}

/// Represents a wrapper around a vimwiki definition list
#[wasm_bindgen]
pub struct DefinitionList(v::DefinitionList<'static>);

#[wasm_bindgen]
impl DefinitionList {
    /// Represents the terms stored in the list
    #[wasm_bindgen(getter)]
    pub fn terms(&self) -> Vec<JsValue> {
        self.0
            .terms()
            .map(ToString::to_string)
            .map(|x| JsValue::from_str(&x))
            .collect()
    }

    /// Returns the definition associated with the specified term
    pub fn get_def(&self, term: &str) -> Option<InlineElementContainer> {
        self.0.get(term).map(|x| {
            InlineElementContainer(v::InlineElementContainer::from(
                x.iter()
                    .map(|x| x.as_inner().as_inner().to_borrowed().into_owned())
                    .collect::<Vec<v::InlineElementContainer>>(),
            ))
        })
    }

    /// Represents total number of terms stored in the list
    #[wasm_bindgen(getter)]
    pub fn term_cnt(&self) -> usize {
        self.0.terms().count()
    }

    /// Represents total number of definitions stored in the list
    #[wasm_bindgen(getter)]
    pub fn def_cnt(&self) -> usize {
        self.0.definitions().count()
    }
}

/// Represents a wrapper around a vimwiki divider
#[wasm_bindgen]
pub struct Divider(v::Divider);

/// Represents a wrapper around a vimwiki header
#[wasm_bindgen]
pub struct Header(v::Header<'static>);

#[wasm_bindgen]
impl Header {
    /// Represents the level of the header
    #[wasm_bindgen(getter)]
    pub fn level(&self) -> usize {
        self.0.level
    }

    /// Represents the content of the header
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> InlineElementContainer {
        InlineElementContainer(self.0.content.to_borrowed().into_owned())
    }

    /// Represents whether or not the header is centered
    #[wasm_bindgen(getter)]
    pub fn centered(&self) -> bool {
        self.0.centered
    }

    /// Converts paragraph to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.content.to_string()
    }
}

/// Represents a wrapper around a vimwiki list
#[wasm_bindgen]
pub struct List(v::List<'static>);

#[wasm_bindgen]
impl List {
    /// Returns list item at the given index if it exists
    pub fn item_at(&self, idx: usize) -> Option<ListItem> {
        self.0
            .items
            .get(idx)
            .map(|x| ListItem(x.to_borrowed().into_owned()))
    }

    /// Represents total number of items within list
    #[wasm_bindgen(getter)]
    pub fn item_cnt(&self) -> usize {
        self.0.items.len()
    }
}

/// Represents a wrapper around a vimwiki list item
#[wasm_bindgen]
pub struct ListItem(v::ListItem<'static>);

#[wasm_bindgen]
impl ListItem {
    /// Represents position of list item within list
    #[wasm_bindgen(getter)]
    pub fn pos(&self) -> usize {
        self.0.pos
    }

    /// Represents contents contained within list item
    #[wasm_bindgen(getter)]
    pub fn contents(&self) -> ListItemContents {
        ListItemContents(self.0.contents.to_borrowed().into_owned())
    }

    /// Represents the prefix of list item (e.g. hyphen or roman numeral)
    #[wasm_bindgen(getter)]
    pub fn prefix(&self) -> String {
        self.0.item_type.to_prefix(self.0.pos, self.0.suffix)
    }

    /// Represents suffix of list item (e.g. period or paren)
    #[wasm_bindgen(getter)]
    pub fn suffix(&self) -> ListItemSuffix {
        ListItemSuffix::from(self.0.suffix)
    }

    /// Represents attributes of list item
    #[wasm_bindgen(getter)]
    pub fn attributes(&self) -> ListItemAttributes {
        ListItemAttributes(self.0.attributes)
    }

    /// Returns true if list item is ordered type
    pub fn is_ordered(&self) -> bool {
        self.0.item_type.is_ordered()
    }

    /// Returns true if list item is unordered type
    pub fn is_unordered(&self) -> bool {
        self.0.item_type.is_unordered()
    }
}

/// Represents contents contained within list item
#[wasm_bindgen]
pub struct ListItemContents(v::ListItemContents<'static>);

#[wasm_bindgen]
impl ListItemContents {
    /// Returns content at the given index if it exists
    pub fn content_at(&self, idx: usize) -> Option<ListItemContent> {
        self.0
            .get(idx)
            .map(|x| ListItemContent(x.to_borrowed().into_owned()))
    }

    /// Represents total number of separate content within the list item contents
    #[wasm_bindgen(getter)]
    pub fn content_cnt(&self) -> usize {
        self.0.len()
    }
}

/// Represents a singular piece of list item content
#[wasm_bindgen]
pub struct ListItemContent(v::ListItemContent<'static>);

#[wasm_bindgen]
impl ListItemContent {
    /// Casts to list if it is one
    pub fn as_list(&self) -> Option<List> {
        match self.0 {
            v::ListItemContent::List(ref x) => {
                Some(List(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Casts to inline element container if it is one
    pub fn as_inline(&self) -> Option<InlineElementContainer> {
        match self.0 {
            v::ListItemContent::InlineContent(ref x) => {
                Some(InlineElementContainer(x.to_borrowed().into_owned()))
            }
            _ => None,
        }
    }

    /// Returns true if content is a sublist
    pub fn is_list(&self) -> bool {
        matches!(self.0, v::ListItemContent::List(_))
    }

    /// Returns true if content is inline content
    pub fn is_inline(&self) -> bool {
        matches!(self.0, v::ListItemContent::InlineContent(_))
    }
}

/// Represents attributes associated with a list item
#[wasm_bindgen]
pub struct ListItemAttributes(v::ListItemAttributes);

#[wasm_bindgen]
impl ListItemAttributes {
    /// Represents the todo status of the list item
    #[wasm_bindgen(getter)]
    pub fn todo_status(&self) -> Option<ListItemTodoStatus> {
        self.0
            .todo_status
            .as_ref()
            .copied()
            .map(ListItemTodoStatus::from)
    }

    pub fn is_todo_incomplete(&self) -> bool {
        matches!(self.0.todo_status, Some(v::ListItemTodoStatus::Incomplete))
    }

    pub fn is_todo_partially_complete_1(&self) -> bool {
        matches!(
            self.0.todo_status,
            Some(v::ListItemTodoStatus::PartiallyComplete1)
        )
    }

    pub fn is_todo_partially_complete_2(&self) -> bool {
        matches!(
            self.0.todo_status,
            Some(v::ListItemTodoStatus::PartiallyComplete2)
        )
    }

    pub fn is_todo_partially_complete_3(&self) -> bool {
        matches!(
            self.0.todo_status,
            Some(v::ListItemTodoStatus::PartiallyComplete3)
        )
    }

    pub fn is_todo_complete(&self) -> bool {
        matches!(self.0.todo_status, Some(v::ListItemTodoStatus::Complete))
    }

    pub fn is_todo_rejected(&self) -> bool {
        matches!(self.0.todo_status, Some(v::ListItemTodoStatus::Rejected))
    }
}

/// Represents the todo status for a list item
#[wasm_bindgen]
pub enum ListItemTodoStatus {
    Incomplete = "incomplete",
    PartiallyComplete1 = "partially_complete_1",
    PartiallyComplete2 = "partially_complete_2",
    PartiallyComplete3 = "partially_complete_3",
    Complete = "complete",
    Rejected = "rejected",
}

impl ListItemTodoStatus {
    pub fn to_vimwiki(&self) -> Option<v::ListItemTodoStatus> {
        match self {
            Self::Incomplete => Some(v::ListItemTodoStatus::Incomplete),
            Self::PartiallyComplete1 => {
                Some(v::ListItemTodoStatus::PartiallyComplete1)
            }
            Self::PartiallyComplete2 => {
                Some(v::ListItemTodoStatus::PartiallyComplete2)
            }
            Self::PartiallyComplete3 => {
                Some(v::ListItemTodoStatus::PartiallyComplete3)
            }
            Self::Complete => Some(v::ListItemTodoStatus::Complete),
            Self::Rejected => Some(v::ListItemTodoStatus::Rejected),
            _ => None,
        }
    }
}

impl From<v::ListItemTodoStatus> for ListItemTodoStatus {
    fn from(x: v::ListItemTodoStatus) -> Self {
        match x {
            v::ListItemTodoStatus::Incomplete => Self::Incomplete,
            v::ListItemTodoStatus::PartiallyComplete1 => {
                Self::PartiallyComplete1
            }
            v::ListItemTodoStatus::PartiallyComplete2 => {
                Self::PartiallyComplete2
            }
            v::ListItemTodoStatus::PartiallyComplete3 => {
                Self::PartiallyComplete3
            }
            v::ListItemTodoStatus::Complete => Self::Complete,
            v::ListItemTodoStatus::Rejected => Self::Rejected,
        }
    }
}

#[wasm_bindgen]
pub enum ListItemSuffix {
    None = "none",
    Period = "period",
    Paren = "paren",
}

impl ListItemSuffix {
    pub fn to_vimwiki(&self) -> Option<v::ListItemSuffix> {
        match self {
            Self::None => Some(v::ListItemSuffix::None),
            Self::Period => Some(v::ListItemSuffix::Period),
            Self::Paren => Some(v::ListItemSuffix::Paren),
            _ => None,
        }
    }
}

impl From<v::ListItemSuffix> for ListItemSuffix {
    fn from(x: v::ListItemSuffix) -> Self {
        match x {
            v::ListItemSuffix::None => Self::None,
            v::ListItemSuffix::Period => Self::Period,
            v::ListItemSuffix::Paren => Self::Paren,
        }
    }
}

/// Represents a wrapper around a vimwiki math block
#[wasm_bindgen]
pub struct MathBlock(v::MathBlock<'static>);

#[wasm_bindgen]
impl MathBlock {
    /// Represents the environment associated with the math block
    #[wasm_bindgen(getter)]
    pub fn environment(&self) -> Option<String> {
        self.0.environment.as_ref().map(ToString::to_string)
    }

    /// Returns line at the given index if it exists
    pub fn line_at(&self, idx: usize) -> Option<String> {
        self.0.lines.get(idx).map(ToString::to_string)
    }

    /// Represents total number of lines within the block
    #[wasm_bindgen(getter)]
    pub fn line_cnt(&self) -> usize {
        self.0.lines.len()
    }
}

/// Represents a wrapper around a vimwiki paragraph
#[wasm_bindgen]
pub struct Paragraph(v::Paragraph<'static>);

#[wasm_bindgen]
impl Paragraph {
    /// Returns line as inline element container at the given index if it exists
    pub fn line_at(&self, idx: usize) -> Option<InlineElementContainer> {
        self.0
            .lines
            .get(idx)
            .map(|x| InlineElementContainer(x.to_borrowed().into_owned()))
    }

    /// Represents total number of lines within the paragraph
    #[wasm_bindgen(getter)]
    pub fn line_cnt(&self) -> usize {
        self.0.lines.len()
    }

    /// Converts paragraph to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0
            .lines
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("\n")
    }
}

/// Represents a wrapper around a vimwiki placeholder
#[wasm_bindgen]
pub struct Placeholder(v::Placeholder<'static>);

#[wasm_bindgen]
impl Placeholder {
    /// Represents the title associated with the placeholder if it has one
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> Option<String> {
        match self.0 {
            v::Placeholder::Title(ref x) => Some(x.to_string()),
            _ => None,
        }
    }

    /// Represents the template associated with the placeholder if it has one
    #[wasm_bindgen(getter)]
    pub fn template(&self) -> Option<String> {
        match self.0 {
            v::Placeholder::Template(ref x) => Some(x.to_string()),
            _ => None,
        }
    }

    /// Represents the date associated with the placeholder if it has one
    #[wasm_bindgen(getter)]
    pub fn date(&self) -> Option<js_sys::Date> {
        use chrono::Datelike;
        match self.0 {
            v::Placeholder::Date(ref x) => {
                Some(js_sys::Date::new_with_year_month_day(
                    x.year() as u32,
                    x.month() as i32,
                    x.day() as i32,
                ))
            }
            _ => None,
        }
    }

    /// Represents the other placeholder's name if it has one
    #[wasm_bindgen(getter)]
    pub fn other_name(&self) -> Option<String> {
        match self.0 {
            v::Placeholder::Other { ref name, .. } => Some(name.to_string()),
            _ => None,
        }
    }

    /// Represents the other placeholder's value if it has one
    #[wasm_bindgen(getter)]
    pub fn other_value(&self) -> Option<String> {
        match self.0 {
            v::Placeholder::Other { ref value, .. } => Some(value.to_string()),
            _ => None,
        }
    }

    /// Returns true if placeholder represents a flag for no html output
    pub fn is_no_html(&self) -> bool {
        matches!(self.0, v::Placeholder::NoHtml)
    }
}

/// Represents a wrapper around a vimwiki table
#[wasm_bindgen]
pub struct Table(v::Table<'static>);

#[wasm_bindgen]
impl Table {
    /// Creates a new table from the given cells and centered status
    ///
    /// Cells are a 2D matrix
    #[wasm_bindgen(constructor)]
    pub fn new(cells: js_sys::Array, centered: bool) -> Result<Table, JsValue> {
        Ok(Self(v::Table::new(
            cells
                .iter()
                .map(js_sys::Array::try_from)
                .enumerate()
                .filter_map(|(row, res)| {
                    res.map(|arr| {
                        arr.iter()
                            .filter_map(|x| {
                                utils::cast_value::<Cell>(x, "Cell").ok()
                            })
                            .enumerate()
                            .map(|(col, x)| {
                                (v::CellPos { row, col }, v::Located::from(x.0))
                            })
                            .collect::<Vec<(v::CellPos, v::Located<v::Cell>)>>()
                    })
                    .ok()
                })
                .flatten(),
            centered,
        )))
    }

    /// Returns cell at the given row & column if it exists
    pub fn cell_at(&self, row: usize, col: usize) -> Option<Cell> {
        self.0
            .get_cell(row, col)
            .map(|x| Cell(x.to_borrowed().into_owned()))
    }

    /// Returns total number of rows
    #[wasm_bindgen(getter)]
    pub fn row_cnt(&self) -> usize {
        self.0.row_cnt()
    }

    /// Returns total number of columns
    #[wasm_bindgen(getter)]
    pub fn col_cnt(&self) -> usize {
        self.0.col_cnt()
    }

    /// Returns true if centered
    #[wasm_bindgen(getter)]
    pub fn centered(&self) -> bool {
        self.0.is_centered()
    }
}

/// Represents a wrapper around a vimwiki table cell
#[wasm_bindgen]
pub struct Cell(v::Cell<'static>);

#[wasm_bindgen]
impl Cell {
    /// Creates a new table cell from the given string
    #[wasm_bindgen(constructor)]
    pub fn new(txt: &str) -> Result<Cell, JsValue> {
        Ok(Self(
            v::Cell::Content(v::Located::from(v::Text::from(txt)).into())
                .into_owned(),
        ))
    }

    /// Represents content contained within cell if it has content
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> Option<InlineElementContainer> {
        self.0
            .get_content()
            .map(|x| InlineElementContainer(x.to_borrowed().into_owned()))
    }

    /// Returns true if cell is a span type
    pub fn is_span(&self) -> bool {
        self.0.is_span()
    }

    /// Returns true if cell is a span from above type
    pub fn is_span_from_above(&self) -> bool {
        self.0
            .get_span()
            .map(|x| matches!(x, v::CellSpan::FromAbove))
            .unwrap_or_default()
    }

    /// Returns true if cell is a span from left type
    pub fn is_span_from_left(&self) -> bool {
        self.0
            .get_span()
            .map(|x| matches!(x, v::CellSpan::FromLeft))
            .unwrap_or_default()
    }

    /// Returns true if cell is a column alignment type
    pub fn is_align(&self) -> bool {
        self.0.is_align()
    }

    /// Returns true if cell is a column alignment left type
    pub fn is_align_left(&self) -> bool {
        self.0
            .get_align()
            .map(|x| matches!(x, v::ColumnAlign::Left))
            .unwrap_or_default()
    }

    /// Returns true if cell is a column alignment centered type
    pub fn is_align_centered(&self) -> bool {
        self.0
            .get_align()
            .map(|x| matches!(x, v::ColumnAlign::Center))
            .unwrap_or_default()
    }

    /// Returns true if cell is a column alignment right type
    pub fn is_align_right(&self) -> bool {
        self.0
            .get_align()
            .map(|x| matches!(x, v::ColumnAlign::Right))
            .unwrap_or_default()
    }

    /// Converts cell to a JavaScript string if it is content
    pub fn to_str(&self) -> Option<String> {
        self.0.get_content().map(ToString::to_string)
    }
}

/// Represents a wrapper around a vimwiki decorated text
#[wasm_bindgen]
pub struct DecoratedText(v::DecoratedText<'static>);

#[wasm_bindgen]
impl DecoratedText {
    /// Creates some new bold text
    pub fn new_bold_text(txt: &str) -> DecoratedText {
        Self(
            v::DecoratedText::Bold(vec![v::Located::from(
                v::DecoratedTextContent::Text(v::Text::from(txt)),
            )])
            .into_owned(),
        )
    }

    /// Creates some new italic text
    pub fn new_italic_text(txt: &str) -> DecoratedText {
        Self(
            v::DecoratedText::Italic(vec![v::Located::from(
                v::DecoratedTextContent::Text(v::Text::from(txt)),
            )])
            .into_owned(),
        )
    }

    /// Creates some new strikeout text
    pub fn new_strikeout_text(txt: &str) -> DecoratedText {
        Self(
            v::DecoratedText::Strikeout(vec![v::Located::from(
                v::DecoratedTextContent::Text(v::Text::from(txt)),
            )])
            .into_owned(),
        )
    }

    /// Creates some new superscript text
    pub fn new_superscript_text(txt: &str) -> DecoratedText {
        Self(
            v::DecoratedText::Superscript(vec![v::Located::from(
                v::DecoratedTextContent::Text(v::Text::from(txt)),
            )])
            .into_owned(),
        )
    }

    /// Creates some new subscript text
    pub fn new_subscript_text(txt: &str) -> DecoratedText {
        Self(
            v::DecoratedText::Subscript(vec![v::Located::from(
                v::DecoratedTextContent::Text(v::Text::from(txt)),
            )])
            .into_owned(),
        )
    }

    /// Returns true if bold
    pub fn is_bold(&self) -> bool {
        matches!(self.0, v::DecoratedText::Bold(_))
    }

    /// Returns true if italic
    pub fn is_italic(&self) -> bool {
        matches!(self.0, v::DecoratedText::Italic(_))
    }

    /// Returns true if strikeout
    pub fn is_strikeout(&self) -> bool {
        matches!(self.0, v::DecoratedText::Strikeout(_))
    }

    /// Returns true if superscript
    pub fn is_superscript(&self) -> bool {
        matches!(self.0, v::DecoratedText::Superscript(_))
    }

    /// Returns true if subscript
    pub fn is_subscript(&self) -> bool {
        matches!(self.0, v::DecoratedText::Subscript(_))
    }

    /// Returns the contents of the decorated text
    #[wasm_bindgen(getter)]
    pub fn contents(&self) -> DecoratedTextContents {
        DecoratedTextContents(
            self.0
                .as_contents()
                .iter()
                .map(|x| x.as_ref().map(|x| x.to_borrowed().into_owned()))
                .collect(),
        )
    }

    /// Converts text to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a collection of decorated text contents
#[wasm_bindgen]
pub struct DecoratedTextContents(
    Vec<v::Located<v::DecoratedTextContent<'static>>>,
);

#[wasm_bindgen]
impl DecoratedTextContents {
    /// Returns the content at the specified index
    pub fn get(&self, idx: usize) -> Option<DecoratedTextContent> {
        self.0
            .get(idx)
            .map(|x| DecoratedTextContent(x.to_borrowed().into_owned()))
    }

    /// Indicates whether or not there is content
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Represents total number of contents contained within
    #[wasm_bindgen(getter)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Represents a singular piece of content within decorated text
#[wasm_bindgen]
pub struct DecoratedTextContent(v::DecoratedTextContent<'static>);

#[wasm_bindgen]
impl DecoratedTextContent {
    /// Converts text to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a wrapper around a vimwiki keyword
#[wasm_bindgen]
pub enum Keyword {
    Todo = "TODO",
    Done = "DONE",
    Started = "STARTED",
    Fixme = "FIXME",
    Fixed = "FIXED",
    Xxx = "XXX",
}

impl Keyword {
    /// Converts to vimwiki instance
    pub fn to_vimwiki(&self) -> Option<v::Keyword> {
        match self {
            Keyword::Todo => Some(v::Keyword::Todo),
            Keyword::Done => Some(v::Keyword::Done),
            Keyword::Started => Some(v::Keyword::Started),
            Keyword::Fixme => Some(v::Keyword::Fixme),
            Keyword::Fixed => Some(v::Keyword::Fixed),
            Keyword::Xxx => Some(v::Keyword::Xxx),
            _ => None,
        }
    }
}

impl From<v::Keyword> for Keyword {
    fn from(x: v::Keyword) -> Self {
        match x {
            v::Keyword::Todo => Self::Todo,
            v::Keyword::Done => Self::Done,
            v::Keyword::Started => Self::Started,
            v::Keyword::Fixme => Self::Fixme,
            v::Keyword::Fixed => Self::Fixed,
            v::Keyword::Xxx => Self::Xxx,
        }
    }
}

/// Represents a wrapper around a vimwiki link
#[wasm_bindgen]
pub struct Link(v::Link<'static>);

#[wasm_bindgen]
impl Link {
    /// Creates a new wiki link
    pub fn new_wiki_link(
        uri: &str,
        description: &str,
    ) -> Result<Link, JsValue> {
        Ok(Self(
            v::Link::new_wiki_link(
                uriparse::URIReference::try_from(uri)
                    .map_err(|x| JsValue::from_str(x.to_string().as_str()))?,
                v::Description::try_from_uri_ref_str(description)
                    .unwrap_or_else(|_| v::Description::from(description)),
            )
            .into_owned(),
        ))
    }

    /// Creates a new indexed interwiki link
    pub fn new_indexed_interwiki_link(
        index: u32,
        uri: &str,
        description: &str,
    ) -> Result<Link, JsValue> {
        Ok(Self(
            v::Link::new_indexed_interwiki_link(
                index,
                uriparse::URIReference::try_from(uri)
                    .map_err(|x| JsValue::from_str(x.to_string().as_str()))?,
                v::Description::try_from_uri_ref_str(description)
                    .unwrap_or_else(|_| v::Description::from(description)),
            )
            .into_owned(),
        ))
    }

    /// Creates a new named interwiki link
    pub fn new_named_interwiki_link(
        name: &str,
        uri: &str,
        description: &str,
    ) -> Result<Link, JsValue> {
        Ok(Self(
            v::Link::new_named_interwiki_link(
                name,
                uriparse::URIReference::try_from(uri)
                    .map_err(|x| JsValue::from_str(x.to_string().as_str()))?,
                v::Description::try_from_uri_ref_str(description)
                    .unwrap_or_else(|_| v::Description::from(description)),
            )
            .into_owned(),
        ))
    }

    /// Creates a new diary link
    pub fn new_diary_link(
        date: js_sys::Date,
        description: &str,
        anchor: &str,
    ) -> Link {
        Self(
            v::Link::new_diary_link(
                chrono::NaiveDate::from_ymd(
                    date.get_utc_full_year() as i32,
                    date.get_utc_month(),
                    date.get_utc_date(),
                ),
                v::Description::try_from_uri_ref_str(description)
                    .unwrap_or_else(|_| v::Description::from(description)),
                v::Anchor::from_uri_fragment(anchor),
            )
            .into_owned(),
        )
    }

    /// Creates a new raw link
    pub fn new_raw_link(uri: &str) -> Result<Link, JsValue> {
        Ok(Self(
            v::Link::new_raw_link(
                uriparse::URIReference::try_from(uri)
                    .map_err(|x| JsValue::from_str(x.to_string().as_str()))?,
            )
            .into_owned(),
        ))
    }

    /// Creates a new transclusion link
    pub fn new_transclusion_link(
        uri: &str,
        description: &str,
        properties: &js_sys::Object,
    ) -> Result<Link, JsValue> {
        let uri = uriparse::URIReference::try_from(uri)
            .map_err(|x| JsValue::from_str(x.to_string().as_str()))?;
        let desc = v::Description::try_from_uri_ref_str(description)
            .unwrap_or_else(|_| v::Description::from(description));
        let props: HashMap<Cow<'_, str>, Cow<'_, str>> =
            js_sys::Object::entries(properties)
                .iter()
                .filter_map(|entry| {
                    use wasm_bindgen::JsCast;
                    entry.dyn_ref::<js_sys::Array>().and_then(|entry| {
                        let key = js_sys::Array::get(entry, 0);
                        let value = js_sys::Array::get(entry, 1);

                        key.as_string().and_then(|key| {
                            value.as_string().map(|value| {
                                (Cow::Owned(key), Cow::Owned(value))
                            })
                        })
                    })
                })
                .collect();

        Ok(Self(
            v::Link::new_transclusion_link(uri, desc, props).into_owned(),
        ))
    }

    /// Returns uri associated with link
    #[wasm_bindgen(getter)]
    pub fn uri(&self) -> String {
        self.0.data().uri_ref().to_string()
    }

    /// Returns description associated with link (if it exists)
    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.0.description().map(ToString::to_string)
    }

    /// Returns object containing properties of link
    #[wasm_bindgen(getter)]
    pub fn properties(&self) -> Option<js_sys::Object> {
        self.0.properties().and_then(|props| {
            let arr = js_sys::Array::new();

            for (key, value) in props.iter() {
                let tuple = js_sys::Array::new();
                tuple.push(&JsValue::from_str(key.as_ref()));
                tuple.push(&JsValue::from_str(value.as_ref()));

                arr.push(&tuple);
            }

            js_sys::Object::from_entries(&arr).ok()
        })
    }

    /// Retrieves the property with the specified name if it exists
    pub fn get_property(&self, name: &str) -> Option<String> {
        self.0
            .properties()
            .and_then(|p| p.get(&Cow::from(name)).map(ToString::to_string))
    }

    /// Returns scheme of link's uri (if it exists)
    #[wasm_bindgen(getter)]
    pub fn scheme(&self) -> Option<String> {
        self.0.scheme().map(ToString::to_string)
    }

    /// Returns date (YYYY-MM-DD) associated with link (if it exists)
    #[wasm_bindgen(getter)]
    pub fn date(&self) -> Option<String> {
        self.0.date().map(|x| x.format("%Y-%m-%d").to_string())
    }

    /// Returns index of wiki pointed to by link if it is different
    #[wasm_bindgen(getter)]
    pub fn wiki_index(&self) -> Option<u32> {
        self.0.index()
    }

    /// Returns name of wiki pointed to by link if it is different
    #[wasm_bindgen(getter)]
    pub fn wiki_name(&self) -> Option<String> {
        self.0.name().map(ToString::to_string)
    }
}

/// Represents a wrapper around a set of vimwiki tags
#[wasm_bindgen]
pub struct Tags(v::Tags<'static>);

#[wasm_bindgen]
impl Tags {
    /// Creates a new tag set instance using the given list of strings
    #[allow(clippy::boxed_local)]
    #[wasm_bindgen(constructor)]
    pub fn new(array: Box<[JsValue]>) -> Result<Tags, JsValue> {
        let array: Vec<String> =
            array.iter().filter_map(|x| x.as_string()).collect();

        Ok(Self(v::Tags::from(array)))
    }

    /// Retrieves the tag at the specified index within the tag set
    pub fn tag_at(&self, idx: usize) -> Option<Tag> {
        self.0.get(idx).map(|x| Tag(x.as_borrowed().into_owned()))
    }

    /// Represents total tags contained within the set
    #[wasm_bindgen(getter)]
    pub fn tag_cnt(&self) -> usize {
        self.0.len()
    }

    /// Converts tags to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a wrapper around a singular vimwiki tag
#[wasm_bindgen]
pub struct Tag(v::Tag<'static>);

#[wasm_bindgen]
impl Tag {
    /// Creates a new tag instance using the given string
    pub fn new(txt: &str) -> Self {
        Self(v::Tag::from(txt).into_owned())
    }

    /// Converts tag to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a wrapper around a vimwiki code inline
#[wasm_bindgen]
pub struct CodeInline(v::CodeInline<'static>);

#[wasm_bindgen]
impl CodeInline {
    /// Creates a new inline code instance using the given string
    #[wasm_bindgen(constructor)]
    pub fn new(txt: &str) -> Self {
        Self(v::CodeInline::new(Cow::from(txt)).into_owned())
    }

    /// Converts inline code to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a wrapper around a vimwiki math inline
#[wasm_bindgen]
pub struct MathInline(v::MathInline<'static>);

#[wasm_bindgen]
impl MathInline {
    /// Creates a new inline math instance using the given string
    #[wasm_bindgen(constructor)]
    pub fn new(txt: &str) -> Self {
        Self(v::MathInline::new(Cow::from(txt)).into_owned())
    }

    /// Converts inline math to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a wrapper around a vimwiki comment
#[wasm_bindgen]
pub struct Comment(v::Comment<'static>);

#[wasm_bindgen]
impl Comment {
    /// Creates a new comment instance, marked as either a line comment
    /// or multiline comment based on the flag
    #[wasm_bindgen(constructor)]
    pub fn new(txt: &str, multiline: bool) -> Self {
        if multiline {
            Self(
                v::Comment::MultiLine(v::MultiLineComment::new(
                    txt.split('\n').map(Cow::from).collect(),
                ))
                .into_owned(),
            )
        } else {
            Self(
                v::Comment::Line(v::LineComment::new(Cow::from(txt)))
                    .into_owned(),
            )
        }
    }

    /// Retrieves the line at the specified index
    pub fn line_at(&self, idx: usize) -> Option<String> {
        match self.0 {
            v::Comment::Line(ref x) if idx == 0 => Some(x.0.to_string()),
            v::Comment::MultiLine(ref x) => {
                x.0.get(idx).map(ToString::to_string)
            }
            _ => None,
        }
    }

    /// Returns total lines contained within the comment
    #[wasm_bindgen(getter)]
    pub fn line_cnt(&self) -> usize {
        match self.0 {
            v::Comment::Line(_) => 1,
            v::Comment::MultiLine(ref x) => x.0.len(),
        }
    }

    /// Converts comment to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a wrapper around a vimwiki text
#[wasm_bindgen]
pub struct Text(v::Text<'static>);

#[wasm_bindgen]
impl Text {
    /// Creates a new text instance using the given string
    #[wasm_bindgen(constructor)]
    pub fn new(txt: &str) -> Self {
        Self(v::Text::new(Cow::from(txt)).into_owned())
    }

    /// Converts text to a JavaScript string
    pub fn to_str(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a wrapper around a vimwiki region
#[wasm_bindgen]
pub struct Region(v::Region);

#[wasm_bindgen]
impl Region {
    /// Creates a new region instance
    #[wasm_bindgen(constructor)]
    pub fn new(offset: usize, len: usize, depth: u16) -> Self {
        Self(v::Region::new_at_depth(offset, len, depth))
    }

    /// Represents the offset (starting from 0) of this region in the text
    #[wasm_bindgen(getter)]
    pub fn offset(&self) -> usize {
        self.0.offset()
    }

    /// Returns true if the region is actually empty (len == 0)
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Represents the length of this region in the text
    #[wasm_bindgen(getter)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Represents the depth of this region in the text
    #[wasm_bindgen(getter)]
    pub fn depth(&self) -> u16 {
        self.0.depth()
    }
}

/// Provide From impl; use @ to not use lifetimes
macro_rules! impl_from {
    (@$name:ident $($tail:tt)*) => {
        impl From<v::$name> for $name {
            fn from(x: v::$name) -> Self {
                Self(x)
            }
        }

        impl_from!($($tail)*);
    };
    ($name:ident $($tail:tt)*) => {
        impl From<v::$name<'static>> for $name {
            fn from(x: v::$name<'static>) -> Self {
                Self(x)
            }
        }

        impl_from!($($tail)*);
    };
    () => {};
}

impl_from!(
    Page Element BlockElement InlineBlockElement InlineElement Blockquote
    CodeBlock DefinitionList @Divider Header List MathBlock Paragraph Table
    DecoratedText Link Tags CodeInline MathInline Comment Text
    InlineElementContainer DecoratedTextContent ListItem ListItemContent
    Placeholder @Region
);

/// Provide conversion functions; use @ to not include html
macro_rules! impl_convert {
    (@$name:ident $($tail:tt)*) => {
        #[wasm_bindgen]
        impl $name {
            /// Convert to a JavaScript value
            pub fn to_js(&self) -> JsValue {
                JsValue::from_serde(&self.0).unwrap()
            }

            /// Convert to a debug string
            pub fn to_debug_str(&self) -> String {
                format!("{:?}", self.0)
            }
        }

        impl_convert!($($tail)*);
    };
    ($name:ident $($tail:tt)*) => {
        #[wasm_bindgen]
        impl $name {
            /// Convert to an HTML string
            pub fn to_html_str(&self) -> Result<String, JsValue> {
                self.0
                    .to_html_string(Default::default())
                    .map_err(|x| x.to_string().into())
            }
        }

        impl_convert!(@$name $($tail)*);
    };
    () => {};
}

impl_convert!(
    Page Element BlockElement InlineBlockElement InlineElement Blockquote
    CodeBlock DefinitionList Divider Header List MathBlock Paragraph Table
    DecoratedText Link Tags CodeInline MathInline Comment Text
    InlineElementContainer DecoratedTextContent ListItem ListItemContent
    Placeholder @Region
);

macro_rules! impl_iter {
    ($name:ident $($tail:tt)*) => {
        #[wasm_bindgen]
        impl $name {
            /// Convert to an array containing all immediate children elements
            #[wasm_bindgen(getter)]
            pub fn children(&self) -> js_sys::Array {
                use vimwiki::IntoChildren;
                self.0
                    .to_borrowed()
                    .into_children()
                    .into_iter()
                    .map(|x| x.to_borrowed().into_owned())
                    .map(v::Element::from)
                    .map(Element::from)
                    .map(JsValue::from)
                    .collect()
            }

            /// Convert to an array of all elements within current object
            pub fn to_all_elements(&self) -> js_sys::Array {
                use vimwiki::IntoChildren;

                // Used to keep track of all elements
                let mut elements = Vec::new();

                // Queue of elements whose children to acquire
                let mut queue: Vec<v::Element<'_>> = self.0
                    .to_borrowed()
                    .into_children()
                    .into_iter()
                    .map(|x| x.to_borrowed().into_owned())
                    .map(v::Element::from)
                    .collect();

                while !queue.is_empty() {
                    let next = queue.remove(0);
                    let children: Vec<v::Element<'_>> = next
                        .clone()
                        .into_children()
                        .into_iter()
                        .map(v::Located::into_inner)
                        .collect();
                    elements.push(next);
                    queue.extend(children);
                }

                // Collect elements into final array
                elements
                    .into_iter()
                    .map(Element::from)
                    .map(JsValue::from)
                    .collect()
            }
        }

        impl_iter!($($tail)*);
    };
    () => {};
}

impl_iter!(
    Page Element BlockElement InlineBlockElement InlineElement
    DefinitionList Header List Paragraph Table
    DecoratedText InlineElementContainer DecoratedTextContent ListItem
);
