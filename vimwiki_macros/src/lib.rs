use paste::paste;
use proc_macro2::{Span, TokenStream};
use vimwiki::{elements, Language, Located};

mod error;
use error::{Error, Result};

mod tokens;
use tokens::{Tokenize, TokenizeContext};

mod utils;

macro_rules! impl_macro {
    ($name:ident, $from_str:ident, $type:ty, $raw_mode:expr) => {
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

                // Validate we did indeed only get a single argument
                utils::require_empty_or_trailing_comma(&mut input)?;

                // Load our input into a string
                let input = utils::input_to_string(first, $raw_mode)?;

                // Perform the action of parsing our language into a
                // structured format
                let element: $type = Language::$from_str(&input)
                    .parse()
                    .map_err(|x| Error::new(Span::call_site(), &format!("{}", x)))?;

                // Stuff our structure language into a proper token stream
                let ctx = TokenizeContext::default();
                let mut stream = TokenStream::new();
                element.tokenize(&ctx, &mut stream);
                Ok(stream)
            }

            // Do the act of expanding our input language into Rust code
            // at compile-time, reporting an error if we fail
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
            impl_macro!([<vimwiki_ $suffix>], from_vimwiki_str, $type, false);
            impl_macro!([<vimwiki_ $suffix _raw>], from_vimwiki_str, $type, true);
        }
    };
}

///////////////////////////////////////////////////////////////////////////////
// Implement macros for vimwiki
///////////////////////////////////////////////////////////////////////////////
impl_macro_vimwiki!(page, elements::Page);
impl_macro_vimwiki!(block_element, Located<elements::BlockElement>);
impl_macro_vimwiki!(
    inline_element_container,
    Located<elements::InlineElementContainer>
);
impl_macro_vimwiki!(inline_element, Located<elements::InlineElement>);
impl_macro_vimwiki!(blockquote, Located<elements::Blockquote>);
impl_macro_vimwiki!(comment, Located<elements::Comment>);
impl_macro_vimwiki!(line_comment, Located<elements::LineComment>);
impl_macro_vimwiki!(multi_line_comment, Located<elements::MultiLineComment>);
impl_macro_vimwiki!(definition_list, Located<elements::DefinitionList>);
impl_macro_vimwiki!(divider, Located<elements::Divider>);
impl_macro_vimwiki!(header, Located<elements::Header>);
impl_macro_vimwiki!(link, Located<elements::Link>);
impl_macro_vimwiki!(diary_link, Located<elements::DiaryLink>);
impl_macro_vimwiki!(external_file_link, Located<elements::ExternalFileLink>);
impl_macro_vimwiki!(raw_link, Located<elements::RawLink>);
impl_macro_vimwiki!(transclusion_link, Located<elements::TransclusionLink>);
impl_macro_vimwiki!(wiki_link, Located<elements::WikiLink>);
impl_macro_vimwiki!(inter_wiki_link, Located<elements::InterWikiLink>);
impl_macro_vimwiki!(list, Located<elements::List>);
impl_macro_vimwiki!(list_item, Located<elements::ListItem>);
impl_macro_vimwiki!(code_inline, Located<elements::CodeInline>);
impl_macro_vimwiki!(math_inline, Located<elements::MathInline>);
impl_macro_vimwiki!(math_block, Located<elements::MathBlock>);
impl_macro_vimwiki!(paragraph, Located<elements::Paragraph>);
impl_macro_vimwiki!(placeholder, Located<elements::Placeholder>);
impl_macro_vimwiki!(preformatted_text, Located<elements::PreformattedText>);
impl_macro_vimwiki!(table, Located<elements::Table>);
impl_macro_vimwiki!(tags, Located<elements::Tags>);
impl_macro_vimwiki!(decorated_text, Located<elements::DecoratedText>);
impl_macro_vimwiki!(keyword, Located<elements::Keyword>);
impl_macro_vimwiki!(text, Located<elements::Text>);
