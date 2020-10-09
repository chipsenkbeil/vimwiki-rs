use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::{collections::HashMap, iter::once, path::PathBuf};
use vimwiki::{
    elements::*,
    vendor::{chrono::NaiveDate, uriparse::URI},
    LocatedElement, Position, Region,
};

#[inline]
fn root_crate() -> TokenStream {
    // TODO: Support detecting if we're within the vimwiki crate
    //       (for unit tests)
    quote! { ::vimwiki }
}

/// Tokenize a value into a stream of tokens.
pub trait Tokenize {
    /// Inject self into a [`TokenStream`].
    fn tokenize(&self, stream: &mut TokenStream);
}

impl Tokenize for bool {
    fn tokenize(&self, stream: &mut TokenStream) {
        self.to_tokens(stream)
    }
}

impl Tokenize for i32 {
    fn tokenize(&self, stream: &mut TokenStream) {
        self.to_tokens(stream)
    }
}

impl Tokenize for u32 {
    fn tokenize(&self, stream: &mut TokenStream) {
        self.to_tokens(stream)
    }
}

impl Tokenize for f32 {
    fn tokenize(&self, stream: &mut TokenStream) {
        self.to_tokens(stream)
    }
}

impl Tokenize for f64 {
    fn tokenize(&self, stream: &mut TokenStream) {
        self.to_tokens(stream)
    }
}

impl<T: Tokenize> Tokenize for LocatedElement<T> {
    fn tokenize(&self, stream: &mut TokenStream) {
        let root = root_crate();
        let mut element = TokenStream::new();
        self.element.tokenize(&mut element);

        let region = tokenize_region(&self.region);

        let self_stream = quote! {
            #root::LocatedElement {
                element: #element,
                region: #region,
            }
        };

        stream.extend(once(self_stream))
    }
}

macro_rules! impl_tokenize {
    ($type_name:ty, $tokenizer:ident) => {
        impl Tokenize for $type_name {
            fn tokenize(&self, stream: &mut TokenStream) {
                stream.extend(once($tokenizer(self)))
            }
        }
    };
}

// Primititives
impl_tokenize!(Region, tokenize_region);
impl_tokenize!(Position, tokenize_position);
impl_tokenize!(String, tokenize_string);
impl_tokenize!(NaiveDate, tokenize_naive_date);
impl_tokenize!(URI<'static>, tokenize_uri);
impl_tokenize!(PathBuf, tokenize_path_buf);

// Top-level types
impl_tokenize!(Page, tokenize_page);
impl_tokenize!(BlockElement, tokenize_block_element);
impl_tokenize!(InlineElementContainer, tokenize_inline_element_container);
impl_tokenize!(InlineElement, tokenize_inline_element);

// Blockquotes
impl_tokenize!(Blockquote, tokenize_blockquote);

// Comments
impl_tokenize!(Comment, tokenize_comment);
impl_tokenize!(LineComment, tokenize_line_comment);
impl_tokenize!(MultiLineComment, tokenize_multi_line_comment);

// Definitions (NOTE: Generic LocatedElement def above handles term & def)
impl_tokenize!(DefinitionList, tokenize_definition_list);
// impl_tokenize!(Definition, tokenize_definition);
// impl_tokenize!(Term, tokenize_term);

// Dividers
impl_tokenize!(Divider, tokenize_divider);

// Headers
impl_tokenize!(Header, tokenize_header);

// Links
impl_tokenize!(Link, tokenize_link);
impl_tokenize!(DiaryLink, tokenize_diary_link);
impl_tokenize!(ExternalFileLink, tokenize_external_file_link);
impl_tokenize!(ExternalFileLinkScheme, tokenize_external_file_link_scheme);
impl_tokenize!(RawLink, tokenize_raw_link);
impl_tokenize!(TransclusionLink, tokenize_transclusion_link);
impl_tokenize!(WikiLink, tokenize_wiki_link);
impl_tokenize!(IndexedInterWikiLink, tokenize_indexed_inter_wiki_link);
impl_tokenize!(InterWikiLink, tokenize_inter_wiki_link);
impl_tokenize!(NamedInterWikiLink, tokenize_named_inter_wiki_link);
impl_tokenize!(Description, tokenize_description);
impl_tokenize!(Anchor, tokenize_anchor);

// Lists
impl_tokenize!(List, tokenize_list);
impl_tokenize!(ListItem, tokenize_list_item);
impl_tokenize!(ListItemContent, tokenize_list_item_content);
impl_tokenize!(ListItemContents, tokenize_list_item_contents);
impl_tokenize!(ListItemSuffix, tokenize_list_item_suffix);
impl_tokenize!(ListItemType, tokenize_list_item_type);
impl_tokenize!(OrderedListItemType, tokenize_ordered_list_item_type);
impl_tokenize!(UnorderedListItemType, tokenize_unordered_list_item_type);
impl_tokenize!(ListItemAttributes, tokenize_list_item_attributes);
impl_tokenize!(ListItemTodoStatus, tokenize_list_item_todo_status);

// Code
impl_tokenize!(CodeInline, tokenize_code_inline);

// Math
impl_tokenize!(MathInline, tokenize_math_inline);
impl_tokenize!(MathBlock, tokenize_math_block);

// Paragraphs
impl_tokenize!(Paragraph, tokenize_paragraph);

// Placeholders
impl_tokenize!(Placeholder, tokenize_placeholder);

// Preformatted Text
impl_tokenize!(PreformattedText, tokenize_preformatted_text);

// Tables
impl_tokenize!(Table, tokenize_table);
impl_tokenize!(Row, tokenize_row);
impl_tokenize!(Cell, tokenize_cell);

// Tags
impl_tokenize!(Tags, tokenize_tags);
impl_tokenize!(Tag, tokenize_tag);

// Typefaces
impl_tokenize!(Text, tokenize_text);
impl_tokenize!(DecoratedTextContent, tokenize_decorated_text_content);
impl_tokenize!(DecoratedText, tokenize_decorated_text);
impl_tokenize!(Keyword, tokenize_keyword);

fn tokenize_page(page: &Page) -> TokenStream {
    let elements = page
        .elements
        .iter()
        .map(|c| tokenize_located_element(c, tokenize_block_element));
    let comments = page
        .comments
        .iter()
        .map(|c| tokenize_located_element(c, tokenize_comment));
    let root = root_crate();
    quote! {
        #root::elements::Page {
            elements: vec![#(#elements),*],
            comments: vec![#(#comments),*],
        }
    }
}

fn tokenize_block_element(block_element: &BlockElement) -> TokenStream {
    let root = root_crate();
    match block_element {
        BlockElement::BlankLine => {
            quote! { #root::elements::BlockElement::BlankLine}
        }
        BlockElement::Blockquote(x) => {
            let t = tokenize_blockquote(&x);
            quote! { #root::elements::BlockElement::Blockquote(#t) }
        }
        BlockElement::DefinitionList(x) => {
            let t = tokenize_definition_list(&x);
            quote! { #root::elements::BlockElement::DefinitionList(#t) }
        }
        BlockElement::Divider(x) => {
            let t = tokenize_divider(&x);
            quote! { #root::elements::BlockElement::Divider(#t) }
        }
        BlockElement::Header(x) => {
            let t = tokenize_header(&x);
            quote! { #root::elements::BlockElement::Header(#t) }
        }
        BlockElement::List(x) => {
            let t = tokenize_list(&x);
            quote! { #root::elements::BlockElement::List(#t) }
        }
        BlockElement::Math(x) => {
            let t = tokenize_math_block(&x);
            quote! { #root::elements::BlockElement::Math(#t) }
        }
        BlockElement::NonBlankLine(x) => {
            let t = quote! { #x.quote() };
            quote! { #root::elements::BlockElement::NonBlankLine(#t) }
        }
        BlockElement::Paragraph(x) => {
            let t = tokenize_paragraph(&x);
            quote! { #root::elements::BlockElement::Paragraph(#t) }
        }
        BlockElement::Placeholder(x) => {
            let t = tokenize_placeholder(&x);
            quote! { #root::elements::BlockElement::Placeholder(#t) }
        }
        BlockElement::PreformattedText(x) => {
            let t = tokenize_preformatted_text(&x);
            quote! { #root::elements::BlockElement::PreformattedText(#t) }
        }
        BlockElement::Table(x) => {
            let t = tokenize_table(&x);
            quote! { #root::elements::BlockElement::Table(#t) }
        }
    }
}

fn tokenize_inline_element_container(
    inline_element_container: &InlineElementContainer,
) -> TokenStream {
    let root = root_crate();
    let elements = inline_element_container
        .elements
        .iter()
        .map(|c| tokenize_located_element(c, tokenize_inline_element));
    quote! {
        #root::elements::InlineElementContainer {
            elements: vec![#(#elements),*],
        }
    }
}

fn tokenize_inline_element(inline_element: &InlineElement) -> TokenStream {
    let root = root_crate();
    match inline_element {
        InlineElement::Text(x) => {
            let t = tokenize_text(&x);
            quote! { #root::elements::InlineElement::Text(#t) }
        }
        InlineElement::DecoratedText(x) => {
            let t = tokenize_decorated_text(&x);
            quote! { #root::elements::InlineElement::DecoratedText(#t) }
        }
        InlineElement::Keyword(x) => {
            let t = tokenize_keyword(&x);
            quote! { #root::elements::InlineElement::Keyword(#t) }
        }
        InlineElement::Link(x) => {
            let t = tokenize_link(&x);
            quote! { #root::elements::InlineElement::Link(#t) }
        }
        InlineElement::Tags(x) => {
            let t = tokenize_tags(&x);
            quote! { #root::elements::InlineElement::Tags(#t) }
        }
        InlineElement::Code(x) => {
            let t = tokenize_code_inline(&x);
            quote! { #root::elements::InlineElement::Code(#t) }
        }
        InlineElement::Math(x) => {
            let t = tokenize_math_inline(&x);
            quote! { #root::elements::InlineElement::Math(#t) }
        }
    }
}

// Blockquotes
fn tokenize_blockquote(blockquote: &Blockquote) -> TokenStream {
    let root = root_crate();
    let lines = blockquote.lines.iter().map(tokenize_string);
    quote! {
        #root::elements::Blockquote {
            lines: vec![#(#lines),*],
        }
    }
}

// Comments
fn tokenize_comment(comment: &Comment) -> TokenStream {
    let root = root_crate();
    match comment {
        Comment::Line(x) => {
            let t = tokenize_line_comment(&x);
            quote! { #root::elements::Comment::Line(#t) }
        }
        Comment::MultiLine(x) => {
            let t = tokenize_multi_line_comment(&x);
            quote! { #root::elements::Comment::MultiLine(#t) }
        }
    }
}

fn tokenize_line_comment(line_comment: &LineComment) -> TokenStream {
    let root = root_crate();
    let t = tokenize_string(&line_comment.0);
    quote! {
        #root::elements::LineComment(#t)
    }
}

fn tokenize_multi_line_comment(
    multi_line_comment: &MultiLineComment,
) -> TokenStream {
    let root = root_crate();
    let t = multi_line_comment.0.iter().map(tokenize_string);
    quote! {
        #root::elements::MultiLineComment(vec![#(#t),*])
    }
}

// Definitions
fn tokenize_definition_list(definition_list: &DefinitionList) -> TokenStream {
    let root = root_crate();
    let td = definition_list.iter().map(tokenize_term_and_definitions);
    quote! {
        #root::elements::DefinitionList::from(vec![#(#td),*])
    }
}

fn tokenize_term_and_definitions(
    term_and_definitions: &TermAndDefinitions,
) -> TokenStream {
    let root = root_crate();
    let term = tokenize_term(&term_and_definitions.term);
    let definitions = term_and_definitions
        .definitions
        .iter()
        .map(tokenize_definition);
    quote! {
        #root::elements::TermAndDefinitions {
            term: #term,
            definitions: vec![#(#definitions),*],
        }
    }
}

fn tokenize_definition(definition: &Definition) -> TokenStream {
    tokenize_inline_element_container(&definition)
}

fn tokenize_term(term: &Term) -> TokenStream {
    tokenize_inline_element_container(&term)
}

// Dividers
fn tokenize_divider(_divider: &Divider) -> TokenStream {
    let root = root_crate();
    quote! {
        #root::elements::Divider
    }
}

// Headers
fn tokenize_header(header: &Header) -> TokenStream {
    let root = root_crate();
    let Header {
        level,
        content,
        centered,
    } = header;
    let content_t = tokenize_inline_element_container(&content);
    quote! {
        #root::elements::Header {
            level: #level,
            content: #content_t,
            centered: #centered,
        }
    }
}

// Links
fn tokenize_link(link: &Link) -> TokenStream {
    let root = root_crate();
    match &link {
        Link::Diary(x) => {
            let t = tokenize_diary_link(&x);
            quote! { #root::elements::Link::Diary(#t) }
        }
        Link::ExternalFile(x) => {
            let t = tokenize_external_file_link(&x);
            quote! { #root::elements::Link::ExternalFile(#t) }
        }
        Link::InterWiki(x) => {
            let t = tokenize_inter_wiki_link(&x);
            quote! { #root::elements::Link::InterWiki(#t) }
        }
        Link::Raw(x) => {
            let t = tokenize_raw_link(&x);
            quote! { #root::elements::Link::Raw(#t) }
        }
        Link::Transclusion(x) => {
            let t = tokenize_transclusion_link(&x);
            quote! { #root::elements::Link::Transclusion(#t) }
        }
        Link::Wiki(x) => {
            let t = tokenize_wiki_link(&x);
            quote! { #root::elements::Link::Wiki(#t) }
        }
    }
}

fn tokenize_diary_link(diary_link: &DiaryLink) -> TokenStream {
    let root = root_crate();
    let date = tokenize_naive_date(&diary_link.date);
    let description =
        tokenize_option(&diary_link.description, tokenize_description);
    let anchor = tokenize_option(&diary_link.anchor, tokenize_anchor);
    quote! {
        #root::elements::DiaryLink {
            date: #date,
            description: #description,
            anchor: #anchor,
        }
    }
}

fn tokenize_external_file_link(
    external_file_link: &ExternalFileLink,
) -> TokenStream {
    let root = root_crate();
    let scheme = tokenize_external_file_link_scheme(&external_file_link.scheme);
    let path = tokenize_path_buf(&external_file_link.path);
    let description =
        tokenize_option(&external_file_link.description, tokenize_description);
    quote! {
        #root::elements::ExternalFileLink {
            scheme: #scheme,
            path: #path,
            description: #description,
        }
    }
}

fn tokenize_external_file_link_scheme(
    external_file_link_scheme: &ExternalFileLinkScheme,
) -> TokenStream {
    let root = root_crate();
    match &external_file_link_scheme {
        ExternalFileLinkScheme::Absolute => {
            quote! { #root::elements::ExternalFileLinkScheme::Absolute }
        }
        ExternalFileLinkScheme::File => {
            quote! { #root::elements::ExternalFileLinkScheme::File }
        }
        ExternalFileLinkScheme::Local => {
            quote! { #root::elements::ExternalFileLinkScheme::Local }
        }
    }
}

fn tokenize_raw_link(raw_link: &RawLink) -> TokenStream {
    let root = root_crate();
    let uri = tokenize_uri(&raw_link.uri);
    quote! {
        #root::elements::RawLink {
            uri: #uri,
        }
    }
}

fn tokenize_transclusion_link(
    transclusion_link: &TransclusionLink,
) -> TokenStream {
    let root = root_crate();
    let uri = tokenize_uri(&transclusion_link.uri);
    let description =
        tokenize_option(&transclusion_link.description, tokenize_description);
    let properties = tokenize_hashmap(
        &transclusion_link.properties,
        tokenize_string_type(),
        tokenize_string_type(),
        tokenize_string,
        tokenize_string,
    );
    quote! {
        #root::elements::TransclusionLink {
            uri: #uri,
            description: #description,
            properties: #properties,
        }
    }
}

fn tokenize_wiki_link(wiki_link: &WikiLink) -> TokenStream {
    let root = root_crate();
    let path = tokenize_path_buf(&wiki_link.path);
    let description =
        tokenize_option(&wiki_link.description, tokenize_description);
    let anchor = tokenize_option(&wiki_link.anchor, tokenize_anchor);
    quote! {
        #root::elements::WikiLink {
            path: #path,
            description: #description,
            anchor: #anchor,
        }
    }
}

fn tokenize_inter_wiki_link(inter_wiki_link: &InterWikiLink) -> TokenStream {
    let root = root_crate();
    match &inter_wiki_link {
        InterWikiLink::Indexed(x) => {
            let t = tokenize_indexed_inter_wiki_link(&x);
            quote! { #root::elements::InterWikiLink::Indexed(#t) }
        }
        InterWikiLink::Named(x) => {
            let t = tokenize_named_inter_wiki_link(&x);
            quote! { #root::elements::InterWikiLink::Named(#t) }
        }
    }
}

fn tokenize_indexed_inter_wiki_link(
    indexed_inter_wiki_link: &IndexedInterWikiLink,
) -> TokenStream {
    let root = root_crate();
    let index = indexed_inter_wiki_link.index;
    let link = tokenize_wiki_link(&indexed_inter_wiki_link.link);
    quote! {
        #root::elements::IndexedInterWikiLink {
            index: #index,
            link: #link,
        }
    }
}

fn tokenize_named_inter_wiki_link(
    named_inter_wiki_link: &NamedInterWikiLink,
) -> TokenStream {
    let root = root_crate();
    let name = tokenize_string(&named_inter_wiki_link.name);
    let link = tokenize_wiki_link(&named_inter_wiki_link.link);
    quote! {
        #root::elements::NamedInterWikiLink {
            name: #name,
            link: #link,
        }
    }
}

fn tokenize_description(description: &Description) -> TokenStream {
    let root = root_crate();
    match &description {
        Description::Text(x) => {
            let t = tokenize_string(&x);
            quote! { #root::elements::Description::Text(#t) }
        }
        Description::URI(x) => {
            let t = tokenize_uri(&x);
            quote! { #root::elements::Description::URI(#t) }
        }
    }
}

fn tokenize_anchor(anchor: &Anchor) -> TokenStream {
    let root = root_crate();
    let elements = anchor.elements.iter().map(tokenize_string);
    quote! {
        #root::elements::Anchor {
            elements: vec![#(#elements),*],
        }
    }
}

fn tokenize_uri(uri: &URI) -> TokenStream {
    let root = root_crate();
    let uri_string = tokenize_string(&uri.to_string());
    quote! {
        {
            use std::convert::TryFrom;
            #root::vendor::uriparse::URI::try_from(#uri_string.as_str())
                .expect("Failed to parse URI").into_owned()
        }
    }
}

// Lists

fn tokenize_list(list: &List) -> TokenStream {
    let root = root_crate();
    let items = list
        .items
        .iter()
        .map(|x| tokenize_located_element(x, tokenize_list_item));
    quote! {
        #root::elements::List {
            items: vec![#(#items),*],
        }
    }
}

fn tokenize_list_item(list_item: &ListItem) -> TokenStream {
    let root = root_crate();
    let ListItem {
        item_type,
        suffix,
        pos,
        contents,
        attributes,
    } = list_item;
    let item_type_t = tokenize_list_item_type(&item_type);
    let suffix_t = tokenize_list_item_suffix(&suffix);
    let contents_t = tokenize_list_item_contents(&contents);
    let attributes_t = tokenize_list_item_attributes(&attributes);
    quote! {
        #root::elements::ListItem {
            item_type: #item_type_t,
            suffix: #suffix_t,
            pos: #pos,
            contents: #contents_t,
            attributes: #attributes_t,
        }
    }
}

fn tokenize_list_item_content(
    list_item_content: &ListItemContent,
) -> TokenStream {
    let root = root_crate();
    match &list_item_content {
        ListItemContent::InlineContent(x) => {
            let t = tokenize_inline_element_container(&x);
            quote! { #root::elements::ListItemContent::InlineContent(#t) }
        }
        ListItemContent::List(x) => {
            let t = tokenize_typed_block_element_of_list(&x);
            quote! { #root::elements::ListItemContent::List(#t) }
        }
    }
}

fn tokenize_list_item_contents(
    list_item_contents: &ListItemContents,
) -> TokenStream {
    let root = root_crate();
    let contents = list_item_contents
        .contents
        .iter()
        .map(|x| tokenize_located_element(x, tokenize_list_item_content));
    quote! {
        #root::elements::ListItemContents {
            contents: vec![#(#contents),*],
        }
    }
}

fn tokenize_list_item_suffix(list_item_suffix: &ListItemSuffix) -> TokenStream {
    let root = root_crate();
    match &list_item_suffix {
        ListItemSuffix::None => {
            quote! { #root::elements::ListItemSuffix::None }
        }
        ListItemSuffix::Period => {
            quote! { #root::elements::ListItemSuffix::Period }
        }
        ListItemSuffix::Paren => {
            quote! { #root::elements::ListItemSuffix::Paren }
        }
    }
}

fn tokenize_list_item_type(list_item_type: &ListItemType) -> TokenStream {
    let root = root_crate();
    match &list_item_type {
        ListItemType::Ordered(x) => {
            let t = tokenize_ordered_list_item_type(&x);
            quote! { #root::elements::ListItemType::Ordered(#t) }
        }
        ListItemType::Unordered(x) => {
            let t = tokenize_unordered_list_item_type(&x);
            quote! { #root::elements::ListItemType::Unordered(#t) }
        }
    }
}

fn tokenize_ordered_list_item_type(
    ordered_list_item_type: &OrderedListItemType,
) -> TokenStream {
    let root = root_crate();
    match &ordered_list_item_type {
        OrderedListItemType::Number => {
            quote! { #root::elements::OrderedListItemType::Number }
        }
        OrderedListItemType::Pound => {
            quote! { #root::elements::OrderedListItemType::Pound }
        }
        OrderedListItemType::LowercaseAlphabet => {
            quote! { #root::elements::OrderedListItemType::LowercaseAlphabet }
        }
        OrderedListItemType::UppercaseAlphabet => {
            quote! { #root::elements::OrderedListItemType::UppercaseAlphabet }
        }
        OrderedListItemType::LowercaseRoman => {
            quote! { #root::elements::OrderedListItemType::LowercaseRoman }
        }
        OrderedListItemType::UppercaseRoman => {
            quote! { #root::elements::OrderedListItemType::UppercaseRoman }
        }
    }
}

fn tokenize_unordered_list_item_type(
    unordered_list_item_type: &UnorderedListItemType,
) -> TokenStream {
    let root = root_crate();
    match &unordered_list_item_type {
        UnorderedListItemType::Hyphen => {
            quote! { #root::elements::UnorderedListItemType::Hyphen }
        }
        UnorderedListItemType::Asterisk => {
            quote! { #root::elements::UnorderedListItemType::Asterisk }
        }
        UnorderedListItemType::Other(x) => {
            let t = tokenize_string(&x);
            quote! { #root::elements::UnorderedListItemType::Other(#t) }
        }
    }
}

fn tokenize_list_item_attributes(
    list_item_attributes: &ListItemAttributes,
) -> TokenStream {
    let root = root_crate();
    let todo_status = tokenize_option(
        &list_item_attributes.todo_status,
        tokenize_list_item_todo_status,
    );
    quote! {
        #root::elements::ListItemAttributes {
            todo_status: #todo_status
        }
    }
}

fn tokenize_list_item_todo_status(
    list_item_todo_status: &ListItemTodoStatus,
) -> TokenStream {
    let root = root_crate();
    match list_item_todo_status {
        ListItemTodoStatus::Incomplete => {
            quote! { #root::elements::ListItemTodoStatus::Incomplete }
        }
        ListItemTodoStatus::PartiallyComplete1 => {
            quote! { #root::elements::ListItemTodoStatus::PartiallyComplete1 }
        }
        ListItemTodoStatus::PartiallyComplete2 => {
            quote! { #root::elements::ListItemTodoStatus::PartiallyComplete2 }
        }
        ListItemTodoStatus::PartiallyComplete3 => {
            quote! { #root::elements::ListItemTodoStatus::PartiallyComplete3 }
        }
        ListItemTodoStatus::Complete => {
            quote! { #root::elements::ListItemTodoStatus::Complete }
        }
        ListItemTodoStatus::Rejected => {
            quote! { #root::elements::ListItemTodoStatus::Rejected }
        }
    }
}

// Code

fn tokenize_code_inline(code_inline: &CodeInline) -> TokenStream {
    let root = root_crate();
    let code = tokenize_string(&code_inline.code);
    quote! {
        #root::elements::CodeInline {
            code: #code,
        }
    }
}

// Math

fn tokenize_math_inline(math_inline: &MathInline) -> TokenStream {
    let root = root_crate();
    let formula = tokenize_string(&math_inline.formula);
    quote! {
        #root::elements::MathInline {
            formula: #formula,
        }
    }
}

fn tokenize_math_block(math_block: &MathBlock) -> TokenStream {
    let root = root_crate();
    let lines = math_block.lines.iter().map(tokenize_string);
    let environment = tokenize_option(&math_block.environment, tokenize_string);
    quote! {
        #root::elements::MathBlock {
            lines: vec![#(#lines),*],
            environment: #environment,
        }
    }
}

// Paragraphs

fn tokenize_paragraph(paragraph: &Paragraph) -> TokenStream {
    let root = root_crate();
    let content = tokenize_inline_element_container(&paragraph.content);
    quote! {
        #root::elements::Paragraph {
            content: #content,
        }
    }
}

// Placeholders

fn tokenize_placeholder(placeholder: &Placeholder) -> TokenStream {
    let root = root_crate();
    match &placeholder {
        Placeholder::Date(x) => {
            let t = tokenize_naive_date(&x);
            quote! { #root::elements::Placeholder::Date(#t) }
        }
        Placeholder::NoHtml => {
            quote! { #root::elements::Placeholder::NoHtml }
        }
        Placeholder::Other { name, value } => {
            let name_t = tokenize_string(&name);
            let value_t = tokenize_string(&value);
            quote! {
                #root::elements::Placeholder::Other {
                    name: #name_t,
                    value: #value_t,
                }
            }
        }
        Placeholder::Template(x) => {
            let t = tokenize_string(&x);
            quote! { #root::elements::Placeholder::Template(#t) }
        }
        Placeholder::Title(x) => {
            let t = tokenize_string(&x);
            quote! { #root::elements::Placeholder::Title(#t) }
        }
    }
}

// Preformatted Text

fn tokenize_preformatted_text(
    preformatted_text: &PreformattedText,
) -> TokenStream {
    let root = root_crate();
    let lang = tokenize_option(&preformatted_text.lang, tokenize_string);
    let metadata = tokenize_hashmap(
        &preformatted_text.metadata,
        tokenize_string_type(),
        tokenize_string_type(),
        tokenize_string,
        tokenize_string,
    );
    let lines = preformatted_text.lines.iter().map(tokenize_string);
    quote! {
        #root::elements::PreformattedText {
            lang: #lang,
            metadata: #metadata,
            lines: vec![#(#lines),*],
        }
    }
}

// Tables

fn tokenize_table(table: &Table) -> TokenStream {
    let root = root_crate();
    let rows = table
        .rows
        .iter()
        .map(|x| tokenize_located_element(x, tokenize_row));
    let centered = table.centered;
    quote! {
        #root::elements::Table {
            rows: vec![#(#rows),*],
            centered: #centered,
        }
    }
}

fn tokenize_row(row: &Row) -> TokenStream {
    let root = root_crate();
    match &row {
        Row::Content { cells } => {
            let t = cells
                .iter()
                .map(|x| tokenize_located_element(x, tokenize_cell));
            quote! { #root::elements::Row::Content { cells: vec![#(#t),*] } }
        }
        Row::Divider => {
            quote! { #root::elements::Row::Divider }
        }
    }
}

fn tokenize_cell(cell: &Cell) -> TokenStream {
    let root = root_crate();
    match &cell {
        Cell::Content(x) => {
            let t = tokenize_inline_element_container(&x);
            quote! { #root::elements::Cell::Content(#t) }
        }
        Cell::SpanAbove => {
            quote! { #root::elements::Cell::SpanAbove }
        }
        Cell::SpanLeft => {
            quote! { #root::elements::Cell::SpanLeft }
        }
    }
}

// Tags

fn tokenize_tags(tags: &Tags) -> TokenStream {
    let root = root_crate();
    let inner = tags.0.iter().map(tokenize_tag);
    quote! {
        #root::elements::Tags(vec![#(#inner),*])
    }
}

fn tokenize_tag(tag: &Tag) -> TokenStream {
    let root = root_crate();
    let inner = tokenize_string(&tag.0);
    quote! {
        #root::elements::Tag(#inner)
    }
}

// Typefaces

fn tokenize_text(text: &Text) -> TokenStream {
    let root = root_crate();
    let inner = tokenize_string(text.as_ref());
    quote! {
        #root::elements::Text::new(#inner)
    }
}

fn tokenize_decorated_text_content(
    decorated_text_content: &DecoratedTextContent,
) -> TokenStream {
    let root = root_crate();
    match &decorated_text_content {
        DecoratedTextContent::Keyword(x) => {
            let t = tokenize_keyword(&x);
            quote! { #root::elements::DecoratedTextContent::Keyword(#t) }
        }
        DecoratedTextContent::Link(x) => {
            let t = tokenize_link(&x);
            quote! { #root::elements::DecoratedTextContent::Link(#t) }
        }
        DecoratedTextContent::Text(x) => {
            let t = tokenize_text(&x);
            quote! { #root::elements::DecoratedTextContent::Text(#t) }
        }
    }
}

fn tokenize_decorated_text(decorated_text: &DecoratedText) -> TokenStream {
    let root = root_crate();

    match decorated_text {
        DecoratedText::Bold(x) => {
            let contents = x.iter().map(|x| {
                tokenize_located_element(x, tokenize_decorated_text_content)
            });
            quote! {
                #root::elements::DecoratedText::Bold(
                    vec![#(#contents),*],
                )
            }
        }
        DecoratedText::BoldItalic(x) => {
            let contents = x.iter().map(|x| {
                tokenize_located_element(x, tokenize_decorated_text_content)
            });
            {
                quote! {
                    #root::elements::DecoratedText::BoldItalic(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Italic(x) => {
            let contents = x.iter().map(|x| {
                tokenize_located_element(x, tokenize_decorated_text_content)
            });
            {
                quote! {
                    #root::elements::DecoratedText::Italic(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Strikeout(x) => {
            let contents = x.iter().map(|x| {
                tokenize_located_element(x, tokenize_decorated_text_content)
            });
            {
                quote! {
                    #root::elements::DecoratedText::Strikeout(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Subscript(x) => {
            let contents = x.iter().map(|x| {
                tokenize_located_element(x, tokenize_decorated_text_content)
            });
            {
                quote! {
                    #root::elements::DecoratedText::Subscript(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Superscript(x) => {
            let contents = x.iter().map(|x| {
                tokenize_located_element(x, tokenize_decorated_text_content)
            });
            {
                quote! {
                    #root::elements::DecoratedText::Superscript(
                        vec![#(#contents),*],
                    )
                }
            }
        }
    }
}

fn tokenize_keyword(keyword: &Keyword) -> TokenStream {
    let root = root_crate();
    match keyword {
        Keyword::DONE => {
            quote! { #root::elements::Keyword::DONE }
        }
        Keyword::FIXED => {
            quote! { #root::elements::Keyword::FIXED }
        }
        Keyword::FIXME => {
            quote! { #root::elements::Keyword::FIXME }
        }
        Keyword::STARTED => {
            quote! { #root::elements::Keyword::STARTED }
        }
        Keyword::TODO => {
            quote! { #root::elements::Keyword::TODO }
        }
        Keyword::XXX => {
            quote! { #root::elements::Keyword::XXX }
        }
    }
}

fn tokenize_located_element<T: Tokenize>(
    le: &LocatedElement<T>,
    f: impl Fn(&T) -> TokenStream,
) -> TokenStream {
    let root = root_crate();
    let element = f(&le.element);
    let region = tokenize_region(&le.region);
    quote! {
        #root::LocatedElement {
            element: #element,
            region: #region,
        }
    }
}

fn tokenize_typed_block_element_of_list(
    typed_block_element: &TypedBlockElement<List>,
) -> TokenStream {
    let root = root_crate();
    let inner = tokenize_list(typed_block_element.as_list());
    quote! {
        #root::elements::TypedBlockElement::from_list(#inner)
    }
}

fn tokenize_region(region: &Region) -> TokenStream {
    let root = root_crate();
    let start = tokenize_position(&region.start);
    let end = tokenize_position(&region.end);
    quote! {
        #root::Region {
            start: #start,
            end: #end,
        }
    }
}

fn tokenize_position(position: &Position) -> TokenStream {
    let root = root_crate();
    let line = position.line;
    let column = position.column;
    quote! {
        #root::Position {
            line: #line,
            column: #column,
        }
    }
}

fn tokenize_hashmap<K: Tokenize, V: Tokenize>(
    m: &HashMap<K, V>,
    kty: TokenStream,
    vty: TokenStream,
    fk: impl Fn(&K) -> TokenStream,
    fv: impl Fn(&V) -> TokenStream,
) -> TokenStream {
    let pairs = m.iter().map(|(k, v)| {
        let tk = fk(k);
        let tv = fv(v);
        quote! { (#tk, #tv) }
    });
    quote! { vec![#(#pairs),*].drain(..).collect::<std::collections::HashMap<#kty,#vty>>() }
}

fn tokenize_option<T: Tokenize>(
    o: &Option<T>,
    f: impl Fn(&T) -> TokenStream,
) -> TokenStream {
    if let Some(ref x) = *o {
        let t = f(x);
        quote! { Some(#t) }
    } else {
        quote! { None }
    }
}

fn tokenize_naive_date(naive_date: &NaiveDate) -> TokenStream {
    use vimwiki::vendor::chrono::Datelike;
    let root = root_crate();
    let year = naive_date.year();
    let month = naive_date.month();
    let day = naive_date.day();
    quote! { #root::vendor::chrono::NaiveDate::from_ymd(#year, #month, #day) }
}

fn tokenize_path_buf(path_buf: &PathBuf) -> TokenStream {
    // TODO: Support cases where pathbuf cannot be converted back to Rust str
    let t = tokenize_str(
        path_buf
            .to_str()
            .expect("PathBuf cannot be converted to &str"),
    );
    quote! {
        std::path::PathBuf::from(#t)
    }
}

fn tokenize_string(s: &String) -> TokenStream {
    quote! { #s.to_owned() }
}

fn tokenize_str(s: &str) -> TokenStream {
    quote! { #s }
}

#[inline]
fn tokenize_string_type() -> TokenStream {
    let t = Ident::new("String", Span::call_site());
    quote! { #t }
}
