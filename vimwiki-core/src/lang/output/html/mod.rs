mod config;
pub use config::*;

mod formatter;
pub use formatter::HtmlFormatter;

mod convert;
pub use convert::{ToHtmlPage, ToHtmlString};

mod error;
pub use error::{HtmlOutputError, HtmlOutputResult};

mod utils;
pub use utils::LinkResolutionError;

use crate::lang::{
    elements::*,
    output::{Output, OutputFormatter},
};
use lazy_static::lazy_static;
use std::{borrow::Cow, collections::HashMap, fmt::Write};
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    html::{self, IncludeBackground},
    parsing::SyntaxSet,
};
use uriparse::URIReference;
use voca_rs::escape;

lazy_static! {
    /// Default syntax set for languages
    static ref DEFAULT_SYNTAX_SET: SyntaxSet =
        SyntaxSet::load_defaults_nonewlines();

    /// Default theme highlight set for languages
    static ref DEFAULT_THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

impl<'a> Output<HtmlFormatter> for Page<'a> {
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        for element in self.elements.iter() {
            element.fmt(f)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for Element<'a> {
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        match self {
            Self::Block(x) => x.fmt(f),
            Self::Inline(x) => x.fmt(f),
            Self::InlineBlock(x) => x.fmt(f),
        }
    }
}

impl<'a> Output<HtmlFormatter> for InlineBlockElement<'a> {
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        match self {
            Self::ListItem(x) => x.fmt(f),
            Self::Term(x) => x.fmt(f),
            Self::Definition(x) => x.fmt(f),
        }
    }
}

impl<'a> Output<HtmlFormatter> for BlockElement<'a> {
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        match self {
            Self::Blockquote(x) => x.fmt(f),
            Self::DefinitionList(x) => x.fmt(f),
            Self::Divider(x) => x.fmt(f),
            Self::Header(x) => x.fmt(f),
            Self::List(x) => x.fmt(f),
            Self::MathBlock(x) => x.fmt(f),
            Self::Paragraph(x) => x.fmt(f),
            Self::Placeholder(x) => x.fmt(f),
            Self::CodeBlock(x) => x.fmt(f),
            Self::Table(x) => x.fmt(f),
        }
    }
}

impl<'a> Output<HtmlFormatter> for Blockquote<'a> {
    /// Writes a blockquote in HTML
    ///
    /// ### Example
    ///
    /// ```html
    /// <blockquote>
    ///     <p>First line in blockquote</p>
    ///     <p>Second line in blockquote</p>
    /// </blockquote>
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        writeln!(f, "<blockquote>")?;

        // If we have more than one group of lines, then we want a paragraph
        // wrapping each group
        if self.line_groups().count() > 1 {
            for lines in self.line_groups() {
                writeln!(
                    f,
                    "<p>{}</p>",
                    escape::escape_html(
                        lines
                            .iter()
                            .map(|line| line.trim())
                            .collect::<Vec<&str>>()
                            .join(" ")
                            .as_str()
                    )
                )?;
            }

        // Otherwise, we want to just drop in the lines verbatim
        } else {
            for line in self {
                writeln!(f, "{}", escape::escape_html(line))?;
            }
        }

        write!(f, "</blockquote>")?;
        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for DefinitionList<'a> {
    /// Writes a definition list in HTML
    ///
    /// ### Example
    ///
    /// ```html
    /// <dl>
    ///     <dt>Term 1</dt>
    ///     <dd>First definition</dd>
    ///     <dd>Second definition</dd>
    ///
    ///     <dt>Term 2</dt>
    ///     <dd>Another definition</dd>
    /// </dl>
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        writeln!(f, "<dl>")?;
        for bundle in self {
            // Write our term in the form <dt>{term}</dt>
            write!(f, "<dt>")?;
            bundle.term.fmt(f)?;
            writeln!(f, "</dt>")?;

            // Write our defs in the form <dd>{def}</dd>
            for def in bundle.definitions.iter() {
                write!(f, "<dd>")?;
                def.fmt(f)?;
                writeln!(f, "</dd>")?;
            }
        }
        write!(f, "</dl>")?;
        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for DefinitionListValue<'a> {
    /// Writes a definition list value in HTML
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        self.as_inner().fmt(f)
    }
}

impl Output<HtmlFormatter> for Divider {
    /// Writes a divider in HTML
    ///
    /// ```html
    /// <hr />
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        write!(f, "<hr />")?;
        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for Header<'a> {
    /// Writes a header in HTML
    ///
    /// ### Standard header
    ///
    /// ```html
    /// <div id="{first level text}-{second level text}-{third level text}">
    ///     <h3 id="{third level text}" class="header">
    ///         <a href="#{id-from-above-div}" class="justcenter">
    ///             <!-- third level header text -->
    ///         </a>
    ///     </h3>
    /// </div>
    /// ```
    ///
    /// ### Table of Contents
    ///
    /// ```html
    /// <div id="{toc text}">
    ///     <h3 id="{toc text}" class="header">
    ///         <!-- toc header text -->
    ///     </h3>
    /// </div>
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        let raw_content = self.content.to_string();
        let header_id = utils::normalize_id(&raw_content);
        let unique_header_id = f.ensure_unique_id(&header_id);
        f.insert_header_text(self.level, header_id.clone());

        // ToC is only available for a top-level header
        let is_toc = self.level == 1
            && raw_content.trim() == f.config().header.table_of_contents;
        if is_toc {
            write!(f, r#"<div class="toc">"#)?;
            write!(f, r#"<h{} id="{}">"#, self.level, unique_header_id)?;
            self.content.fmt(f)?;
            write!(f, "</h{}></div>", self.level)?;
        } else {
            // Build our full id using each of the most recent header's
            // contents (earlier levels) up to and including the current header
            let complete_header_id =
                build_complete_id(f, self.level, &header_id)?;

            let has_different_complete_id = complete_header_id != header_id;
            let unique_complete_header_id = if has_different_complete_id {
                Cow::Owned(f.ensure_unique_id(&complete_header_id))
            } else {
                Cow::Borrowed(&unique_header_id)
            };

            // If we have a nested header, then we need to provide a div
            // that has a full id to it alongside the existing single unique
            // id
            if has_different_complete_id {
                write!(f, r#"<div id="{}">"#, unique_complete_header_id)?;
            }

            write!(
                f,
                r#"<h{} id="{}" class="header">"#,
                self.level, unique_header_id
            )?;

            // NOTE: It's fine to use the unique complete header here as if
            //       we are a top-level header then this would be the same
            //       as unique_header_id
            write!(f, r##"<a href="#{}">"##, unique_complete_header_id)?;
            self.content.fmt(f)?;
            write!(f, "</a></h{}>", self.level)?;

            // If we have a nested header, then we produced a div with a
            // complete id and we need to close it
            if has_different_complete_id {
                write!(f, "</div>")?;
            }
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for List<'a> {
    /// Writes a list in HTML
    ///
    /// ### Unordered list
    ///
    /// ```html
    /// <ul>
    ///     <li>...</li>
    ///     <li>...</li>
    /// </ul>
    /// ```
    ///
    /// ### Ordered list
    ///
    /// ```html
    /// <ol>
    ///     <li>...</li>
    ///     <li>...</li>
    /// </ol>
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        // TODO: This should be used for list items... how?
        let _ignore_newlines = f.config().list.ignore_newline;

        // If the list is ordered, we use an ordered HTML list
        if self.is_ordered() {
            writeln!(f, "<ol>")?;

        // Otherwise, if the list is unordered (or has nothing) we use
        // an unordered HTML list
        } else {
            writeln!(f, "<ul>")?;
        }

        for item in self {
            item.fmt(f)?;
            writeln!(f)?;
        }

        if self.is_ordered() {
            write!(f, "</ol>")?;
        } else {
            write!(f, "</ul>")?;
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for ListItem<'a> {
    /// Writes a list item in HTML
    ///
    /// ### Plain item
    ///
    /// ```html
    /// <li>...</li>
    /// ```
    ///
    /// ### Incomplete todo item
    ///
    /// ```html
    /// <li class="done0">...</li>
    /// ```
    ///
    /// ### Partially completed todo items
    ///
    /// ```html
    /// <li class="done1">...</li>
    /// <li class="done2">...</li>
    /// <li class="done3">...</li>
    /// ```
    ///
    /// ### Completed todo item
    ///
    /// ```html
    /// <li class="done4">...</li>
    /// ```
    ///
    /// ### Rejected todo item
    ///
    /// ```html
    /// <li class="rejected">...</li>
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        // TODO: This should be used for list items... how?
        let _ignore_newlines = f.config().list.ignore_newline;

        // First, figure out what class we should be using
        let todo_class = if self.is_todo_incomplete() {
            "done0"
        } else if self.is_todo_partially_complete_1() {
            "done1"
        } else if self.is_todo_partially_complete_2() {
            "done2"
        } else if self.is_todo_partially_complete_3() {
            "done3"
        } else if self.is_todo_complete() {
            "done4"
        } else if self.is_todo_rejected() {
            "rejected"
        } else {
            ""
        };

        // Second, construct the list item
        if !todo_class.is_empty() {
            write!(f, r#"<li class="{}">"#, todo_class)?;
        } else {
            write!(f, "<li>")?;
        }

        self.contents.fmt(f)?;

        write!(f, "</li>")?;

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for ListItemContents<'a> {
    /// Writes a list item's contents in HTML
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        for content in self {
            content.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for MathBlock<'a> {
    /// Writes a math block in HTML
    ///
    /// This leverages MathJAX to transform the dom, and MathJAX expects
    /// block-level math to look like the following:
    ///
    /// ```html
    /// \[some math enclosed in block notation\]
    /// ```
    ///
    /// ### With environment
    ///
    /// ```html
    /// \begin{environment}
    /// some math enclosed in block notation
    /// \end{environment}
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        if let Some(env) = self.environment.as_deref() {
            writeln!(f, r"\begin{{{}}}", env)?;
            for line in self {
                writeln!(f, "{}", escape::escape_html(line))?;
            }
            write!(f, r"\end{{{}}}", env)?;
        } else {
            // TODO: vimwiki appears to support a class if it is on the same
            //       line as the start of the math block, which we currently
            //       do not parse. This would be appended to the end of the
            //       starting notation \[<CLASS>
            writeln!(f, r"\[")?;
            for line in self {
                writeln!(f, "{}", escape::escape_html(line))?;
            }
            write!(f, r"\]")?;
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for Placeholder<'a> {
    /// Writes placeholders in HTML
    ///
    /// Note that this doesn't actually do any writing, but instead updates
    /// settings in the formatter with specific details such as a title, date,
    /// or alternative template to use
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        match self {
            Self::Title(x) => f.set_title(x),
            Self::Date(x) => f.set_date(x),
            Self::Template(x) => f.set_template(x.as_ref()),
            _ => {}
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for CodeBlock<'a> {
    /// Writes a code block block in HTML
    ///
    /// ### Client-side
    ///
    /// Supporting browser highlighters written in JavaScript such as
    /// `highlight.js`:
    ///
    /// ```html
    /// <pre>
    ///     <code class="{language}">
    ///         // Rust source
    ///         fn main() {
    ///             println!("Hello World!");
    ///         }
    ///     </code>
    /// </pre>
    /// ```
    ///
    /// ### Server-side
    ///
    /// When supporting CSS classes:
    ///
    /// ```html
    /// <pre class="code">
    ///     <span class="source rust">
    ///         <span class="comment line double-slash rust">
    ///             <span class="punctuation definition comment rust">//</span> Rust source</span>
    ///         ...
    /// </pre>
    /// ```
    ///
    /// When inlining all stylings:
    ///
    /// ```html
    /// <pre style="background-color:#2b303b;">
    ///     <span style="color:#c0c5ce;">// Rust source</span>
    ///     ...
    /// </pre>
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        // If we are told to perform a server-side render of styles, we
        // build out the <pre> tag and then inject a variety of <span> wrapping
        // individual text elements with associated stylings
        if f.config().code.server_side {
            // Load and use the syntax set from the specified directory if
            // given, otherwise use the default syntax set
            let custom_ss = f
                .config()
                .code
                .syntax_dir
                .as_ref()
                .map(SyntaxSet::load_from_folder)
                .transpose()
                .map_err(HtmlOutputError::from)?;
            let ss = custom_ss.as_ref().unwrap_or(&DEFAULT_SYNTAX_SET);

            // Load and use the theme set from the specified directory if
            // given, otherwise use the default theme set
            let custom_ts = f
                .config()
                .code
                .theme_dir
                .as_ref()
                .map(ThemeSet::load_from_folder)
                .transpose()
                .map_err(HtmlOutputError::from)?;
            let ts = custom_ts.as_ref().unwrap_or(&DEFAULT_THEME_SET);

            // Get syntax using language specifier, otherwise use plain text
            let syntax = if let Some(lang) = self.language.as_ref() {
                ss.find_syntax_by_token(lang)
                    .unwrap_or_else(|| ss.find_syntax_plain_text())
            } else {
                ss.find_syntax_plain_text()
            };

            // Load the specified theme, reporting an error if missing
            let theme =
                ts.themes.get(&f.config().code.theme).ok_or_else(|| {
                    HtmlOutputError::ThemeMissing(
                        f.config().code.theme.to_string(),
                    )
                })?;
            let mut h = HighlightLines::new(syntax, theme);

            // NOTE: The function to create the <pre> tag includes a newline
            //       at the end, which is why we use write! instead of writeln!
            write!(f, "{}", html::start_highlighted_html_snippet(theme).0)?;

            // TODO: The preferred way is to iterate with line endings
            //       included, which we don't have. Want to avoid allocating
            //       new strings just to include line endings, so code blocks
            //       may need to be retooled to be just the entire text
            //       including line endings while supporting an iterator over
            //       the lines
            for line in self {
                let regions = h.highlight(line, ss);
                writeln!(
                    f,
                    "{}",
                    html::styled_line_to_highlighted_html(
                        &regions[..],
                        IncludeBackground::No,
                    )
                )?;
            }

            write!(f, "</pre>")?;

        // Otherwise, we produce <pre> and <code class="{lang}"> for use with
        // frontend highlighters like highlight.js
        } else {
            write!(f, "<pre>")?;

            // Build out our <code ...> tag
            {
                write!(f, "<code")?;

                // If provided with a language, fill it in as the class
                if let Some(lang) = self.language.as_ref() {
                    write!(f, r#" class="{}""#, lang)?;
                }

                // For each metadata assignment, treat it as an HTML attribute
                for (attr, value) in self.metadata.iter() {
                    write!(f, r#" {}="{}""#, attr, value)?;
                }

                // NOTE: We do NOT include a newline here because it results
                //       in the output having a newline at the beginning of
                //       the code block
                write!(f, ">")?;
            }

            for (idx, line) in self.lines.iter().enumerate() {
                let is_last_line = idx == self.lines.len() - 1;
                let line = escape::escape_html(line);

                if is_last_line {
                    write!(f, "{}", line)?;
                } else {
                    writeln!(f, "{}", line)?;
                }
            }

            write!(f, "</code></pre>")?;
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for Paragraph<'a> {
    /// Writes a paragraph in HTML
    ///
    /// ### Ignoring newlines
    ///
    /// This will trim lines and join them together using a single space
    ///
    /// ```html
    /// <p>Some paragraph text on multiple lines</p>
    /// ```
    ///
    /// ### Respecting newlines
    ///
    /// This will trim lines and join them together using a <br> tag
    /// to respect line breaks
    ///
    /// ```html
    /// <p>Some paragraph text<br />on multiple lines</p>
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        let ignore_newlines = f.config().paragraph.ignore_newline;
        let is_blank = self.is_blank();

        // TODO: CHIP CHIP CHIP -- need to handle situation where a paragraph
        //       is comprised ONLY of comments inside itself. In that situation,
        //       we don't want to render the <p></p>, but we do want to
        //       attempt to render comments in case the <!-- --> happens
        //
        //       Add to inline element container a method that checks through
        //       all of the children to see if they are comments; then, we
        //       can use this across all lines in a paragraph to do the same
        //
        //       We probably also need to support this for definition lists,
        //       lists, and other places (???) where inline element containers
        //       can exist

        // Only render opening tag if not blank (meaning comprised of more
        // than just comments)
        if !is_blank {
            write!(f, "<p>")?;
        }

        for (idx, line) in self.lines.iter().enumerate() {
            let is_last_line = idx < self.lines.len() - 1;

            for element in line.iter() {
                element.fmt(f)?;
            }

            // If we are not ignoring newlines, then at the end of each line
            // we want to introduce a hard break (except the last line)
            if is_last_line && !ignore_newlines {
                write!(f, "<br />")?;
            // Otherwise, we want to add a space inbetween the lines of the
            // paragraph if it isn't the last one
            } else if is_last_line {
                write!(f, " ")?;
            }
        }

        // Only render closing tag if not blank (meaning comprised of more
        // than just comments)
        if !is_blank {
            write!(f, "</p>")?;
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for Table<'a> {
    /// Writes a table in HTML
    ///
    /// ### Normal
    ///
    /// ```html
    /// <table>
    ///     <tbody>
    ///         <tr>
    ///             <td>Data 1</td>
    ///             <td>Data 2</td>
    ///         </tr>
    ///         <tr>
    ///             <td>Data 3</td>
    ///             <td>Data 4</td>
    ///         </tr>
    ///     </tbody>
    /// </table>
    /// ```
    ///
    /// ### With a header
    ///
    /// ```html
    /// <table>
    ///     <thead>
    ///         <tr>
    ///             <th>Column 1</th>
    ///             <th>Column 2</th>
    ///         </tr>
    ///     </thead>
    ///     <tbody>
    ///         <tr>
    ///             <td>Data 1</td>
    ///             <td>Data 2</td>
    ///         </tr>
    ///         <tr>
    ///             <td>Data 3</td>
    ///             <td>Data 4</td>
    ///         </tr>
    ///     </tbody>
    /// </table>
    /// ```
    ///
    /// ### Centered
    ///
    /// If the table is considered centered, it will add a **center** class:
    ///
    /// ```html
    /// <table class="center">
    ///     <!-- ... -->
    /// </table>
    /// ```
    ///
    /// ### Cell spans
    ///
    /// If `>` or `\/` is used, the cells to the left or above will have
    /// a `rowspan` or `colspan` attribute added:
    ///
    /// ```html
    /// <table>
    ///     <tbody>
    ///         <tr>
    ///             <td rowspan="2">Data 1</td>
    ///             <td rowspan="3" colspan="2">Data 2</td>
    ///             <td colspan="2">Data 3</td>
    ///         </tr>
    ///     </tbody>
    /// </table>
    /// ```
    ///
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        if self.centered {
            writeln!(f, "<table class=\"center\">")?;
        } else {
            writeln!(f, "<table>")?;
        }

        if self.has_header_rows() {
            writeln!(f, "<thead>")?;
            for row in self.header_rows() {
                // Only produce a row if content exists
                if !row.has_content() {
                    continue;
                }

                writeln!(f, "<tr>")?;
                for (pos, cell) in row.zip_with_position() {
                    if let Some(content) = cell.get_content() {
                        write!(f, "<th")?;

                        let rowspan = self.get_cell_rowspan(pos.row, pos.col);
                        if rowspan > 1 {
                            write!(f, " rowspan=\"{}\"", rowspan)?;
                        }

                        let colspan = self.get_cell_colspan(pos.row, pos.col);
                        if colspan > 1 {
                            write!(f, " colspan=\"{}\"", colspan)?;
                        }

                        write!(f, ">")?;
                        content.fmt(f)?;
                        writeln!(f, "</th>")?;
                    }
                }
                writeln!(f, "</tr>")?;
            }
            writeln!(f, "</thead>")?;
        }

        if self.has_body_rows() {
            writeln!(f, "<tbody>")?;
            for row in self.body_rows() {
                // Only produce a row if content exists
                if !row.has_content() {
                    continue;
                }

                writeln!(f, "<tr>")?;
                for (pos, cell) in row.zip_with_position() {
                    if let Some(content) = cell.get_content() {
                        write!(f, "<td")?;

                        let rowspan = self.get_cell_rowspan(pos.row, pos.col);
                        if rowspan > 1 {
                            write!(f, " rowspan=\"{}\"", rowspan)?;
                        }

                        let colspan = self.get_cell_colspan(pos.row, pos.col);
                        if colspan > 1 {
                            write!(f, " colspan=\"{}\"", colspan)?;
                        }

                        write!(f, ">")?;
                        content.fmt(f)?;
                        writeln!(f, "</td>")?;
                    }
                }
                writeln!(f, "</tr>")?;
            }
            writeln!(f, "</tbody>")?;
        }

        write!(f, "</table>")?;

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for InlineElementContainer<'a> {
    /// Writes a collection of inline elements in HTML
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        for element in self {
            element.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for InlineElement<'a> {
    /// Writes an inline element in HTML
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
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

impl<'a> Output<HtmlFormatter> for Text<'a> {
    /// Writes text in HTML, escaping any HTML-specific characters
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        write!(f, "{}", escape::escape_html(self.as_str()))?;
        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for DecoratedText<'a> {
    /// Writes decorated text in HTML
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        // First, we figure out the type of decoration to apply with bold
        // having the most unique situation as it can also act as an anchor
        match self {
            Self::Bold(contents) => {
                // First, build up the isolated id using contents
                let mut id = String::new();
                for content in contents {
                    write!(&mut id, "{}", content)?;
                }
                id = utils::normalize_id(&id);
                let unique_id = f.ensure_unique_id(&id);

                // Second, produce a span in front if we are nested at some
                // level when it comes to previous ids
                if f.max_header_level().is_some() {
                    let complete_id = build_complete_id(
                        f,
                        f.max_header_level().unwrap_or_default() + 1,
                        id.as_str(),
                    )?;
                    let unique_complete_id = f.ensure_unique_id(&complete_id);
                    write!(f, "<span id=\"{}\"></span>", unique_complete_id)?;
                }

                // Third, write out all of the contents inbetween <strong> tag
                // with the strong tag having a unique bold id
                write!(f, "<strong id=\"{}\">", unique_id)?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, "</strong>")?;
            }
            Self::Italic(contents) => {
                write!(f, "<em>")?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, "</em>")?;
            }
            Self::Strikeout(contents) => {
                write!(f, "<del>")?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, "</del>")?;
            }
            Self::Superscript(contents) => {
                write!(f, "<sup><small>")?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, "</small></sup>")?;
            }
            Self::Subscript(contents) => {
                write!(f, "<sub><small>")?;
                for content in contents {
                    content.fmt(f)?;
                }
                write!(f, "</small></sub>")?;
            }
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for DecoratedTextContent<'a> {
    /// Writes decorated text content in HTML
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        match self {
            Self::Text(x) => x.fmt(f),
            Self::DecoratedText(x) => x.fmt(f),
            Self::Keyword(x) => x.fmt(f),
            Self::Link(x) => x.fmt(f),
        }
    }
}

impl Output<HtmlFormatter> for Keyword {
    /// Writes keyword in HTML
    ///
    /// Unable to be implemented via Output<HtmlFormatter> trait as generic associated types
    /// would be required.
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        // For all keywords other than todo, they are treated as plain output
        // for HTML. For todo, it is wrapped in a span with a todo class
        match self {
            Self::Todo => write!(f, "<span class=\"todo\">TODO</span>")?,
            Self::Done => write!(f, "DONE")?,
            Self::Started => write!(f, "STARTED")?,
            Self::Fixme => write!(f, "FIXME")?,
            Self::Fixed => write!(f, "FIXED")?,
            Self::Xxx => write!(f, "XXX")?,
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for Link<'a> {
    /// Writes a link in HTML
    ///
    /// ### Wiki/Interwiki Link
    ///
    /// 1. Plain link
    ///
    ///    For `[[url]]` in vimwiki:
    ///
    ///    ```html
    ///    <a href="url.html">url</a>
    ///    ```
    ///
    /// 2. Link with description
    ///
    ///    For `[[url|descr]]` in vimwiki:
    ///
    ///    ```html
    ///    <a href="url.html">descr</a>
    ///    ```
    ///
    /// 3. Link with embedded image
    ///
    ///    For `[[url|{{...}}]]` in vimwiki:
    ///
    ///    ```html
    ///    <a href="url.html"> ... </a>
    ///    ```
    ///
    /// 4. Link with anchors
    ///
    ///    For `[[url#a1#a2]]` in vimwiki:
    ///
    ///    ```html
    ///    <a href="url.html#a1-a2">url#a1#a2</a>
    ///    ```
    ///
    /// 5. Only anchors
    ///
    ///    For `[[#a1#a2]]` in vimwiki:
    ///
    ///    ```html
    ///    <a href="#a1-a2">#a1#a2</a>
    ///    ```
    ///
    /// ### Diary Link
    ///
    /// For `[[diary:2021-03-05]]` and `[[diary:2021-03-05|description]]`:
    ///
    /// ```html
    /// <a href="diary/2021-03-05.html">diary:2021-03-05</a>
    /// <a href="diary/2021-03-05.html">description</a>
    /// ```
    ///
    /// ### Raw Link
    ///
    /// For `https://example.com`:
    ///
    /// ```html
    /// <a href="https://example.com">https://example.com</a>
    /// ```
    ///
    /// ### Link to file
    ///
    /// For `[[fileurl.ext|descr]]` in vimwiki:
    ///
    /// ```html
    /// <a href="fileurl.ext">descr</a>
    /// ```
    ///
    /// ### Link to directory
    ///
    /// For `[[dirurl/|descr]]` in vimwiki:
    ///
    /// ```html
    /// <a href="dirurl/index.html">descr</a>
    /// ```
    ///
    /// ### Transclusion Link
    ///
    /// For `{{path/to/img.png}}`, `{{path/to/img.png|descr}}`, and
    /// `{{path/to/img.png|descr|style="A"}}`:
    ///
    /// ```html
    /// <img src="path/to/img.png" />
    /// <img src="path/to/img.png" alt="descr" />
    /// <img src="path/to/img.png" alt="descr" style="A" />
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        // Produces a link tag of <a href=".." ...>link/description</a>
        // based on the link data and a given base url representing the root
        // of the wiki if needed
        fn write_link(
            f: &mut HtmlFormatter,
            href: &URIReference<'_>,
            description: Option<&Description>,
            properties: Option<&HashMap<Cow<'_, str>, Cow<'_, str>>>,
            use_img_tag: bool,
        ) -> HtmlOutputResult {
            if use_img_tag {
                write!(f, "<img src=\"{}\"", href)?;

                if let Some(desc) = description {
                    write!(
                        f,
                        " alt=\"{}\"",
                        escape::escape_html(desc.to_string().as_str())
                    )?;
                }

                if let Some(properties) = properties {
                    for (k, v) in properties.iter() {
                        write!(f, " {}=\"{}\"", k, escape::escape_html(v))?;
                    }
                }

                write!(f, " />")?;
            } else {
                write!(f, "<a href=\"{}\"", href)?;

                if let Some(properties) = properties {
                    for (k, v) in properties.iter() {
                        write!(f, " {}=\"{}\"", k, escape::escape_html(v))?;
                    }
                }

                write!(f, ">")?;

                match description {
                    Some(Description::Text(x)) => {
                        write!(f, "{}", escape::escape_html(x))?
                    }

                    // TODO: Figure out more optimal way to perform nested
                    //       transclusion that needs to resolve the link
                    //       within it
                    Some(Description::TransclusionLink(data)) => {
                        Link::Transclusion {
                            data: *data.clone(),
                        }
                        .fmt(f)?
                    }
                    None => write!(f, "{}", href)?,
                }

                write!(f, "</a>")?;
            }

            Ok(())
        }

        let uri_ref = utils::resolve_link(
            f.config(),
            &f.config().to_current_wiki(),
            f.config().as_active_page_path_within_wiki(),
            self,
        )
        .map_err(HtmlOutputError::from)?;

        write_link(
            f,
            &uri_ref,
            self.to_description_or_fallback().as_ref(),
            self.properties(),
            matches!(self, Self::Transclusion { .. }),
        )
    }
}

impl<'a> Output<HtmlFormatter> for Tags<'a> {
    /// Writes tags in HTML
    ///
    /// ### Example
    ///
    /// If placed after a header called *Header 1*, the tag will inject a span
    /// in front of itself that acts as an anchor to itself:
    ///
    /// ```html
    /// <span id="Header 1-tag1"></span><span class="tag" id="tag1">tag1</span>
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        for tag in self {
            let id = utils::normalize_id(tag.as_str());
            let unique_id = f.ensure_unique_id(&id);

            // Only produce a span in front if we are nested at some level
            // when it comes to previous ids
            if f.max_header_level().is_some() {
                let complete_id = build_complete_id(
                    f,
                    f.max_header_level().unwrap_or_default() + 1,
                    id.as_str(),
                )?;
                let unique_complete_id = f.ensure_unique_id(&complete_id);
                write!(f, "<span id=\"{}\"></span>", unique_complete_id)?;
            }

            write!(
                f,
                "<span class=\"tag\" id=\"{}\">{}</span>",
                unique_id, id
            )?;
        }

        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for CodeInline<'a> {
    /// Writes inline code in HTML
    ///
    /// ### Example
    ///
    /// ```html
    /// <code>some code</code>
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        write!(f, "<code>{}</code>", escape::escape_html(self.as_str()))?;
        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for MathInline<'a> {
    /// Writes inline math in HTML
    ///
    /// ### Example
    ///
    /// ```html
    /// \(some math\)
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        write!(f, r"\({}\)", escape::escape_html(self.as_str()))?;
        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for Comment<'a> {
    /// Writes a comment in HTML
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        match self {
            Self::Line(x) => x.fmt(f),
            Self::MultiLine(x) => x.fmt(f),
        }
    }
}

impl<'a> Output<HtmlFormatter> for LineComment<'a> {
    /// Writes a line comment in HTML
    ///
    /// ### Example
    ///
    /// If `config.comment.include` is true, will output the following:
    ///
    /// ```html
    /// <!-- {line} -->
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        if f.config().comment.include {
            write!(f, "<!-- {} -->", self.as_str())?;
        }
        Ok(())
    }
}

impl<'a> Output<HtmlFormatter> for MultiLineComment<'a> {
    /// Writes a multiline comment in HTML
    ///
    /// ### Example
    ///
    /// If `config.comment.include` is true, will output the following:
    ///
    /// ```html
    /// <!--
    /// {line1}
    /// {line2}
    /// ...
    /// {lineN}
    /// -->
    /// ```
    fn fmt(&self, f: &mut HtmlFormatter) -> HtmlOutputResult {
        if f.config().comment.include {
            writeln!(f, "<!--")?;
            for line in self {
                writeln!(f, "{}", line)?;
            }
            write!(f, "-->")?;
        }
        Ok(())
    }
}

fn build_complete_id(
    f: &mut HtmlFormatter,
    max_level: usize,
    id: &str,
) -> Result<String, HtmlOutputError> {
    let mut complete_id = String::new();

    // Add all of the header text up to (not including) the level specified
    // to form the complete id
    for i in 1..max_level {
        if let Some(id) = f.get_header_text(i) {
            write!(&mut complete_id, "{}-", id)?;
        }
    }
    write!(&mut complete_id, "{}", id)?;

    Ok(complete_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use indoc::indoc;
    use similar_asserts::{assert_eq, assert_str_eq};
    use std::{
        borrow::Cow,
        collections::HashMap,
        convert::TryFrom,
        path::{Path, PathBuf},
    };
    use uriparse::URIReference;

    /// Produces an html config with a singular wiki for some test page
    /// provided
    fn test_html_config<P1: AsRef<Path>, P2: AsRef<Path>>(
        wiki: P1,
        page: P2,
    ) -> HtmlConfig {
        let wiki = wiki.as_ref().to_string_lossy();
        let page = page.as_ref().to_string_lossy();
        HtmlConfig {
            wikis: vec![HtmlWikiConfig {
                path: make_path_from_pieces(vec!["wiki", wiki.as_ref()]),
                path_html: make_path_from_pieces(vec!["html", wiki.as_ref()]),
                ..Default::default()
            }],
            runtime: HtmlRuntimeConfig {
                wiki_index: Some(0),
                page: make_path_from_pieces(vec![
                    "wiki",
                    wiki.as_ref(),
                    page.as_ref(),
                ]),
            },
            ..Default::default()
        }
    }

    /// Adds a wiki to the config for interwiki testing
    fn add_wiki<P: AsRef<Path>>(c: &mut HtmlConfig, wiki: P) {
        let wiki = wiki.as_ref().to_string_lossy();
        c.wikis.push(HtmlWikiConfig {
            path: make_path_from_pieces(vec!["wiki", wiki.as_ref()]),
            path_html: make_path_from_pieces(vec!["html", wiki.as_ref()]),
            ..Default::default()
        });
    }

    /// Adds a wiki to the config for interwiki testing
    fn add_wiki_with_name<P: AsRef<Path>, N: AsRef<str>>(
        c: &mut HtmlConfig,
        wiki: P,
        name: N,
    ) {
        let wiki = wiki.as_ref().to_string_lossy();
        c.wikis.push(HtmlWikiConfig {
            path: make_path_from_pieces(vec!["wiki", wiki.as_ref()]),
            path_html: make_path_from_pieces(vec!["html", wiki.as_ref()]),
            name: Some(name.as_ref().to_string()),
            ..Default::default()
        });
    }

    fn make_path_from_pieces<'a, I: IntoIterator<Item = &'a str>>(
        iter: I,
    ) -> PathBuf {
        let rel_path: PathBuf = iter.into_iter().collect();
        std::path::Path::new(&std::path::Component::RootDir)
            .join(rel_path.as_path())
    }

    fn text_to_inline_element_container(s: &str) -> InlineElementContainer {
        InlineElementContainer::new(vec![Located::from(InlineElement::Text(
            Text::from(s),
        ))])
    }

    #[test]
    fn page_should_add_a_newline_after_each_block_element_rendered() {
        let page = Page::new(vec![
            Located::from(BlockElement::from(Paragraph::new(vec![
                text_to_inline_element_container("paragraph 1"),
            ]))),
            Located::from(BlockElement::from(Paragraph::new(vec![
                text_to_inline_element_container("paragraph 2"),
            ]))),
        ]);

        let mut f = HtmlFormatter::default();
        page.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            "<p>paragraph 1</p>\n<p>paragraph 2</p>\n"
        );
    }

    #[test]
    fn blockquote_with_multiple_line_groups_should_output_blockquote_tag_with_paragraph_for_each_group_of_lines(
    ) {
        let blockquote = Blockquote::new(vec![
            Cow::from("line1"),
            Cow::from("line2"),
            Cow::from(""),
            Cow::from("line3"),
        ]);
        let mut f = HtmlFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <blockquote>
                <p>line1 line2</p>
                <p>line3</p>
                </blockquote>
            "}
            .trim(),
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
        let mut f = HtmlFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <blockquote>
                line1
                line2
                line3
                </blockquote>
            "}
            .trim(),
        );
    }

    #[test]
    fn blockquote_should_escape_html_in_each_line_of_a_singular_line_group() {
        let blockquote = Blockquote::new(vec![
            Cow::from("<test1>"),
            Cow::from("<test2>"),
            Cow::from("<test3>"),
        ]);
        let mut f = HtmlFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <blockquote>
                &lt;test1&gt;
                &lt;test2&gt;
                &lt;test3&gt;
                </blockquote>
            "}
            .trim(),
        );
    }

    #[test]
    fn blockquote_should_escape_html_in_each_line_of_multiple_line_groups() {
        let blockquote = Blockquote::new(vec![
            Cow::from("<test1>"),
            Cow::from("<test2>"),
            Cow::from(""),
            Cow::from("<test3>"),
        ]);
        let mut f = HtmlFormatter::default();
        blockquote.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <blockquote>
                <p>&lt;test1&gt; &lt;test2&gt;</p>
                <p>&lt;test3&gt;</p>
                </blockquote>
            "}
            .trim(),
        );
    }

    #[test]
    fn definition_list_should_output_list_tag_with_term_and_definition_tags_together(
    ) {
        // Test no definitions
        let list: DefinitionList = vec![(
            Located::from(DefinitionListValue::from("term1")),
            Located::from(DefinitionBundle::empty()),
        )]
        .into_iter()
        .collect();

        let mut f = HtmlFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <dl>
                <dt>term1</dt>
                </dl>
            "}
            .trim(),
        );

        // Test single definition
        let list: DefinitionList = vec![(
            Located::from(DefinitionListValue::from("term1")),
            Located::from(DefinitionBundle::new(vec![Located::from(
                DefinitionListValue::from("def1"),
            )])),
        )]
        .into_iter()
        .collect();

        let mut f = HtmlFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <dl>
                <dt>term1</dt>
                <dd>def1</dd>
                </dl>
            "}
            .trim(),
        );

        // Test multiple definitions
        let list: DefinitionList = vec![(
            Located::from(DefinitionListValue::from("term1")),
            Located::from(DefinitionBundle::new(vec![
                Located::from(DefinitionListValue::from("def1")),
                Located::from(DefinitionListValue::from("def2")),
            ])),
        )]
        .into_iter()
        .collect();

        let mut f = HtmlFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <dl>
                <dt>term1</dt>
                <dd>def1</dd>
                <dd>def2</dd>
                </dl>
            "}
            .trim(),
        );
    }

    #[test]
    fn divider_should_output_hr_tag() {
        let divider = Divider;

        let mut f = HtmlFormatter::default();
        divider.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), "<hr />");
    }

    #[test]
    fn header_should_output_h_and_a_tags() {
        let header = Header::new(
            text_to_inline_element_container("some header"),
            3,
            false,
        );

        let mut f = HtmlFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            [
                "<h3 id=\"some-header\" class=\"header\">",
                "<a href=\"#some-header\">",
                "some header",
                "</a>",
                "</h3>",
            ]
            .join(""),
        );
    }

    #[test]
    fn header_should_support_toc_variant() {
        let text = HtmlHeaderConfig::default_table_of_contents();
        let header =
            Header::new(text_to_inline_element_container(&text), 1, false);

        let mut f = HtmlFormatter::default();
        header.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            [
                "<div class=\"toc\">",
                "<h1 id=\"contents\">",
                "Contents",
                "</h1>",
                "</div>",
            ]
            .join(""),
        );
    }

    #[test]
    fn header_should_escape_html_in_ids() {
        let header =
            Header::new(text_to_inline_element_container("<test>"), 3, false);

        // Configure to use a different table of contents string
        // that has characters that should be escaped
        let mut f = HtmlFormatter::default();

        // Add some header ids prior to this one to verify that they aren't used
        f.insert_header_text(1, "h1");
        f.insert_header_text(2, "h2");

        header.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            [
                "<div id=\"h1-h2-&lt;test&gt;\">",
                "<h3 id=\"&lt;test&gt;\" class=\"header\">",
                "<a href=\"#h1-h2-&lt;test&gt;\">",
                "&lt;test&gt;",
                "</a>",
                "</h3>",
                "</div>",
            ]
            .join(""),
        );
    }

    #[test]
    fn header_should_escape_html_in_ids_for_toc() {
        let header =
            Header::new(text_to_inline_element_container("<test>"), 1, false);

        // Configure to use a different table of contents string
        // that has characters that should be escaped
        let mut f = HtmlFormatter::new(HtmlConfig {
            header: HtmlHeaderConfig {
                table_of_contents: String::from("<test>"),
            },
            ..Default::default()
        });

        header.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            "<div class=\"toc\"><h1 id=\"&lt;test&gt;\">&lt;test&gt;</h1></div>",
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

        let mut f = HtmlFormatter::default();
        header1.fmt(&mut f).unwrap();
        header2.fmt(&mut f).unwrap();
        header3.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            [
                // First header
                "<h3 id=\"some-header\" class=\"header\">",
                "<a href=\"#some-header\">",
                "some header",
                "</a>",
                "</h3>",
                // Second header
                "<h3 id=\"some-header-1\" class=\"header\">",
                "<a href=\"#some-header-1\">",
                "some header",
                "</a>",
                "</h3>",
                // Third header
                "<h3 id=\"some-header-2\" class=\"header\">",
                "<a href=\"#some-header-2\">",
                "some header",
                "</a>",
                "</h3>",
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

        let mut f = HtmlFormatter::default();
        f.insert_header_text(1, "a");
        f.insert_header_text(2, "b");
        header1.fmt(&mut f).unwrap();
        header2.fmt(&mut f).unwrap();

        f.insert_header_text(2, "c");
        header3.fmt(&mut f).unwrap();

        assert_str_eq!(
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
                // Second header
                "<div id=\"a-b-some-header-1\">",
                "<h3 id=\"some-header-1\" class=\"header\">",
                "<a href=\"#a-b-some-header-1\">",
                "some header",
                "</a>",
                "</h3>",
                "</div>",
                // Third header
                "<div id=\"a-c-some-header\">",
                "<h3 id=\"some-header-2\" class=\"header\">",
                "<a href=\"#a-c-some-header\">",
                "some header",
                "</a>",
                "</h3>",
                "</div>",
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
            ListItemContents::new(vec![Located::from(BlockElement::from(
                Paragraph::new(vec![text_to_inline_element_container(
                    "some list item",
                )]),
            ))]),
            ListItemAttributes::default(),
        ))]);
        let mut f = HtmlFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <ol>
                <li><p>some list item</p></li>
                </ol>
            "}
            .trim(),
        );
    }

    #[test]
    fn list_should_output_unordered_list_if_unordered_type() {
        let list = List::new(vec![Located::from(ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(BlockElement::from(
                Paragraph::new(vec![text_to_inline_element_container(
                    "some list item",
                )]),
            ))]),
            ListItemAttributes::default(),
        ))]);
        let mut f = HtmlFormatter::default();
        list.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <ul>
                <li><p>some list item</p></li>
                </ul>
            "}
            .trim(),
        );
    }

    #[test]
    fn list_item_should_output_li_tag() {
        let item = ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(BlockElement::from(
                Paragraph::new(vec![text_to_inline_element_container(
                    "some list item",
                )]),
            ))]),
            ListItemAttributes::default(),
        );
        let mut f = HtmlFormatter::default();
        item.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), "<li><p>some list item</p></li>");
    }

    #[test]
    fn list_item_should_support_adding_class_based_on_todo_status() {
        let mut item = ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(BlockElement::from(
                Paragraph::new(vec![text_to_inline_element_container(
                    "some list item",
                )]),
            ))]),
            ListItemAttributes::default(),
        );

        let mut f = HtmlFormatter::default();
        item.attributes.todo_status = Some(ListItemTodoStatus::Incomplete);
        item.fmt(&mut f).unwrap();
        assert_str_eq!(
            f.get_content(),
            "<li class=\"done0\"><p>some list item</p></li>"
        );

        let mut f = HtmlFormatter::default();
        item.attributes.todo_status =
            Some(ListItemTodoStatus::PartiallyComplete1);
        item.fmt(&mut f).unwrap();
        assert_str_eq!(
            f.get_content(),
            "<li class=\"done1\"><p>some list item</p></li>"
        );

        let mut f = HtmlFormatter::default();
        item.attributes.todo_status =
            Some(ListItemTodoStatus::PartiallyComplete2);
        item.fmt(&mut f).unwrap();
        assert_str_eq!(
            f.get_content(),
            "<li class=\"done2\"><p>some list item</p></li>"
        );

        let mut f = HtmlFormatter::default();
        item.attributes.todo_status =
            Some(ListItemTodoStatus::PartiallyComplete3);
        item.fmt(&mut f).unwrap();
        assert_str_eq!(
            f.get_content(),
            "<li class=\"done3\"><p>some list item</p></li>"
        );

        let mut f = HtmlFormatter::default();
        item.attributes.todo_status = Some(ListItemTodoStatus::Complete);
        item.fmt(&mut f).unwrap();
        assert_str_eq!(
            f.get_content(),
            "<li class=\"done4\"><p>some list item</p></li>"
        );

        let mut f = HtmlFormatter::default();
        item.attributes.todo_status = Some(ListItemTodoStatus::Rejected);
        item.fmt(&mut f).unwrap();
        assert_str_eq!(
            f.get_content(),
            "<li class=\"rejected\"><p>some list item</p></li>"
        );
    }

    #[test]
    fn math_block_should_output_a_mathjax_notation() {
        let math = MathBlock::from_lines(vec!["some lines", "of math"]);
        let mut f = HtmlFormatter::default();
        math.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {r"
                \[
                some lines
                of math
                \]
            "}
            .trim(),
        );
    }

    #[test]
    fn math_block_should_support_environments() {
        let math = MathBlock::new(
            vec![Cow::from("some lines"), Cow::from("of math")],
            Some(Cow::from("test environment")),
        );
        let mut f = HtmlFormatter::default();
        math.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {r"
                \begin{test environment}
                some lines
                of math
                \end{test environment}
            "}
            .trim(),
        );
    }

    #[test]
    fn placeholder_should_set_title_if_specified() {
        let placeholder = Placeholder::title_from_str("test title");
        let mut f = HtmlFormatter::default();
        placeholder.fmt(&mut f).unwrap();

        assert_eq!(f.get_title(), Some("test title"));
    }

    #[test]
    fn placeholder_should_set_date_if_specified() {
        let placeholder = Placeholder::Date(NaiveDate::from_ymd(2021, 4, 27));
        let mut f = HtmlFormatter::default();
        placeholder.fmt(&mut f).unwrap();

        assert_eq!(f.get_date(), Some(&NaiveDate::from_ymd(2021, 4, 27)));
    }

    #[test]
    fn placeholder_should_set_template_if_specified() {
        let placeholder = Placeholder::template_from_str("template file");
        let mut f = HtmlFormatter::default();
        placeholder.fmt(&mut f).unwrap();

        assert_eq!(f.get_template(), Some(Path::new("template file")));
    }

    #[test]
    fn code_block_should_output_pre_code_tags_for_clientside_render() {
        let code = CodeBlock::from_lines(vec!["some lines", "of code"]);
        let mut f = HtmlFormatter::default();
        code.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {"
                <pre><code>some lines
                of code</code></pre>
            "}
            .trim(),
        );
    }

    #[test]
    fn code_block_should_escape_output_clientside() {
        let code = CodeBlock::from_lines(vec!["<test>"]);
        let mut f = HtmlFormatter::default();
        code.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r"<pre><code>&lt;test&gt;</code></pre>",
        );
    }

    #[test]
    fn code_block_should_support_serverside_render() {
        let code = CodeBlock::new(
            Some(Cow::from("rust")),
            HashMap::new(),
            vec![Cow::from("fn my_func() -> String { String::new() }")],
        );
        let mut f = HtmlFormatter::new(HtmlConfig {
            code: HtmlCodeConfig {
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
            "</pre>",
        ]
        .join("");

        assert_str_eq!(f.get_content(), expected);
    }

    #[test]
    fn code_block_should_support_serverside_render_with_no_language() {
        let code = CodeBlock::from_lines(vec!["some lines", "of code"]);
        let mut f = HtmlFormatter::new(HtmlConfig {
            code: HtmlCodeConfig {
                server_side: true,
                ..Default::default()
            },
            ..Default::default()
        });
        code.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {r#"
                <pre style="background-color:#ffffff;">
                <span style="color:#323232;">some lines</span>
                <span style="color:#323232;">of code</span>
                </pre>
            "#}
            .trim()
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
        let mut f = HtmlFormatter::default();
        paragraph.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), "<p>some text and more text</p>");
    }

    #[test]
    fn paragraph_should_support_linebreaks_if_configured() {
        let paragraph = Paragraph::new(vec![
            text_to_inline_element_container("some text"),
            text_to_inline_element_container("and more text"),
        ]);
        let mut f = HtmlFormatter::new(HtmlConfig {
            paragraph: HtmlParagraphConfig {
                ignore_newline: false,
            },
            ..Default::default()
        });
        paragraph.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), "<p>some text<br />and more text</p>");
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
        let mut f = HtmlFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_str_eq!(
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
            "}
            .trim(),
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
        let mut f = HtmlFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {r#"
                <table>
                <thead>
                <tr>
                <th rowspan="2">some text</th>
                </tr>
                </thead>
                </table>
            "#}
            .trim(),
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
        let mut f = HtmlFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {r#"
                <table>
                <thead>
                <tr>
                <th colspan="2">some text</th>
                </tr>
                </thead>
                </table>
            "#}
            .trim(),
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
        let mut f = HtmlFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {r#"
                <table>
                <tbody>
                <tr>
                <td rowspan="2">some text</td>
                </tr>
                </tbody>
                </table>
            "#}
            .trim(),
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
        let mut f = HtmlFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            indoc! {r#"
                <table>
                <tbody>
                <tr>
                <td colspan="2">some text</td>
                </tr>
                </tbody>
                </table>
            "#}
            .trim(),
        );
    }

    #[test]
    fn table_should_support_being_centered() {
        let table = Table::new(Vec::new(), true);
        let mut f = HtmlFormatter::default();
        table.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), "<table class=\"center\">\n</table>");
    }

    #[test]
    fn text_should_output_inner_str() {
        let text = Text::from("some text");
        let mut f = HtmlFormatter::default();
        text.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), "some text");
    }

    #[test]
    fn text_should_escape_html() {
        let text = Text::from("<some>html</some>");
        let mut f = HtmlFormatter::default();
        text.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), r"&lt;some&gt;html&lt;/some&gt;");
    }

    #[test]
    fn decorated_text_should_output_strong_tag_for_bold_text() {
        let decorated_text = DecoratedText::Bold(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = HtmlFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_str_eq!(
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
        let mut f = HtmlFormatter::default();
        f.insert_header_text(1, "one");
        f.insert_header_text(2, "two");
        f.insert_header_text(3, "three");
        decorated_text.fmt(&mut f).unwrap();

        assert_str_eq!(
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
        let mut f = HtmlFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_str_eq!(
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

        let mut f = HtmlFormatter::default();
        bold1.fmt(&mut f).unwrap();
        bold2.fmt(&mut f).unwrap();

        assert_str_eq!(
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

        let mut f = HtmlFormatter::default();
        f.insert_header_text(1, "a");
        f.insert_header_text(2, "b");
        bold1.fmt(&mut f).unwrap();
        bold2.fmt(&mut f).unwrap();

        f.insert_header_text(2, "c");
        bold3.fmt(&mut f).unwrap();

        assert_str_eq!(
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
        let mut f = HtmlFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), r#"<em>some text</em>"#);
    }

    #[test]
    fn decorated_text_should_output_del_tag_for_strikeout_text() {
        let decorated_text = DecoratedText::Strikeout(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = HtmlFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), r#"<del>some text</del>"#);
    }

    #[test]
    fn decorated_text_should_output_sup_tag_for_superscript_text() {
        let decorated_text = DecoratedText::Superscript(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = HtmlFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<sup><small>some text</small></sup>"#
        );
    }

    #[test]
    fn decorated_text_should_output_sub_tag_for_subscript_text() {
        let decorated_text = DecoratedText::Subscript(vec![Located::from(
            DecoratedTextContent::Text(Text::from("some text")),
        )]);
        let mut f = HtmlFormatter::default();
        decorated_text.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<sub><small>some text</small></sub>"#
        );
    }

    #[test]
    fn keyword_should_output_span_with_class_for_todo() {
        let keyword = Keyword::Todo;

        let mut f = HtmlFormatter::default();
        keyword.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), r#"<span class="todo">TODO</span>"#);
    }

    #[test]
    fn keyword_should_output_self_in_all_caps() {
        let keyword = Keyword::Done;

        let mut f = HtmlFormatter::default();
        keyword.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), "DONE");
    }

    #[test]
    fn wiki_link_should_output_a_tag() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page").unwrap(),
            None,
        );
        let mut f = HtmlFormatter::new(test_html_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="some/page.html">some/page</a>"#
        );
    }

    #[test]
    fn wiki_link_should_support_anchors() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page#some-anchor").unwrap(),
            None,
        );
        let mut f = HtmlFormatter::new(test_html_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="some/page.html#some-anchor">some/page#some-anchor</a>"#
        );
    }

    #[test]
    fn wiki_link_should_support_standalone_anchors() {
        let link = Link::new_wiki_link(
            URIReference::try_from("#some-anchor").unwrap(),
            None,
        );
        let mut f = HtmlFormatter::new(test_html_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r##"<a href="#some-anchor">#some-anchor</a>"##
        );
    }

    #[test]
    fn wiki_link_should_support_text_description() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page").unwrap(),
            Description::from("some description"),
        );
        let mut f = HtmlFormatter::new(test_html_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="some/page.html">some description</a>"#
        );
    }

    #[test]
    fn wiki_link_should_support_transclusion_link_description() {
        let link = Link::new_wiki_link(
            URIReference::try_from("some/page").unwrap(),
            Description::try_from_uri_ref_str("some/img.png").unwrap(),
        );
        let mut f = HtmlFormatter::new(test_html_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="some/page.html"><img src="some/img.png" /></a>"#
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
        let mut c = test_html_config("wiki", "test.wiki");
        add_wiki(&mut c, "wiki2");

        let mut f = HtmlFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.html">some/page</a>"#
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
        let mut c = test_html_config("wiki", "test.wiki");
        add_wiki(&mut c, "wiki2");

        let mut f = HtmlFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.html#some-anchor">some/page#some-anchor</a>"#
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
        let mut c = test_html_config("wiki", "test.wiki");
        add_wiki(&mut c, "wiki2");

        let mut f = HtmlFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.html">some description</a>"#
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
        let mut c = test_html_config("wiki", "test.wiki");
        add_wiki(&mut c, "wiki2");

        let mut f = HtmlFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.html"><img src="some/img.png" /></a>"#
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
        let mut c = test_html_config("wiki", "test.wiki");
        add_wiki_with_name(&mut c, "wiki2", "my-wiki");

        let mut f = HtmlFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.html">some/page</a>"#
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
        let mut c = test_html_config("wiki", "test.wiki");
        add_wiki_with_name(&mut c, "wiki2", "my-wiki");

        let mut f = HtmlFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.html#some-anchor">some/page#some-anchor</a>"#
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
        let mut c = test_html_config("wiki", "test.wiki");
        add_wiki_with_name(&mut c, "wiki2", "my-wiki");

        let mut f = HtmlFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.html">some description</a>"#
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
        let mut c = test_html_config("wiki", "test.wiki");
        add_wiki_with_name(&mut c, "wiki2", "my-wiki");

        let mut f = HtmlFormatter::new(c);
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="../wiki2/some/page.html"><img src="some/img.png" /></a>"#
        );
    }

    #[test]
    fn diary_link_should_output_a_tag() {
        let link =
            Link::new_diary_link(NaiveDate::from_ymd(2021, 5, 27), None, None);
        let mut f = HtmlFormatter::new(test_html_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="diary/2021-05-27.html">diary:2021-05-27</a>"#
        );
    }

    #[test]
    fn diary_link_should_support_text_description() {
        let link = Link::new_diary_link(
            NaiveDate::from_ymd(2021, 5, 27),
            Description::from("some description"),
            None,
        );
        let mut f = HtmlFormatter::new(test_html_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="diary/2021-05-27.html">some description</a>"#
        );
    }

    #[test]
    fn diary_link_should_support_transclusion_link_description() {
        let link = Link::new_diary_link(
            NaiveDate::from_ymd(2021, 5, 27),
            Description::try_from_uri_ref_str("some/img.png").unwrap(),
            None,
        );
        let mut f = HtmlFormatter::new(test_html_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<a href="diary/2021-05-27.html"><img src="some/img.png" /></a>"#
        );
    }

    #[test]
    fn raw_link_should_output_a_tag() {
        let link = Link::new_raw_link(
            URIReference::try_from("https://example.com").unwrap(),
        );

        let mut f = HtmlFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
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

        let mut f = HtmlFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
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

        let mut f = HtmlFormatter::new(test_html_config("wiki", "test.wiki"));
        link.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), r#"<img src="img/pic.png" />"#);
    }

    #[test]
    fn transclusion_link_should_use_description_as_alt_text() {
        let link = Link::new_transclusion_link(
            URIReference::try_from("https://example.com/img.jpg").unwrap(),
            Some(Description::from("some description")),
            None,
        );

        let mut f = HtmlFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
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

        let mut f = HtmlFormatter::default();
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
    fn transclusion_link_should_escape_html() {
        let mut properties: HashMap<Cow<str>, Cow<str>> = HashMap::new();
        properties.insert(Cow::from("key1"), Cow::from("<test>value1</test>"));

        let link = Link::new_transclusion_link(
            URIReference::try_from("https://example.com/img.jpg?a=b&c=d")
                .unwrap(),
            Some(Description::from("<test>some description</test>")),
            properties,
        );

        let mut f = HtmlFormatter::default();
        link.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r#"<img src="https://example.com/img.jpg?a=b&c=d" alt="&lt;test&gt;some description&lt;/test&gt;" key1="&lt;test&gt;value1&lt;/test&gt;" />"#
        );
    }

    #[test]
    fn tags_should_output_span_per_tag() {
        let tags: Tags = vec!["one", "two"].into_iter().collect();
        let mut f = HtmlFormatter::default();
        tags.fmt(&mut f).unwrap();

        assert_str_eq!(
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
        let mut f = HtmlFormatter::default();
        f.insert_header_text(1, "first-id");
        f.insert_header_text(3, "third-id");

        tags.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), [
            r#"<span id="first-id-third-id-one"></span><span class="tag" id="one">one</span>"#,
            r#"<span id="first-id-third-id-two"></span><span class="tag" id="two">two</span>"#,
        ].join(""));
    }

    #[test]
    fn tags_should_escape_html() {
        let tags: Tags = vec!["one&", "two>"].into_iter().collect();
        let mut f = HtmlFormatter::default();
        tags.fmt(&mut f).unwrap();

        assert_str_eq!(
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

        let mut f = HtmlFormatter::default();
        tags1.fmt(&mut f).unwrap();
        tags2.fmt(&mut f).unwrap();

        assert_str_eq!(
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

        let mut f = HtmlFormatter::default();
        f.insert_header_text(1, "a");
        f.insert_header_text(2, "b");
        tags1.fmt(&mut f).unwrap();
        tags2.fmt(&mut f).unwrap();

        f.insert_header_text(2, "c");
        tags3.fmt(&mut f).unwrap();

        assert_str_eq!(
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
        let mut f = HtmlFormatter::default();
        code_inline.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), "<code>some code</code>");
    }

    #[test]
    fn code_inline_should_escape_html() {
        let code_inline = CodeInline::from("<test>some code</test>");
        let mut f = HtmlFormatter::default();
        code_inline.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            "<code>&lt;test&gt;some code&lt;/test&gt;</code>"
        );
    }

    #[test]
    fn math_inline_should_output_a_mathjax_notation() {
        let math_inline = MathInline::from("some math");
        let mut f = HtmlFormatter::default();
        math_inline.fmt(&mut f).unwrap();

        assert_str_eq!(f.get_content(), r"\(some math\)");
    }

    #[test]
    fn math_inline_should_escape_html() {
        let math_inline = MathInline::from("<test>some math</test>");
        let mut f = HtmlFormatter::default();
        math_inline.fmt(&mut f).unwrap();

        assert_str_eq!(
            f.get_content(),
            r"\(&lt;test&gt;some math&lt;/test&gt;\)"
        );
    }

    #[test]
    fn comment_should_output_tag_based_on_inner_element() {
        let comment = Comment::from(LineComment::from("some comment"));
        let mut f = HtmlFormatter::new(HtmlConfig {
            comment: HtmlCommentConfig { include: true },
            ..Default::default()
        });
        comment.fmt(&mut f).unwrap();
        assert_str_eq!(f.get_content(), "<!-- some comment -->");

        let comment = Comment::from(MultiLineComment::new(vec![
            Cow::Borrowed("some comment"),
            Cow::Borrowed("on multiple lines"),
        ]));
        let mut f = HtmlFormatter::new(HtmlConfig {
            comment: HtmlCommentConfig { include: true },
            ..Default::default()
        });
        comment.fmt(&mut f).unwrap();
        assert_str_eq!(
            f.get_content(),
            "<!--\nsome comment\non multiple lines\n-->"
        );
    }

    #[test]
    fn line_comment_should_output_html_comment_if_flagged() {
        let comment = LineComment::from("some comment");

        // By default, no comment will be output
        let mut f = HtmlFormatter::default();
        comment.fmt(&mut f).unwrap();
        assert_str_eq!(f.get_content(), "");

        // If configured to output comments, should use HTML syntax
        let mut f = HtmlFormatter::new(HtmlConfig {
            comment: HtmlCommentConfig { include: true },
            ..Default::default()
        });
        comment.fmt(&mut f).unwrap();
        assert_str_eq!(f.get_content(), "<!-- some comment -->");
    }

    #[test]
    fn multi_line_comment_should_output_html_comment_if_flagged() {
        let comment = MultiLineComment::new(vec![
            Cow::Borrowed("some comment"),
            Cow::Borrowed("on multiple lines"),
        ]);

        // By default, no comment will be output
        let mut f = HtmlFormatter::default();
        comment.fmt(&mut f).unwrap();
        assert_str_eq!(f.get_content(), "");

        // If configured to output comments, should use HTML syntax
        let mut f = HtmlFormatter::new(HtmlConfig {
            comment: HtmlCommentConfig { include: true },
            ..Default::default()
        });
        comment.fmt(&mut f).unwrap();
        assert_str_eq!(
            f.get_content(),
            "<!--\nsome comment\non multiple lines\n-->"
        );
    }
}
