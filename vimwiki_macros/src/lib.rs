use paste::paste;
use proc_macro2::{Span, TokenStream};
use syn::parse_macro_input;
use vimwiki::*;

mod error;
use error::{Error, Result};

mod tokens;
use tokens::{Tokenize, TokenizeContext};

mod args;
use args::FormatArgs;

mod formatter;
use formatter::Formatter;

mod utils;

macro_rules! impl_macro {
    ($name:ident, $from_str:ident, $type:ty, $raw_mode:expr, $verbatim:expr) => {
        #[proc_macro]
        pub fn $name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            let input_2 = input.clone();
            let args = parse_macro_input!(input as FormatArgs);
            let input = TokenStream::from(input_2);

            fn try_expand(input: TokenStream, args: FormatArgs) -> Result<TokenStream> {
                let mut input = input.into_iter();

                let first = input.next().ok_or_else(|| {
                    Error::new(
                        Span::call_site(),
                        "unexpected end of macro invocation, expected format string",
                    )
                })?;

                // Validate we did indeed only get a single argument
                // utils::require_empty_or_trailing_comma(&mut input)?;

                // Load our input into a string
                let input = utils::input_to_string(first, $raw_mode)?;

                // Perform the action of parsing our language into a
                // structured format
                let element: $type = Language::$from_str(&input)
                    .parse()
                    .map_err(|x| Error::new(Span::call_site(), &format!("{}", x)))?;

                // Stuff our structure language into a proper token stream
                let ctx = TokenizeContext {
                    formatter: Formatter::new(args),
                    verbatim: $verbatim,
                };
                let mut stream = TokenStream::new();
                element.tokenize(&ctx, &mut stream);
                Ok(stream)
            }

            // Do the act of expanding our input language into Rust code
            // at compile-time, reporting an error if we fail
            let output = match try_expand(input, args) {
                Ok(tokens) => tokens,
                Err(err) => err.to_compile_error(),
            };

            proc_macro::TokenStream::from(output)
        }
    };
}

/// Macro that generates four macros in the form of
///
/// 1. vimwiki_${suffix}
/// 2. vimwiki_${suffix}_raw
/// 3. vimwiki_${suffix}_format
/// 4. vimwiki_${suffix}_raw_format
///
/// All convert the given text to the specified vimwiki type at compile time.
///
/// The raw versions use the string literal as-is while the non-raw
/// version removes all leading and trailing blank lines AND determines the
/// minimum indentation level (poor man's indoc) and removes that from the
/// beginning of each line.
///
/// The format versions perform variable substitution in the same way that
/// format!, println!, and write! can inject content.
macro_rules! impl_macro_vimwiki {
    ($suffix:ident, $type:ty) => {
        paste! {
            impl_macro!([<vimwiki_ $suffix>], from_vimwiki_str, $type, false, true);
            impl_macro!([<vimwiki_ $suffix _raw>], from_vimwiki_str, $type, true, true);
            impl_macro!([<vimwiki_ $suffix _format>], from_vimwiki_str, $type, false, false);
            impl_macro!([<vimwiki_ $suffix _raw_format>], from_vimwiki_str, $type, true, false);
        }
    };
}

///////////////////////////////////////////////////////////////////////////////
// Implement macros for vimwiki
///////////////////////////////////////////////////////////////////////////////
impl_macro_vimwiki!(page, Page);
impl_macro_vimwiki!(block_element, Located<BlockElement>);
impl_macro_vimwiki!(inline_element_container, Located<InlineElementContainer>);
impl_macro_vimwiki!(inline_element, Located<InlineElement>);
impl_macro_vimwiki!(blockquote, Located<Blockquote>);
impl_macro_vimwiki!(comment, Located<Comment>);
impl_macro_vimwiki!(line_comment, Located<LineComment>);
impl_macro_vimwiki!(multi_line_comment, Located<MultiLineComment>);
impl_macro_vimwiki!(definition_list, Located<DefinitionList>);
impl_macro_vimwiki!(divider, Located<Divider>);
impl_macro_vimwiki!(header, Located<Header>);
impl_macro_vimwiki!(link, Located<Link>);
impl_macro_vimwiki!(list, Located<List>);
impl_macro_vimwiki!(list_item, Located<ListItem>);
impl_macro_vimwiki!(code_inline, Located<CodeInline>);
impl_macro_vimwiki!(math_inline, Located<MathInline>);
impl_macro_vimwiki!(math_block, Located<MathBlock>);
impl_macro_vimwiki!(paragraph, Located<Paragraph>);
impl_macro_vimwiki!(placeholder, Located<Placeholder>);
impl_macro_vimwiki!(preformatted_text, Located<PreformattedText>);
impl_macro_vimwiki!(table, Located<Table>);
impl_macro_vimwiki!(tags, Located<Tags>);
impl_macro_vimwiki!(decorated_text, Located<DecoratedText>);
impl_macro_vimwiki!(keyword, Located<Keyword>);
impl_macro_vimwiki!(text, Located<Text>);
