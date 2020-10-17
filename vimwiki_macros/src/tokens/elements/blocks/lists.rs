use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_list, List<'a>, 'a);
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

impl_tokenize!(tokenize_list_item, ListItem<'a>, 'a);
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

impl_tokenize!(tokenize_list_item_content, ListItemContent<'a>, 'a);
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

impl_tokenize!(tokenize_list_item_contents, ListItemContents<'a>, 'a);
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

impl_tokenize!(tokenize_list_item_suffix, ListItemSuffix);
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

impl_tokenize!(tokenize_list_item_type, ListItemType<'a>, 'a);
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

impl_tokenize!(tokenize_ordered_list_item_type, OrderedListItemType);
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

impl_tokenize!(tokenize_unordered_list_item_type, UnorderedListItemType<'a>, 'a);
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

impl_tokenize!(tokenize_list_item_attributes, ListItemAttributes);
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

impl_tokenize!(tokenize_list_item_todo_status, ListItemTodoStatus);
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
