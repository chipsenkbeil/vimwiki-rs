use super::{HtmlConfig, HtmlOutputError, OutputFormatter};
use chrono::NaiveDate;
use std::{
    borrow::Cow,
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

    /// Cache of all ids used and total times used thus far
    id_cache: HashMap<String, usize>,

    /// Contains the title to be used for the page
    title: Option<String>,

    /// Contains the date to be used for the page
    date: Option<NaiveDate>,

    /// Contains the template to be used for the page
    template: Option<PathBuf>,

    /// Contains the content to be injected into a template
    content: String,
}

impl OutputFormatter for HtmlFormatter {
    type Error = HtmlOutputError;
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
            id_cache: HashMap::new(),
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
    pub fn insert_header_text<Text: Into<String>>(
        &mut self,
        level: usize,
        text: Text,
    ) {
        self.last_seen_headers.insert(level, text.into());
    }

    /// Returns the text of the last header seen at the given level
    pub fn get_header_text(&self, level: usize) -> Option<&str> {
        self.last_seen_headers.get(&level).map(String::as_str)
    }

    /// Returns the level of the biggest header (level) stored at the moment
    pub fn max_header_level(&self) -> Option<usize> {
        self.last_seen_headers.keys().max().copied()
    }

    /// Given some input id, will output an id that is guaranteed to be unique
    /// through a format of {ID}-{NUMBER}
    pub fn ensure_unique_id<'a>(&mut self, id: &'a str) -> Cow<'a, str> {
        let unique_id = if self.id_cache.contains_key(id) {
            let mut id = Cow::Borrowed(id);

            while let Some(count) = self.id_cache.get(id.as_ref()).copied() {
                let tmp = format!("{}-{}", id, count + 1);

                if !self.id_cache.contains_key(&tmp) {
                    self.id_cache.insert(id.to_string(), count + 1);
                    id = Cow::Owned(tmp);
                } else {
                    id = Cow::Owned(format!("{}-1", id));
                }
            }

            id
        } else {
            self.id_cache.insert(id.to_string(), 0);
            Cow::Borrowed(id)
        };

        unique_id
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_unique_id_should_return_existing_id_if_not_already_used() {
        let mut f = HtmlFormatter::default();
        assert_eq!(f.ensure_unique_id("id"), "id");
    }

    #[test]
    fn ensure_unique_id_should_return_id_with_numeric_suffix_if_already_used() {
        let mut f = HtmlFormatter::default();

        f.ensure_unique_id("id");
        assert_eq!(f.ensure_unique_id("id"), "id-1");
    }

    #[test]
    fn ensure_unique_id_should_return_id_with_numeric_suffix_incremented_if_already_exists(
    ) {
        let mut f = HtmlFormatter::default();

        assert_eq!(f.ensure_unique_id("id"), "id");
        assert_eq!(f.ensure_unique_id("id"), "id-1");
        assert_eq!(f.ensure_unique_id("id"), "id-2");
    }

    #[test]
    fn ensure_unique_id_should_return_id_with_extra_suffix_if_increment_already_exists(
    ) {
        let mut f = HtmlFormatter::default();

        assert_eq!(f.ensure_unique_id("id"), "id");
        assert_eq!(f.ensure_unique_id("id-1"), "id-1");
        assert_eq!(f.ensure_unique_id("id"), "id-1-1");
        assert_eq!(f.ensure_unique_id("id"), "id-1-2");
        assert_eq!(f.ensure_unique_id("id-1"), "id-1-3");
    }

    #[test]
    fn ensure_unique_id_should_not_cache_generated_ids() {
        let mut f = HtmlFormatter::default();

        // Notice that even though id -> id-1 is produced, id-1 -> id-1 is
        // the next result; this mirrors what blackfriday (markdown) does
        assert_eq!(f.ensure_unique_id("id"), "id");
        assert_eq!(f.ensure_unique_id("id"), "id-1");
        assert_eq!(f.ensure_unique_id("id-1"), "id-1");
        assert_eq!(f.ensure_unique_id("id"), "id-2");
    }
}
