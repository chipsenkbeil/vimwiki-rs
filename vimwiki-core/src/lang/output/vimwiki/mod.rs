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
use std::{borrow::Cow, collections::HashMap, fmt::Write};
use uriparse::URIReference;

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

        // TODO: Need to calculate largest cell in each column

        fn write_row(
            f: &mut VimwikiFormatter,
            row: Row<'_, '_>,
            cell_max_size: usize,
            no_padding: bool,
        ) -> VimwikiOutputResult {
            f.write_indent()?;
            write!(f, "|")?;

            for cell in row {
                if !no_padding {
                    write!(f, " ")?;
                }

                match cell {
                    Cell::Content(x) => x.fmt(f)?,
                    Cell::Span(CellSpan::FromLeft) => write!(f, ">")?,
                    Cell::Span(CellSpan::FromAbove) => write!(f, r"\/")?,
                    Cell::Align(ColumnAlign::None) => {
                        write!(f, "{}", "-".repeat(cell_max_size))?
                    }
                    Cell::Align(ColumnAlign::Left) => {
                        write!(f, ":{}", "-".repeat(cell_max_size - 1))?
                    }
                    Cell::Align(ColumnAlign::Center) => {
                        write!(f, ":{}:", "-".repeat(cell_max_size - 2))?
                    }
                    Cell::Align(ColumnAlign::Right) => {
                        write!(f, "{}:", "-".repeat(cell_max_size - 1))?
                    }
                }

                if !no_padding {
                    write!(f, " ")?;
                }

                write!(f, "|")?;
            }

            Ok(())
        }

        // First, write any and all header rows
        for row in self.header_rows() {
            write_row(f, row, no_padding)?;
        }

        // Second, write a divider row if we had at least one header row
        if self.has_header_rows() {
            // Need to know how many dashes to add for each column, which means
            // that we need to keep track of characters written
            todo!();
        }

        // Third, write all body rows
        for row in self.body_rows() {
            write_row(f, row, no_padding)?;
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
    fn blockquote_with_multiple_line_groups_should_output_blockquote_tag_with_paragraph_for_each_group_of_lines(
    ) {
        let blockquote = Blockquote::new(vec![
            Cow::from("line1"),
            Cow::from("line2"),
            Cow::from(""),
            Cow::from("line3"),
        ]);
        let mut f = VimwikiFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <blockquote>
                <p>line1 line2</p>
                <p>line3</p>
                </blockquote>
            "}
        );
    }

    #[test]
    fn blockquote_with_single_line_group_should_output_blockquote_tag_with_no_paragraph(
    ) {
        let blockquote = Blockquote::new(vec![
            Cow::from("line1"),
            Cow::from("line2"),
            Cow::from("line3"),
        ]);
        let mut f = VimwikiFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <blockquote>
                line1
                line2
                line3
                </blockquote>
            "}
        );
    }

    #[test]
    fn blockquote_should_escape_vimwiki_in_each_line_of_a_singular_line_group()
    {
        let blockquote = Blockquote::new(vec![
            Cow::from("<test1>"),
            Cow::from("<test2>"),
            Cow::from("<test3>"),
        ]);
        let mut f = VimwikiFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <blockquote>
                &lt;test1&gt;
                &lt;test2&gt;
                &lt;test3&gt;
                </blockquote>
            "}
        );
    }

    #[test]
    fn blockquote_should_escape_vimwiki_in_each_line_of_multiple_line_groups() {
        let blockquote = Blockquote::new(vec![
            Cow::from("<test1>"),
            Cow::from("<test2>"),
            Cow::from(""),
            Cow::from("<test3>"),
        ]);
        let mut f = VimwikiFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <blockquote>
                <p>&lt;test1&gt; &lt;test2&gt;</p>
                <p>&lt;test3&gt;</p>
                </blockquote>
            "}
        );
    }

    #[test]
    fn definition_list_should_output_list_tag_with_term_and_definition_tags_together(
    ) {
        // Test no definitions
        let list: DefinitionList = vec![(
            Located::from(DefinitionListValue::from("term1")),
            Vec::new(),
        )]
        .into_iter()
        .collect();

        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <dl>
                <dt>term1</dt>
                </dl>
            "}
        );

        // Test single definition
        let list: DefinitionList = vec![(
            Located::from(DefinitionListValue::from("term1")),
            vec![Located::from(DefinitionListValue::from("def1"))],
        )]
        .into_iter()
        .collect();

        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <dl>
                <dt>term1</dt>
                <dd>def1</dd>
                </dl>
            "}
        );

        // Test multiple definitions
        let list: DefinitionList = vec![(
            Located::from(DefinitionListValue::from("term1")),
            vec![
                Located::from(DefinitionListValue::from("def1")),
                Located::from(DefinitionListValue::from("def2")),
            ],
        )]
        .into_iter()
        .collect();

        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <dl>
                <dt>term1</dt>
                <dd>def1</dd>
                <dd>def2</dd>
                </dl>
            "}
        );
    }

    #[test]
    fn divider_should_output_hr_tag() {
        let divider = Divider;

        let mut f = VimwikiFormatter::default();
        divider.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "<hr />\n");
    }

    #[test]
    fn header_should_output_h_and_a_tags() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            3,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                "<h3 id=\"some-header\" class=\"header\">",
                "<a href=\"#some-header\">",
                "some header",
                "</a>",
                "</h3>",
                "\n",
            ]
            .join(""),
        );
    }

    #[test]
    fn header_should_support_toc_variant() {
        let text = VimwikiHeaderConfig::default_table_of_contents();
        let header =
            Header::new(text_to_inline_element_container(&text), 1, false);

        let mut f = VimwikiFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                "<div class=\"toc\">",
                "<h1 id=\"contents\">",
                "Contents",
                "</h1>",
                "</div>",
                "\n",
            ]
            .join(""),
        );
    }

    #[test]
    fn header_should_escape_vimwiki_in_ids() {
        let header =
            Header::new(text_to_inline_element_container("<test>"), 3, false);

        // Configure to use a different table of contents string
        // that has characters that should be escaped
        let mut f = VimwikiFormatter::default();

        // Add some header ids prior to this one to verify that they aren't used
        f.insert_header_text(1, "h1");
        f.insert_header_text(2, "h2");

        header.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                "<div id=\"h1-h2-&lt;test&gt;\">",
                "<h3 id=\"&lt;test&gt;\" class=\"header\">",
                "<a href=\"#h1-h2-&lt;test&gt;\">",
                "&lt;test&gt;",
                "</a>",
                "</h3>",
                "</div>",
                "\n",
            ]
            .join(""),
        );
    }

    #[test]
    fn header_should_escape_vimwiki_in_ids_for_toc() {
        let header =
            Header::new(text_to_inline_element_container("<test>"), 1, false);

        // Configure to use a different table of contents string
        // that has characters that should be escaped
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            header: VimwikiHeaderConfig {
                table_of_contents: String::from("<test>"),
            },
            ..Default::default()
        });

        header.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "<div class=\"toc\"><h1 id=\"&lt;test&gt;\">&lt;test&gt;</h1></div>\n",
        );
    }

    #[test]
    fn header_should_produce_unique_ids_from_repeated_same_header() {
        let header1 = Header::new(
            text_to_inline_element_container("some header"),
            3,
            false,
        );
        let header2 = Header::new(
            text_to_inline_element_container("some header"),
            3,
            false,
        );
        let header3 = Header::new(
            text_to_inline_element_container("some header"),
            3,
            false,
        );

        let mut f = VimwikiFormatter::default();
        header1.fmt(&mut f).unwrap();
        header2.fmt(&mut f).unwrap();
        header3.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                // First header
                "<h3 id=\"some-header\" class=\"header\">",
                "<a href=\"#some-header\">",
                "some header",
                "</a>",
                "</h3>",
                "\n",
                // Second header
                "<h3 id=\"some-header-1\" class=\"header\">",
                "<a href=\"#some-header-1\">",
                "some header",
                "</a>",
                "</h3>",
                "\n",
                // Third header
                "<h3 id=\"some-header-2\" class=\"header\">",
                "<a href=\"#some-header-2\">",
                "some header",
                "</a>",
                "</h3>",
                "\n",
            ]
            .join(""),
        );
    }

    #[test]
    fn header_should_produce_unique_ids_from_repeated_same_header_with_nested_headers(
    ) {
        let header1 = Header::new(
            text_to_inline_element_container("some header"),
            3,
            false,
        );
        let header2 = Header::new(
            text_to_inline_element_container("some header"),
            3,
            false,
        );
        let header3 = Header::new(
            text_to_inline_element_container("some header"),
            3,
            false,
        );

        let mut f = VimwikiFormatter::default();
        f.insert_header_text(1, "a");
        f.insert_header_text(2, "b");
        header1.fmt(&mut f).unwrap();
        header2.fmt(&mut f).unwrap();

        f.insert_header_text(2, "c");
        header3.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                // First header
                "<div id=\"a-b-some-header\">",
                "<h3 id=\"some-header\" class=\"header\">",
                "<a href=\"#a-b-some-header\">",
                "some header",
                "</a>",
                "</h3>",
                "</div>",
                "\n",
                // Second header
                "<div id=\"a-b-some-header-1\">",
                "<h3 id=\"some-header-1\" class=\"header\">",
                "<a href=\"#a-b-some-header-1\">",
                "some header",
                "</a>",
                "</h3>",
                "</div>",
                "\n",
                // Third header
                "<div id=\"a-c-some-header\">",
                "<h3 id=\"some-header-2\" class=\"header\">",
                "<a href=\"#a-c-some-header\">",
                "some header",
                "</a>",
                "</h3>",
                "</div>",
                "\n",
            ]
            .join(""),
        );
    }

    #[test]
    fn list_should_output_ordered_list_if_ordered_type() {
        let list = List::new(vec![Located::from(ListItem::new(
            ListItemType::Ordered(OrderedListItemType::Number),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(
                ListItemContent::InlineContent(
                    text_to_inline_element_container("some list item"),
                ),
            )]),
            ListItemAttributes::default(),
        ))]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <ol>
                <li>some list item</li>
                </ol>
            "}
        );
    }

    #[test]
    fn list_should_output_unordered_list_if_unordered_type() {
        let list = List::new(vec![Located::from(ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(
                ListItemContent::InlineContent(
                    text_to_inline_element_container("some list item"),
                ),
            )]),
            ListItemAttributes::default(),
        ))]);
        let mut f = VimwikiFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <ul>
                <li>some list item</li>
                </ul>
            "}
        );
    }

    #[test]
    fn list_item_should_output_li_tag() {
        let item = ListItem::new(
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
        item.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "<li>some list item</li>\n");
    }

    #[test]
    fn list_item_should_support_adding_class_based_on_todo_status() {
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
        assert_eq!(
            f.get_content(),
            "<li class=\"done0\">some list item</li>\n"
        );

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status =
            Some(ListItemTodoStatus::PartiallyComplete1);
        item.fmt(&mut f).unwrap();
        assert_eq!(
            f.get_content(),
            "<li class=\"done1\">some list item</li>\n"
        );

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status =
            Some(ListItemTodoStatus::PartiallyComplete2);
        item.fmt(&mut f).unwrap();
        assert_eq!(
            f.get_content(),
            "<li class=\"done2\">some list item</li>\n"
        );

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status =
            Some(ListItemTodoStatus::PartiallyComplete3);
        item.fmt(&mut f).unwrap();
        assert_eq!(
            f.get_content(),
            "<li class=\"done3\">some list item</li>\n"
        );

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status = Some(ListItemTodoStatus::Complete);
        item.fmt(&mut f).unwrap();
        assert_eq!(
            f.get_content(),
            "<li class=\"done4\">some list item</li>\n"
        );

        let mut f = VimwikiFormatter::default();
        item.attributes.todo_status = Some(ListItemTodoStatus::Rejected);
        item.fmt(&mut f).unwrap();
        assert_eq!(
            f.get_content(),
            "<li class=\"rejected\">some list item</li>\n"
        );
    }

    #[test]
    fn math_block_should_output_a_mathjax_notation() {
        let math = MathBlock::from_lines(vec!["some lines", "of math"]);
        let mut f = VimwikiFormatter::default();
        math.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {r"
                \[
                some lines
                of math
                \]
            "}
        );
    }

    #[test]
    fn math_block_should_support_environments() {
        let math = MathBlock::new(
            vec![Cow::from("some lines"), Cow::from("of math")],
            Some(Cow::from("test environment")),
        );
        let mut f = VimwikiFormatter::default();
        math.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {r"
                \begin{test environment}
                some lines
                of math
                \end{test environment}
            "}
        );
    }

    #[test]
    fn placeholder_should_set_title_if_specified() {
        let placeholder = Placeholder::title_from_str("test title");
        let mut f = VimwikiFormatter::default();
        placeholder.fmt(&mut f).unwrap();

        assert_eq!(f.get_title(), Some("test title"));
    }

    #[test]
    fn placeholder_should_set_date_if_specified() {
        let placeholder = Placeholder::Date(NaiveDate::from_ymd(2021, 4, 27));
        let mut f = VimwikiFormatter::default();
        placeholder.fmt(&mut f).unwrap();

        assert_eq!(f.get_date(), Some(&NaiveDate::from_ymd(2021, 4, 27)));
    }

    #[test]
    fn placeholder_should_set_template_if_specified() {
        let placeholder = Placeholder::template_from_str("template file");
        let mut f = VimwikiFormatter::default();
        placeholder.fmt(&mut f).unwrap();

        assert_eq!(f.get_template(), Some(Path::new("template file")));
    }

    #[test]
    fn code_block_should_output_pre_code_tags_for_clientside_render() {
        let code = CodeBlock::from_lines(vec!["some lines", "of code"]);
        let mut f = VimwikiFormatter::default();
        code.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <pre><code>some lines
                of code</code></pre>
            "}
        );
    }

    #[test]
    fn code_block_should_escape_output_clientside() {
        let code = CodeBlock::from_lines(vec!["<test>"]);
        let mut f = VimwikiFormatter::default();
        code.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <pre><code>&lt;test&gt;</code></pre>
            "}
        );
    }

    #[test]
    fn code_block_should_support_serverside_render() {
        let code = CodeBlock::new(
            Some(Cow::from("rust")),
            HashMap::new(),
            vec![Cow::from("fn my_func() -> String { String::new() }")],
        );
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            code: VimwikiCodeConfig {
                server_side: true,
                ..Default::default()
            },
            ..Default::default()
        });
        code.fmt(&mut f).unwrap();

        let expected = [
            r#"<pre style="background-color:#ffffff;">"#,
            "\n",
            r#"<span style="font-weight:bold;color:#a71d5d;">fn </span>"#,
            r#"<span style="font-weight:bold;color:#795da3;">my_func</span>"#,
            r#"<span style="color:#323232;">() -&gt; String { </span>"#,
            r#"<span style="color:#0086b3;">String</span>"#,
            r#"<span style="color:#323232;">::new() }</span>"#,
            "\n",
            "</pre>\n",
        ]
        .join("");

        assert_eq!(f.get_content(), expected);
    }

    #[test]
    fn code_block_should_support_serverside_render_with_no_language() {
        let code = CodeBlock::from_lines(vec!["some lines", "of code"]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            code: VimwikiCodeConfig {
                server_side: true,
                ..Default::default()
            },
            ..Default::default()
        });
        code.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {r#"
                <pre style="background-color:#ffffff;">
                <span style="color:#323232;">some lines</span>
                <span style="color:#323232;">of code</span>
                </pre>
            "#}
        );
    }

    #[test]
    #[ignore]
    fn code_block_should_support_serverside_render_with_custom_syntax_dir() {
        todo!();
    }

    #[test]
    #[ignore]
    fn code_block_should_support_serverside_render_with_custom_theme_dir() {
        todo!();
    }

    #[test]
    fn paragraph_should_output_p_tag() {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container("some text"),
            text_to_inline_element_container("and more text"),
        ]);
        let mut f = VimwikiFormatter::default();
        paragraph.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "<p>some text and more text</p>\n");
    }

    #[test]
    fn paragraph_should_support_linebreaks_if_configured() {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container("some text"),
            text_to_inline_element_container("and more text"),
        ]);
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            paragraph: VimwikiParagraphConfig {
                ignore_newline: false,
            },
            ..Default::default()
        });
        paragraph.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "<p>some text<br />and more text</p>\n");
    }

    #[test]
    fn table_should_output_table_and_other_relevant_tags_for_header_and_body() {
        let table = Table::new(
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
            false,
        );
        let mut f = VimwikiFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {"
                <table>
                <thead>
                <tr>
                <th>some header</th>
                </tr>
                </thead>
                <tbody>
                <tr>
                <td>some text</td>
                </tr>
                </tbody>
                </table>
            "},
        );
    }

    #[test]
    fn table_should_support_rowspan_attr_on_header_cells() {
        let table = Table::new(
            vec![
                (
                    CellPos { row: 0, col: 0 },
                    Located::from(Cell::Content(
                        text_to_inline_element_container("some text"),
                    )),
                ),
                (
                    CellPos { row: 1, col: 0 },
                    Located::from(Cell::Span(CellSpan::FromAbove)),
                ),
                (
                    CellPos { row: 2, col: 0 },
                    Located::from(Cell::Align(ColumnAlign::default())),
                ),
            ],
            false,
        );
        let mut f = VimwikiFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {r#"
                <table>
                <thead>
                <tr>
                <th rowspan="2">some text</th>
                </tr>
                </thead>
                </table>
            "#},
        );
    }

    #[test]
    fn table_should_support_colspan_attr_on_header_cells() {
        let table = Table::new(
            vec![
                (
                    CellPos { row: 0, col: 0 },
                    Located::from(Cell::Content(
                        text_to_inline_element_container("some text"),
                    )),
                ),
                (
                    CellPos { row: 0, col: 1 },
                    Located::from(Cell::Span(CellSpan::FromLeft)),
                ),
                (
                    CellPos { row: 1, col: 0 },
                    Located::from(Cell::Align(ColumnAlign::default())),
                ),
                (
                    CellPos { row: 1, col: 1 },
                    Located::from(Cell::Align(ColumnAlign::default())),
                ),
            ],
            false,
        );
        let mut f = VimwikiFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {r#"
                <table>
                <thead>
                <tr>
                <th colspan="2">some text</th>
                </tr>
                </thead>
                </table>
            "#},
        );
    }

    #[test]
    fn table_should_support_rowspan_attr_on_body_cells() {
        let table = Table::new(
            vec![
                (
                    CellPos { row: 0, col: 0 },
                    Located::from(Cell::Content(
                        text_to_inline_element_container("some text"),
                    )),
                ),
                (
                    CellPos { row: 1, col: 0 },
                    Located::from(Cell::Span(CellSpan::FromAbove)),
                ),
            ],
            false,
        );
        let mut f = VimwikiFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {r#"
                <table>
                <tbody>
                <tr>
                <td rowspan="2">some text</td>
                </tr>
                </tbody>
                </table>
            "#},
        );
    }

    #[test]
    fn table_should_support_colspan_attr_on_body_cells() {
        let table = Table::new(
            vec![
                (
                    CellPos { row: 0, col: 0 },
                    Located::from(Cell::Content(
                        text_to_inline_element_container("some text"),
                    )),
                ),
                (
                    CellPos { row: 0, col: 1 },
                    Located::from(Cell::Span(CellSpan::FromLeft)),
                ),
            ],
            false,
        );
        let mut f = VimwikiFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            indoc! {r#"
                <table>
                <tbody>
                <tr>
                <td colspan="2">some text</td>
                </tr>
                </tbody>
                </table>
            "#},
        );
    }

    #[test]
    fn table_should_support_being_centered() {
        let table = Table::new(Vec::new(), true);
        let mut f = VimwikiFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "<table class=\"center\">\n</table>\n");
    }

    #[test]
    fn text_should_output_inner_str() {
        let text = Text::from("some text");
        let mut f = VimwikiFormatter::default();
        text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "some text");
    }

    #[test]
    fn text_should_escape_vimwiki() {
        let text = Text::from("<some>vimwiki</some>");
        let mut f = VimwikiFormatter::default();
        text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), r"&lt;some&gt;vimwiki&lt;/some&gt;");
    }

    #[test]
    fn decorated_text_should_output_strong_tag_for_bold_text() {
        let decorated_text = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<strong id="some-text">some text</strong>"#,
        );
    }

    #[test]
    fn decorated_text_should_include_extra_span_with_id_comprised_of_previous_headers_for_bold_text(
    ) {
        let decorated_text = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        f.insert_header_text(1, "one");
        f.insert_header_text(2, "two");
        f.insert_header_text(3, "three");
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                r#"<span id="one-two-three-some-text"></span>"#,
                r#"<strong id="some-text">some text</strong>"#,
            ]
            .join(""),
        );
    }

    #[test]
    fn decorated_text_should_escape_id_and_text_for_bold_text() {
        let decorated_text = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some <test> text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<strong id="some-&lt;test&gt;-text">some &lt;test&gt; text</strong>"#,
        );
    }

    #[test]
    fn decorated_text_should_produce_unique_ids_from_repeated_bold_text() {
        let bold1 = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("bold")),
        )]);
        let bold2 = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("bold")),
        )]);

        let mut f = VimwikiFormatter::default();
        bold1.fmt(&mut f).unwrap();
        bold2.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                r#"<strong id="bold">bold</strong>"#,
                r#"<strong id="bold-1">bold</strong>"#,
            ]
            .join("")
        );
    }

    #[test]
    fn decorated_text_should_produce_unique_ids_from_repeated_bold_text_with_nested_headers(
    ) {
        let bold1 = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("bold")),
        )]);
        let bold2 = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("bold")),
        )]);
        let bold3 = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("bold")),
        )]);

        let mut f = VimwikiFormatter::default();
        f.insert_header_text(1, "a");
        f.insert_header_text(2, "b");
        bold1.fmt(&mut f).unwrap();
        bold2.fmt(&mut f).unwrap();

        f.insert_header_text(2, "c");
        bold3.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                r#"<span id="a-b-bold"></span><strong id="bold">bold</strong>"#,
                r#"<span id="a-b-bold-1"></span><strong id="bold-1">bold</strong>"#,
                r#"<span id="a-c-bold"></span><strong id="bold-2">bold</strong>"#,
            ]
            .join("")
        );
    }

    #[test]
    fn decorated_text_should_output_em_tag_for_italic_text() {
        let decorated_text = DecoratedText::Italic(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), r#"<em>some text</em>"#);
    }

    #[test]
    fn decorated_text_should_output_del_tag_for_strikeout_text() {
        let decorated_text = DecoratedText::Strikeout(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), r#"<del>some text</del>"#);
    }

    #[test]
    fn decorated_text_should_output_sup_tag_for_superscript_text() {
        let decorated_text = DecoratedText::Superscript(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), r#"<sup><small>some text</small></sup>"#);
    }

    #[test]
    fn decorated_text_should_output_sub_tag_for_subscript_text() {
        let decorated_text = DecoratedText::Subscript(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = VimwikiFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), r#"<sub><small>some text</small></sub>"#);
    }

    #[test]
    fn keyword_should_output_span_with_class_for_todo() {
        let keyword = Keyword::Todo;

        let mut f = VimwikiFormatter::default();
        keyword.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), r#"<span class="todo">TODO</span>"#);
    }

    #[test]
    fn keyword_should_output_self_in_all_caps() {
        let keyword = Keyword::Done;

        let mut f = VimwikiFormatter::default();
        keyword.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "DONE");
    }

    #[test]
    fn wiki_link_should_output_a_tag() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page").unwrap(),
            None,
        );
        let mut f =
            VimwikiFormatter::new(test_vimwiki_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="some/page.vimwiki">some/page</a>"#
        );
    }

    #[test]
    fn wiki_link_should_support_anchors() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page#some-anchor").unwrap(),
            None,
        );
        let mut f =
            VimwikiFormatter::new(test_vimwiki_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="some/page.vimwiki#some-anchor">some/page#some-anchor</a>"#
        );
    }

    #[test]
    fn wiki_link_should_support_text_description() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page").unwrap(),
            Description::from("some description"),
        );
        let mut f =
            VimwikiFormatter::new(test_vimwiki_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="some/page.vimwiki">some description</a>"#
        );
    }

    #[test]
    fn wiki_link_should_support_transclusion_link_description() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page").unwrap(),
            Description::try_from_uri_ref_str("some/img.png").unwrap(),
        );
        let mut f =
            VimwikiFormatter::new(test_vimwiki_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="some/page.vimwiki"><img src="some/img.png" /></a>"#
        );
    }

    #[test]
    fn indexed_inter_wiki_link_should_output_a_tag() {
        let link = Link::new_indexed_interwiki_link(
            1,
            URIReference::try_from("some/page").unwrap(),
            None,
        );

        // Make a config with two wikis so we can refer to the other one
        let mut c = test_vimwiki_config("wiki", "test.wiki");
        add_wiki(&mut c, "wiki2");

        let mut f = VimwikiFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.vimwiki">some/page</a>"#
        );
    }

    #[test]
    fn indexed_inter_wiki_link_should_support_anchors() {
        let link = Link::new_indexed_interwiki_link(
            1,
            URIReference::try_from("some/page#some-anchor").unwrap(),
            None,
        );

        // Make a config with two wikis so we can refer to the other one
        let mut c = test_vimwiki_config("wiki", "test.wiki");
        add_wiki(&mut c, "wiki2");

        let mut f = VimwikiFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.vimwiki#some-anchor">some/page#some-anchor</a>"#
        );
    }

    #[test]
    fn indexed_inter_wiki_link_should_support_text_description() {
        let link = Link::new_indexed_interwiki_link(
            1,
            URIReference::try_from("some/page").unwrap(),
            Description::from("some description"),
        );

        // Make a config with two wikis so we can refer to the other one
        let mut c = test_vimwiki_config("wiki", "test.wiki");
        add_wiki(&mut c, "wiki2");

        let mut f = VimwikiFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.vimwiki">some description</a>"#
        );
    }

    #[test]
    fn indexed_inter_wiki_link_should_support_transclusion_link_description() {
        let link = Link::new_indexed_interwiki_link(
            1,
            URIReference::try_from("some/page").unwrap(),
            Description::try_from_uri_ref_str("some/img.png").unwrap(),
        );

        // Make a config with two wikis so we can refer to the other one
        let mut c = test_vimwiki_config("wiki", "test.wiki");
        add_wiki(&mut c, "wiki2");

        let mut f = VimwikiFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.vimwiki"><img src="some/img.png" /></a>"#
        );
    }

    #[test]
    fn named_inter_wiki_link_should_output_a_tag() {
        let link = Link::new_named_interwiki_link(
            "my-wiki",
            URIReference::try_from("some/page").unwrap(),
            None,
        );

        // Make a config with two wikis so we can refer to the other one
        let mut c = test_vimwiki_config("wiki", "test.wiki");
        add_wiki_with_name(&mut c, "wiki2", "my-wiki");

        let mut f = VimwikiFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.vimwiki">some/page</a>"#
        );
    }

    #[test]
    fn named_inter_wiki_link_should_support_anchors() {
        let link = Link::new_named_interwiki_link(
            "my-wiki",
            URIReference::try_from("some/page#some-anchor").unwrap(),
            None,
        );

        // Make a config with two wikis so we can refer to the other one
        let mut c = test_vimwiki_config("wiki", "test.wiki");
        add_wiki_with_name(&mut c, "wiki2", "my-wiki");

        let mut f = VimwikiFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.vimwiki#some-anchor">some/page#some-anchor</a>"#
        );
    }

    #[test]
    fn named_inter_wiki_link_should_support_text_description() {
        let link = Link::new_named_interwiki_link(
            "my-wiki",
            URIReference::try_from("some/page").unwrap(),
            Description::from("some description"),
        );

        // Make a config with two wikis so we can refer to the other one
        let mut c = test_vimwiki_config("wiki", "test.wiki");
        add_wiki_with_name(&mut c, "wiki2", "my-wiki");

        let mut f = VimwikiFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.vimwiki">some description</a>"#
        );
    }

    #[test]
    fn named_inter_wiki_link_should_support_transclusion_link_description() {
        let link = Link::new_named_interwiki_link(
            "my-wiki",
            URIReference::try_from("some/page").unwrap(),
            Description::try_from_uri_ref_str("some/img.png").unwrap(),
        );

        // Make a config with two wikis so we can refer to the other one
        let mut c = test_vimwiki_config("wiki", "test.wiki");
        add_wiki_with_name(&mut c, "wiki2", "my-wiki");

        let mut f = VimwikiFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.vimwiki"><img src="some/img.png" /></a>"#
        );
    }

    #[test]
    fn diary_link_should_output_a_tag() {
        let link =
            Link::new_diary_link(NaiveDate::from_ymd(2021, 5, 27), None, None);
        let mut f =
            VimwikiFormatter::new(test_vimwiki_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="diary/2021-05-27.vimwiki">diary:2021-05-27</a>"#
        );
    }

    #[test]
    fn diary_link_should_support_text_description() {
        let link = Link::new_diary_link(
            NaiveDate::from_ymd(2021, 5, 27),
            Description::from("some description"),
            None,
        );
        let mut f =
            VimwikiFormatter::new(test_vimwiki_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="diary/2021-05-27.vimwiki">some description</a>"#
        );
    }

    #[test]
    fn diary_link_should_support_transclusion_link_description() {
        let link = Link::new_diary_link(
            NaiveDate::from_ymd(2021, 5, 27),
            Description::try_from_uri_ref_str("some/img.png").unwrap(),
            None,
        );
        let mut f =
            VimwikiFormatter::new(test_vimwiki_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="diary/2021-05-27.vimwiki"><img src="some/img.png" /></a>"#
        );
    }

    #[test]
    fn raw_link_should_output_a_tag() {
        let link = Link::new_raw_link(
            URIReference::try_from("https://example.com").unwrap(),
        );

        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<a href="https://example.com/">https://example.com/</a>"#
        );
    }

    #[test]
    fn transclusion_link_should_output_img_tag() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("https://example.com/img.jpg").unwrap(),
            None,
            None,
        );

        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<img src="https://example.com/img.jpg" />"#
        );
    }

    #[test]
    fn transclusion_link_should_support_local_uris() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("img/pic.png").unwrap(),
            None,
            None,
        );

        let mut f =
            VimwikiFormatter::new(test_vimwiki_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), r#"<img src="img/pic.png" />"#);
    }

    #[test]
    fn transclusion_link_should_use_description_as_alt_text() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("https://example.com/img.jpg").unwrap(),
            Some(Description::from("some description")),
            None,
        );

        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<img src="https://example.com/img.jpg" alt="some description" />"#
        );
    }

    #[test]
    fn transclusion_link_should_support_arbitrary_attrs_on_img() {
        let mut properties: HashMap<Cow<str>, Cow<str>> = HashMap::new();
        properties.insert(Cow::from("key1"), Cow::from("value1"));
        properties.insert(Cow::from("key2"), Cow::from("value2"));

        let link = Link::new_transclusion_link(
            URIReference::try_from("https://example.com/img.jpg").unwrap(),
            Some(Description::from("some description")),
            properties,
        );

        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        // NOTE: The order of properties isn't guaranteed, so we have to check
        //       both possibilities
        let equal1 = f.get_content()
            == r#"<img src="https://example.com/img.jpg" alt="some description" key1="value1" key2="value2" />"#;
        let equal2 = f.get_content()
            == r#"<img src="https://example.com/img.jpg" alt="some description" key2="value2" key1="value1" />"#;
        assert!(equal1 || equal2);
    }

    #[test]
    fn transclusion_link_should_escape_vimwiki() {
        let mut properties: HashMap<Cow<str>, Cow<str>> = HashMap::new();
        properties.insert(Cow::from("key1"), Cow::from("<test>value1</test>"));

        let link = Link::new_transclusion_link(
            URIReference::try_from("https://example.com/img.jpg?a=b&c=d")
                .unwrap(),
            Some(Description::from("<test>some description</test>")),
            properties,
        );

        let mut f = VimwikiFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            r#"<img src="https://example.com/img.jpg?a=b&c=d" alt="&lt;test&gt;some description&lt;/test&gt;" key1="&lt;test&gt;value1&lt;/test&gt;" />"#
        );
    }

    #[test]
    fn tags_should_output_span_per_tag() {
        let tags: Tags = vec!["one", "two"].into_iter().collect();
        let mut f = VimwikiFormatter::default();
        tags.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                r#"<span class="tag" id="one">one</span>"#,
                r#"<span class="tag" id="two">two</span>"#,
            ]
            .join("")
        );
    }

    #[test]
    fn tags_should_include_extra_span_with_id_comprised_of_previous_headers() {
        let tags: Tags = vec!["one", "two"].into_iter().collect();
        let mut f = VimwikiFormatter::default();
        f.insert_header_text(1, "first-id");
        f.insert_header_text(3, "third-id");

        tags.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), [
            r#"<span id="first-id-third-id-one"></span><span class="tag" id="one">one</span>"#,
            r#"<span id="first-id-third-id-two"></span><span class="tag" id="two">two</span>"#,
        ].join(""));
    }

    #[test]
    fn tags_should_escape_vimwiki() {
        let tags: Tags = vec!["one&", "two>"].into_iter().collect();
        let mut f = VimwikiFormatter::default();
        tags.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                r#"<span class="tag" id="one&amp;">one&amp;</span>"#,
                r#"<span class="tag" id="two&gt;">two&gt;</span>"#,
            ]
            .join("")
        );
    }

    #[test]
    fn tags_should_produce_unique_ids_from_repeated_same_tags() {
        let tags1: Tags = vec!["one", "two"].into_iter().collect();
        let tags2: Tags = vec!["one", "two"].into_iter().collect();

        let mut f = VimwikiFormatter::default();
        tags1.fmt(&mut f).unwrap();
        tags2.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                r#"<span class="tag" id="one">one</span>"#,
                r#"<span class="tag" id="two">two</span>"#,
                r#"<span class="tag" id="one-1">one</span>"#,
                r#"<span class="tag" id="two-1">two</span>"#,
            ]
            .join("")
        );
    }

    #[test]
    fn tags_should_produce_unique_ids_from_repeated_same_tags_with_nested_headers(
    ) {
        let tags1: Tags = vec!["one", "two"].into_iter().collect();
        let tags2: Tags = vec!["one", "two"].into_iter().collect();
        let tags3: Tags = vec!["one", "two"].into_iter().collect();

        let mut f = VimwikiFormatter::default();
        f.insert_header_text(1, "a");
        f.insert_header_text(2, "b");
        tags1.fmt(&mut f).unwrap();
        tags2.fmt(&mut f).unwrap();

        f.insert_header_text(2, "c");
        tags3.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            [
                r#"<span id="a-b-one"></span><span class="tag" id="one">one</span>"#,
                r#"<span id="a-b-two"></span><span class="tag" id="two">two</span>"#,
                r#"<span id="a-b-one-1"></span><span class="tag" id="one-1">one</span>"#,
                r#"<span id="a-b-two-1"></span><span class="tag" id="two-1">two</span>"#,
                r#"<span id="a-c-one"></span><span class="tag" id="one-2">one</span>"#,
                r#"<span id="a-c-two"></span><span class="tag" id="two-2">two</span>"#,
            ]
            .join("")
        );
    }

    #[test]
    fn code_inline_should_output_code_tag() {
        let code_inline = CodeInline::from("some code");
        let mut f = VimwikiFormatter::default();
        code_inline.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), "<code>some code</code>");
    }

    #[test]
    fn code_inline_should_escape_vimwiki() {
        let code_inline = CodeInline::from("<test>some code</test>");
        let mut f = VimwikiFormatter::default();
        code_inline.fmt(&mut f).unwrap();

        assert_eq!(
            f.get_content(),
            "<code>&lt;test&gt;some code&lt;/test&gt;</code>"
        );
    }

    #[test]
    fn math_inline_should_output_a_mathjax_notation() {
        let math_inline = MathInline::from("some math");
        let mut f = VimwikiFormatter::default();
        math_inline.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), r"\(some math\)");
    }

    #[test]
    fn math_inline_should_escape_vimwiki() {
        let math_inline = MathInline::from("<test>some math</test>");
        let mut f = VimwikiFormatter::default();
        math_inline.fmt(&mut f).unwrap();

        assert_eq!(f.get_content(), r"\(&lt;test&gt;some math&lt;/test&gt;\)");
    }

    #[test]
    fn comment_should_output_tag_based_on_inner_element() {
        let comment = Comment::from(LineComment::from("some comment"));
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            comment: VimwikiCommentConfig { include: true },
            ..Default::default()
        });
        comment.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "<!-- some comment -->");

        let comment = Comment::from(MultiLineComment::new(vec![
            Cow::Borrowed("some comment"),
            Cow::Borrowed("on multiple lines"),
        ]));
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            comment: VimwikiCommentConfig { include: true },
            ..Default::default()
        });
        comment.fmt(&mut f).unwrap();
        assert_eq!(
            f.get_content(),
            "<!--\nsome comment\non multiple lines\n-->"
        );
    }

    #[test]
    fn line_comment_should_output_vimwiki_comment_if_flagged() {
        let comment = LineComment::from("some comment");

        // By default, no comment will be output
        let mut f = VimwikiFormatter::default();
        comment.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "");

        // If configured to output comments, should use vimwiki syntax
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            comment: VimwikiCommentConfig { include: true },
            ..Default::default()
        });
        comment.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "<!-- some comment -->");
    }

    #[test]
    fn multi_line_comment_should_output_vimwiki_comment_if_flagged() {
        let comment = MultiLineComment::new(vec![
            Cow::Borrowed("some comment"),
            Cow::Borrowed("on multiple lines"),
        ]);

        // By default, no comment will be output
        let mut f = VimwikiFormatter::default();
        comment.fmt(&mut f).unwrap();
        assert_eq!(f.get_content(), "");

        // If configured to output comments, should use vimwiki syntax
        let mut f = VimwikiFormatter::new(VimwikiConfig {
            comment: VimwikiCommentConfig { include: true },
            ..Default::default()
        });
        comment.fmt(&mut f).unwrap();
        assert_eq!(
            f.get_content(),
            "<!--\nsome comment\non multiple lines\n-->"
        );
    }
}
