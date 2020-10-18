pub mod elements;
pub mod parsers;

use derive_more::Display;
use elements::*;
use parsers::{vimwiki, Span};
use std::borrow::Cow;

/// Parse a value from a `Language`
pub trait FromLanguage<'a>: Sized {
    type Error;

    /// Parses a `Language` to return a value of this type
    fn from_language(s: &'a Language<'a>) -> Result<Self, Self::Error>;
}

/// Represents a raw, unparsed representation of some language
/// (vimwiki, markdown, mediawiki)
#[derive(Clone, Debug, Eq, PartialEq, Display)]
pub enum Language<'a> {
    Vimwiki(Cow<'a, str>),
    Markdown(Cow<'a, str>),
    Mediawiki(Cow<'a, str>),
}

impl<'a> Language<'a> {
    /// Wraps provided `&str` as a `Language` for *vimwiki*
    pub fn from_vimwiki_str(inner: &'a str) -> Self {
        Self::Vimwiki(Cow::from(inner))
    }

    /// Wraps provided `String` as a `Language` for *vimwiki*
    pub fn from_vimwiki_string(inner: String) -> Self {
        Self::Vimwiki(Cow::from(inner))
    }

    /// Wraps provided `&str` as a `Language` for *markdown*
    pub fn from_markdown_str(inner: &'a str) -> Self {
        Self::Markdown(Cow::from(inner))
    }

    /// Wraps provided `String` as a `Language` for *markdown*
    pub fn from_markdown_string(inner: String) -> Self {
        Self::Markdown(Cow::from(inner))
    }

    /// Wraps provided `&str` as a `Language` for *mediawiki*
    pub fn from_mediawiki_str(inner: &'a str) -> Self {
        Self::Mediawiki(Cow::from(inner))
    }

    /// Wraps provided `String` as a `Language` for *mediawiki*
    pub fn from_mediawiki_string(inner: String) -> Self {
        Self::Mediawiki(Cow::from(inner))
    }

    /// Converts into a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }

    /// Converts into a str slice
    pub fn as_str(&self) -> &str {
        self.as_inner()
    }

    /// Converts into a mut str slice
    pub fn as_mut_str(&mut self) -> &mut str {
        self.as_mut_inner().to_mut()
    }

    /// Converts into ref to inner `Cow<'_, str>` type
    pub fn as_inner(&self) -> &Cow<'a, str> {
        match self {
            Self::Vimwiki(ref x) => x,
            Self::Markdown(ref x) => x,
            Self::Mediawiki(ref x) => x,
        }
    }

    /// Converts into mut ref to inner `Cow<'_, str>` type
    pub fn as_mut_inner(&mut self) -> &mut Cow<'a, str> {
        match self {
            Self::Vimwiki(ref mut x) => x,
            Self::Markdown(ref mut x) => x,
            Self::Mediawiki(ref mut x) => x,
        }
    }

    /// Converts into inner `Cow<'_, str>` type
    pub fn into_inner(self) -> Cow<'a, str> {
        match self {
            Self::Vimwiki(x) => x,
            Self::Markdown(x) => x,
            Self::Mediawiki(x) => x,
        }
    }

    /// Parses this `Language` into another type
    pub fn parse<F: FromLanguage<'a>>(&'a self) -> Result<F, F::Error> {
        FromLanguage::from_language(self)
    }
}

impl Language<'_> {
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

    /// Whether or not this `Language` is only borrowing the underlying data
    pub fn is_borrowed(&self) -> bool {
        matches!(self.as_inner(), Cow::Borrowed(_))
    }

    /// Whether or not this `Language` owns the underlying data
    pub fn is_owned(&self) -> bool {
        matches!(self.as_inner(), Cow::Owned(_))
    }

    /// Converts `Language` into owned version, allocating a new string if
    /// necessary, or just yielding itself if already owned
    pub fn into_owned(self) -> Language<'static> {
        match self {
            Self::Vimwiki(x) => Language::Vimwiki(Cow::from(x.into_owned())),
            Self::Markdown(x) => Language::Markdown(Cow::from(x.into_owned())),
            Self::Mediawiki(x) => {
                Language::Mediawiki(Cow::from(x.into_owned()))
            }
        }
    }

    /// Returns a new `Language` which is identical but has as lifetime tied to
    /// this `Language`
    pub fn as_borrowed(&self) -> Language {
        use self::Cow::*;

        macro_rules! make_borrowed {
            ($value:expr) => {
                Cow::Borrowed(match $value {
                    Borrowed(borrowed) => *borrowed,
                    Owned(owned) => owned.as_str(),
                })
            };
        }

        match self {
            Self::Vimwiki(ref x) => Language::Vimwiki(make_borrowed!(x)),
            Self::Markdown(ref x) => Language::Markdown(make_borrowed!(x)),
            Self::Mediawiki(ref x) => Language::Mediawiki(make_borrowed!(x)),
        }
    }
}

macro_rules! impl_from_language {
    ($t:ty, $f:expr) => {
        impl<'a> FromLanguage<'a> for $t {
            type Error = nom::Err<parsers::Error>;

            fn from_language(s: &'a Language<'a>) -> Result<Self, Self::Error> {
                match s {
                    Language::Vimwiki(x) => Ok($f(Span::from(x.as_bytes()))?.1),
                    _ => Err(nom::Err::Failure(parsers::Error::unsupported())),
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
