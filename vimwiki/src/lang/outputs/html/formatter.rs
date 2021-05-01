use super::HtmlConfig;
use chrono::NaiveDate;
use std::{
    collections::HashMap,
    fmt::{self, Write},
    path::{Path, PathBuf},
};

/// Represents the formatter to use to write HTML output that includes various
/// options that can be set as well as a context for use when writing output
pub struct HtmlFormatter<'a> {
    /// Represents the configuration associated with the formatter
    config: &'a HtmlConfig,

    /// Mapping of header level -> text (with details stripped)
    last_seen_headers: HashMap<usize, String>,

    /// Contains the title to be used for the page
    title: &'a mut String,

    /// Contains the date to be used for the page
    date: &'a mut String,

    /// Contains the template to be used for the page
    template: &'a mut PathBuf,

    /// Contains the content to be injected into a template
    content: &'a mut (dyn Write + 'a),
}

impl<'a> Write for HtmlFormatter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.content.write_str(s)
    }
}

impl<'a> HtmlFormatter<'a> {
    /// Represents the config contained within the formatter
    #[inline]
    pub fn config(&self) -> &HtmlConfig {
        self.config
    }

    /// Inserts text for the header at the given level to be remembered when
    /// keeping track of the last header seen at a given level
    pub fn insert_header_text(&mut self, level: usize, text: String) {
        self.last_seen_headers.insert(level, text);
    }

    /// Returns the text of the last header seen at the given level
    pub fn get_header_text(&self, level: usize) -> Option<&str> {
        self.last_seen_headers.get(&level).map(String::as_str)
    }

    /// Returns the level of the biggest header (level) stored at the moment
    pub fn max_header_level(&self) -> Option<usize> {
        self.last_seen_headers.keys().max().copied()
    }

    /// Sets the title referenced by the formatter
    pub fn set_title(&mut self, title: &str) {
        self.title.clear();
        write!(self.title, "{}", title)
            .expect("Writing title should never fail")
    }

    /// Sets the date referenced by the formatter
    pub fn set_date(&mut self, date: &NaiveDate) {
        self.date.clear();
        write!(self.date, "{}", date).expect("Writing date should never fail")
    }

    /// Sets the template referenced by the formatter
    pub fn set_template(&mut self, template: impl AsRef<Path>) {
        self.template.clear();
        self.template.push(template);
    }
}
