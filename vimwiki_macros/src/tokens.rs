//! The [`Tokenize`] trait, turning [glsl](https://crates.io/crates/glsl) into [`TokenStream`]s.
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::{HashMap, HashSet};
use std::iter::once;
use std::path::PathBuf;
use vimwiki::{
    components::*,
    vendor::{chrono::NaiveDate, uriparse::URI},
    LocatedComponent, Position, Region,
};

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

impl<T: Tokenize> Tokenize for LocatedComponent<T> {
    fn tokenize(&self, stream: &mut TokenStream) {
        let mut component = TokenStream::new();
        self.component.tokenize(&mut component);

        let region = tokenize_region(&self.region);

        let self_stream = quote! {
            vimwiki::LocatedComponent {
                component: #component,
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
impl_tokenize!(BlockComponent, tokenize_block_component);
impl_tokenize!(
    InlineComponentContainer,
    tokenize_inline_component_container
);
impl_tokenize!(InlineComponent, tokenize_inline_component);

// Blockquotes
impl_tokenize!(Blockquote, tokenize_blockquote);

// Comments
impl_tokenize!(Comment, tokenize_comment);
impl_tokenize!(MultiLineComment, tokenize_multi_line_comment);

// Definitions (NOTE: Generic LocatedComponent def above handles term & def)
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
impl_tokenize!(EnhancedListItem, tokenize_enhanced_list_item);
impl_tokenize!(
    EnhancedListItemAttribute,
    tokenize_enhanced_list_item_attribute
);
impl_tokenize!(ListItem, tokenize_list_item);
impl_tokenize!(ListItemContent, tokenize_list_item_content);
impl_tokenize!(ListItemContents, tokenize_list_item_contents);
impl_tokenize!(ListItemSuffix, tokenize_list_item_suffix);
impl_tokenize!(ListItemType, tokenize_list_item_type);
impl_tokenize!(OrderedListItemType, tokenize_ordered_list_item_type);
impl_tokenize!(UnorderedListItemType, tokenize_unordered_list_item_type);

// Math
impl_tokenize!(Math, tokenize_math);
impl_tokenize!(MathInline, tokenize_math_inline);
impl_tokenize!(MathBlock, tokenize_math_block);

// Paragraphs
impl_tokenize!(Paragraph, tokenize_paragraph);

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
impl_tokenize!(Decoration, tokenize_decoration);
impl_tokenize!(DecoratedTextContent, tokenize_decorated_text_content);
impl_tokenize!(DecoratedText, tokenize_decorated_text);
impl_tokenize!(Keyword, tokenize_keyword);

fn tokenize_page(page: &Page) -> TokenStream {
    let components = page
        .components
        .iter()
        .map(|c| tokenize_located_component(c, tokenize_block_component));
    quote! {
        vimwiki::components::Page {
            components: vec![#(#components),*],
        }
    }
}

fn tokenize_block_component(block_component: &BlockComponent) -> TokenStream {
    match block_component {
        BlockComponent::BlankLine => {
            quote! { vimwiki::components::BlockComponent::BlankLine}
        }
        BlockComponent::Blockquote(x) => {
            let t = tokenize_blockquote(&x);
            quote! { vimwiki::components::BlockComponent::Blockquote(#t) }
        }
        BlockComponent::DefinitionList(x) => {
            let t = tokenize_definition_list(&x);
            quote! { vimwiki::components::BlockComponent::DefinitionList(#t) }
        }
        BlockComponent::Divider(x) => {
            let t = tokenize_divider(&x);
            quote! { vimwiki::components::BlockComponent::Divider(#t) }
        }
        BlockComponent::Header(x) => {
            let t = tokenize_header(&x);
            quote! { vimwiki::components::BlockComponent::Header(#t) }
        }
        BlockComponent::List(x) => {
            let t = tokenize_list(&x);
            quote! { vimwiki::components::BlockComponent::List(#t) }
        }
        BlockComponent::Math(x) => {
            let t = tokenize_math_block(&x);
            quote! { vimwiki::components::BlockComponent::Math(#t) }
        }
        BlockComponent::NonBlankLine(x) => {
            let t = quote! { #x.quote() };
            quote! { vimwiki::components::BlockComponent::NonBlankLine(#t) }
        }
        BlockComponent::Paragraph(x) => {
            let t = tokenize_paragraph(&x);
            quote! { vimwiki::components::BlockComponent::Paragraph(#t) }
        }
        BlockComponent::PreformattedText(x) => {
            let t = tokenize_preformatted_text(&x);
            quote! { vimwiki::components::BlockComponent::PreformattedText(#t) }
        }
        BlockComponent::Table(x) => {
            let t = tokenize_table(&x);
            quote! { vimwiki::components::BlockComponent::Table(#t) }
        }
        BlockComponent::Tags(x) => {
            let t = tokenize_tags(&x);
            quote! { vimwiki::components::BlockComponent::Tags(#t) }
        }
    }
}

fn tokenize_inline_component_container(
    inline_component_container: &InlineComponentContainer,
) -> TokenStream {
    let components = inline_component_container
        .components
        .iter()
        .map(|c| tokenize_located_component(c, tokenize_inline_component));
    quote! {
        vimwiki::components::InlineComponentContainer {
            components: vec![#(#components),*],
        }
    }
}

fn tokenize_inline_component(
    inline_component: &InlineComponent,
) -> TokenStream {
    match inline_component {
        InlineComponent::Text(x) => {
            let t = tokenize_string(&x);
            quote! { vimwiki::components::InlineComponent::Text(#t) }
        }
        InlineComponent::DecoratedText(x) => {
            let t = tokenize_decorated_text(&x);
            quote! { vimwiki::components::InlineComponent::DecoratedText(#t) }
        }
        InlineComponent::Keyword(x) => {
            let t = tokenize_keyword(&x);
            quote! { vimwiki::components::InlineComponent::Keyword(#t) }
        }
        InlineComponent::Link(x) => {
            let t = tokenize_link(&x);
            quote! { vimwiki::components::InlineComponent::Link(#t) }
        }
        InlineComponent::Tags(x) => {
            let t = tokenize_tags(&x);
            quote! { vimwiki::components::InlineComponent::Tags(#t) }
        }
        InlineComponent::Math(x) => {
            let t = tokenize_math_inline(&x);
            quote! { vimwiki::components::InlineComponent::Math(#t) }
        }
    }
}

// Blockquotes
fn tokenize_blockquote(blockquote: &Blockquote) -> TokenStream {
    let lines = blockquote.lines.iter().map(tokenize_string);
    quote! {
        vimwiki::components::Blockquote {
            lines: vec![#(#lines),*],
        }
    }
}

// Comments
fn tokenize_comment(comment: &Comment) -> TokenStream {
    match comment {
        Comment::Line(x) => {
            let t = tokenize_line_comment(&x);
            quote! { vimwiki::components::Comment::Line(#t) }
        }
        Comment::MultiLine(x) => {
            let t = tokenize_multi_line_comment(&x);
            quote! { vimwiki::components::Comment::MultiLine(#t) }
        }
    }
}

fn tokenize_line_comment(line_comment: &LineComment) -> TokenStream {
    let t = tokenize_string(&line_comment.0);
    quote! {
        vimwiki::components::LineComment(#t)
    }
}

fn tokenize_multi_line_comment(
    multi_line_comment: &MultiLineComment,
) -> TokenStream {
    let t = multi_line_comment.0.iter().map(tokenize_string);
    quote! {
        vimwiki::components::MultiLineComment(vec![#(#t),*])
    }
}

// Definitions
fn tokenize_definition_list(definition_list: &DefinitionList) -> TokenStream {
    let td = definition_list.iter().map(tokenize_term_and_definitions);
    quote! {
        vimwiki::components::DefinitionList::from(vec![#(#td),*])
    }
}

fn tokenize_term_and_definitions(
    term_and_definitions: &TermAndDefinitions,
) -> TokenStream {
    let term = tokenize_term(&term_and_definitions.term);
    let definitions = term_and_definitions
        .definitions
        .iter()
        .map(tokenize_definition);
    quote! {
        vimwiki::components::TermAndDefinitions {
            term: #term,
            definitions: vec![#(#definitions),*],
        }
    }
}

fn tokenize_definition(definition: &Definition) -> TokenStream {
    tokenize_located_component(definition, tokenize_string)
}

fn tokenize_term(term: &Term) -> TokenStream {
    tokenize_located_component(term, tokenize_string)
}

// Dividers
fn tokenize_divider(_divider: &Divider) -> TokenStream {
    quote! {
        vimwiki::components::Divider
    }
}

// Headers
fn tokenize_header(header: &Header) -> TokenStream {
    let Header {
        level,
        text,
        centered,
    } = header;
    let t = tokenize_string(&text);
    quote! {
        vimwiki::components::Header {
            level: #level,
            text: #t,
            centered: #centered,
        }
    }
}

// Links
fn tokenize_link(link: &Link) -> TokenStream {
    match &link {
        Link::Diary(x) => {
            let t = tokenize_diary_link(&x);
            quote! { vimwiki::components::Link::Diary(#t) }
        }
        Link::ExternalFile(x) => {
            let t = tokenize_external_file_link(&x);
            quote! { vimwiki::components::Link::ExternalFile(#t) }
        }
        Link::InterWiki(x) => {
            let t = tokenize_inter_wiki_link(&x);
            quote! { vimwiki::components::Link::InterWiki(#t) }
        }
        Link::Raw(x) => {
            let t = tokenize_raw_link(&x);
            quote! { vimwiki::components::Link::Raw(#t) }
        }
        Link::Transclusion(x) => {
            let t = tokenize_transclusion_link(&x);
            quote! { vimwiki::components::Link::Transclusion(#t) }
        }
        Link::Wiki(x) => {
            let t = tokenize_wiki_link(&x);
            quote! { vimwiki::components::Link::Wiki(#t) }
        }
    }
}

fn tokenize_diary_link(diary_link: &DiaryLink) -> TokenStream {
    let date = tokenize_naive_date(&diary_link.date);
    let description =
        tokenize_option(&diary_link.description, tokenize_description);
    let anchor = tokenize_option(&diary_link.anchor, tokenize_anchor);
    quote! {
        vimwiki::components::DiaryLink {
            date: #date,
            description: #description,
            anchor: #anchor,
        }
    }
}

fn tokenize_external_file_link(
    external_file_link: &ExternalFileLink,
) -> TokenStream {
    let scheme = tokenize_external_file_link_scheme(&external_file_link.scheme);
    let path = tokenize_path_buf(&external_file_link.path);
    let description =
        tokenize_option(&external_file_link.description, tokenize_description);
    quote! {
        vimwiki::components::ExternalFileLink {
            scheme: #scheme,
            path: #path,
            description: #description,
        }
    }
}

fn tokenize_external_file_link_scheme(
    external_file_link_scheme: &ExternalFileLinkScheme,
) -> TokenStream {
    match &external_file_link_scheme {
        ExternalFileLinkScheme::Absolute => {
            quote! { vimwiki::components::ExternalFileLinkScheme::Absolute }
        }
        ExternalFileLinkScheme::File => {
            quote! { vimwiki::components::ExternalFileLinkScheme::File }
        }
        ExternalFileLinkScheme::Local => {
            quote! { vimwiki::components::ExternalFileLinkScheme::Local }
        }
    }
}

fn tokenize_raw_link(raw_link: &RawLink) -> TokenStream {
    let uri = tokenize_uri(&raw_link.uri);
    quote! {
        vimwiki::components::RawLink {
            uri: #uri,
        }
    }
}

fn tokenize_transclusion_link(
    transclusion_link: &TransclusionLink,
) -> TokenStream {
    let uri = tokenize_uri(&transclusion_link.uri);
    let description =
        tokenize_option(&transclusion_link.description, tokenize_description);
    let properties = tokenize_hashmap(
        &transclusion_link.properties,
        tokenize_string,
        tokenize_string,
    );
    quote! {
        vimwiki::components::TransclusionLink {
            uri: #uri,
            description: #description,
            properties: #properties,
        }
    }
}

fn tokenize_wiki_link(wiki_link: &WikiLink) -> TokenStream {
    let path = tokenize_path_buf(&wiki_link.path);
    let description =
        tokenize_option(&wiki_link.description, tokenize_description);
    let anchor = tokenize_option(&wiki_link.anchor, tokenize_anchor);
    quote! {
        vimwiki::components::WikiLink {
            path: #path,
            description: #description,
            anchor: #anchor,
        }
    }
}

fn tokenize_inter_wiki_link(inter_wiki_link: &InterWikiLink) -> TokenStream {
    match &inter_wiki_link {
        InterWikiLink::Indexed(x) => {
            let t = tokenize_indexed_inter_wiki_link(&x);
            quote! { vimwiki::components::InterWikiLink::Indexed(#t) }
        }
        InterWikiLink::Named(x) => {
            let t = tokenize_named_inter_wiki_link(&x);
            quote! { vimwiki::components::InterWikiLink::Named(#t) }
        }
    }
}

fn tokenize_indexed_inter_wiki_link(
    indexed_inter_wiki_link: &IndexedInterWikiLink,
) -> TokenStream {
    let index = indexed_inter_wiki_link.index;
    let link = tokenize_wiki_link(&indexed_inter_wiki_link.link);
    quote! {
        vimwiki::components::IndexedInterWikiLink {
            index: #index,
            link: #link,
        }
    }
}

fn tokenize_named_inter_wiki_link(
    named_inter_wiki_link: &NamedInterWikiLink,
) -> TokenStream {
    let name = tokenize_string(&named_inter_wiki_link.name);
    let link = tokenize_wiki_link(&named_inter_wiki_link.link);
    quote! {
        vimwiki::components::NamedInterWikiLink {
            name: #name,
            link: #link,
        }
    }
}

fn tokenize_description(description: &Description) -> TokenStream {
    match &description {
        Description::Text(x) => {
            let t = tokenize_string(&x);
            quote! { vimwiki::components::Description::Text(#t) }
        }
        Description::URI(x) => {
            let t = tokenize_uri(&x);
            quote! { vimwiki::components::Description::URI(#t) }
        }
    }
}

fn tokenize_anchor(anchor: &Anchor) -> TokenStream {
    let components = anchor.components.iter().map(tokenize_string);
    quote! {
        vimwiki::components::Anchor {
            components: vec![#(#components),*],
        }
    }
}

fn tokenize_uri(uri: &URI) -> TokenStream {
    let uri_string = tokenize_string(&uri.to_string());
    quote! {
        {
            use std::convert::TryFrom;
            vimwiki::uri::URI::try_from(#uri_string)
                .expect("Failed to parse URI").into_owned()
        }
    }
}

// Lists

fn tokenize_list(list: &List) -> TokenStream {
    let items = list
        .items
        .iter()
        .map(|x| tokenize_located_component(x, tokenize_enhanced_list_item));
    quote! {
        vimwiki::components::List {
            items: vec![#(#items),*],
        }
    }
}

fn tokenize_enhanced_list_item(
    enhanced_list_item: &EnhancedListItem,
) -> TokenStream {
    let item = tokenize_list_item(&enhanced_list_item.item);
    let attributes = tokenize_hashset(
        &enhanced_list_item.attributes,
        tokenize_enhanced_list_item_attribute,
    );
    quote! {
        vimwiki::components::EnhancedListItem {
            item: #item,
            attributes: #attributes,
        }
    }
}

fn tokenize_enhanced_list_item_attribute(
    enhanced_list_item_attribute: &EnhancedListItemAttribute,
) -> TokenStream {
    match &enhanced_list_item_attribute {
        EnhancedListItemAttribute::TodoIncomplete => {
            quote! { vimwiki::components::EnhancedListItemAttribute::TodoIncomplete }
        }
        EnhancedListItemAttribute::TodoPartiallyComplete1 => {
            quote! { vimwiki::components::EnhancedListItemAttribute::TodoPartiallyComplete1 }
        }
        EnhancedListItemAttribute::TodoPartiallyComplete2 => {
            quote! { vimwiki::components::EnhancedListItemAttribute::TodoPartiallyComplete2 }
        }
        EnhancedListItemAttribute::TodoPartiallyComplete3 => {
            quote! { vimwiki::components::EnhancedListItemAttribute::TodoPartiallyComplete3 }
        }
        EnhancedListItemAttribute::TodoComplete => {
            quote! { vimwiki::components::EnhancedListItemAttribute::TodoComplete }
        }
        EnhancedListItemAttribute::TodoRejected => {
            quote! { vimwiki::components::EnhancedListItemAttribute::TodoRejected }
        }
    }
}

fn tokenize_list_item(list_item: &ListItem) -> TokenStream {
    let ListItem {
        item_type,
        suffix,
        pos,
        contents,
    } = list_item;
    let item_type_t = tokenize_list_item_type(&item_type);
    let suffix_t = tokenize_list_item_suffix(&suffix);
    let contents_t = tokenize_list_item_contents(&contents);
    quote! {
        vimwiki::components::ListItem {
            item_type: #item_type_t,
            suffix: #suffix_t,
            pos: #pos,
            contents: #contents_t,
        }
    }
}

fn tokenize_list_item_content(
    list_item_content: &ListItemContent,
) -> TokenStream {
    match &list_item_content {
        ListItemContent::InlineContent(x) => {
            let t = tokenize_inline_component_container(&x);
            quote! { vimwiki::components::ListItemContent::InlineContent(#t) }
        }
        ListItemContent::List(x) => {
            let t = tokenize_list(&x);
            quote! { vimwiki::components::ListItemContent::List(#t) }
        }
    }
}

fn tokenize_list_item_contents(
    list_item_contents: &ListItemContents,
) -> TokenStream {
    let contents = list_item_contents
        .contents
        .iter()
        .map(|x| tokenize_located_component(x, tokenize_list_item_content));
    quote! {
        vimwiki::components::ListItemContents {
            contents: vec![#(#contents),*],
        }
    }
}

fn tokenize_list_item_suffix(list_item_suffix: &ListItemSuffix) -> TokenStream {
    match &list_item_suffix {
        ListItemSuffix::None => {
            quote! { vimwiki::components::ListItemSuffix::None }
        }
        ListItemSuffix::Period => {
            quote! { vimwiki::components::ListItemSuffix::Period }
        }
        ListItemSuffix::Paren => {
            quote! { vimwiki::components::ListItemSuffix::Paren }
        }
    }
}

fn tokenize_list_item_type(list_item_type: &ListItemType) -> TokenStream {
    match &list_item_type {
        ListItemType::Ordered(x) => {
            let t = tokenize_ordered_list_item_type(&x);
            quote! { vimwiki::components::ListItemType::Ordered(#t) }
        }
        ListItemType::Unordered(x) => {
            let t = tokenize_unordered_list_item_type(&x);
            quote! { vimwiki::components::ListItemType::Unordered(#t) }
        }
    }
}

fn tokenize_ordered_list_item_type(
    ordered_list_item_type: &OrderedListItemType,
) -> TokenStream {
    match &ordered_list_item_type {
        OrderedListItemType::Number => {
            quote! { vimwiki::components::OrderedListItemType::Number }
        }
        OrderedListItemType::Pound => {
            quote! { vimwiki::components::OrderedListItemType::Pound }
        }
        OrderedListItemType::LowercaseAlphabet => {
            quote! { vimwiki::components::OrderedListItemType::LowercaseAlphabet }
        }
        OrderedListItemType::UppercaseAlphabet => {
            quote! { vimwiki::components::OrderedListItemType::UppercaseAlphabet }
        }
        OrderedListItemType::LowercaseRoman => {
            quote! { vimwiki::components::OrderedListItemType::LowercaseRoman }
        }
        OrderedListItemType::UppercaseRoman => {
            quote! { vimwiki::components::OrderedListItemType::UppercaseRoman }
        }
    }
}

fn tokenize_unordered_list_item_type(
    unordered_list_item_type: &UnorderedListItemType,
) -> TokenStream {
    match &unordered_list_item_type {
        UnorderedListItemType::Hyphen => {
            quote! { vimwiki::components::UnorderedListItemType::Hyphen }
        }
        UnorderedListItemType::Asterisk => {
            quote! { vimwiki::components::UnorderedListItemType::Asterisk }
        }
        UnorderedListItemType::Other(x) => {
            let t = tokenize_string(&x);
            quote! { vimwiki::components::UnorderedListItemType::Other(#t) }
        }
    }
}

// Math

fn tokenize_math(math: &Math) -> TokenStream {
    match &math {
        Math::Block(x) => {
            let t = tokenize_math_block(&x);
            quote! { vimwiki::components::Math::Block(#t) }
        }
        Math::Inline(x) => {
            let t = tokenize_math_inline(&x);
            quote! { vimwiki::components::Math::Inline(#t) }
        }
    }
}

fn tokenize_math_inline(math_inline: &MathInline) -> TokenStream {
    let formula = tokenize_string(&math_inline.formula);
    quote! {
        vimwiki::components::MathInline {
            formula: #formula,
        }
    }
}

fn tokenize_math_block(math_block: &MathBlock) -> TokenStream {
    let lines = math_block.lines.iter().map(tokenize_string);
    let environment = tokenize_option(&math_block.environment, tokenize_string);
    quote! {
        vimwiki::components::MathBlock {
            lines: vec![#(#lines),*],
            environment: #environment,
        }
    }
}

// Paragraphs

fn tokenize_paragraph(paragraph: &Paragraph) -> TokenStream {
    let content = tokenize_inline_component_container(&paragraph.content);
    quote! {
        vimwiki::components::Paragraph {
            content: #content,
        }
    }
}

// Preformatted Text

fn tokenize_preformatted_text(
    preformatted_text: &PreformattedText,
) -> TokenStream {
    let metadata = tokenize_hashmap(
        &preformatted_text.metadata,
        tokenize_string,
        tokenize_string,
    );
    let lines = preformatted_text.lines.iter().map(tokenize_string);
    quote! {
        vimwiki::components::PreformattedText {
            metadata: #metadata,
            lines: vec![#(#lines),*],
        }
    }
}

// Tables

fn tokenize_table(table: &Table) -> TokenStream {
    let rows = table
        .rows
        .iter()
        .map(|x| tokenize_located_component(x, tokenize_row));
    let centered = table.centered;
    quote! {
        vimwiki::components::Table {
            rows: vec![#(#rows),*],
            centered: #centered,
        }
    }
}

fn tokenize_row(row: &Row) -> TokenStream {
    match &row {
        Row::Content { cells } => {
            let t = cells
                .iter()
                .map(|x| tokenize_located_component(x, tokenize_cell));
            quote! { vimwiki::components::Row::Content { cells: vec![#(#t),*] } }
        }
        Row::Divider => {
            quote! { vimwiki::components::Row::Divider }
        }
    }
}

fn tokenize_cell(cell: &Cell) -> TokenStream {
    match &cell {
        Cell::Content(x) => {
            let t = tokenize_inline_component_container(&x);
            quote! { vimwiki::components::Cell::Content(#t) }
        }
        Cell::SpanAbove => {
            quote! { vimwiki::components::Cell::SpanAbove }
        }
        Cell::SpanLeft => {
            quote! { vimwiki::components::Cell::SpanLeft }
        }
    }
}

// Tags

fn tokenize_tags(tags: &Tags) -> TokenStream {
    let inner = tags.0.iter().map(tokenize_tag);
    quote! {
        vimwiki::components::Tags(vec![#(#inner),*])
    }
}

fn tokenize_tag(tag: &Tag) -> TokenStream {
    let inner = tokenize_string(&tag.0);
    quote! {
        vimwiki::components::Tag(#inner)
    }
}

// Typefaces

fn tokenize_decoration(decoration: &Decoration) -> TokenStream {
    match &decoration {
        Decoration::Bold => quote! { vimwiki::components::Decoration::Bold },
        Decoration::BoldItalic => {
            quote! { vimwiki::components::Decoration::BoldItalic }
        }
        Decoration::Code => quote! { vimwiki::components::Decoration::Code },
        Decoration::Italic => {
            quote! { vimwiki::components::Decoration::Italic }
        }
        Decoration::Strikeout => {
            quote! { vimwiki::components::Decoration::Strikeout }
        }
        Decoration::Subscript => {
            quote! { vimwiki::components::Decoration::Subscript }
        }
        Decoration::Superscript => {
            quote! { vimwiki::components::Decoration::Superscript }
        }
    }
}

fn tokenize_decorated_text_content(
    decorated_text_content: &DecoratedTextContent,
) -> TokenStream {
    match &decorated_text_content {
        DecoratedTextContent::DecoratedText(x) => {
            let t = tokenize_decorated_text(&x);
            quote! { vimwiki::components::DecoratedTextContent::DecoratedText(#t) }
        }
        DecoratedTextContent::Keyword(x) => {
            let t = tokenize_keyword(&x);
            quote! { vimwiki::components::DecoratedTextContent::Keyword(#t) }
        }
        DecoratedTextContent::Link(x) => {
            let t = tokenize_link(&x);
            quote! { vimwiki::components::DecoratedTextContent::Link(#t) }
        }
        DecoratedTextContent::Text(x) => {
            let t = tokenize_string(&x);
            quote! { vimwiki::components::DecoratedTextContent::Text(#t) }
        }
    }
}

fn tokenize_decorated_text(decorated_text: &DecoratedText) -> TokenStream {
    let contents = decorated_text.contents.iter().map(|x| {
        tokenize_located_component(x, tokenize_decorated_text_content)
    });
    let decoration = tokenize_decoration(&decorated_text.decoration);
    quote! {
        vimwiki::components::DecoratedText {
            contents: vec![#(#contents),*],
            decoration: #decoration,
        }
    }
}

fn tokenize_keyword(keyword: &Keyword) -> TokenStream {
    match keyword {
        Keyword::DONE => {
            quote! { vimwiki::components::Keyword::DONE }
        }
        Keyword::FIXED => {
            quote! { vimwiki::components::Keyword::FIXED }
        }
        Keyword::FIXME => {
            quote! { vimwiki::components::Keyword::FIXME }
        }
        Keyword::STARTED => {
            quote! { vimwiki::components::Keyword::STARTED }
        }
        Keyword::TODO => {
            quote! { vimwiki::components::Keyword::TODO }
        }
        Keyword::XXX => {
            quote! { vimwiki::components::Keyword::XXX }
        }
    }
}

fn tokenize_located_component<T: Tokenize>(
    lc: &LocatedComponent<T>,
    f: impl Fn(&T) -> TokenStream,
) -> TokenStream {
    let component = f(&lc.component);
    let region = tokenize_region(&lc.region);
    quote! {
        vimwiki::LocatedComponent {
            component: #component,
            region: #region,
        }
    }
}

fn tokenize_region(region: &Region) -> TokenStream {
    let start = tokenize_position(&region.start);
    let end = tokenize_position(&region.end);
    quote! {
        vimwiki::Region {
            start: #start,
            end: #end,
        }
    }
}

fn tokenize_position(position: &Position) -> TokenStream {
    let line = position.line;
    let column = position.column;
    quote! {
        vimwiki::Position {
            line: #line,
            column: #column,
        }
    }
}

fn tokenize_hashmap<K: Tokenize, V: Tokenize>(
    m: &HashMap<K, V>,
    fk: impl Fn(&K) -> TokenStream,
    fv: impl Fn(&V) -> TokenStream,
) -> TokenStream {
    let pairs = m.iter().map(|(k, v)| {
        let tk = fk(k);
        let tv = fv(v);
        quote! { (#tk, #tv) }
    });
    quote! { std::collections::HashMap::from(vec![#(#pairs),*]) }
}

fn tokenize_hashset<T: Tokenize>(
    s: &HashSet<T>,
    f: impl Fn(&T) -> TokenStream,
) -> TokenStream {
    let items = s.iter().map(f);
    quote! { std::collections::HashSet::from(vec![#(#items),*]) }
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
    let year = naive_date.year();
    let month = naive_date.month();
    let day = naive_date.day();
    quote! { vimwiki::vendor::chrono::NaiveDate::from_ymd(#year, #month, #day) }
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
    tokenize_str(s)
}

fn tokenize_str(s: &str) -> TokenStream {
    quote! { #s.to_owned() }
}
