use paste::paste;
use proc_macro2::{Span, TokenStream};
use std::convert::TryInto;
use vimwiki::{elements, RawStr, LE};

mod error;
use error::{Error, Result};

mod tokens;
use tokens::Tokenize;

mod utils;

macro_rules! impl_macro {
    ($name:ident, $raw_str:ident, $type:ty, $raw_mode:expr) => {
        #[proc_macro]
        pub fn $name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            let input = TokenStream::from(input);

            fn try_expand(input: TokenStream) -> Result<TokenStream> {
                let mut input = input.into_iter();

                let first = input.next().ok_or_else(|| {
                    Error::new(
                        Span::call_site(),
                        "unexpected end of macro invocation, expected format string",
                    )
                })?;

                let raw_source = utils::input_to_string(first, $raw_mode)?;
                let element: $type = RawStr::$raw_str(&raw_source)
                    .try_into()
                    .map_err(|x| Error::new(Span::call_site(), &format!("{}", x)))?;

                utils::require_empty_or_trailing_comma(&mut input)?;

                let mut stream = TokenStream::new();
                element.tokenize(&mut stream);
                Ok(stream)
            }

            let output = match try_expand(input) {
                Ok(tokens) => tokens,
                Err(err) => err.to_compile_error(),
            };

            proc_macro::TokenStream::from(output)
        }
    };
}

/// Macro that generates two macros in the form of
///
///     vimwiki_${suffix}
///     vimwiki_${suffix}_raw
///
/// Both convert the given text to the specified vimwiki type at compile time,
/// but the raw version uses the string literal as-is while the non-raw
/// version removes all leading and trailing blank lines AND determines the
/// minimum indentation level (poor man's indoc) and removes that from the
/// beginning of each line.
macro_rules! impl_macro_vimwiki {
    ($suffix:ident, $type:ty) => {
        paste! {
            impl_macro!([<vimwiki_ $suffix>], Vimwiki, $type, false);
            impl_macro!([<vimwiki_ $suffix _raw>], Vimwiki, $type, true);
        }
    };
}

///////////////////////////////////////////////////////////////////////////////
// Implement macros for vimwiki
///////////////////////////////////////////////////////////////////////////////
impl_macro_vimwiki!(page, LE<elements::Page>);
impl_macro_vimwiki!(block_element, LE<elements::BlockElement>);
impl_macro_vimwiki!(
    inline_element_container,
    LE<elements::InlineElementContainer>
);
impl_macro_vimwiki!(inline_element, LE<elements::InlineElement>);
impl_macro_vimwiki!(blockquote, LE<elements::Blockquote>);
impl_macro_vimwiki!(comment, LE<elements::Comment>);
impl_macro_vimwiki!(line_comment, LE<elements::LineComment>);
impl_macro_vimwiki!(multi_line_comment, LE<elements::MultiLineComment>);
impl_macro_vimwiki!(definition_list, LE<elements::DefinitionList>);
impl_macro_vimwiki!(divider, LE<elements::Divider>);
impl_macro_vimwiki!(header, LE<elements::Header>);
impl_macro_vimwiki!(link, LE<elements::Link>);
impl_macro_vimwiki!(diary_link, LE<elements::DiaryLink>);
impl_macro_vimwiki!(external_file_link, LE<elements::ExternalFileLink>);
impl_macro_vimwiki!(raw_link, LE<elements::RawLink>);
impl_macro_vimwiki!(transclusion_link, LE<elements::TransclusionLink>);
impl_macro_vimwiki!(wiki_link, LE<elements::WikiLink>);
impl_macro_vimwiki!(inter_wiki_link, LE<elements::InterWikiLink>);
impl_macro_vimwiki!(list, LE<elements::List>);
impl_macro_vimwiki!(math_inline, LE<elements::MathInline>);
impl_macro_vimwiki!(math_block, LE<elements::MathBlock>);
impl_macro_vimwiki!(paragraph, LE<elements::Paragraph>);
impl_macro_vimwiki!(placeholder, LE<elements::Placeholder>);
impl_macro_vimwiki!(preformatted_text, LE<elements::PreformattedText>);
impl_macro_vimwiki!(table, LE<elements::Table>);
impl_macro_vimwiki!(tags, LE<elements::Tags>);
impl_macro_vimwiki!(decorated_text, LE<elements::DecoratedText>);
impl_macro_vimwiki!(keyword, LE<elements::Keyword>);
impl_macro_vimwiki!(string, LE<String>);
