use crate::tokens::{utils::element_path, utils::tokenize_option, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_list, List<'a>, 'a);
fn tokenize_list(list: &List) -> TokenStream {
    let root = element_path();
    let items = list.items.iter().map(|x| do_tokenize!(x));
    quote! {
        #root::List {
            items: ::std::vec![#(#items),*],
        }
    }
}

impl_tokenize!(tokenize_list_item, ListItem<'a>, 'a);
fn tokenize_list_item(list_item: &ListItem) -> TokenStream {
    let root = element_path();
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
        #root::ListItem {
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
    let root = element_path();
    match &list_item_content {
        ListItemContent::InlineContent(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::ListItemContent::InlineContent(#t) }
        }
        ListItemContent::List(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::ListItemContent::List(#t) }
        }
    }
}

impl_tokenize!(tokenize_list_item_contents, ListItemContents<'a>, 'a);
fn tokenize_list_item_contents(
    list_item_contents: &ListItemContents,
) -> TokenStream {
    let root = element_path();
    let contents = list_item_contents.contents.iter().map(|x| do_tokenize!(x));
    quote! {
        #root::ListItemContents {
            contents: ::std::vec![#(#contents),*],
        }
    }
}

impl_tokenize!(tokenize_list_item_suffix, ListItemSuffix);
fn tokenize_list_item_suffix(list_item_suffix: &ListItemSuffix) -> TokenStream {
    let root = element_path();
    match &list_item_suffix {
        ListItemSuffix::None => {
            quote! { #root::ListItemSuffix::None }
        }
        ListItemSuffix::Period => {
            quote! { #root::ListItemSuffix::Period }
        }
        ListItemSuffix::Paren => {
            quote! { #root::ListItemSuffix::Paren }
        }
    }
}

impl_tokenize!(tokenize_list_item_type, ListItemType<'a>, 'a);
fn tokenize_list_item_type(list_item_type: &ListItemType) -> TokenStream {
    let root = element_path();
    match &list_item_type {
        ListItemType::Ordered(x) => {
            let t = tokenize_ordered_list_item_type(&x);
            quote! { #root::ListItemType::Ordered(#t) }
        }
        ListItemType::Unordered(x) => {
            let t = tokenize_unordered_list_item_type(&x);
            quote! { #root::ListItemType::Unordered(#t) }
        }
    }
}

impl_tokenize!(tokenize_ordered_list_item_type, OrderedListItemType);
fn tokenize_ordered_list_item_type(
    ordered_list_item_type: &OrderedListItemType,
) -> TokenStream {
    let root = element_path();
    match &ordered_list_item_type {
        OrderedListItemType::Number => {
            quote! { #root::OrderedListItemType::Number }
        }
        OrderedListItemType::Pound => {
            quote! { #root::OrderedListItemType::Pound }
        }
        OrderedListItemType::LowercaseAlphabet => {
            quote! { #root::OrderedListItemType::LowercaseAlphabet }
        }
        OrderedListItemType::UppercaseAlphabet => {
            quote! { #root::OrderedListItemType::UppercaseAlphabet }
        }
        OrderedListItemType::LowercaseRoman => {
            quote! { #root::OrderedListItemType::LowercaseRoman }
        }
        OrderedListItemType::UppercaseRoman => {
            quote! { #root::OrderedListItemType::UppercaseRoman }
        }
    }
}

impl_tokenize!(tokenize_unordered_list_item_type, UnorderedListItemType<'a>, 'a);
fn tokenize_unordered_list_item_type(
    unordered_list_item_type: &UnorderedListItemType,
) -> TokenStream {
    let root = element_path();
    match &unordered_list_item_type {
        UnorderedListItemType::Hyphen => {
            quote! { #root::UnorderedListItemType::Hyphen }
        }
        UnorderedListItemType::Asterisk => {
            quote! { #root::UnorderedListItemType::Asterisk }
        }
        UnorderedListItemType::Other(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::UnorderedListItemType::Other(#t) }
        }
    }
}

impl_tokenize!(tokenize_list_item_attributes, ListItemAttributes);
fn tokenize_list_item_attributes(
    list_item_attributes: &ListItemAttributes,
) -> TokenStream {
    let root = element_path();
    let todo_status =
        tokenize_option(&list_item_attributes.todo_status, |x| do_tokenize!(x));
    quote! {
        #root::ListItemAttributes {
            todo_status: #todo_status
        }
    }
}

impl_tokenize!(tokenize_list_item_todo_status, ListItemTodoStatus);
fn tokenize_list_item_todo_status(
    list_item_todo_status: &ListItemTodoStatus,
) -> TokenStream {
    let root = element_path();
    match list_item_todo_status {
        ListItemTodoStatus::Incomplete => {
            quote! { #root::ListItemTodoStatus::Incomplete }
        }
        ListItemTodoStatus::PartiallyComplete1 => {
            quote! { #root::ListItemTodoStatus::PartiallyComplete1 }
        }
        ListItemTodoStatus::PartiallyComplete2 => {
            quote! { #root::ListItemTodoStatus::PartiallyComplete2 }
        }
        ListItemTodoStatus::PartiallyComplete3 => {
            quote! { #root::ListItemTodoStatus::PartiallyComplete3 }
        }
        ListItemTodoStatus::Complete => {
            quote! { #root::ListItemTodoStatus::Complete }
        }
        ListItemTodoStatus::Rejected => {
            quote! { #root::ListItemTodoStatus::Rejected }
        }
    }
}
