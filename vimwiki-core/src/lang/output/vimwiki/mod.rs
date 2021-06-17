mod config;
pub use config::*;

mod formatter;
pub use formatter::VimwikiFormatter;

mod convert;
pub use convert::ToVimwikiString;

mod error;
pub use error::{VimwikiOutputError, VimwikiOutputResult};

use crate::lang::{
    elements::*,
    output::{Output, OutputFormatter},
};
use percent_encoding::percent_decode_str;
use std::{collections::HashMap, fmt::Write};

impl<'a> Output<VimwikiFormatter> for Page<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        for element in self.elements.iter() {
            element.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for Element<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::Block(x) => x.fmt(f),
            Self::Inline(x) => x.fmt(f),
            Self::InlineBlock(x) => x.fmt(f),
        }
    }
}

impl<'a> Output<VimwikiFormatter> for InlineBlockElement<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::ListItem(x) => x.fmt(f),
            Self::Term(x) => x.fmt(f),
            Self::Definition(x) => x.fmt(f),
        }
    }
}

impl<'a> Output<VimwikiFormatter> for BlockElement<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::Blockquote(x) => x.fmt(f),
            Self::DefinitionList(x) => x.fmt(f),
            Self::Divider(x) => x.fmt(f),
            Self::Header(x) => x.fmt(f),
            Self::List(x) => x.fmt(f),
            Self::Math(x) => x.fmt(f),
            Self::Paragraph(x) => x.fmt(f),
            Self::Placeholder(x) => x.fmt(f),
            Self::CodeBlock(x) => x.fmt(f),
            Self::Table(x) => x.fmt(f),
        }
    }
}

impl<'a> Output<VimwikiFormatter> for Blockquote<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        let VimwikiBlockquoteConfig {
            prefer_indented_blockquote,
            trim_lines,
        } = f.config().blockquote;

        for line in self {
            f.write_indent()?;

            // TODO: Support determining when to use each type of blockquote
            //       as default instead of forcing one type or another
            // TODO: Support spacing on multiple >>> for nested blockquotes
            //       once those are implemented
            if prefer_indented_blockquote {
                write!(f, "    ")?;
            } else {
                write!(f, "> ")?;
            }

            if trim_lines {
                f.and_trim(|f| line.fmt(f))?;
            } else {
                line.fmt(f)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for DefinitionList<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        let VimwikiDefinitionListConfig {
            term_on_line_by_itself,
            trim_terms,
            trim_definitions,
        } = f.config().definition_list;

        for (term, defs) in self {
            f.write_indent()?;
            if trim_terms {
                f.and_trim(|f| term.fmt(f))?;
            } else {
                term.fmt(f)?;
            }
            write!(f, "::")?;

            for (idx, def) in defs.iter().enumerate() {
                if idx == 0 && !term_on_line_by_itself {
                    write!(f, " ")?;
                } else {
                    writeln!(f);
                    f.write_indent()?;
                    write!(f, ":: ")?;
                }

                if trim_definitions {
                    f.and_trim(|f| def.fmt(f))?;
                } else {
                    def.fmt(f)?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for DefinitionListValue<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        self.as_inner().fmt(f)
    }
}

impl Output<VimwikiFormatter> for Divider {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        writeln!(f, "----")?;
        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for Header<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        let VimwikiHeaderConfig {
            no_padding,
            trim_content,
        } = f.config().header;

        // If centered, we have to indent by some amount
        // TODO: Support configuring spaces for centered header
        if self.centered {
            f.write_indent()?;
        }

        // Beginning portion of header
        for _ in 0..self.level {
            write!(f, "=")?;
        }

        // Padding after beginning portion of header
        if !no_padding {
            write!(f, " ")?;
        }

        // Write the header's content, trimming whitespace if specified
        if trim_content {
            f.and_trim(|f| self.content.fmt(f))?;
        } else {
            self.content.fmt(f)?;
        }

        // Padding after ending portion of header
        if !no_padding {
            write!(f, " ")?;
        }

        // Ending portion of header
        for _ in 0..self.level {
            write!(f, "=")?;
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for List<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        for item in self {
            item.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for ListItem<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        let VimwikiListConfig { trim_lines } = f.config().list;

        for (idx, content) in self.contents.iter().enumerate() {
            // Apply indentation to place list item at right starting location
            f.write_indent()?;

            // If first line of content, write the prefix such as 1. or -
            // alongside content
            if idx == 0 {
                write!(f, "{} ", self.to_prefix())?;
            }

            // Write line(s) at proper indentation level with trim if specified
            if trim_lines {
                f.and_trim(|f| f.and_indent(|f| content.fmt(f)))?;
            } else {
                f.and_indent(|f| content.fmt(f))?;
            }
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for ListItemContent<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::List(x) => x.fmt(f)?,
            Self::InlineContent(x) => x.fmt(f)?,
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for MathBlock<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        // First, write starting line of math block
        f.write_indent()?;
        writeln!(
            f,
            "{{{{${}",
            self.environment
                .map(|e| format!("%{}%", e))
                .unwrap_or_default()
        )?;

        // Second, write all lines within math block
        for line in self {
            f.write_indent()?;
            writeln!(f, "{}", line)?;
        }

        // Third, write closing line of math block
        f.write_indent()?;
        writeln!(f, "}}}}$")?;

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for Placeholder<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::Title(x) => writeln!(f, "%title {}", x)?,
            Self::Date(x) => writeln!(f, "%date {}", x)?,
            Self::Template(x) => writeln!(f, "%template {}", x)?,
            Self::NoHtml => writeln!(f, "%nohtml")?,
            Self::Other { name, value } => writeln!(f, "%{} {}", name, value)?,
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for CodeBlock<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        // First, write starting line of code block
        f.write_indent()?;
        write!(f, "{{{{{{")?;

        if let Some(lang) = self.language.as_ref() {
            write!(f, "{}", lang)?;
        }

        for (idx, (key, value)) in self.metadata.iter().enumerate() {
            // If a language is not preceeding the metadata, don't add a space
            if idx != 0 || self.language.is_none() {
                write!(f, " ")?;
            }

            write!(f, "{}=\"{}\"", key, value)?;
        }

        writeln!(f);

        // Second, write all lines within code block
        for line in self {
            f.write_indent()?;
            writeln!(f, "{}", line)?;
        }

        // Third, write closing line of code block
        f.write_indent()?;
        writeln!(f, "}}}}}}")?;

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for Paragraph<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        let VimwikiParagraphConfig {
            line_wrap_column,
            no_line_wrap,
            trim_lines,
        } = f.config().paragraph;

        // TODO: Need to handle inline element container directly rather than
        //       calling fmt(...) because we need to know how many characters
        //       have been written to support line wrapping; this means that
        //       we actually need to revise output's fmt(...) to return the
        //       total characters written (or maybe just bytes) - alternatively,
        //       we can have a character counter within the formatter that
        //       gets incremented and can be cleared before writing (probably easier)
        for line in self {
            f.write_indent()?;

            if trim_lines {
                f.and_trim(|f| line.fmt(f))?;
            } else {
                line.fmt(f)?;
            }
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for Table<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        let VimwikiTableConfig { no_padding } = f.config().table;

        // First, we calculate the output for each content/span cell so we can
        // figure out how big each column's largest cell will be
        let fixed_size_cells: HashMap<CellPos, String> = self
            .cells()
            .zip_with_position()
            .filter_map(|(pos, cell)| match cell {
                Cell::Content(x) => Some({
                    // Create a copy of our formatter without the content so we can
                    // write the content fresh and pass it back as a string
                    let mut formatter = f.clone_without_content();
                    x.fmt(&mut formatter)?;
                    formatter.into_content()
                }),
                Cell::Span(CellSpan::FromLeft) => Some(">"),
                Cell::Span(CellSpan::FromAbove) => Some(r"\/"),
                _ => None,
            })
            .collect();

        // Second, we calculate largest cell in each column (col -> max size)
        let max_column_sizes: HashMap<usize, usize> = fixed_size_cells
            .iter()
            .fold(HashMap::new(), |(mut acc, (pos, text))| {
                let col = pos.col;
                let new_size = text.len();
                let cur_size = acc.entry(col).or_insert(new_size);
                if new_size > cur_size {
                    acc.insert(col, new_size);
                }
                acc
            });

        // Third, we iterate through all cells, one row at a time, and write
        // out the table using the size information
        for row in 0..self.row_cnt() {
            write!(f, "|")?;
            for col in 0..self.col_cnt() {
                // Get the max size, using 0 if nothing with a fixed size is
                // in the column
                let mut max_size =
                    max_column_sizes.get(col).unwrap_or_default();

                // If adding padding on each size, adjust the max cell size
                if !no_padding {
                    max_size += 2;
                }

                // If we have fixed content, write it with optional padding
                if let Some(text) = fixed_size_cells.get(CellPos { row, col }) {
                    if !no_padding {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", text)?;

                    if !no_padding {
                        write!(f, " ")?;
                    }

                // Otherwise, we have some form of divider and want to write it
                } else {
                    match self.get_cell(row, col) {
                        Cell::Align(ColumnAlign::None) => {
                            write!(f, "{}", "-".repeat(max_size))?
                        }
                        Cell::Align(ColumnAlign::Left) => {
                            write!(f, ":{}", "-".repeat(max_size - 1))?
                        }
                        Cell::Align(ColumnAlign::Center) => {
                            write!(f, ":{}:", "-".repeat(max_size - 2))?
                        }
                        Cell::Align(ColumnAlign::Right) => {
                            write!(f, "{}:", "-".repeat(max_size - 1))?
                        }
                        _ => write!(f, "{}", " ".repeat(max_size))?,
                    }
                }

                write!(f, "|")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for InlineElementContainer<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        for element in self {
            element.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for InlineElement<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::Text(x) => x.fmt(f),
            Self::DecoratedText(x) => x.fmt(f),
            Self::Keyword(x) => x.fmt(f),
            Self::Link(x) => x.fmt(f),
            Self::Tags(x) => x.fmt(f),
            Self::Code(x) => x.fmt(f),
            Self::Math(x) => x.fmt(f),
            Self::Comment(x) => x.fmt(f),
        }
    }
}

impl<'a> Output<VimwikiFormatter> for Text<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        write!(f, "{}", self.as_str())?;
        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for DecoratedText<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::Bold(contents) => {
                write!(f, "*")?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, "*")?;
            }
            Self::Italic(contents) => {
                write!(f, "_")?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, "_")?;
            }
            Self::Strikeout(contents) => {
                write!(f, "~~")?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, "~~")?;
            }
            Self::Superscript(contents) => {
                write!(f, "^")?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, "^")?;
            }
            Self::Subscript(contents) => {
                write!(f, ",,")?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, ",,")?;
            }
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for DecoratedTextContent<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::Text(x) => x.fmt(f),
            Self::DecoratedText(x) => x.fmt(f),
            Self::Keyword(x) => x.fmt(f),
            Self::Link(x) => x.fmt(f),
        }
    }
}

impl Output<VimwikiFormatter> for Keyword {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        write!(f, "{}", self)?;
        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for Link<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::Wiki { data } => {
                write!(f, "[[")?;
                write!(f, "{}", percent_decode_str(data.uri_ref()))?;
                if let Some(desc) = data.description() {
                    write!(f, "|{}", desc)?;
                }
                write!(f, "]]")?;
            }
            Self::IndexedInterWiki { index, data } => {
                write!(f, "[[")?;
                write!(f, "wiki{}:", index)?;
                write!(f, "{}", percent_decode_str(data.uri_ref()))?;
                if let Some(desc) = data.description() {
                    write!(f, "|{}", desc)?;
                }
                write!(f, "]]")?;
            }
            Self::NamedInterWiki { name, data } => {
                write!(f, "[[")?;
                write!(f, "wn.{}:", name)?;
                write!(f, "{}", percent_decode_str(data.uri_ref()))?;
                if let Some(desc) = data.description() {
                    write!(f, "|{}", desc)?;
                }
                write!(f, "]]")?;
            }
            Self::Diary { date, data } => {
                write!(f, "[[")?;
                write!(f, "{}", date)?;
                if let Some(anchor) = data.to_anchor() {
                    write!(f, "{}", anchor)?;
                }
                if let Some(desc) = data.description() {
                    write!(f, "|{}", desc)?;
                }
                write!(f, "]]")?;
            }
            Self::Raw { data } => {
                write!(f, "{}", data.uri_ref())?;
            }
            Self::Transclusion { data } => {
                write!(f, "{{{{")?;
                write!(f, "{}", percent_decode_str(data.uri_ref()))?;
                if let Some(desc) = data.description() {
                    write!(f, "|{}", desc)?;
                }
                if let Some(properties) = data.properties() {
                    // Transclusion requires a description prior to properties,
                    // so we make sure there is one, even if empty
                    if data.description().is_none() {
                        write!(f, "|")?;
                    }

                    for (key, value) in properties {
                        write!(f, "|{}=\"{}\"", key, value)?;
                    }
                }
                write!(f, "}}}}")?;
            }
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for Tags<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        write!(f, ":")?;

        for tag in self {
            write!(f, "{}:", tag)?;
        }

        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for CodeInline<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        write!(f, "`{}`", self)?;
        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for MathInline<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        write!(f, "${}$", self)?;
        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for Comment<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        match self {
            Self::Line(x) => x.fmt(f),
            Self::MultiLine(x) => x.fmt(f),
        }
    }
}

impl<'a> Output<VimwikiFormatter> for LineComment<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        write!(f, "%%{}", self)?;
        Ok(())
    }
}

impl<'a> Output<VimwikiFormatter> for MultiLineComment<'a> {
    fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
        write!(f, "%%+")?;
        for line in self {
            writeln!(f, "{}", line)?;
        }
        write!(f, "+%%")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use indoc::indoc;
    use std::{
        borrow::Cow,
        collections::HashMap,
        convert::TryFrom,
        path::{Path, PathBuf},
    };
    use uriparse::URIReference;

    #[test]
    fn blockquote_should_default_to_arrow_style() {
        let blockquote = Blockquote::new(vec![
            Cow::from("some lines"),
            Cow::from("of text"),
            Cow::from(""),
            Cow::from("with an empty line"),
        ]);
        let mut f = VimwikiFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                > some lines
                > of text
                >
                > with an empty line
            "}
        );
    }

    #[test]
    fn blockquote_should_trim_lines_by_default() {
        let blockquote = Blockquote::new(vec![
            Cow::from("\t some lines \t"),
            Cow::from("\t\tof text\r"),
        ]);
        let mut f = VimwikiFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "> some lines\n> of text\n");
    }

    #[test]
    fn blockquote_should_use_arrow_style_if_indented_setting_disabled() {
        let blockquote = Blockquote::new(vec![
            Cow::from("some lines"),
            Cow::from("of text"),
        ]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            blockquote: VimwikiBlockquoteConfig {
                prefer_indented_blockquote: false,
                ..Default::default()
            },
            ..Default::default()
        });
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "> some lines\n> of text\n");
    }

    #[test]
    fn blockquote_should_use_indented_style_if_setting_enabled() {
        let blockquote = Blockquote::new(vec![
            Cow::from("some lines"),
            Cow::from("of text"),
        ]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            blockquote: VimwikiBlockquoteConfig {
                prefer_indented_blockquote: true,
                ..Default::default()
            },
            ..Default::default()
        });
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "    some lines\n    of text\n");
    }

    #[test]
    fn blockquote_should_trim_lines_if_setting_enabled() {
        let blockquote = Blockquote::new(vec![
            Cow::from("\t some lines \t"),
            Cow::from("\t\tof text\r"),
        ]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            blockquote: VimwikiBlockquoteConfig {
                trim_lines: true,
                ..Default::default()
            },
            ..Default::default()
        });
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                > some lines
                > of text
            "}
        );
    }

    #[test]
    fn blockquote_should_not_trim_lines_if_setting_disabled() {
        let blockquote = Blockquote::new(vec![
            Cow::from("\t some lines \t"),
            Cow::from("\t\tof text\r"),
        ]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            blockquote: VimwikiBlockquoteConfig {
                trim_lines: true,
                ..Default::default()
            },
            ..Default::default()
        });
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                > \t some lines \t
                > \t\tof text\r
            "}
        );
    }

    #[test]
    fn blockquote_should_support_indentation() {
        todo!();
    }

    fn build_def_list<I: IntoIterator<D>, D: Into<DefinitionListValue>>(
        term: &str,
        defs: I,
    ) -> DefinitionList {
        vec![(
            term,
            defs.into_iter()
                .map(Into::into)
                .collect::<Vec<DefinitionListValue>>(),
        )]
        .into_iter()
        .collect()
    }

    #[test]
    fn definition_list_should_place_first_definition_on_same_line_as_term_by_default(
    ) {
        // Test no definitions
        let list = build_def_list("term1", []);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1::\n");

        // Test single definition
        let list = build_def_list("term1", ["def1"]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n");

        // Test multiple definitions
        let list = build_def_list("term1", ["def1", "def2"]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n:: def2\n");
    }

    #[test]
    fn definition_list_should_trim_terms_by_default() {
        // Test no definitions
        let list = build_def_list(" \rterm1\t ", []);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1::\n");

        // Test single definition
        let list = build_def_list(" \rterm1\t ", ["def1"]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n");

        // Test multiple definitions
        let list = build_def_list(" \rterm1\t ", ["def1", "def2"]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n:: def2\n");
    }

    #[test]
    fn definition_list_should_trim_definitions_by_default() {
        // Test single definition
        let list = build_def_list("term1", [" \rdef1\t "]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n");

        // Test multiple definitions
        let list = build_def_list("term1", [" \rdef1\t ", " \rdef2\t "]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n:: def2\n");
    }

    #[test]
    fn definition_list_should_not_place_first_definition_on_same_line_as_term_if_setting_disabled(
    ) {
        let config = VimwikiConfig {
            definition_list: VimwikiDefinitionListConfig {
                term_on_line_by_itself: false,
                ..Default::default()
            },
            ..Default::default()
        };

        // Test no definitions
        let list = build_def_list("term1", []);
        let mut f = VimwikiFormatter::new(config.clone());
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1::\n");

        // Test single definition
        let list = build_def_list("term1", ["def1"]);
        let mut f = VimwikiFormatter::new(config.clone());
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n");

        // Test multiple definitions
        let list = build_def_list("term1", ["def1", "def2"]);
        let mut f = VimwikiFormatter::new(config.clone());
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n:: def2\n");
    }

    #[test]
    fn definition_list_should_place_first_definition_on_same_line_as_term_if_setting_enabled(
    ) {
        let config = VimwikiConfig {
            definition_list: VimwikiDefinitionListConfig {
                term_on_line_by_itself: true,
                ..Default::default()
            },
            ..Default::default()
        };

        // Test no definitions
        let list = build_def_list("term1", []);
        let mut f = VimwikiFormatter::new(config.clone());
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1::\n");

        // Test single definition
        let list = build_def_list("term1", ["def1"]);
        let mut f = VimwikiFormatter::new(config.clone());
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1::\n:: def1\n");

        // Test multiple definitions
        let list = build_def_list("term1", ["def1", "def2"]);
        let mut f = VimwikiFormatter::new(config.clone());
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1::\n:: def1\n:: def2\n");
    }

    #[test]
    fn definition_list_should_trim_terms_if_setting_enabled() {
        let config = VimwikiConfig {
            definition_list: VimwikiDefinitionListConfig {
                trim_terms: true,
                ..Default::default()
            },
            ..Default::default()
        };

        // Test no definitions
        let list = build_def_list(" \rterm1\t ", []);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1::\n");

        // Test single definition
        let list = build_def_list(" \rterm1\t ", ["def1"]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n");

        // Test multiple definitions
        let list = build_def_list(" \rterm1\t ", ["def1", "def2"]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n:: def2\n");
    }

    #[test]
    fn definition_list_should_not_trim_terms_if_setting_disabled() {
        let config = VimwikiConfig {
            definition_list: VimwikiDefinitionListConfig {
                trim_terms: false,
                ..Default::default()
            },
            ..Default::default()
        };

        // Test no definitions
        let list = build_def_list(" \rterm1\t ", []);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), " \rterm1\t ::\n");

        // Test single definition
        let list = build_def_list(" \rterm1\t ", ["def1"]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), " \rterm1\t :: def1\n");

        // Test multiple definitions
        let list = build_def_list(" \rterm1\t ", ["def1", "def2"]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), " \rterm1\t :: def1\n:: def2\n");
    }

    #[test]
    fn definition_list_should_trim_definitions_if_setting_enabled() {
        let config = VimwikiConfig {
            definition_list: VimwikiDefinitionListConfig {
                trim_definitions: true,
                ..Default::default()
            },
            ..Default::default()
        };

        // Test single definition
        let list = build_def_list("term1", [" \rdef1\t "]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n");

        // Test multiple definitions
        let list = build_def_list("term1", [" \rdef1\t ", " \rdef2\t "]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1:: def1\n:: def2\n");
    }

    #[test]
    fn definition_list_should_not_trim_definitions_if_setting_disabled() {
        let config = VimwikiConfig {
            definition_list: VimwikiDefinitionListConfig {
                trim_definitions: false,
                ..Default::default()
            },
            ..Default::default()
        };

        // Test single definition
        let list = build_def_list("term1", [" \rdef1\t "]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1::  \rdef1\t \n");

        // Test multiple definitions
        let list = build_def_list("term1", [" \rdef1\t ", " \rdef2\t "]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "term1::  \rdef1\t \n::  \rdef2\t \n");
    }

    #[test]
    fn definition_list_should_support_indentation() {
        todo!();
    }

    #[test]
    fn divider_should_output_vimwiki_syntax() {
        let divider = Divider;

        let mut f = VimwikiFormatter::default();
        divider.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "----\n");
    }

    #[test]
    fn divider_should_not_support_indentation() {
        todo!();
    }

    #[test]
    fn header_should_trim_content_by_default() {
        let header = Header::new(
            text_to_inline_element_container(" \r\tsome header \r\t"),
            1,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "= some header =\n");
    }

    #[test]
    fn header_should_pad_content_by_default() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            1,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "= some header =\n");
    }

    #[test]
    fn header_should_trim_content_if_setting_enabled() {
        let header = Header::new(
            text_to_inline_element_container(" \r\tsome header \r\t"),
            1,
            false,
        );

        let mut f = VimwikiFormatter::new(VimwikiConfig {
            header: VimwikiHeaderConfig {
                trim_content: true,
                ..Default::default()
            },
            ..Default::default()
        });
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "= some header =\n");
    }

    #[test]
    fn header_should_not_trim_content_if_setting_disabled() {
        let header = Header::new(
            text_to_inline_element_container(" \r\tsome header \r\t"),
            1,
            false,
        );

        let mut f = VimwikiFormatter::new(VimwikiConfig {
            header: VimwikiHeaderConfig {
                trim_content: false,
                ..Default::default()
            },
            ..Default::default()
        });
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "=  \r\tsome header \r\t =\n");
    }

    #[test]
    fn header_should_pad_content_if_setting_enabled() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            1,
            false,
        );

        let mut f = VimwikiFormatter::new(VimwikiConfig {
            header: VimwikiHeaderConfig {
                no_padding: false,
                ..Default::default()
            },
            ..Default::default()
        });
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "= some header =\n");
    }

    #[test]
    fn header_should_not_pad_content_if_setting_disabled() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            1,
            false,
        );

        let mut f = VimwikiFormatter::new(VimwikiConfig {
            header: VimwikiHeaderConfig {
                no_padding: true,
                ..Default::default()
            },
            ..Default::default()
        });
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "=some header=\n");
    }

    #[test]
    fn header_should_support_level_1_header() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            1,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "= some header =\n");
    }

    #[test]
    fn header_should_support_level_2_header() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            2,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "== some header ==\n");
    }

    #[test]
    fn header_should_support_level_3_header() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            3,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "=== some header ===\n");
    }

    #[test]
    fn header_should_support_level_4_header() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            4,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "==== some header ====\n");
    }

    #[test]
    fn header_should_support_level_5_header() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            5,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "===== some header =====\n");
    }

    #[test]
    fn header_should_support_level_6_header() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            6,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "====== some header ======\n");
    }

    #[test]
    fn header_should_support_being_centered() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            1,
            true,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "    = some header =\n");
    }

    #[test]
    fn header_should_not_support_indentation() {
        todo!();
    }

    #[test]
    fn list_should_output_list_items() {
        let list = List::new(vec![
            Located::from(ListItem::new(
                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                ListItemSuffix::None,
                0,
                ListItemContents::new(vec![Located::from(
                    ListItemContent::InlineContent(
                        text_to_inline_element_container("some list item"),
                    ),
                )]),
                ListItemAttributes::default(),
            )),
            Located::from(ListItem::new(
                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                ListItemSuffix::None,
                1,
                ListItemContents::new(vec![Located::from(
                    ListItemContent::InlineContent(
                        text_to_inline_element_container("another list item"),
                    ),
                )]),
                ListItemAttributes::default(),
            )),
        ]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "- some list item\n- another list item\n");
    }

    #[test]
    fn list_item_should_trim_lines_by_default() {
        let list_item = ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(
                ListItemContent::InlineContent(
                    text_to_inline_element_container(
                        " \r\tsome list item \r\t",
                    ),
                ),
            )]),
            ListItemAttributes::default(),
        );
        let mut f = VimwikiFormatter::default();
        list_item.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "- some list item");
    }

    #[test]
    fn list_item_should_trim_lines_if_setting_enabled() {
        let list_item = ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(
                ListItemContent::InlineContent(
                    text_to_inline_element_container(
                        " \r\tsome list item \r\t",
                    ),
                ),
            )]),
            ListItemAttributes::default(),
        );
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            list: VimwikiListConfig { trim_lines: true },
            ..Default::default()
        });
        list_item.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "- some list item");
    }

    #[test]
    fn list_item_should_not_trim_lines_if_setting_disabled() {
        let list_item = ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(
                ListItemContent::InlineContent(
                    text_to_inline_element_container(
                        " \r\tsome list item \r\t",
                    ),
                ),
            )]),
            ListItemAttributes::default(),
        );
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            list: VimwikiListConfig { trim_lines: false },
            ..Default::default()
        });
        list_item.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "-  \r\tsome list item \r\t");
    }

    #[test]
    fn list_item_should_include_prefix_at_beginning() {
        fn new_list_item<T: Into<ListItemType>>(
            ty: T,
            suffix: ListItemSuffix,
        ) -> ListItem {
            ListItem::new(
                ty.into(),
                suffix,
                0,
                ListItemContents::new(vec![Located::from(
                    ListItemContent::InlineContent(
                        text_to_inline_element_container("some list item"),
                    ),
                )]),
                ListItemAttributes::default(),
            )
        }

        let item =
            new_list_item(UnorderedListItemType::Hyphen, ListItemSuffix::None);
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "- some list item");

        let item = new_list_item(
            UnorderedListItemType::Asterisk,
            ListItemSuffix::None,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "* some list item");

        let item = new_list_item(
            UnorderedListItemType::Other(Cow::from("xXx")),
            ListItemSuffix::None,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "xXx some list item");

        let item =
            new_list_item(OrderedListItemType::Number, ListItemSuffix::Period);
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "1. some list item");

        let item =
            new_list_item(OrderedListItemType::Number, ListItemSuffix::Paren);
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "1) some list item");

        let item =
            new_list_item(OrderedListItemType::Pound, ListItemSuffix::None);
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "# some list item");

        let item = new_list_item(
            OrderedListItemType::LowercaseAlphabet,
            ListItemSuffix::Period,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "a. some list item");

        let item = new_list_item(
            OrderedListItemType::LowercaseAlphabet,
            ListItemSuffix::Paren,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "a) some list item");

        let item = new_list_item(
            OrderedListItemType::UppercaseAlphabet,
            ListItemSuffix::Period,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "A. some list item");

        let item = new_list_item(
            OrderedListItemType::UppercaseAlphabet,
            ListItemSuffix::Paren,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "A) some list item");

        let item = new_list_item(
            OrderedListItemType::LowercaseRoman,
            ListItemSuffix::Period,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "i. some list item");

        let item = new_list_item(
            OrderedListItemType::LowercaseRoman,
            ListItemSuffix::Paren,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "i) some list item");

        let item = new_list_item(
            OrderedListItemType::UppercaseRoman,
            ListItemSuffix::Period,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "I. some list item");

        let item = new_list_item(
            OrderedListItemType::UppercaseRoman,
            ListItemSuffix::Paren,
        );
        let mut f = VimwikiFormatter::default();
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "I) some list item");
    }

    #[test]
    fn list_item_should_include_todo_status() {
        let mut item = ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(
                ListItemContent::InlineContent(
                    text_to_inline_element_container("some list item"),
                ),
            )]),
            ListItemAttributes::default(),
        );

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status = Some(ListItemTodoStatus::Incomplete);
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "- [ ] some list item\n");

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status =
            Some(ListItemTodoStatus::PartiallyComplete1);
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "- [.] some list item\n");

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status =
            Some(ListItemTodoStatus::PartiallyComplete2);
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "- [o] some list item\n");

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status =
            Some(ListItemTodoStatus::PartiallyComplete3);
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "- [O] some list item\n");

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status = Some(ListItemTodoStatus::Complete);
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "- [X] some list item\n");

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status = Some(ListItemTodoStatus::Rejected);
        item.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "- [-] some list item\n");
    }

    #[test]
    fn list_item_should_support_indentation() {
        todo!();
    }

    #[test]
    fn math_block_should_output_vimwiki() {
        let math = MathBlock::from_lines(vec!["some lines", "of math"]);
        let mut f = VimwikiFormatter::default();
        math.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "{{$\nsome lines\nof math\n}}$\n");
    }

    #[test]
    fn math_block_should_support_environment() {
        let math = MathBlock::new(
            vec![Cow::from("some lines"), Cow::from("of math")],
            Some(Cow::from("test environment")),
        );
        let mut f = VimwikiFormatter::default();
        math.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "{{$%test environment%\nsome lines\nof math\n}}$\n"
        );
    }

    #[test]
    fn math_block_should_support_indentation() {
        let math = MathBlock::from_lines(vec!["some lines", "of math"]);
        let mut f = VimwikiFormatter::default();
        f.and_indent(|f| math.fmt(&mut f)).unwrap();

        assert_eq!(
            f.get_content(),
            "    {{$\n    some lines\n    of math\n    }}$\n",
        );
    }

    #[test]
    fn placeholder_should_support_title() {
        let placeholder = Placeholder::title_from_str("test title");
        let mut f = VimwikiFormatter::default();
        placeholder.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "%title test title");
    }

    #[test]
    fn placeholder_should_support_date() {
        let placeholder = Placeholder::Date(NaiveDate::from_ymd(2021, 6, 17));
        let mut f = VimwikiFormatter::default();
        placeholder.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "%date 2021-06-17");
    }

    #[test]
    fn placeholder_should_support_template() {
        let placeholder = Placeholder::template_from_str("test template");
        let mut f = VimwikiFormatter::default();
        placeholder.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "%template test template");
    }

    #[test]
    fn placeholder_should_support_nohtml() {
        let placeholder = Placeholder::NoHtml;
        let mut f = VimwikiFormatter::default();
        placeholder.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "%nohtml");
    }

    #[test]
    fn placeholder_should_support_other() {
        let placeholder = Placeholder::other_from_str("name", "value");
        let mut f = VimwikiFormatter::default();
        placeholder.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "%name value");
    }

    #[test]
    fn placeholder_should_not_support_indentation() {
        todo!();
    }

    #[test]
    fn code_block_should_output_vimwiki() {
        let code = CodeBlock::new(
            None,
            Default::default(),
            vec![Cow::from("some lines"), Cow::from("of code")],
        );
        let mut f = VimwikiFormatter::default();
        code.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "{{{\nsome lines\nof code\n}}}\n");
    }

    #[test]
    fn code_block_should_support_language() {
        let code = CodeBlock::new(
            Some(Cow::from("language")),
            Default::default(),
            vec![Cow::from("some lines"), Cow::from("of code")],
        );
        let mut f = VimwikiFormatter::default();
        code.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "{{{language\nsome lines\nof code\n}}}\n");
    }

    #[test]
    fn code_block_should_support_indentation() {
        let code = CodeBlock::new(
            None,
            Default::default(),
            vec![Cow::from("some lines"), Cow::from("of code")],
        );
        let mut f = VimwikiFormatter::default();
        f.and_indent(|f| code.fmt(&mut f)).unwrap();

        assert_eq!(
            f.get_content(),
            "    {{{\n    some lines\n    of code\n    }}}\n",
        );
    }

    #[test]
    fn paragraph_should_wrap_lines_to_80_characters_split_by_words_by_default()
    {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container("some text that is over 80 characters in length should wrap based on word boundaries"),
            text_to_inline_element_container("with boundaries being on the next line"),
        ]);
        let mut f = VimwikiFormatter::default();
        paragraph.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "some text that is over 80 characters in length should wrap based on word\nboundaries with boundaries being on the next line\n");
    }

    #[test]
    fn paragraph_should_wrap_lines_to_80_characters_split_by_words_if_setting_enabled(
    ) {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container("some text that is over 80 characters in length should wrap based on word boundaries"),
            text_to_inline_element_container("with boundaries being on the next line"),
        ]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            paragraph: VimwikiParagraphConfig {
                no_line_wrap: false,
                line_wrap_column: 80,
                ..Default::default()
            },
            ..Default::default()
        });
        paragraph.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "some text that is over 80 characters in length should wrap based on word\nboundaries with boundaries being on the next line\n");
    }

    #[test]
    fn paragraph_should_not_wrap_lines_to_80_characters_split_by_words_if_setting_disabled(
    ) {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container("some text that is over 80 characters in length should wrap based on word boundaries"),
            text_to_inline_element_container("with boundaries being on the next line"),
        ]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            paragraph: VimwikiParagraphConfig {
                no_line_wrap: true,
                line_wrap_column: 80,
                ..Default::default()
            },
            ..Default::default()
        });
        paragraph.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "some text that is over 80 characters in length should wrap based on word boundaries\nwith boundaries being on the next line\n");
    }

    #[test]
    fn paragraph_should_trim_lines_by_default() {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container(" \r\tsome text \r\t"),
            text_to_inline_element_container(" \r\tand more text \r\t"),
        ]);
        let mut f = VimwikiFormatter::default();
        paragraph.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "some text\nand more text\n");
    }

    #[test]
    fn paragraph_should_trim_lines_if_setting_enabled() {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container(" \r\tsome text \r\t"),
            text_to_inline_element_container(" \r\tand more text \r\t"),
        ]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            paragraph: VimwikiParagraphConfig {
                trim_lines: true,
                ..Default::default()
            },
            ..Default::default()
        });
        paragraph.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "some text\nand more text\n");
    }

    #[test]
    fn paragraph_should_not_trim_lines_if_setting_disabled() {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container(" \r\tsome text \r\t"),
            text_to_inline_element_container(" \r\tand more text \r\t"),
        ]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            paragraph: VimwikiParagraphConfig {
                trim_lines: false,
                ..Default::default()
            },
            ..Default::default()
        });
        paragraph.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            " \r\tsome text \r\t\n \r\tand more text \r\t\n",
        );
    }

    #[test]
    fn paragraph_should_support_indentation() {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container("some text"),
            text_to_inline_element_container("and more text"),
        ]);
        let mut f = VimwikiFormatter::default();
        f.and_indent(|f| paragraph.fmt(&mut f)).unwrap();

        assert_eq!(f.get_content(), "   some text\n    and more text\n");
    }

    #[inline]
    fn single_column_table(centered: bool) -> Table {
        Table::new(
            vec![
                (
                    CellPos { row: 0, col: 0 },
                    Located::from(Cell::Content(
                        text_to_inline_element_container("some header"),
                    )),
                ),
                (
                    CellPos { row: 1, col: 0 },
                    Located::from(Cell::Align(ColumnAlign::default())),
                ),
                (
                    CellPos { row: 2, col: 0 },
                    Located::from(Cell::Content(
                        text_to_inline_element_container("some text"),
                    )),
                ),
            ],
            centered,
        )
    }

    #[test]
    fn table_should_pad_cells_by_default() {
        let table = single_column_table(false);
        let mut f = VimwikiFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                | some header |
                |-------------|
                | some text   |
            "},
        );
    }

    #[test]
    fn table_should_pad_cells_if_setting_enabled() {
        let table = single_column_table(false);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            table: VimwikiTableConfig { no_padding: false },
            ..Default::default()
        });
        table.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                | some header |
                |-------------|
                | some text   |
            "},
        );
    }

    #[test]
    fn table_should_not_pad_cells_if_setting_disabled() {
        let table = single_column_table(false);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            table: VimwikiTableConfig { no_padding: true },
            ..Default::default()
        });
        table.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                |some header|
                |-----------|
                |some text  |
            "},
        );
    }

    #[test]
    fn table_should_adjust_cell_size_to_fit_largest_cell_in_column() {
        todo!("Do 4 columns with content, span left, span above, and nothing");
    }

    #[test]
    fn table_should_adjust_cell_size_accounting_for_padding() {
        todo!("Do 4 columns with content, span left, span above, and nothing");
    }

    #[test]
    fn table_should_stretch_divider_rows_to_fit_column_sizes() {
        todo!("Do 4 columns with each of 4 types of column divider");
    }

    #[test]
    fn table_should_support_being_centered() {
        todo!();
    }

    #[test]
    fn table_should_support_indentation() {
        todo!();
    }

    #[test]
    fn text_should_output_the_same() {
        let text = Text::from("some text");
        let mut f = VimwikiFormatter::default();
        text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "some text");
    }

    #[test]
    fn decorated_text_should_output_bold_text_with_asterisks() {
        let decorated_text = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "*some text*");
    }

    #[test]
    fn decorated_text_should_output_italic_text_with_underscores() {
        let decorated_text = DecoratedText::Italic(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "_some text_");
    }

    #[test]
    fn decorated_text_should_output_strikeout_text_with_double_tilde() {
        let decorated_text = DecoratedText::Strikeout(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "~~some text~~");
    }

    #[test]
    fn decorated_text_should_output_superscript_text_with_carrot() {
        let decorated_text = DecoratedText::Superscript(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "^some text^");
    }

    #[test]
    fn decorated_text_should_output_subscript_text_with_double_comma() {
        let decorated_text = DecoratedText::Subscript(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), ",,some text,,");
    }

    #[test]
    fn keyword_should_output_self_in_all_caps() {
        let inputs_and_outputs = [
            (Keyword::Todo, "TODO"),
            (Keyword::Done, "DONE"),
            (Keyword::Started, "STARTED"),
            (Keyword::Fixme, "FIXME"),
            (Keyword::Fixed, "FIXED"),
            (Keyword::Xxx, "XXX"),
        ];

        for (keyword, output) in inputs_and_outputs {
            let mut f = VimwikiFormatter::default();
            keyword.fmt(&mut f).unwrap();
            assert_eq!(f.get_content(), output);
        }
    }

    #[test]
    fn wiki_link_should_output_vimwiki() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page").unwrap(),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[some/page]]");
    }

    #[test]
    fn wiki_link_should_output_vimwiki_with_uri_percent_decoded() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page%20with%20spaces").unwrap(),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[some/page with spaces]]");
    }

    #[test]
    fn wiki_link_should_support_text_descriptions() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page").unwrap(),
            Some(Description::from("text description")),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[some/page|text description]]");
    }

    #[test]
    fn wiki_link_should_support_transclusion_descriptions() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page").unwrap(),
            Some(
                Description::try_from_uri_ref_str(
                    "https://example.com/img.png",
                )
                .unwrap(),
            ),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "[[some/page|{{https://example.com/img.png}}]]"
        );
    }

    #[test]
    fn indexed_interwiki_link_should_output_vimwiki() {
        let link = Link::new_indexed_interwiki_link(
            123,
            URIReference::try_from("some/page").unwrap(),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[wiki123:some/page]]");
    }

    #[test]
    fn indexed_interwiki_link_should_output_vimwiki_with_uri_percent_decoded() {
        let link = Link::new_interwiki_link(
            123,
            URIReference::try_from("some/page%20with%20spaces").unwrap(),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[wiki123:some/page with spaces]]");
    }

    #[test]
    fn indexed_interwiki_link_should_support_text_descriptions() {
        let link = Link::new_indexed_interwiki_link(
            123,
            URIReference::try_from("some/page").unwrap(),
            Some(Description::from("text description")),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[wiki123:some/page|text description]]");
    }

    #[test]
    fn indexed_interwiki_link_should_support_transclusion_descriptions() {
        let link = Link::new_indexed_interwiki_link(
            123,
            URIReference::try_from("some/page").unwrap(),
            Some(
                Description::try_from_uri_ref_str(
                    "https://example.com/img.png",
                )
                .unwrap(),
            ),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "[[wiki123:some/page|{{https://example.com/img.png}}]]"
        );
    }

    #[test]
    fn named_interwiki_link_should_output_vimwiki() {
        let link = Link::new_named_interwiki_link(
            "my wiki",
            URIReference::try_from("some/page").unwrap(),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[wn.my wiki:some/page]]");
    }

    #[test]
    fn named_interwiki_link_should_output_vimwiki_with_uri_percent_decoded() {
        let link = Link::new_interwiki_link(
            "my wiki",
            URIReference::try_from("some/page%20with%20spaces").unwrap(),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[wn.my wiki:some/page with spaces]]");
    }

    #[test]
    fn named_interwiki_link_should_support_text_descriptions() {
        let link = Link::new_named_interwiki_link(
            "my wiki",
            URIReference::try_from("some/page").unwrap(),
            Some(Description::from("text description")),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "[[wn.my wiki:some/page|text description]]"
        );
    }

    #[test]
    fn named_interwiki_link_should_support_transclusion_descriptions() {
        let link = Link::new_named_interwiki_link(
            "my wiki",
            URIReference::try_from("some/page").unwrap(),
            Some(
                Description::try_from_uri_ref_str(
                    "https://example.com/img.png",
                )
                .unwrap(),
            ),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "[[wn.my wiki:some/page|{{https://example.com/img.png}}]]"
        );
    }

    #[test]
    fn diary_link_should_output_vimwiki() {
        let link =
            Link::new_diary_link(NaiveDate::from_ymd(2021, 6, 17), None, None);
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[2021-06-17]]");
    }

    #[test]
    fn diary_link_should_support_text_descriptions() {
        let link = Link::new_diary_link(
            NaiveDate::from_ymd(2021, 6, 17),
            Description::from("text description"),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[2021-06-17|text description]]");
    }

    #[test]
    fn diary_link_should_support_transclusion_descriptions() {
        let link = Link::new_diary_link(
            NaiveDate::from_ymd(2021, 6, 17),
            Description::try_from_uri_ref_str("https://example.com/img.png")
                .unwrap(),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "[[2021-06-17|{{https://example.com/img.png}}]]"
        );
    }

    #[test]
    fn diary_link_should_support_anchors() {
        let link = Link::new_diary_link(
            NaiveDate::from_ymd(2021, 6, 17),
            None,
            Anchor::from_uri_fragment("#one#two#three").unwrap(),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "[[2021-06-17#one#two#three]]");
    }

    #[test]
    fn diary_link_should_support_anchors_and_descriptions_together() {
        let link = Link::new_diary_link(
            NaiveDate::from_ymd(2021, 6, 17),
            Description::from("text description"),
            Anchor::from_uri_fragment("#one#two#three").unwrap(),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "[[2021-06-17#one#two#three|text description]]"
        );
    }

    #[test]
    fn raw_link_should_output_vimwiki() {
        let link = Link::try_new_raw_link("https://example.com/");
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "https://example.com/");
    }

    #[test]
    fn transclusion_link_should_output_vimwiki() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("some/img.png").unwrap(),
            None,
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "{{some/img.png}}");
    }

    #[test]
    fn transclusion_link_should_output_with_uri_percent_decoded() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("some/img%20with%20spaces.png").unwrap(),
            None,
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "{{some/img with spaces.png}}");
    }

    #[test]
    fn transclusion_link_should_support_text_descriptions() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("some/img.png").unwrap(),
            Some(Description::from("text description")),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "{{some/img.png|text description}}");
    }

    #[test]
    fn transclusion_link_should_support_transclusion_descriptions() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("some/img.png").unwrap(),
            Some(
                Description::try_from_uri_ref_str(
                    "https://example.com/img.png",
                )
                .unwrap(),
            ),
            None,
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "{{some/img.png|{{https://example.com/img.png}}}}"
        );
    }

    #[test]
    fn transclusion_link_should_support_properties() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("some/img.png").unwrap(),
            None,
            [(Cow::Borrowed("key"), Cow::Borrowed("value"))]
                .into_iter()
                .collect::<HashMap<Cow<str>, Cow<str>>>(),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "{{some/img.png||key=\"value\"}}");
    }

    #[test]
    fn transclusion_link_should_support_description_properties() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("some/img.png").unwrap(),
            Description::from("text description"),
            [(Cow::Borrowed("key"), Cow::Borrowed("value"))]
                .into_iter()
                .collect::<HashMap<Cow<str>, Cow<str>>>(),
        );
        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "{{some/img.png|text description|key=\"value\"}}"
        );
    }

    #[test]
    fn tags_should_output_vimwiki() {
        let tags: Tags = vec!["one", "two"].into_iter().collect();
        let mut f = VimwikiFormatter::default();
        tags.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), ":one:two:");
    }

    #[test]
    fn tags_should_output_single_tag() {
        let tags: Tags = vec!["one"].into_iter().collect();
        let mut f = VimwikiFormatter::default();
        tags.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), ":one:");
    }

    #[test]
    fn code_inline_should_output_vimwiki() {
        let code_inline = CodeInline::from("some code");
        let mut f = VimwikiFormatter::default();
        code_inline.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "`some code`");
    }

    #[test]
    fn math_inline_should_output_vimwiki() {
        let math_inline = MathInline::from("some math");
        let mut f = VimwikiFormatter::default();
        math_inline.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "$some math$");
    }

    #[test]
    fn line_comment_should_output_vimwiki() {
        let comment = LineComment::from("some comment");
        let mut f = VimwikiFormatter::default();
        comment.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "%%some comment");
    }

    #[test]
    fn multi_line_comment_should_output_vimwiki() {
        let comment = MultiLineComment::from("some single line comment");
        let mut f = VimwikiFormatter::default();
        comment.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "%%+some single line comment+%%");
    }

    #[test]
    fn multi_line_comment_should_support_multiple_lines() {
        let comment: MultiLineComment =
            ["line one", "line two"].into_iter().collect();
        let mut f = VimwikiFormatter::default();
        comment.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "%%+line one\nline two+%%");
    }
}
