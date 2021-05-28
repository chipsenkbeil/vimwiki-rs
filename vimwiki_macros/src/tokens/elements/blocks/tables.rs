use crate::tokens::{
    utils::{root_crate, tokenize_hashmap},
    Tokenize, TokenizeContext,
};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::{Cell, CellPos, CellSpan, ColumnAlign, Table};

impl_tokenize!(tokenize_table, Table<'a>, 'a);
fn tokenize_table(ctx: &TokenizeContext, table: &Table) -> TokenStream {
    let root = root_crate();
    let centered = table.is_centered();
    let cells = tokenize_hashmap(
        table.as_data(),
        quote!(#root::CellPos),
        quote!(#root::Located<#root::Cell>),
        |x| do_tokenize!(ctx, x),
        |x| do_tokenize!(ctx, x),
    );

    quote! {
        #root::Table::new(
            #cells,
            #centered,
        )
    }
}

impl_tokenize!(tokenize_cell, Cell<'a>, 'a);
fn tokenize_cell(ctx: &TokenizeContext, cell: &Cell) -> TokenStream {
    let root = root_crate();
    match &cell {
        Cell::Content(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::Cell::Content(#t) }
        }
        Cell::Span(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::Cell::Span(#t) }
        }
        Cell::Align(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::Cell::Align(#t) }
        }
    }
}

impl_tokenize!(tokenize_column_align, ColumnAlign);
fn tokenize_column_align(
    _ctx: &TokenizeContext,
    column_align: &ColumnAlign,
) -> TokenStream {
    let root = root_crate();
    match column_align {
        ColumnAlign::Left => {
            quote! { #root::ColumnAlign::Left }
        }
        ColumnAlign::Center => {
            quote! { #root::ColumnAlign::Center }
        }
        ColumnAlign::Right => {
            quote! { #root::ColumnAlign::Right }
        }
    }
}

impl_tokenize!(tokenize_cell_span, CellSpan);
fn tokenize_cell_span(
    _ctx: &TokenizeContext,
    cell_span: &CellSpan,
) -> TokenStream {
    let root = root_crate();
    match cell_span {
        CellSpan::FromAbove => {
            quote! { #root::CellSpan::FromAbove }
        }
        CellSpan::FromLeft => {
            quote! { #root::CellSpan::FromLeft }
        }
    }
}

impl_tokenize!(tokenize_cell_pos, CellPos);
fn tokenize_cell_pos(
    _ctx: &TokenizeContext,
    cell_pos: &CellPos,
) -> TokenStream {
    let root = root_crate();
    let row = cell_pos.row;
    let col = cell_pos.col;
    quote! { #root::CellPos::new(#row, #col) }
}
