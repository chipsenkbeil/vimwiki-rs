pub mod elements;
pub mod parsers;

use derive_more::Display;
use elements::*;
use parsers::{vimwiki, IResult, Span};

/// Parse a value from a `Language`
pub trait FromLanguage<'a>: Sized {
    type Error;

    /// Parses a `Language` to return a value of this type
    fn from_language(language: Language<'a>) -> Result<Self, Self::Error>;
}

/// Represents a raw, unparsed representation of some language
/// (vimwiki, markdown, mediawiki)
///
/// ## Examples
///
/// ```
/// use vimwiki::{Language, Page};
///
/// // Load some language as a string
/// let language = Language::from_vimwiki_str(r#"
/// = My Header =
///
/// Some paragraph with *decorations* and [[links]] that you would normally
/// see in a vimwiki file.
/// "#);
///
/// // Parse the input as a page using vimwiki format
/// let page: Page = language.parse().unwrap();
/// ```
///
#[derive(Copy, Clone, Debug, Eq, PartialEq, Display)]
pub enum Language<'a> {
    Vimwiki(&'a str),
    Markdown(&'a str),
    Mediawiki(&'a str),
}

impl<'a> Language<'a> {
    /// Wraps provided `&str` as a `Language` for *vimwiki*
    pub fn from_vimwiki_str(inner: &'a str) -> Self {
        Self::Vimwiki(inner)
    }

    /// Wraps provided `&str` as a `Language` for *markdown*
    pub fn from_markdown_str(inner: &'a str) -> Self {
        Self::Markdown(inner)
    }

    /// Wraps provided `&str` as a `Language` for *mediawiki*
    pub fn from_mediawiki_str(inner: &'a str) -> Self {
        Self::Mediawiki(inner)
    }

    /// Whether or not this represents a vimwiki format
    pub fn is_vimwiki(&self) -> bool {
        matches!(self, Self::Vimwiki(_))
    }

    /// Whether or not this represents a markdown format
    pub fn is_markdown(&self) -> bool {
        matches!(self, Self::Markdown(_))
    }

    /// Whether or not this represents a mediawiki format
    pub fn is_mediawiki(&self) -> bool {
        matches!(self, Self::Mediawiki(_))
    }

    pub fn as_inner(&self) -> &str {
        match self {
            Self::Vimwiki(x) => x,
            Self::Markdown(x) => x,
            Self::Mediawiki(x) => x,
        }
    }

    /// Borrows this language and parses it into another type
    pub fn parse<F: FromLanguage<'a>>(&self) -> Result<F, F::Error> {
        FromLanguage::from_language(*self)
    }
}

macro_rules! impl_from_language {
    ($t:ty, $f:expr) => {
        impl<'a> FromLanguage<'a> for $t {
            type Error = parsers::Error<'a>;

            fn from_language(l: Language<'a>) -> Result<Self, Self::Error> {
                match l {
                    Language::Vimwiki(x) => Ok($f(Span::from(x))?.1),
                    _ => Err(parsers::Error::unsupported()),
                }
            }
        }
    };
}

// Top-level types
impl_from_language!(Page<'a>, vimwiki::page);
impl_from_language!(Located<BlockElement<'a>>, vimwiki::blocks::block_element);
impl_from_language!(
    Located<InlineElementContainer<'a>>,
    vimwiki::blocks::inline::inline_element_container
);
impl_from_language!(
    Located<InlineElement<'a>>,
    vimwiki::blocks::inline::inline_element
);

// Blockquotes
impl_from_language!(
    Located<Blockquote<'a>>,
    vimwiki::blocks::blockquotes::blockquote
);

// Code
impl_from_language!(
    Located<CodeInline<'a>>,
    vimwiki::blocks::inline::code::code_inline
);

// Comments
impl_from_language!(
    Located<Comment<'a>>,
    vimwiki::blocks::inline::comments::comment
);
impl_from_language!(
    Located<LineComment<'a>>,
    vimwiki::blocks::inline::comments::line_comment
);
impl_from_language!(
    Located<MultiLineComment<'a>>,
    vimwiki::blocks::inline::comments::multi_line_comment
);

// Definitions (NOTE: Generic LocatedElement def above handles term & def)
impl_from_language!(
    Located<DefinitionList<'a>>,
    vimwiki::blocks::definitions::definition_list
);
// impl_from_language!(Located<Definition>, vimwiki::definition);
// impl_from_language!(Located<Term>, vimwiki::term);

// Dividers
impl_from_language!(Located<Divider>, vimwiki::blocks::dividers::divider);

// Headers
impl_from_language!(Located<Header<'a>>, vimwiki::blocks::headers::header);

// Links
impl_from_language!(Located<Link<'a>>, vimwiki::blocks::inline::links::link);
impl_from_language!(
    Located<DiaryLink<'a>>,
    vimwiki::blocks::inline::links::diary::diary_link
);
impl_from_language!(
    Located<ExternalFileLink<'a>>,
    vimwiki::blocks::inline::links::external::external_file_link
);
impl_from_language!(
    Located<RawLink<'a>>,
    vimwiki::blocks::inline::links::raw::raw_link
);
impl_from_language!(
    Located<TransclusionLink<'a>>,
    vimwiki::blocks::inline::links::transclusion::transclusion_link
);
impl_from_language!(
    Located<WikiLink<'a>>,
    vimwiki::blocks::inline::links::wiki::wiki_link
);
impl_from_language!(
    Located<InterWikiLink<'a>>,
    vimwiki::blocks::inline::links::interwiki::inter_wiki_link
);

// Lists
impl_from_language!(Located<List<'a>>, vimwiki::blocks::lists::list);
impl_from_language!(Located<ListItem<'a>>, parse_list_item);
fn parse_list_item<'a>(input: Span<'a>) -> IResult<Located<ListItem<'a>>> {
    nom::combinator::map(
        vimwiki::blocks::lists::list_item,
        |(_, item): (usize, Located<ListItem>)| item,
    )(input)
}

// Math
impl_from_language!(
    Located<MathInline<'a>>,
    vimwiki::blocks::inline::math::math_inline
);
impl_from_language!(Located<MathBlock<'a>>, vimwiki::blocks::math::math_block);

// Paragraphs
impl_from_language!(
    Located<Paragraph<'a>>,
    vimwiki::blocks::paragraphs::paragraph
);

// Placeholders
impl_from_language!(
    Located<Placeholder<'a>>,
    vimwiki::blocks::placeholders::placeholder
);

// Preformatted Text
impl_from_language!(
    Located<PreformattedText<'a>>,
    vimwiki::blocks::preformatted::preformatted_text
);

// Tables
impl_from_language!(Located<Table<'a>>, vimwiki::blocks::tables::table);

// Tags
impl_from_language!(Located<Tags<'a>>, vimwiki::blocks::inline::tags::tags);

// Typefaces
impl_from_language!(
    Located<Text<'a>>,
    vimwiki::blocks::inline::typefaces::text
);
impl_from_language!(
    Located<DecoratedText<'a>>,
    vimwiki::blocks::inline::typefaces::decorated_text
);
impl_from_language!(
    Located<Keyword>,
    vimwiki::blocks::inline::typefaces::keyword
);

#[cfg(test)]
mod tests {
    use super::*;

    /// Contains tests for the vimwiki language parsers
    mod vimwiki {
        use super::*;

        #[test]
        fn parse_to_page() {
            let input = Language::from_vimwiki_str("some text");
            let _result: Page = input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_block_element() {
            let input = Language::from_vimwiki_str("some text");
            let _result: Located<BlockElement> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_inline_element_container() {
            let input = Language::from_vimwiki_str("some text");
            let _result: Located<InlineElementContainer> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_inline_element() {
            let input = Language::from_vimwiki_str("some text");
            let _result: Located<InlineElement> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_blockquote() {
            let input = Language::from_vimwiki_str("> some text");
            let _result: Located<Blockquote> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_code_inline() {
            let input = Language::from_vimwiki_str("`code`");
            let _result: Located<CodeInline> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_comment() {
            let input = Language::from_vimwiki_str("%% some comment");
            let _result: Located<Comment> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_line_comment() {
            let input = Language::from_vimwiki_str("%% some comment");
            let _result: Located<LineComment> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_multi_line_comment() {
            let input = Language::from_vimwiki_str("%%+ some comment +%%");
            let _result: Located<MultiLineComment> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_definition_list() {
            let input = Language::from_vimwiki_str("term:: definition");
            let _result: Located<DefinitionList> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_divider() {
            let input = Language::from_vimwiki_str("----");
            let _result: Located<Divider> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_header() {
            let input = Language::from_vimwiki_str("= header =");
            let _result: Located<Header> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_link() {
            let input = Language::from_vimwiki_str("[[link]]");
            let _result: Located<Link> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_diary_link() {
            let input = Language::from_vimwiki_str("[[diary:2012-03-05]]");
            let _result: Located<DiaryLink> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_external_file_link() {
            let input = Language::from_vimwiki_str("[[file:path/to/file]]");
            let _result: Located<ExternalFileLink> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_raw_link() {
            let input = Language::from_vimwiki_str("https://example.com");
            let _result: Located<RawLink> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_transclusion_link() {
            let input =
                Language::from_vimwiki_str("{{https://example.com/img.jpg}}");
            let _result: Located<TransclusionLink> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_wiki_link() {
            let input = Language::from_vimwiki_str("[[link]]");
            let _result: Located<WikiLink> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_inter_wiki_link() {
            let input = Language::from_vimwiki_str("[[wiki1:link]]");
            let _result: Located<InterWikiLink> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_list() {
            let input = Language::from_vimwiki_str("- some list item");
            let _result: Located<List> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_math_inline() {
            let input = Language::from_vimwiki_str("$math$");
            let _result: Located<MathInline> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_math_block() {
            let input = Language::from_vimwiki_str("{{$\nmath\n}}$");
            let _result: Located<MathBlock> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_paragraph() {
            let input = Language::from_vimwiki_str("some text");
            let _result: Located<Paragraph> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_placeholder() {
            let input = Language::from_vimwiki_str("%title some text");
            let _result: Located<Placeholder> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_preformatted_text() {
            let input = Language::from_vimwiki_str("{{{\nsome code\n}}}");
            let _result: Located<PreformattedText> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_table() {
            let input = Language::from_vimwiki_str("|cell|");
            let _result: Located<Table> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_tags() {
            let input = Language::from_vimwiki_str(":tag:");
            let _result: Located<Tags> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_text() {
            let input = Language::from_vimwiki_str("some text");
            let _result: Located<Text> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_decorated_text() {
            let input = Language::from_vimwiki_str("*some text*");
            let _result: Located<DecoratedText> =
                input.parse().expect("Failed to parse");
        }

        #[test]
        fn parse_to_located_keyword() {
            let input = Language::from_vimwiki_str("TODO");
            let _result: Located<Keyword> =
                input.parse().expect("Failed to parse");
        }
    }
}
