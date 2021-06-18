use super::{OutputFormatter, VimwikiConfig, VimwikiOutputError};
use std::fmt::{self, Write};

/// Represents the formatter to use to write ivimwiki output that includes various
/// options that can be set as well as a context for use when writing output
#[derive(Clone, Default)]
pub struct VimwikiFormatter {
    /// Represents the configuration associated with the formatter
    config: VimwikiConfig,

    /// Contains the content to be injected into a template
    content: String,

    /// Current level of indentation to use when writing vimwiki elements that
    /// care about indentation
    indent_level: usize,

    /// If true, will skip writing whitespace until the first non-whitespace
    /// character is provided, in which case this is reset to false
    skip_whitespace: bool,
}

impl OutputFormatter for VimwikiFormatter {
    type Error = VimwikiOutputError;
}

impl Write for VimwikiFormatter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // If flagged to skip whitespace, we want to skip all whitespace
        // until we see the first non-whitespace character
        let s = if self.skip_whitespace {
            let tmp = s.trim_start();
            if !tmp.is_empty() {
                self.skip_whitespace = false;
            }
            tmp
        } else {
            s
        };

        self.content.write_str(s)
    }
}

impl VimwikiFormatter {
    pub fn new(config: VimwikiConfig) -> Self {
        Self {
            config,
            content: String::new(),
            indent_level: 0,
            skip_whitespace: false,
        }
    }

    pub fn clone_without_content(&self) -> Self {
        Self {
            content: String::new(),
            ..self.clone()
        }
    }

    /// Writes a string representing the indentation for the current level
    pub fn write_indent(&mut self) -> Result<(), VimwikiOutputError> {
        let indent_str = self.config.format.indent_str.to_string();

        for _ in 0..self.indent_level {
            write!(self, "{}", indent_str)?;
        }

        Ok(())
    }

    /// Invokes the given function, passing it a mutable reference to this
    /// formatter where the indentation level has been incremented by 1 and
    /// will be decremented at the end of the function call
    pub fn and_indent<F>(&mut self, f: F) -> Result<(), VimwikiOutputError>
    where
        F: FnOnce(&mut Self) -> Result<(), VimwikiOutputError>,
    {
        self.indent_level += 1;
        let result = f(self);
        self.indent_level -= 1;
        result
    }

    /// Invokes the given function, passing it a mutable reference to this
    /// formatter where all leading and trailing whitespace the mutable ref
    /// produces will be removed
    pub fn and_trim<F>(&mut self, f: F) -> Result<(), VimwikiOutputError>
    where
        F: FnOnce(&mut Self) -> Result<(), VimwikiOutputError>,
    {
        self.skip_whitespace(f)?;
        self.trim_end();
        Ok(())
    }

    /// Invokes the given function, passing it a mutable reference to this
    /// formatter with a flag set to skip all whitespace until the first
    /// non-whitespace character is written to it
    pub fn skip_whitespace<F>(&mut self, f: F) -> Result<(), VimwikiOutputError>
    where
        F: FnOnce(&mut Self) -> Result<(), VimwikiOutputError>,
    {
        self.skip_whitespace = true;
        let result = f(self);
        self.skip_whitespace = false;
        result
    }

    /// Removes whitespace from end of current output content
    pub fn trim_end(&mut self) {
        let diff = self.content.len() - self.content.trim_end().len();
        self.content.truncate(self.content.len() - diff);
    }

    /// Represents the config contained within the formatter
    #[inline]
    pub fn config(&self) -> &VimwikiConfig {
        &self.config
    }

    pub fn get_content(&self) -> &str {
        self.content.as_str()
    }

    pub fn into_content(self) -> String {
        self.content
    }
}
