use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::{HashMap, HashSet};
use std::iter::once;
use std::path::PathBuf;
use vimwiki::{
    components::*,
    vendor::{chrono::NaiveDate, uriparse::URI},
    LocatedComponent, Position, Region,
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

impl<T: Tokenize> Tokenize for LocatedComponent<T> {
    fn tokenize(&self, stream: &mut TokenStream) {
        let root = root_crate();
        let mut component = TokenStream::new();
        self.component.tokenize(&mut component);

        let region = tokenize_region(&self.region);

        let self_stream = quote! {
            #root::LocatedComponent {
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
impl_tokenize!(LineComment, tokenize_line_comment);
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
impl_tokenize!(Decoration, tokenize_decoration);
impl_tokenize!(DecoratedTextContent, tokenize_decorated_text_content);
impl_tokenize!(DecoratedText, tokenize_decorated_text);
impl_tokenize!(Keyword, tokenize_keyword);

fn tokenize_page(page: &Page) -> TokenStream {
    let components = page
        .components
        .iter()
        .map(|c| tokenize_located_component(c, tokenize_block_component));
    let comments = page
        .comments
        .iter()
        .map(|c| tokenize_located_component(c, tokenize_comment));
    let root = root_crate();
    quote! {
        #root::components::Page {
            components: vec![#(#components),*],
            comments: vec![#(#comments),*],
        }
    }
}

fn tokenize_block_component(block_component: &BlockComponent) -> TokenStream {
    let root = root_crate();
    match block_component {
        BlockComponent::BlankLine => {
            quote! { #root::components::BlockComponent::BlankLine}
        }
        BlockComponent::Blockquote(x) => {
            let t = tokenize_blockquote(&x);
            quote! { #root::components::BlockComponent::Blockquote(#t) }
        }
        BlockComponent::DefinitionList(x) => {
            let t = tokenize_definition_list(&x);
            quote! { #root::components::BlockComponent::DefinitionList(#t) }
        }
        BlockComponent::Divider(x) => {
            let t = tokenize_divider(&x);
            quote! { #root::components::BlockComponent::Divider(#t) }
        }
        BlockComponent::Header(x) => {
            let t = tokenize_header(&x);
            quote! { #root::components::BlockComponent::Header(#t) }
        }
        BlockComponent::List(x) => {
            let t = tokenize_list(&x);
            quote! { #root::components::BlockComponent::List(#t) }
        }
        BlockComponent::Math(x) => {
            let t = tokenize_math_block(&x);
            quote! { #root::components::BlockComponent::Math(#t) }
        }
        BlockComponent::NonBlankLine(x) => {
            let t = quote! { #x.quote() };
            quote! { #root::components::BlockComponent::NonBlankLine(#t) }
        }
        BlockComponent::Paragraph(x) => {
            let t = tokenize_paragraph(&x);
            quote! { #root::components::BlockComponent::Paragraph(#t) }
        }
        BlockComponent::Placeholder(x) => {
            let t = tokenize_placeholder(&x);
            quote! { #root::components::BlockComponent::Placeholder(#t) }
        }
        BlockComponent::PreformattedText(x) => {
            let t = tokenize_preformatted_text(&x);
            quote! { #root::components::BlockComponent::PreformattedText(#t) }
        }
        BlockComponent::Table(x) => {
            let t = tokenize_table(&x);
            quote! { #root::components::BlockComponent::Table(#t) }
        }
        BlockComponent::Tags(x) => {
            let t = tokenize_tags(&x);
            quote! { #root::components::BlockComponent::Tags(#t) }
        }
    }
}

fn tokenize_inline_component_container(
    inline_component_container: &InlineComponentContainer,
) -> TokenStream {
    let root = root_crate();
    let components = inline_component_container
        .components
        .iter()
        .map(|c| tokenize_located_component(c, tokenize_inline_component));
    quote! {
        #root::components::InlineComponentContainer {
            components: vec![#(#components),*],
        }
    }
}

fn tokenize_inline_component(
    inline_component: &InlineComponent,
) -> TokenStream {
    let root = root_crate();
    match inline_component {
        InlineComponent::Text(x) => {
            let t = tokenize_string(&x);
            quote! { #root::components::InlineComponent::Text(#t) }
        }
        InlineComponent::DecoratedText(x) => {
            let t = tokenize_decorated_text(&x);
            quote! { #root::components::InlineComponent::DecoratedText(#t) }
        }
        InlineComponent::Keyword(x) => {
            let t = tokenize_keyword(&x);
            quote! { #root::components::InlineComponent::Keyword(#t) }
        }
        InlineComponent::Link(x) => {
            let t = tokenize_link(&x);
            quote! { #root::components::InlineComponent::Link(#t) }
        }
        InlineComponent::Tags(x) => {
            let t = tokenize_tags(&x);
            quote! { #root::components::InlineComponent::Tags(#t) }
        }
        InlineComponent::Math(x) => {
            let t = tokenize_math_inline(&x);
            quote! { #root::components::InlineComponent::Math(#t) }
        }
    }
}

// Blockquotes
fn tokenize_blockquote(blockquote: &Blockquote) -> TokenStream {
    let root = root_crate();
    let lines = blockquote.lines.iter().map(tokenize_string);
    quote! {
        #root::components::Blockquote {
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
            quote! { #root::components::Comment::Line(#t) }
        }
        Comment::MultiLine(x) => {
            let t = tokenize_multi_line_comment(&x);
            quote! { #root::components::Comment::MultiLine(#t) }
        }
    }
}

fn tokenize_line_comment(line_comment: &LineComment) -> TokenStream {
    let root = root_crate();
    let t = tokenize_string(&line_comment.0);
    quote! {
        #root::components::LineComment(#t)
    }
}

fn tokenize_multi_line_comment(
    multi_line_comment: &MultiLineComment,
) -> TokenStream {
    let root = root_crate();
    let t = multi_line_comment.0.iter().map(tokenize_string);
    quote! {
        #root::components::MultiLineComment(vec![#(#t),*])
    }
}

// Definitions
fn tokenize_definition_list(definition_list: &DefinitionList) -> TokenStream {
    let root = root_crate();
    let td = definition_list.iter().map(tokenize_term_and_definitions);
    quote! {
        #root::components::DefinitionList::from(vec![#(#td),*])
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
        #root::components::TermAndDefinitions {
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
    let root = root_crate();
    quote! {
        #root::components::Divider
    }
}

// Headers
fn tokenize_header(header: &Header) -> TokenStream {
    let root = root_crate();
    let Header {
        level,
        text,
        centered,
    } = header;
    let t = tokenize_string(&text);
    quote! {
        #root::components::Header {
            level: #level,
            text: #t,
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
            quote! { #root::components::Link::Diary(#t) }
        }
        Link::ExternalFile(x) => {
            let t = tokenize_external_file_link(&x);
            quote! { #root::components::Link::ExternalFile(#t) }
        }
        Link::InterWiki(x) => {
            let t = tokenize_inter_wiki_link(&x);
            quote! { #root::components::Link::InterWiki(#t) }
        }
        Link::Raw(x) => {
            let t = tokenize_raw_link(&x);
            quote! { #root::components::Link::Raw(#t) }
        }
        Link::Transclusion(x) => {
            let t = tokenize_transclusion_link(&x);
            quote! { #root::components::Link::Transclusion(#t) }
        }
        Link::Wiki(x) => {
            let t = tokenize_wiki_link(&x);
            quote! { #root::components::Link::Wiki(#t) }
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
        #root::components::DiaryLink {
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
        #root::components::ExternalFileLink {
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
            quote! { #root::components::ExternalFileLinkScheme::Absolute }
        }
        ExternalFileLinkScheme::File => {
            quote! { #root::components::ExternalFileLinkScheme::File }
        }
        ExternalFileLinkScheme::Local => {
            quote! { #root::components::ExternalFileLinkScheme::Local }
        }
    }
}

fn tokenize_raw_link(raw_link: &RawLink) -> TokenStream {
    let root = root_crate();
    let uri = tokenize_uri(&raw_link.uri);
    quote! {
        #root::components::RawLink {
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
        #root::components::TransclusionLink {
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
        #root::components::WikiLink {
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
            quote! { #root::components::InterWikiLink::Indexed(#t) }
        }
        InterWikiLink::Named(x) => {
            let t = tokenize_named_inter_wiki_link(&x);
            quote! { #root::components::InterWikiLink::Named(#t) }
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
        #root::components::IndexedInterWikiLink {
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
        #root::components::NamedInterWikiLink {
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
            quote! { #root::components::Description::Text(#t) }
        }
        Description::URI(x) => {
            let t = tokenize_uri(&x);
            quote! { #root::components::Description::URI(#t) }
        }
    }
}

fn tokenize_anchor(anchor: &Anchor) -> TokenStream {
    let root = root_crate();
    let components = anchor.components.iter().map(tokenize_string);
    quote! {
        #root::components::Anchor {
            components: vec![#(#components),*],
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
        .map(|x| tokenize_located_component(x, tokenize_enhanced_list_item));
    quote! {
        #root::components::List {
            items: vec![#(#items),*],
        }
    }
}

fn tokenize_enhanced_list_item(
    enhanced_list_item: &EnhancedListItem,
) -> TokenStream {
    let root = root_crate();
    let item = tokenize_list_item(&enhanced_list_item.item);
    let attributes = tokenize_hashset(
        &enhanced_list_item.attributes,
        quote! { #root::components::EnhancedListItemAttribute },
        tokenize_enhanced_list_item_attribute,
    );
    quote! {
        #root::components::EnhancedListItem {
            item: #item,
            attributes: #attributes,
        }
    }
}

fn tokenize_enhanced_list_item_attribute(
    enhanced_list_item_attribute: &EnhancedListItemAttribute,
) -> TokenStream {
    let root = root_crate();
    match &enhanced_list_item_attribute {
        EnhancedListItemAttribute::TodoIncomplete => {
            quote! { #root::components::EnhancedListItemAttribute::TodoIncomplete }
        }
        EnhancedListItemAttribute::TodoPartiallyComplete1 => {
            quote! { #root::components::EnhancedListItemAttribute::TodoPartiallyComplete1 }
        }
        EnhancedListItemAttribute::TodoPartiallyComplete2 => {
            quote! { #root::components::EnhancedListItemAttribute::TodoPartiallyComplete2 }
        }
        EnhancedListItemAttribute::TodoPartiallyComplete3 => {
            quote! { #root::components::EnhancedListItemAttribute::TodoPartiallyComplete3 }
        }
        EnhancedListItemAttribute::TodoComplete => {
            quote! { #root::components::EnhancedListItemAttribute::TodoComplete }
        }
        EnhancedListItemAttribute::TodoRejected => {
            quote! { #root::components::EnhancedListItemAttribute::TodoRejected }
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
    } = list_item;
    let item_type_t = tokenize_list_item_type(&item_type);
    let suffix_t = tokenize_list_item_suffix(&suffix);
    let contents_t = tokenize_list_item_contents(&contents);
    quote! {
        #root::components::ListItem {
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
    let root = root_crate();
    match &list_item_content {
        ListItemContent::InlineContent(x) => {
            let t = tokenize_inline_component_container(&x);
            quote! { #root::components::ListItemContent::InlineContent(#t) }
        }
        ListItemContent::List(x) => {
            let t = tokenize_list(&x);
            quote! { #root::components::ListItemContent::List(#t) }
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
        .map(|x| tokenize_located_component(x, tokenize_list_item_content));
    quote! {
        #root::components::ListItemContents {
            contents: vec![#(#contents),*],
        }
    }
}

fn tokenize_list_item_suffix(list_item_suffix: &ListItemSuffix) -> TokenStream {
    let root = root_crate();
    match &list_item_suffix {
        ListItemSuffix::None => {
            quote! { #root::components::ListItemSuffix::None }
        }
        ListItemSuffix::Period => {
            quote! { #root::components::ListItemSuffix::Period }
        }
        ListItemSuffix::Paren => {
            quote! { #root::components::ListItemSuffix::Paren }
        }
    }
}

fn tokenize_list_item_type(list_item_type: &ListItemType) -> TokenStream {
    let root = root_crate();
    match &list_item_type {
        ListItemType::Ordered(x) => {
            let t = tokenize_ordered_list_item_type(&x);
            quote! { #root::components::ListItemType::Ordered(#t) }
        }
        ListItemType::Unordered(x) => {
            let t = tokenize_unordered_list_item_type(&x);
            quote! { #root::components::ListItemType::Unordered(#t) }
        }
    }
}

fn tokenize_ordered_list_item_type(
    ordered_list_item_type: &OrderedListItemType,
) -> TokenStream {
    let root = root_crate();
    match &ordered_list_item_type {
        OrderedListItemType::Number => {
            quote! { #root::components::OrderedListItemType::Number }
        }
        OrderedListItemType::Pound => {
            quote! { #root::components::OrderedListItemType::Pound }
        }
        OrderedListItemType::LowercaseAlphabet => {
            quote! { #root::components::OrderedListItemType::LowercaseAlphabet }
        }
        OrderedListItemType::UppercaseAlphabet => {
            quote! { #root::components::OrderedListItemType::UppercaseAlphabet }
        }
        OrderedListItemType::LowercaseRoman => {
            quote! { #root::components::OrderedListItemType::LowercaseRoman }
        }
        OrderedListItemType::UppercaseRoman => {
            quote! { #root::components::OrderedListItemType::UppercaseRoman }
        }
    }
}

fn tokenize_unordered_list_item_type(
    unordered_list_item_type: &UnorderedListItemType,
) -> TokenStream {
    let root = root_crate();
    match &unordered_list_item_type {
        UnorderedListItemType::Hyphen => {
            quote! { #root::components::UnorderedListItemType::Hyphen }
        }
        UnorderedListItemType::Asterisk => {
            quote! { #root::components::UnorderedListItemType::Asterisk }
        }
        UnorderedListItemType::Other(x) => {
            let t = tokenize_string(&x);
            quote! { #root::components::UnorderedListItemType::Other(#t) }
        }
    }
}

// Math

fn tokenize_math(math: &Math) -> TokenStream {
    let root = root_crate();
    match &math {
        Math::Block(x) => {
            let t = tokenize_math_block(&x);
            quote! { #root::components::Math::Block(#t) }
        }
        Math::Inline(x) => {
            let t = tokenize_math_inline(&x);
            quote! { #root::components::Math::Inline(#t) }
        }
    }
}

fn tokenize_math_inline(math_inline: &MathInline) -> TokenStream {
    let root = root_crate();
    let formula = tokenize_string(&math_inline.formula);
    quote! {
        #root::components::MathInline {
            formula: #formula,
        }
    }
}

fn tokenize_math_block(math_block: &MathBlock) -> TokenStream {
    let root = root_crate();
    let lines = math_block.lines.iter().map(tokenize_string);
    let environment = tokenize_option(&math_block.environment, tokenize_string);
    quote! {
        #root::components::MathBlock {
            lines: vec![#(#lines),*],
            environment: #environment,
        }
    }
}

// Paragraphs

fn tokenize_paragraph(paragraph: &Paragraph) -> TokenStream {
    let root = root_crate();
    let content = tokenize_inline_component_container(&paragraph.content);
    quote! {
        #root::components::Paragraph {
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
            quote! { #root::components::Placeholder::Date(#t) }
        }
        Placeholder::NoHtml => {
            quote! { #root::components::Placeholder::NoHtml }
        }
        Placeholder::Other { name, value } => {
            let name_t = tokenize_string(&name);
            let value_t = tokenize_string(&value);
            quote! {
                #root::components::Placeholder::Other {
                    name: #name_t,
                    value: #value_t,
                }
            }
        }
        Placeholder::Template(x) => {
            let t = tokenize_string(&x);
            quote! { #root::components::Placeholder::Template(#t) }
        }
        Placeholder::Title(x) => {
            let t = tokenize_string(&x);
            quote! { #root::components::Placeholder::Title(#t) }
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
        #root::components::PreformattedText {
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
        .map(|x| tokenize_located_component(x, tokenize_row));
    let centered = table.centered;
    quote! {
        #root::components::Table {
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
                .map(|x| tokenize_located_component(x, tokenize_cell));
            quote! { #root::components::Row::Content { cells: vec![#(#t),*] } }
        }
        Row::Divider => {
            quote! { #root::components::Row::Divider }
        }
    }
}

fn tokenize_cell(cell: &Cell) -> TokenStream {
    let root = root_crate();
    match &cell {
        Cell::Content(x) => {
            let t = tokenize_inline_component_container(&x);
            quote! { #root::components::Cell::Content(#t) }
        }
        Cell::SpanAbove => {
            quote! { #root::components::Cell::SpanAbove }
        }
        Cell::SpanLeft => {
            quote! { #root::components::Cell::SpanLeft }
        }
    }
}

// Tags

fn tokenize_tags(tags: &Tags) -> TokenStream {
    let root = root_crate();
    let inner = tags.0.iter().map(tokenize_tag);
    quote! {
        #root::components::Tags(vec![#(#inner),*])
    }
}

fn tokenize_tag(tag: &Tag) -> TokenStream {
    let root = root_crate();
    let inner = tokenize_string(&tag.0);
    quote! {
        #root::components::Tag(#inner)
    }
}

// Typefaces

fn tokenize_decoration(decoration: &Decoration) -> TokenStream {
    let root = root_crate();
    match &decoration {
        Decoration::Bold => quote! { #root::components::Decoration::Bold },
        Decoration::BoldItalic => {
            quote! { #root::components::Decoration::BoldItalic }
        }
        Decoration::Code => quote! { #root::components::Decoration::Code },
        Decoration::Italic => {
            quote! { #root::components::Decoration::Italic }
        }
        Decoration::Strikeout => {
            quote! { #root::components::Decoration::Strikeout }
        }
        Decoration::Subscript => {
            quote! { #root::components::Decoration::Subscript }
        }
        Decoration::Superscript => {
            quote! { #root::components::Decoration::Superscript }
        }
    }
}

fn tokenize_decorated_text_content(
    decorated_text_content: &DecoratedTextContent,
) -> TokenStream {
    let root = root_crate();
    match &decorated_text_content {
        DecoratedTextContent::DecoratedText(x) => {
            let t = tokenize_decorated_text(&x);
            quote! { #root::components::DecoratedTextContent::DecoratedText(#t) }
        }
        DecoratedTextContent::Keyword(x) => {
            let t = tokenize_keyword(&x);
            quote! { #root::components::DecoratedTextContent::Keyword(#t) }
        }
        DecoratedTextContent::Link(x) => {
            let t = tokenize_link(&x);
            quote! { #root::components::DecoratedTextContent::Link(#t) }
        }
        DecoratedTextContent::Text(x) => {
            let t = tokenize_string(&x);
            quote! { #root::components::DecoratedTextContent::Text(#t) }
        }
    }
}

fn tokenize_decorated_text(decorated_text: &DecoratedText) -> TokenStream {
    let root = root_crate();
    let contents = decorated_text.contents.iter().map(|x| {
        tokenize_located_component(x, tokenize_decorated_text_content)
    });
    let decoration = tokenize_decoration(&decorated_text.decoration);
    quote! {
        #root::components::DecoratedText {
            contents: vec![#(#contents),*],
            decoration: #decoration,
        }
    }
}

fn tokenize_keyword(keyword: &Keyword) -> TokenStream {
    let root = root_crate();
    match keyword {
        Keyword::DONE => {
            quote! { #root::components::Keyword::DONE }
        }
        Keyword::FIXED => {
            quote! { #root::components::Keyword::FIXED }
        }
        Keyword::FIXME => {
            quote! { #root::components::Keyword::FIXME }
        }
        Keyword::STARTED => {
            quote! { #root::components::Keyword::STARTED }
        }
        Keyword::TODO => {
            quote! { #root::components::Keyword::TODO }
        }
        Keyword::XXX => {
            quote! { #root::components::Keyword::XXX }
        }
    }
}

fn tokenize_located_component<T: Tokenize>(
    lc: &LocatedComponent<T>,
    f: impl Fn(&T) -> TokenStream,
) -> TokenStream {
    let root = root_crate();
    let component = f(&lc.component);
    let region = tokenize_region(&lc.region);
    quote! {
        #root::LocatedComponent {
            component: #component,
            region: #region,
        }
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

fn tokenize_hashset<T: Tokenize>(
    s: &HashSet<T>,
    tty: TokenStream,
    f: impl Fn(&T) -> TokenStream,
) -> TokenStream {
    let items = s.iter().map(f);
    quote! { vec![#(#items),*].drain(..).collect::<std::collections::HashSet<#tty>>() }
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
