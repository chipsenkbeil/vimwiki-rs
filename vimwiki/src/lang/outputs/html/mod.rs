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
use std::fmt::Write;

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
    type Formatter = HtmlFormatter<'a>;

    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        for element in self.elements.iter() {
            element.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output for BlockElement<'a> {
    type Formatter = HtmlFormatter<'a>;

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

        Ok(())
    }
}

impl<'a> Output for Blockquote<'a> {
    type Formatter = HtmlFormatter<'a>;

    /// Writes a blockquote in HTML
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
    type Formatter = HtmlFormatter<'a>;

    /// Writes a definition list in HTML
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

impl<'a> Output for &'a Divider {
    type Formatter = HtmlFormatter<'a>;

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
    type Formatter = HtmlFormatter<'a>;

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
            let mut complete_header_id = String::new();
            for i in 1..self.level {
                if let Some(id) = f.get_header_text(self.level) {
                    write!(&mut complete_header_id, "{}-", id)?;
                }
            }
            write!(&mut complete_header_id, "{}", header_id)?;

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
    type Formatter = HtmlFormatter<'a>;

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
        let ignore_newlines = f.config().list.ignore_newline;

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
    type Formatter = HtmlFormatter<'a>;

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
        let ignore_newlines = f.config().list.ignore_newline;

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
    type Formatter = HtmlFormatter<'a>;

    /// Writes a list item's contents in HTML
    fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
        for content in self.contents.iter() {
            content.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Output for ListItemContent<'a> {
    type Formatter = HtmlFormatter<'a>;

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
    type Formatter = HtmlFormatter<'a>;

    /// Writes a math block in HTML
    ///
    /// This leverages MathJAX to transform the dom, and MathJAX expects
    /// block-level math to look like the following:
    ///
    /// ```html
    /// \[some math enclosed in block notation\]
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
    type Formatter = HtmlFormatter<'a>;

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
    type Formatter = HtmlFormatter<'a>;

    /// Writes a preformatted text block in HTML
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
            let ss = if let Some(dir) = f.config().code.syntax_dir.as_ref() {
                &SyntaxSet::load_from_folder(dir).map_err(OutputError::from)?
            } else {
                &DEFAULT_SYNTAX_SET
            };

            // Load and use the theme set from the specified directory if
            // given, otherwise use the default theme set
            let ts = if let Some(dir) = f.config().code.theme_dir.as_ref() {
                &ThemeSet::load_from_folder(dir).map_err(OutputError::from)?
            } else {
                &DEFAULT_THEME_SET
            };

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
                );
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

            for line in self.lines {
                writeln!(f, "{}", line)?;
            }

            writeln!(f, "</code>")?;
            writeln!(f, "</pre>")?;
        }

        Ok(())
    }
}

impl<'a> Output for Paragraph<'a> {
    type Formatter = HtmlFormatter<'a>;

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

        for line in self.lines {
            for element in line {
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
    type Formatter = HtmlFormatter<'a>;

    /// Writes a table in HTML
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
