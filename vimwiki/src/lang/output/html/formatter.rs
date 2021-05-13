use super::HtmlConfig;
use chrono::NaiveDate;
use std::{
    collections::HashMap,
    fmt::{self, Write},
    path::{Path, PathBuf},
};

/// Represents the formatter to use to write HTML output that includes various
/// options that can be set as well as a context for use when writing output
#[derive(Default)]
pub struct HtmlFormatter {
    /// Represents the configuration associated with the formatter
    config: HtmlConfig,

    /// Mapping of header level -> text (with details stripped)
    last_seen_headers: HashMap<usize, String>,

    /// Contains the title to be used for the page
    title: Option<String>,

    /// Contains the date to be used for the page
    date: Option<NaiveDate>,

    /// Contains the template to be used for the page
    template: Option<PathBuf>,

    /// Contains the content to be injected into a template
    content: String,
}

impl Write for HtmlFormatter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.content.write_str(s)
    }
}

impl HtmlFormatter {
    pub fn new(config: HtmlConfig) -> Self {
        Self {
            config,
            last_seen_headers: HashMap::new(),
            title: None,
            date: None,
            template: None,
            content: String::new(),
        }
    }

    /// Represents the config contained within the formatter
    #[inline]
    pub fn config(&self) -> &HtmlConfig {
        &self.config
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

    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    pub fn get_title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn take_title(&mut self) -> Option<String> {
        self.title.take()
    }

    pub fn set_date(&mut self, date: &NaiveDate) {
        self.date = Some(*date);
    }

    pub fn get_date(&self) -> Option<&NaiveDate> {
        self.date.as_ref()
    }

    pub fn take_date(&mut self) -> Option<NaiveDate> {
        self.date.take()
    }

    pub fn set_template(&mut self, template: impl AsRef<Path>) {
        self.template = Some(template.as_ref().to_path_buf());
    }

    pub fn get_template(&self) -> Option<&Path> {
        self.template.as_deref()
    }

    pub fn take_template(&mut self) -> Option<PathBuf> {
        self.template.take()
    }

    pub fn get_content(&self) -> &str {
        self.content.as_str()
    }

    pub fn into_content(self) -> String {
        self.content
    }
}
