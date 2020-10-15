pub mod elements;
pub mod parsers;

use derive_more::Display;
use elements::*;
use parsers::{vimwiki, Span};
use std::{borrow::Cow, convert::TryFrom};

/// Represents a raw string for a type of language
/// (vimwiki, markdown, mediawiki)
#[derive(Clone, Debug, Eq, PartialEq, Display)]
pub enum RawStr<'a> {
    Vimwiki(Cow<'a, str>),
    Markdown(Cow<'a, str>),
    Mediawiki(Cow<'a, str>),
}

impl<'a> RawStr<'a> {
    /// Wraps provided `&str` as a `RawStr` for *vimwiki*
    pub fn from_vimwiki_str(inner: &'a str) -> Self {
        Self::Vimwiki(Cow::from(inner))
    }

    /// Wraps provided `String` as a `RawStr` for *vimwiki*
    pub fn from_vimwiki_string(inner: String) -> Self {
        Self::Vimwiki(Cow::from(inner))
    }

    /// Wraps provided `&str` as a `RawStr` for *markdown*
    pub fn from_markdown_str(inner: &'a str) -> Self {
        Self::Markdown(Cow::from(inner))
    }

    /// Wraps provided `String` as a `RawStr` for *markdown*
    pub fn from_markdown_string(inner: String) -> Self {
        Self::Markdown(Cow::from(inner))
    }

    /// Wraps provided `&str` as a `RawStr` for *mediawiki*
    pub fn from_mediawiki_str(inner: &'a str) -> Self {
        Self::Mediawiki(Cow::from(inner))
    }

    /// Wraps provided `String` as a `RawStr` for *mediawiki*
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
}

impl RawStr<'_> {
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

    /// Whether or not this `RawStr` is only borrowing the underlying data
    pub fn is_borrowed(&self) -> bool {
        matches!(self.as_inner(), Cow::Borrowed(_))
    }

    /// Whether or not this `RawStr` owns the underlying data
    pub fn is_owned(&self) -> bool {
        matches!(self.as_inner(), Cow::Owned(_))
    }

    /// Converts `RawStr` into owned version, allocating a new string if
    /// necessary, or just yielding itself if already owned
    pub fn into_owned(self) -> RawStr<'static> {
        match self {
            Self::Vimwiki(x) => RawStr::Vimwiki(Cow::from(x.into_owned())),
            Self::Markdown(x) => RawStr::Markdown(Cow::from(x.into_owned())),
            Self::Mediawiki(x) => RawStr::Mediawiki(Cow::from(x.into_owned())),
        }
    }

    /// Returns a new `RawStr` which is identical but has as lifetime tied to
    /// this `RawStr`
    pub fn as_borrowed(&self) -> RawStr {
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
            Self::Vimwiki(ref x) => RawStr::Vimwiki(make_borrowed!(x)),
            Self::Markdown(ref x) => RawStr::Markdown(make_borrowed!(x)),
            Self::Mediawiki(ref x) => RawStr::Mediawiki(make_borrowed!(x)),
        }
    }
}

impl<'a> TryFrom<RawStr<'a>> for Located<Page<'a>> {
    type Error = nom::Err<parsers::Error>;

    fn try_from(s: RawStr<'a>) -> Result<Self, Self::Error> {
        if s.is_vimwiki() {
            vimwiki::page(s.into_inner().into_owned())
        } else {
            Err(nom::Err::Failure(parsers::Error::unsupported()))
        }
    }
}

macro_rules! impl_try_from {
    ($t:ty, $f:expr) => {
        impl<'a> TryFrom<RawStr<'a>> for $t {
            type Error = nom::Err<parsers::Error>;

            fn try_from(s: RawStr<'a>) -> Result<Self, Self::Error> {
                if s.is_vimwiki() {
                    let input = Span::from(s.as_str());
                    Ok($f(input)?.1)
                } else {
                    Err(nom::Err::Failure(parsers::Error::unsupported()))
                }
            }
        }
    };
}

// Top-level types
impl_try_from!(Located<BlockElement<'a>>, vimwiki::blocks::block_element);
impl_try_from!(
    Located<InlineElementContainer<'a>>,
    vimwiki::blocks::inline::inline_element_container
);
impl_try_from!(
    Located<InlineElement<'a>>,
    vimwiki::blocks::inline::inline_element
);

// Blockquotes
impl_try_from!(
    Located<Blockquote<'a>>,
    vimwiki::blocks::blockquotes::blockquote
);

// Code
impl_try_from!(
    Located<CodeInline<'a>>,
    vimwiki::blocks::inline::code::code_inline
);

// Comments
impl_try_from!(Located<Comment<'a>>, vimwiki::comments::comment);
impl_try_from!(Located<LineComment<'a>>, vimwiki::comments::line_comment);
impl_try_from!(
    Located<MultiLineComment<'a>>,
    vimwiki::comments::multi_line_comment
);

// Definitions (NOTE: Generic LocatedElement def above handles term & def)
impl_try_from!(
    Located<DefinitionList<'a>>,
    vimwiki::blocks::definitions::definition_list
);
// impl_try_from!(Located<Definition>, vimwiki::definition);
// impl_try_from!(Located<Term>, vimwiki::term);

// Dividers
impl_try_from!(Located<Divider>, vimwiki::blocks::dividers::divider);

// Headers
impl_try_from!(Located<Header<'a>>, vimwiki::blocks::headers::header);

// Links
impl_try_from!(Located<Link<'a>>, vimwiki::blocks::inline::links::link);
impl_try_from!(
    Located<DiaryLink<'a>>,
    vimwiki::blocks::inline::links::diary::diary_link
);
impl_try_from!(
    Located<ExternalFileLink<'a>>,
    vimwiki::blocks::inline::links::external::external_file_link
);
impl_try_from!(
    Located<RawLink<'a>>,
    vimwiki::blocks::inline::links::raw::raw_link
);
impl_try_from!(
    Located<TransclusionLink<'a>>,
    vimwiki::blocks::inline::links::transclusion::transclusion_link
);
impl_try_from!(
    Located<WikiLink<'a>>,
    vimwiki::blocks::inline::links::wiki::wiki_link
);
impl_try_from!(
    Located<InterWikiLink<'a>>,
    vimwiki::blocks::inline::links::interwiki::inter_wiki_link
);

// Lists
impl_try_from!(Located<List<'a>>, vimwiki::blocks::lists::list);

// Math
impl_try_from!(
    Located<MathInline<'a>>,
    vimwiki::blocks::inline::math::math_inline
);
impl_try_from!(Located<MathBlock<'a>>, vimwiki::blocks::math::math_block);

// Paragraphs
impl_try_from!(
    Located<Paragraph<'a>>,
    vimwiki::blocks::paragraphs::paragraph
);

// Placeholders
impl_try_from!(
    Located<Placeholder<'a>>,
    vimwiki::blocks::placeholders::placeholder
);

// Preformatted Text
impl_try_from!(
    Located<PreformattedText<'a>>,
    vimwiki::blocks::preformatted::preformatted_text
);

// Tables
impl_try_from!(Located<Table<'a>>, vimwiki::blocks::tables::table);

// Tags
impl_try_from!(Located<Tags<'a>>, vimwiki::blocks::inline::tags::tags);

// Typefaces
impl_try_from!(Located<Text<'a>>, vimwiki::blocks::inline::typefaces::text);
impl_try_from!(
    Located<DecoratedText<'a>>,
    vimwiki::blocks::inline::typefaces::decorated_text
);
impl_try_from!(
    Located<Keyword>,
    vimwiki::blocks::inline::typefaces::keyword
);

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    /// Contains tests for the vimwiki language parsers
    mod vimwiki {
        use super::*;

        #[test]
        fn try_from_raw_str_to_le_page() {
            let input = RawStr::from_vimwiki_str("some text");
            let _result: Located<Page> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_block_element() {
            let input = RawStr::from_vimwiki_str("some text");
            let _result: Located<BlockElement> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_inline_element_container() {
            let input = RawStr::from_vimwiki_str("some text");
            let _result: Located<InlineElementContainer> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_inline_element() {
            let input = RawStr::from_vimwiki_str("some text");
            let _result: Located<InlineElement> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_blockquote() {
            let input = RawStr::from_vimwiki_str("> some text");
            let _result: Located<Blockquote> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_code_inline() {
            let input = RawStr::from_vimwiki_str("`code`");
            let _result: Located<CodeInline> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_comment() {
            let input = RawStr::from_vimwiki_str("%% some comment");
            let _result: Located<Comment> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_line_comment() {
            let input = RawStr::from_vimwiki_str("%% some comment");
            let _result: Located<LineComment> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_multi_line_comment() {
            let input = RawStr::from_vimwiki_str("%%+ some comment +%%");
            let _result: Located<MultiLineComment> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_definition_list() {
            let input = RawStr::from_vimwiki_str("term:: definition");
            let _result: Located<DefinitionList> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_divider() {
            let input = RawStr::from_vimwiki_str("----");
            let _result: Located<Divider> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_header() {
            let input = RawStr::from_vimwiki_str("= header =");
            let _result: Located<Header> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_link() {
            let input = RawStr::from_vimwiki_str("[[link]]");
            let _result: Located<Link> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_diary_link() {
            let input = RawStr::from_vimwiki_str("[[diary:2012-03-05]]");
            let _result: Located<DiaryLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_external_file_link() {
            let input = RawStr::from_vimwiki_str("[[file:path/to/file]]");
            let _result: Located<ExternalFileLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_raw_link() {
            let input = RawStr::from_vimwiki_str("https://example.com");
            let _result: Located<RawLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_transclusion_link() {
            let input =
                RawStr::from_vimwiki_str("{{https://example.com/img.jpg}}");
            let _result: Located<TransclusionLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_wiki_link() {
            let input = RawStr::from_vimwiki_str("[[link]]");
            let _result: Located<WikiLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_inter_wiki_link() {
            let input = RawStr::from_vimwiki_str("[[wiki1:link]]");
            let _result: Located<InterWikiLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_list() {
            let input = RawStr::from_vimwiki_str("- some list item");
            let _result: Located<List> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_math_inline() {
            let input = RawStr::from_vimwiki_str("$math$");
            let _result: Located<MathInline> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_math_block() {
            let input = RawStr::from_vimwiki_str("{{$\nmath\n}}$");
            let _result: Located<MathBlock> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_paragraph() {
            let input = RawStr::from_vimwiki_str("some text");
            let _result: Located<Paragraph> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_placeholder() {
            let input = RawStr::from_vimwiki_str("%title some text");
            let _result: Located<Placeholder> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_preformatted_text() {
            let input = RawStr::from_vimwiki_str("{{{\nsome code\n}}}");
            let _result: Located<PreformattedText> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_table() {
            let input = RawStr::from_vimwiki_str("|cell|");
            let _result: Located<Table> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_tags() {
            let input = RawStr::from_vimwiki_str(":tag:");
            let _result: Located<Tags> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_text() {
            let input = RawStr::from_vimwiki_str("some text");
            let _result: Located<Text> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_decorated_text() {
            let input = RawStr::from_vimwiki_str("*some text*");
            let _result: Located<DecoratedText> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_keyword() {
            let input = RawStr::from_vimwiki_str("TODO");
            let _result: Located<Keyword> =
                input.try_into().expect("Failed to parse");
        }
    }
}
