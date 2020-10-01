pub mod components;
mod parsers;
pub mod utils;

use components::*;
use derive_more::Display;
pub use parsers::LangParserError;
use parsers::{print_timekeeper_report, vimwiki};
use std::convert::TryFrom;
use utils::{Span, LC};

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
                print_timekeeper_report(true);

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
impl_try_from!(LC<Page>, vimwiki::page);
impl_try_from!(
    LC<BlockComponent>,
    vimwiki::block_components::block_component
);
impl_try_from!(
    LC<InlineComponentContainer>,
    vimwiki::block_components::inline::inline_component_container
);
impl_try_from!(
    LC<InlineComponent>,
    vimwiki::block_components::inline::inline_component
);

// Blockquotes
impl_try_from!(
    LC<Blockquote>,
    vimwiki::block_components::blockquotes::blockquote
);

// Comments
impl_try_from!(LC<Comment>, vimwiki::comments::comment);
impl_try_from!(LC<LineComment>, vimwiki::comments::line_comment);
impl_try_from!(LC<MultiLineComment>, vimwiki::comments::multi_line_comment);

// Definitions (NOTE: Generic LocatedComponent def above handles term & def)
impl_try_from!(
    LC<DefinitionList>,
    vimwiki::block_components::definitions::definition_list
);
// impl_try_from!(LC<Definition>, vimwiki::definition);
// impl_try_from!(LC<Term>, vimwiki::term);

// Dividers
impl_try_from!(LC<Divider>, vimwiki::block_components::dividers::divider);

// Headers
impl_try_from!(LC<Header>, vimwiki::block_components::headers::header);

// Links
impl_try_from!(LC<Link>, vimwiki::block_components::inline::links::link);
impl_try_from!(
    LC<DiaryLink>,
    vimwiki::block_components::inline::links::diary::diary_link
);
impl_try_from!(
    LC<ExternalFileLink>,
    vimwiki::block_components::inline::links::external::external_file_link
);
impl_try_from!(
    LC<RawLink>,
    vimwiki::block_components::inline::links::raw::raw_link
);
impl_try_from!(
    LC<TransclusionLink>,
    vimwiki::block_components::inline::links::transclusion::transclusion_link
);
impl_try_from!(
    LC<WikiLink>,
    vimwiki::block_components::inline::links::wiki::wiki_link
);
impl_try_from!(
    LC<InterWikiLink>,
    vimwiki::block_components::inline::links::interwiki::inter_wiki_link
);

// Lists
impl_try_from!(LC<List>, vimwiki::block_components::lists::list);

// Math
impl_try_from!(
    LC<MathInline>,
    vimwiki::block_components::inline::math::math_inline
);
impl_try_from!(LC<MathBlock>, vimwiki::block_components::math::math_block);

// Paragraphs
impl_try_from!(
    LC<Paragraph>,
    vimwiki::block_components::paragraphs::paragraph
);

// Placeholders
impl_try_from!(
    LC<Placeholder>,
    vimwiki::block_components::placeholders::placeholder
);

// Preformatted Text
impl_try_from!(
    LC<PreformattedText>,
    vimwiki::block_components::preformatted::preformatted_text
);

// Tables
impl_try_from!(LC<Table>, vimwiki::block_components::tables::table);

// Tags
impl_try_from!(LC<Tags>, vimwiki::block_components::inline::tags::tags);

// Typefaces
impl_try_from!(
    LC<String>,
    vimwiki::block_components::inline::typefaces::text
);
impl_try_from!(
    LC<DecoratedText>,
    vimwiki::block_components::inline::typefaces::decorated_text
);
impl_try_from!(
    LC<Keyword>,
    vimwiki::block_components::inline::typefaces::keyword
);

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    /// Contains tests for the vimwiki language parsers
    mod vimwiki {
        use super::*;

        #[test]
        fn try_from_raw_str_to_lc_page() {
            let input = RawStr::Vimwiki("some text");
            let _result: LC<Page> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_block_component() {
            let input = RawStr::Vimwiki("some text");
            let _result: LC<BlockComponent> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_inline_component_container() {
            let input = RawStr::Vimwiki("some text");
            let _result: LC<InlineComponentContainer> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_inline_component() {
            let input = RawStr::Vimwiki("some text");
            let _result: LC<InlineComponent> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_blockquote() {
            let input = RawStr::Vimwiki("> some text");
            let _result: LC<Blockquote> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_comment() {
            let input = RawStr::Vimwiki("%% some comment");
            let _result: LC<Comment> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_line_comment() {
            let input = RawStr::Vimwiki("%% some comment");
            let _result: LC<LineComment> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_multi_line_comment() {
            let input = RawStr::Vimwiki("%%+ some comment +%%");
            let _result: LC<MultiLineComment> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_definition_list() {
            let input = RawStr::Vimwiki("term:: definition");
            let _result: LC<DefinitionList> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_divider() {
            let input = RawStr::Vimwiki("----");
            let _result: LC<Divider> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_header() {
            let input = RawStr::Vimwiki("= header =");
            let _result: LC<Header> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_link() {
            let input = RawStr::Vimwiki("[[link]]");
            let _result: LC<Link> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_diary_link() {
            let input = RawStr::Vimwiki("[[diary:2012-03-05]]");
            let _result: LC<DiaryLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_external_file_link() {
            let input = RawStr::Vimwiki("[[file:path/to/file]]");
            let _result: LC<ExternalFileLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_raw_link() {
            let input = RawStr::Vimwiki("https://example.com");
            let _result: LC<RawLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_transclusion_link() {
            let input = RawStr::Vimwiki("{{https://example.com/img.jpg}}");
            let _result: LC<TransclusionLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_wiki_link() {
            let input = RawStr::Vimwiki("[[link]]");
            let _result: LC<WikiLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_inter_wiki_link() {
            let input = RawStr::Vimwiki("[[wiki1:link]]");
            let _result: LC<InterWikiLink> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_list() {
            let input = RawStr::Vimwiki("- some list item");
            let _result: LC<List> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_math_inline() {
            let input = RawStr::Vimwiki("$math$");
            let _result: LC<MathInline> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_math_block() {
            let input = RawStr::Vimwiki("{{$\nmath\n}}$");
            let _result: LC<MathBlock> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_paragraph() {
            let input = RawStr::Vimwiki("some text");
            let _result: LC<Paragraph> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_placeholder() {
            let input = RawStr::Vimwiki("%title some text");
            let _result: LC<Placeholder> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_preformatted_text() {
            let input = RawStr::Vimwiki("{{{\nsome code\n}}}");
            let _result: LC<PreformattedText> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_table() {
            let input = RawStr::Vimwiki("|cell|");
            let _result: LC<Table> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_tags() {
            let input = RawStr::Vimwiki(":tag:");
            let _result: LC<Tags> = input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_string() {
            let input = RawStr::Vimwiki("some text");
            let _result: LC<String> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_decorated_text() {
            let input = RawStr::Vimwiki("*some text*");
            let _result: LC<DecoratedText> =
                input.try_into().expect("Failed to parse");
        }

        #[test]
        fn try_from_raw_str_to_lc_keyword() {
            let input = RawStr::Vimwiki("TODO");
            let _result: LC<Keyword> =
                input.try_into().expect("Failed to parse");
        }
    }
}
