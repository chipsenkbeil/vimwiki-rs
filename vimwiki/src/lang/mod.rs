pub mod elements;
mod parsers;
pub mod utils;

use derive_more::Display;
use elements::*;
use parsers::vimwiki;
pub use parsers::LangParserError;
use std::convert::TryFrom;
use utils::{Span, LE};

/// Represents a raw string for a type of language
/// (vimwiki, markdown, mediawiki)
#[derive(Clone, Debug, Eq, PartialEq, Display)]
pub enum RawStr<'a> {
    Vimwiki(&'a str),
    Markdown(&'a str),
    Mediawiki(&'a str),
}

macro_rules! parse {
    ($raw_str:ident, $f:expr) => {
        match &$raw_str {
            RawStr::Vimwiki(s) => {
                let input = Span::from(*s);
                let result = $f(input)?.1;

                // For debugging purposes, we will print out a report of what
                // parts of our parsers took the longest
                #[cfg(feature = "timekeeper")]
                parsers::print_timekeeper_report(true);

                Ok(result)
            }
            RawStr::Markdown(_) => {
                Err(nom::Err::Failure(LangParserError::unsupported()))
            }
            RawStr::Mediawiki(_) => {
                Err(nom::Err::Failure(LangParserError::unsupported()))
            }
        }
    };
}

macro_rules! impl_try_from {
    ($t:ty, $f:expr) => {
        impl<'a> TryFrom<RawStr<'a>> for $t {
            type Error = nom::Err<LangParserError>;

            fn try_from(s: RawStr<'a>) -> Result<Self, Self::Error> {
                parse!(s, $f)
            }
        }
    };
}

// Top-level types
impl_try_from!(LE<Page>, vimwiki::page);
impl_try_from!(LE<BlockElement>, vimwiki::blocks::block_element);
impl_try_from!(
    LE<InlineElementContainer>,
    vimwiki::blocks::inline::inline_element_container
);
impl_try_from!(LE<InlineElement>, vimwiki::blocks::inline::inline_element);

// Blockquotes
impl_try_from!(LE<Blockquote>, vimwiki::blocks::blockquotes::blockquote);

// Code
impl_try_from!(LE<CodeInline>, vimwiki::blocks::inline::code::code_inline);

// Comments
impl_try_from!(LE<Comment>, vimwiki::comments::comment);
impl_try_from!(LE<LineComment>, vimwiki::comments::line_comment);
impl_try_from!(LE<MultiLineComment>, vimwiki::comments::multi_line_comment);

// Definitions (NOTE: Generic LocatedElement def above handles term & def)
impl_try_from!(
    LE<DefinitionList>,
    vimwiki::blocks::definitions::definition_list
);
// impl_try_from!(LE<Definition>, vimwiki::definition);
// impl_try_from!(LE<Term>, vimwiki::term);

// Dividers
impl_try_from!(LE<Divider>, vimwiki::blocks::dividers::divider);

// Headers
impl_try_from!(LE<Header>, vimwiki::blocks::headers::header);

// Links
impl_try_from!(LE<Link>, vimwiki::blocks::inline::links::link);
impl_try_from!(
    LE<DiaryLink>,
    vimwiki::blocks::inline::links::diary::diary_link
);
impl_try_from!(
    LE<ExternalFileLink>,
    vimwiki::blocks::inline::links::external::external_file_link
);
impl_try_from!(LE<RawLink>, vimwiki::blocks::inline::links::raw::raw_link);
impl_try_from!(
    LE<TransclusionLink>,
    vimwiki::blocks::inline::links::transclusion::transclusion_link
);
impl_try_from!(
    LE<WikiLink>,
    vimwiki::blocks::inline::links::wiki::wiki_link
);
impl_try_from!(
    LE<InterWikiLink>,
    vimwiki::blocks::inline::links::interwiki::inter_wiki_link
);

// Lists
impl_try_from!(LE<List>, vimwiki::blocks::lists::list);

// Math
impl_try_from!(LE<MathInline>, vimwiki::blocks::inline::math::math_inline);
impl_try_from!(LE<MathBlock>, vimwiki::blocks::math::math_block);

// Paragraphs
impl_try_from!(LE<Paragraph>, vimwiki::blocks::paragraphs::paragraph);

// Placeholders
impl_try_from!(LE<Placeholder>, vimwiki::blocks::placeholders::placeholder);

// Preformatted Text
impl_try_from!(
    LE<PreformattedText>,
    vimwiki::blocks::preformatted::preformatted_text
);

// Tables
impl_try_from!(LE<Table>, vimwiki::blocks::tables::table);

// Tags
impl_try_from!(LE<Tags>, vimwiki::blocks::inline::tags::tags);

// Typefaces
impl_try_from!(LE<String>, vimwiki::blocks::inline::typefaces::text);
impl_try_from!(
    LE<DecoratedText>,
    vimwiki::blocks::inline::typefaces::decorated_text
);
impl_try_from!(LE<Keyword>, vimwiki::blocks::inline::typefaces::keyword);

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    /// Contains tests for the vimwiki language parsers
    mod vimwiki {
        use super::*;

        #[test]
        fn try_from_raw_str_to_le_page() {
            let input = RawStr::Vimwiki("some text");
            let _result: LE<Page> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_block_element() {
            let input = RawStr::Vimwiki("some text");
            let _result: LE<BlockElement> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_inline_element_container() {
            let input = RawStr::Vimwiki("some text");
            let _result: LE<InlineElementContainer> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_inline_element() {
            let input = RawStr::Vimwiki("some text");
            let _result: LE<InlineElement> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_blockquote() {
            let input = RawStr::Vimwiki("> some text");
            let _result: LE<Blockquote> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_code_inline() {
            let input = RawStr::Vimwiki("`code`");
            let _result: LE<CodeInline> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_comment() {
            let input = RawStr::Vimwiki("%% some comment");
            let _result: LE<Comment> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_line_comment() {
            let input = RawStr::Vimwiki("%% some comment");
            let _result: LE<LineComment> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_multi_line_comment() {
            let input = RawStr::Vimwiki("%%+ some comment +%%");
            let _result: LE<MultiLineComment> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_definition_list() {
            let input = RawStr::Vimwiki("term:: definition");
            let _result: LE<DefinitionList> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_divider() {
            let input = RawStr::Vimwiki("----");
            let _result: LE<Divider> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_header() {
            let input = RawStr::Vimwiki("= header =");
            let _result: LE<Header> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_link() {
            let input = RawStr::Vimwiki("[[link]]");
            let _result: LE<Link> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_diary_link() {
            let input = RawStr::Vimwiki("[[diary:2012-03-05]]");
            let _result: LE<DiaryLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_external_file_link() {
            let input = RawStr::Vimwiki("[[file:path/to/file]]");
            let _result: LE<ExternalFileLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_raw_link() {
            let input = RawStr::Vimwiki("https://example.com");
            let _result: LE<RawLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_transclusion_link() {
            let input = RawStr::Vimwiki("{{https://example.com/img.jpg}}");
            let _result: LE<TransclusionLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_wiki_link() {
            let input = RawStr::Vimwiki("[[link]]");
            let _result: LE<WikiLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_inter_wiki_link() {
            let input = RawStr::Vimwiki("[[wiki1:link]]");
            let _result: LE<InterWikiLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_list() {
            let input = RawStr::Vimwiki("- some list item");
            let _result: LE<List> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_math_inline() {
            let input = RawStr::Vimwiki("$math$");
            let _result: LE<MathInline> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_math_block() {
            let input = RawStr::Vimwiki("{{$\nmath\n}}$");
            let _result: LE<MathBlock> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_paragraph() {
            let input = RawStr::Vimwiki("some text");
            let _result: LE<Paragraph> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_placeholder() {
            let input = RawStr::Vimwiki("%title some text");
            let _result: LE<Placeholder> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_preformatted_text() {
            let input = RawStr::Vimwiki("{{{\nsome code\n}}}");
            let _result: LE<PreformattedText> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_table() {
            let input = RawStr::Vimwiki("|cell|");
            let _result: LE<Table> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_tags() {
            let input = RawStr::Vimwiki(":tag:");
            let _result: LE<Tags> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_string() {
            let input = RawStr::Vimwiki("some text");
            let _result: LE<String> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_decorated_text() {
            let input = RawStr::Vimwiki("*some text*");
            let _result: LE<DecoratedText> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_le_keyword() {
            let input = RawStr::Vimwiki("TODO");
            let _result: LE<Keyword> =
                input.try_into().expect("Failed to parse");
        }
    }
}
