use crate::tokens::{
    utils::root_crate, utils::tokenize_option, Tokenize, TokenizeContext,
};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::{
    List, ListItem, ListItemAttributes, ListItemContent, ListItemContents,
    ListItemSuffix, ListItemTodoStatus, ListItemType, OrderedListItemType,
    UnorderedListItemType,
};

impl_tokenize!(tokenize_list, List<'a>, 'a);
fn tokenize_list(ctx: &TokenizeContext, list: &List) -> TokenStream {
    let root = root_crate();
    let items = list.iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::List::new(::std::vec![#(#items),*])
    }
}

impl_tokenize!(tokenize_list_item, ListItem<'a>, 'a);
fn tokenize_list_item(ctx: &TokenizeContext, item: &ListItem) -> TokenStream {
    let root = root_crate();

    let pos = item.pos;
    let item_type_t = tokenize_list_item_type(ctx, &item.ty);
    let suffix_t = tokenize_list_item_suffix(ctx, &item.suffix);
    let contents_t = tokenize_list_item_contents(ctx, &item.contents);
    let attributes_t = tokenize_list_item_attributes(ctx, &item.attributes);
    quote! {
        #root::ListItem::new(
            #item_type_t,
            #suffix_t,
            #pos,
            #contents_t,
            #attributes_t,
        )
    }
}

impl_tokenize!(tokenize_list_item_content, ListItemContent<'a>, 'a);
fn tokenize_list_item_content(
    ctx: &TokenizeContext,
    list_item_content: &ListItemContent,
) -> TokenStream {
    let root = root_crate();
    match &list_item_content {
        ListItemContent::InlineContent(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::ListItemContent::InlineContent(#t) }
        }
        ListItemContent::List(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::ListItemContent::List(#t) }
        }
    }
}

impl_tokenize!(tokenize_list_item_contents, ListItemContents<'a>, 'a);
fn tokenize_list_item_contents(
    ctx: &TokenizeContext,
    list_item_contents: &ListItemContents,
) -> TokenStream {
    let root = root_crate();
    let contents = list_item_contents.iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::ListItemContents::new(::std::vec![#(#contents),*])
    }
}

impl_tokenize!(tokenize_list_item_suffix, ListItemSuffix);
fn tokenize_list_item_suffix(
    _ctx: &TokenizeContext,
    list_item_suffix: &ListItemSuffix,
) -> TokenStream {
    let root = root_crate();
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
fn tokenize_list_item_type(
    ctx: &TokenizeContext,
    list_item_type: &ListItemType,
) -> TokenStream {
    let root = root_crate();
    match &list_item_type {
        ListItemType::Ordered(x) => {
            let t = tokenize_ordered_list_item_type(ctx, &x);
            quote! { #root::ListItemType::Ordered(#t) }
        }
        ListItemType::Unordered(x) => {
            let t = tokenize_unordered_list_item_type(ctx, &x);
            quote! { #root::ListItemType::Unordered(#t) }
        }
    }
}

impl_tokenize!(tokenize_ordered_list_item_type, OrderedListItemType);
fn tokenize_ordered_list_item_type(
    _ctx: &TokenizeContext,
    ordered_list_item_type: &OrderedListItemType,
) -> TokenStream {
    let root = root_crate();
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
    ctx: &TokenizeContext,
    unordered_list_item_type: &UnorderedListItemType,
) -> TokenStream {
    let root = root_crate();
    match &unordered_list_item_type {
        UnorderedListItemType::Hyphen => {
            quote! { #root::UnorderedListItemType::Hyphen }
        }
        UnorderedListItemType::Asterisk => {
            quote! { #root::UnorderedListItemType::Asterisk }
        }
        UnorderedListItemType::Other(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::UnorderedListItemType::Other(#t) }
        }
    }
}

impl_tokenize!(tokenize_list_item_attributes, ListItemAttributes);
fn tokenize_list_item_attributes(
    ctx: &TokenizeContext,
    list_item_attributes: &ListItemAttributes,
) -> TokenStream {
    let root = root_crate();
    let todo_status =
        tokenize_option(ctx, &list_item_attributes.todo_status, |ctx, x| {
            do_tokenize!(ctx, x)
        });
    quote! {
        #root::ListItemAttributes {
            todo_status: #todo_status
        }
    }
}

impl_tokenize!(tokenize_list_item_todo_status, ListItemTodoStatus);
fn tokenize_list_item_todo_status(
    _ctx: &TokenizeContext,
    list_item_todo_status: &ListItemTodoStatus,
) -> TokenStream {
    let root = root_crate();
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
