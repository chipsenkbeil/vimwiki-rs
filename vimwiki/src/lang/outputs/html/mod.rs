mod config;
pub use config::*;

mod formatter;
pub use formatter::HtmlFormatter;

mod convert;
pub use convert::ToHtmlString;

use crate::lang::{
    elements::*,
    outputs::{Output, OutputError, OutputResult},
};
use lazy_static::lazy_static;
use std::{fmt::Write, path::Path};

use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    html::{self, IncludeBackground},
    parsing::SyntaxSet,
};

lazy_static! {
    /// Default syntax set for languages
    static ref DEFAULT_SYNTAX_SET: SyntaxSet =
        SyntaxSet::load_defaults_nonewlines();

    /// Default theme highlight set for languages
    static ref DEFAULT_THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

impl<'a> Output for Page<'a> {
    type Formatter = HtmlFormatter;

    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        for element in self.elements.iter() {
            element.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output for BlockElement<'a> {
    type Formatter = HtmlFormatter;

    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        match self {
            Self::Blockquote(x) => x.fmt(f),
            Self::DefinitionList(x) => x.fmt(f),
            Self::Divider(x) => x.fmt(f),
            Self::Header(x) => x.fmt(f),
            Self::List(x) => x.fmt(f),
            Self::Math(x) => x.fmt(f),
            Self::Paragraph(x) => x.fmt(f),
            Self::Placeholder(x) => x.fmt(f),
            Self::PreformattedText(x) => x.fmt(f),
            Self::Table(x) => x.fmt(f),
        }
    }
}

impl<'a> Output for Blockquote<'a> {
    type Formatter = HtmlFormatter;

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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        // TODO: Blockquote output is handled differently if it comes from
        //       indented blockquote versus arrow/chevron (>)
        //
        // We don't have a way to determine this yet (needs to be added to
        // blockquote data structure). It also isn't clear to me how this is
        // done differently as the code I'm seeing in vimwiki plugin is for
        // the arrow style, which is what we'll be doing for now
        writeln!(f, "<blockquote>")?;
        for line in self.lines.iter() {
            writeln!(f, "<p>{}</p>", line.trim())?;
        }
        writeln!(f, "</blockquote>")?;
        Ok(())
    }
}

impl<'a> Output for DefinitionList<'a> {
    type Formatter = HtmlFormatter;

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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        writeln!(f, "<dl>")?;
        for (term, defs) in self.iter() {
            // Write our term in the form <dt>{term}</dt>
            write!(f, "<dt>")?;
            term.fmt(f)?;
            writeln!(f, "</dt>")?;

            // Write our defs in the form <dd>{def}</dd>
            for def in defs.iter() {
                write!(f, "<dd>")?;
                def.fmt(f)?;
                writeln!(f, "</dd>")?;
            }
        }
        writeln!(f, "</dl>")?;
        Ok(())
    }
}

impl<'a> Output for DefinitionListValue<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a definition list value in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        self.as_inner().fmt(f)
    }
}

impl Output for Divider {
    type Formatter = HtmlFormatter;

    /// Writes a divider in HTML
    ///
    /// ```html
    /// <hr />
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        writeln!(f, "<hr />")?;
        Ok(())
    }
}

impl<'a> Output for Header<'a> {
    type Formatter = HtmlFormatter;

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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        let header_id = self.content.to_string();
        f.insert_header_text(self.level, header_id.clone());

        let is_toc = header_id.trim() == f.config().header.table_of_contents;
        if is_toc {
            write!(f, r#"<div id="{}">"#, header_id)?;
            write!(
                f,
                r#"<h{} id="{}" class="header">"#,
                self.level, header_id
            )?;
            self.content.fmt(f)?;
            writeln!(f, "</h{}></div>", self.level)?;
        } else {
            // Build our full id using each of the most recent header's
            // contents (earlier levels) up to and including the current header
            let complete_header_id =
                build_complete_id(f, self.level, &header_id)?;

            write!(f, r#"<div id="{}">"#, complete_header_id)?;
            write!(
                f,
                r#"<h{} id="{}" class="header">"#,
                self.level, header_id
            )?;
            write!(f, r##"<a href="#{}">"##, complete_header_id)?;
            self.content.fmt(f)?;
            writeln!(f, "</a></h{}></div>", self.level)?;
        }

        Ok(())
    }
}

impl<'a> Output for List<'a> {
    type Formatter = HtmlFormatter;

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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
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

        for item in self.items.iter() {
            item.fmt(f)?;
        }

        if self.is_ordered() {
            writeln!(f, "</ol>")?;
        } else {
            writeln!(f, "</ul>")?;
        }

        Ok(())
    }
}

impl<'a> Output for ListItem<'a> {
    type Formatter = HtmlFormatter;

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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
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

        writeln!(f, "</li>")?;

        Ok(())
    }
}

impl<'a> Output for ListItemContents<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a list item's contents in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        for content in self.contents.iter() {
            content.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output for ListItemContent<'a> {
    type Formatter = HtmlFormatter;

    /// Writes one piece of content within a list item in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        match self {
            Self::List(x) => x.fmt(f)?,
            Self::InlineContent(x) => x.fmt(f)?,
        }

        Ok(())
    }
}

impl<'a> Output for MathBlock<'a> {
    type Formatter = HtmlFormatter;

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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        if let Some(env) = self.environment.as_deref() {
            writeln!(f, r"\begin{{{}}}", env)?;
            for line in self.lines.iter() {
                writeln!(f, "{}", line)?;
            }
            writeln!(f, r"\end{{{}}}", env)?;
        } else {
            // TODO: vimwiki appears to support a class if it is on the same
            //       line as the start of the math block, which we currently
            //       do not parse. This would be appended to the end of the
            //       starting notation \[<CLASS>
            writeln!(f, r"\[")?;
            for line in self.lines.iter() {
                writeln!(f, "{}", line)?;
            }
            writeln!(f, r"\]")?;
        }

        Ok(())
    }
}

impl<'a> Output for Placeholder<'a> {
    type Formatter = HtmlFormatter;

    /// Writes placeholders in HTML
    ///
    /// Note that this doesn't actually do any writing, but instead updates
    /// settings in the formatter with specific details such as a title, date,
    /// or alternative template to use
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        match self {
            Self::Title(x) => f.set_title(x),
            Self::Date(x) => f.set_date(x),
            Self::Template(x) => f.set_template(x.as_ref()),
            _ => {}
        }

        Ok(())
    }
}

impl<'a> Output for PreformattedText<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a preformatted text block in HTML
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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        // TODO: Support different ways of generating
        //
        // 1. Use https://github.com/trishume/syntect to produce colored
        //    <pre> tags on the backend
        // 2. Produce <pre><code class="lang">...</code></pre> for highlight.js
        // 3. Compatibility with vimwiki plugin

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
                .map_err(OutputError::from)?;
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
                .map_err(OutputError::from)?;
            let ts = custom_ts.as_ref().unwrap_or(&DEFAULT_THEME_SET);

            // Get syntax using language specifier, otherwise use plain text
            let syntax = if let Some(lang) = self.lang.as_ref() {
                ss.find_syntax_by_name(lang)
                    .unwrap_or_else(|| ss.find_syntax_plain_text())
            } else {
                ss.find_syntax_plain_text()
            };

            // Load the specified theme, reporting an error if missing
            let theme =
                ts.themes.get(&f.config().code.theme).ok_or_else(|| {
                    OutputError::ThemeMissing(f.config().code.theme.to_string())
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
            for line in self.lines.iter() {
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

            writeln!(f, "</pre>")?;

        // Otherwise, we produce <pre> and <code class="{lang}"> for use with
        // frontend highlighters like highlight.js
        } else {
            writeln!(f, "<pre>")?;

            // Build out our <code ...> tag
            {
                write!(f, "<code")?;

                // If provided with a language, fill it in as the class
                if let Some(lang) = self.lang.as_ref() {
                    write!(f, r#" class="{}""#, lang)?;
                }

                // For each metadata assignment, treat it as an HTML attribute
                for (attr, value) in self.metadata.iter() {
                    write!(f, r#" {}="{}""#, attr, value)?;
                }

                writeln!(f, ">")?;
            }

            for line in self.lines.iter() {
                writeln!(f, "{}", line)?;
            }

            writeln!(f, "</code>")?;
            writeln!(f, "</pre>")?;
        }

        Ok(())
    }
}

impl<'a> Output for Paragraph<'a> {
    type Formatter = HtmlFormatter;

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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        let ignore_newlines = f.config().text.ignore_newline;

        write!(f, "<p>")?;

        for line in self.lines.iter() {
            for element in line.elements.iter() {
                element.fmt(f)?;
            }

            // If we are not ignoring newlines, then at the end of each line
            // we want to introduce a hard break
            if !ignore_newlines {
                write!(f, "<br />")?;
            }
        }

        writeln!(f, "</p>")?;

        Ok(())
    }
}

impl<'a> Output for Table<'a> {
    type Formatter = HtmlFormatter;

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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        if self.is_centered() {
            writeln!(f, "<table class\"center\">")?;
        } else {
            writeln!(f, "<table>")?;
        }

        if self.has_header_rows() {
            writeln!(f, "<thead>")?;
            for row in self.header_rows() {
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

                        writeln!(f, ">")?;
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

                        writeln!(f, ">")?;
                        content.fmt(f)?;
                        writeln!(f, "</td>")?;
                    }
                }
                writeln!(f, "</tr>")?;
            }
            writeln!(f, "</tbody>")?;
        }

        writeln!(f, "</table>")?;

        Ok(())
    }
}

impl<'a> Output for InlineElementContainer<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a collection of inline elements in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        for element in self.elements.iter() {
            element.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output for InlineElement<'a> {
    type Formatter = HtmlFormatter;

    /// Writes an inline element in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
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

impl<'a> Output for Text<'a> {
    type Formatter = HtmlFormatter;

    /// Writes text in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl<'a> Output for DecoratedText<'a> {
    type Formatter = HtmlFormatter;

    /// Writes decorated text in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        // First, we figure out the type of decoration to apply with bold
        // having the most unique situation as it can also act as an anchor
        match self {
            Self::Bold(contents) => {
                // First, build up the isolated id using contents
                let mut id = String::new();
                for content in contents {
                    write!(&mut id, "{}", content.to_string())?;
                }

                // Second, build up the full id using all headers leading up
                // to this bold text
                let complete_id = build_complete_id(
                    f,
                    f.max_header_level().unwrap_or_default() + 1,
                    &id,
                )?;

                // Third, produce HTML span (anchor) in front of <strong> tag
                // using the complete id produced
                write!(f, r#"<span id="{}"></span><strong>"#, complete_id)?;

                // Fourth, write out all of the contents and then close the
                // <strong> tag
                for content in contents {
                    content.fmt(f)?;
                }
                writeln!(f, "</strong>")?;
            }
            Self::Italic(contents) => {
                write!(f, "<em>")?;
                for content in contents {
                    content.fmt(f)?;
                }
                writeln!(f, "</em>")?;
            }
            Self::Strikeout(contents) => {
                write!(f, "<del>")?;
                for content in contents {
                    content.fmt(f)?;
                }
                writeln!(f, "</del>")?;
            }
            Self::Superscript(contents) => {
                write!(f, "<sup><small>")?;
                for content in contents {
                    content.fmt(f)?;
                }
                writeln!(f, "</small></sup>")?;
            }
            Self::Subscript(contents) => {
                write!(f, "<sub><small>")?;
                for content in contents {
                    content.fmt(f)?;
                }
                writeln!(f, "</small></sub>")?;
            }
        }

        Ok(())
    }
}

impl<'a> Output for DecoratedTextContent<'a> {
    type Formatter = HtmlFormatter;

    /// Writes decorated text content in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        match self {
            Self::Text(x) => x.fmt(f),
            Self::DecoratedText(x) => x.fmt(f),
            Self::Keyword(x) => x.fmt(f),
            Self::Link(x) => x.fmt(f),
        }
    }
}

impl Output for Keyword {
    type Formatter = HtmlFormatter;

    /// Writes keyword in HTML
    ///
    /// Unable to be implemented via Output trait as generic associated types
    /// would be required.
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
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

impl<'a> Output for Link<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a link in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        match self {
            Self::Wiki(x) => x.fmt(f)?,
            Self::InterWiki(x) => x.fmt(f)?,
            Self::Diary(x) => x.fmt(f)?,
            Self::Raw(x) => x.fmt(f)?,
            Self::ExternalFile(x) => x.fmt(f)?,
            Self::Transclusion(x) => x.fmt(f)?,
        }

        Ok(())
    }
}

impl<'a> Output for WikiLink<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a wiki link in HTML
    ///
    /// ### Plain link
    ///
    /// For `[[url]]` in vimwiki:
    ///
    /// ```html
    /// <a href="url.html">url</a>
    /// ```
    ///
    /// ### Link with description
    ///
    /// For `[[url|descr]]` in vimwiki:
    ///
    /// ```html
    /// <a href="url.html">descr</a>
    /// ```
    ///
    /// ### Link with embedded image
    ///
    /// For `[[url|{{...}}]]` in vimwiki:
    ///
    /// ```html
    /// <a href="url.html"> ... </a>
    /// ```
    ///
    ///
    /// ### Link with anchors
    ///
    /// For `[[url#a1#a2]]` in vimwiki:
    ///
    /// ```html
    /// <a href="url.html#a1-a2">url#a1#a2</a>
    /// ```
    ///
    /// ### Only anchors
    ///
    /// For `[[#a1#a2]]` in vimwiki:
    ///
    /// ```html
    /// <a href="#a1-a2">#a1#a2</a>
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        write_link(
            f,
            self.path.as_ref(),
            self.anchor.as_ref(),
            self.description.as_ref(),
        )
    }
}

impl<'a> Output for InterWikiLink<'a> {
    type Formatter = HtmlFormatter;

    /// Writes an interwiki link in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        match self {
            Self::Indexed(x) => x.fmt(f)?,
            Self::Named(x) => x.fmt(f)?,
        }

        Ok(())
    }
}

impl<'a> Output for IndexedInterWikiLink<'a> {
    type Formatter = HtmlFormatter;

    /// Writes an indexed interwiki link in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        // TODO: Need to do link resolution as this should
        //
        // 1. Look up the wiki with the given index (return error if fails to resolve)
        // 2. Grab the path to the wiki
        // 3. Convert path to a relative link in the form of
        //    ../{other wiki}/page.html
        write_link(
            f,
            self.link.path.as_ref(),
            self.link.anchor.as_ref(),
            self.link.description.as_ref(),
        )
    }
}

impl<'a> Output for NamedInterWikiLink<'a> {
    type Formatter = HtmlFormatter;

    /// Writes an named interwiki link in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        // TODO: Need to do link resolution as this should
        //
        // 1. Look up the wiki with the given name (return error if fails to resolve)
        // 2. Grab the path to the wiki
        // 3. Convert path to a relative link in the form of
        //    ../{other wiki}/page.html
        write_link(
            f,
            self.link.path.as_ref(),
            self.link.anchor.as_ref(),
            self.link.description.as_ref(),
        )
    }
}

impl<'a> Output for DiaryLink<'a> {
    type Formatter = HtmlFormatter;

    /// Writes an diary link in HTML
    ///
    /// ### Example
    ///
    /// For `[[diary:2021-03-05]]` and `[[diary:2021-03-05|description]]`:
    ///
    /// ```html
    /// <a href="diary/2021-03-05.html">diary:2021-03-05</a>
    /// <a href="diary/2021-03-05.html">description</a>
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        // TODO: Need some sort of base wiki path for us to provide the
        //       diary link; add our end_path_str to the end of the base path
        let end_path_str = format!("diary/{}.html", self.date.to_string());
        write_link(f, end_path_str, None, self.description.as_ref())
    }
}

impl<'a> Output for RawLink<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a raw link in HTML
    ///
    /// ### Example
    ///
    /// For `https://example.com`:
    ///
    /// ```html
    /// <a href="https://example.com">https://example.com</a>
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        write_link(f, self.uri.to_string(), None, None)
    }
}

impl<'a> Output for ExternalFileLink<'a> {
    type Formatter = HtmlFormatter;

    /// Writes an external file link in HTML
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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        // TODO: Need to construct relative path based on file/dir relative
        //       to the wiki containing it
        let path = if self.path.is_dir() {
            self.path.as_ref().join("index.html")
        } else {
            self.path.to_path_buf()
        };
        write_link(f, path, None, self.description.as_ref())
    }
}

impl<'a> Output for TransclusionLink<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a transclusion link in HTML
    ///
    /// ### Images
    ///
    /// For `{{path/to/img.png}}`, `{{path/to/img.png|descr}}`, and
    /// `{{path/to/img.png|descr|style="A"}}`:
    ///
    /// ```html
    /// <img src="path/to/img.png" />
    /// <img src="path/to/img.png" alt="descr" />
    /// <img src="path/to/img.png" alt="descr" style="A" />
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        write!(f, "<img src=\"{}\"", self.uri)?;
        if let Some(description) = self.description.as_ref() {
            write!(f, " alt=\"{}\"", description.to_string())?;
        }
        for (k, v) in self.properties.iter() {
            write!(f, " {}=\"{}\"", k, v)?;
        }
        write!(f, " />")?;
        Ok(())
    }
}

impl<'a> Output for Tags<'a> {
    type Formatter = HtmlFormatter;

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
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        for tag in self.iter() {
            let id = tag.as_str();
            let complete_id = build_complete_id(
                f,
                f.max_header_level().unwrap_or_default() + 1,
                id,
            )?;
            write!(f, "<span id=\"{}\"></span>", complete_id)?;
            write!(f, "<span class=\"tag\" id=\"{}\">{}</span>", id, id)?;
        }

        Ok(())
    }
}

impl<'a> Output for CodeInline<'a> {
    type Formatter = HtmlFormatter;

    /// Writes inline code in HTML
    ///
    /// ### Example
    ///
    /// ```html
    /// <code>some code</code>
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        write!(f, "<code>{}</code>", self.code)?;
        Ok(())
    }
}

impl<'a> Output for MathInline<'a> {
    type Formatter = HtmlFormatter;

    /// Writes inline math in HTML
    ///
    /// ### Example
    ///
    /// ```html
    /// \(some math\)
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        write!(f, r"\({}\)", self.formula)?;
        Ok(())
    }
}

impl<'a> Output for Comment<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a comment in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        match self {
            Self::Line(x) => x.fmt(f),
            Self::MultiLine(x) => x.fmt(f),
        }
    }
}

impl<'a> Output for LineComment<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a line comment in HTML
    ///
    /// ### Example
    ///
    /// If `config.comment.include` is true, will output the following:
    ///
    /// ```html
    /// <!-- {line} -->
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        if f.config().comment.include {
            write!(f, "<!-- {} -->", self.as_str())?;
        }
        Ok(())
    }
}

impl<'a> Output for MultiLineComment<'a> {
    type Formatter = HtmlFormatter;

    /// Writes a multiline comment in HTML
    ///
    /// ### Example
    ///
    /// If `config.comment.include` is true, will output the following:
    ///
    /// ```html
    /// <!-- {line1}
    /// {line2}
    /// ...
    /// {lineN} -->
    /// ```
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        if f.config().comment.include {
            write!(f, "<!-- ")?;
            for line in self.as_lines() {
                writeln!(f, "{}", line)?;
            }
            write!(f, " -->")?;
        }
        Ok(())
    }
}

fn build_complete_id(
    f: &mut HtmlFormatter,
    max_level: usize,
    id: &str,
) -> Result<String, OutputError> {
    let mut complete_id = String::new();
    for i in 1..max_level {
        if let Some(id) = f.get_header_text(i) {
            write!(&mut complete_id, "{}-", id)?;
        }
    }
    write!(&mut complete_id, "{}", id)?;

    Ok(complete_id)
}

fn write_link(
    f: &mut HtmlFormatter,
    path: impl AsRef<Path>,
    maybe_anchor: Option<&Anchor>,
    maybe_description: Option<&Description>,
) -> OutputResult {
    // Build url#a1-a2
    let mut src = path.as_ref().to_string_lossy().to_string();
    if let Some(anchor) = maybe_anchor {
        write!(&mut src, "#{}", anchor.elements.join("-"))?;
    }

    // Build descr or url#a1#a2 or embed an image
    let mut text = String::new();
    if let Some(description) = maybe_description {
        match description {
            Description::Text(x) => write!(&mut text, "{}", x)?,

            // If description is a url, this signifies it is something we want
            // to pull in rather than use directly
            //
            // TODO: vimwiki supports the following while we only support
            //       the raw url; so, we need to update description to take
            //       in extra information that isn't just the url
            //
            //      {{imgurl|arg1|arg2}}    -> ???
            //      {{imgurl}}                -> <img src="imgurl"/>
            //      {{imgurl|descr|style="A"}} -> <img src="imgurl" alt="descr" style="A" />
            //      {{imgurl|descr|class="B"}} -> <img src="imgurl" alt="descr" class="B" />
            Description::Uri(x) => write!(&mut text, "<img src=\"{}\" />", x)?,
        }
    } else {
        write!(&mut text, "{}", path.as_ref().to_string_lossy())?;

        if let Some(anchor) = maybe_anchor {
            for element in anchor.elements.iter() {
                write!(&mut text, "{}", element)?;
            }
        }
    }

    write!(f, r#"<a href="{}">{}</a>"#, src, text)?;
    Ok(())
}
