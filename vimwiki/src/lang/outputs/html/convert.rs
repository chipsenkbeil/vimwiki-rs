use super::{HtmlConfig, HtmlFormatter, Output};
use std::{collections::HashMap, path::PathBuf};

/// Provides an interface to convert some vimwiki element to HTML
pub trait ToHtmlString {
    /// Produces a string representing the output generated as HTML
    fn to_html_string(&self) -> String;
}

/// # Panics
///
/// In this implementation, the `to_html_string` method panics if the `Output`
/// implementation returns an error.  This indicates an incorrect `Output`
/// implementation since `fmt::Write for String` never returns an error itself.
impl<'a, T: Output<Formatter = HtmlFormatter<'a>>> ToHtmlString for T {
    /// Blanket implementation that uses the html formatter to produce
    /// appropriate html output
    fn to_html_string(&self) -> String {
        let config = HtmlConfig::default();

        // TODO: Use template to load the appropriate template for HTML output,
        //       defaulting to a static template we'll keep in file (from config)
        //       otherwise
        // TODO: Do a find & replace of %title% for the assigned title if the
        //       string is not empty, otherwise use the filename (from config)
        // TODO: Do a find & replace of %date% for the assigned date if the
        //       string is not empty, otherwise use the current date
        let mut title = String::new();
        let mut date = String::new();
        let mut template = PathBuf::new();
        let mut content = String::new();
        let mut formatter = HtmlFormatter {
            config: &config,
            last_seen_headers: HashMap::new(),
            title: &mut title,
            date: &mut date,
            template: &mut template,
            content: &mut content,
        };

        self.fmt(&mut formatter)
            .expect("Writing strings should not fail");

        // TODO: This should include the filled out template
        //
        // TODO: Does this need to be a blanket implementation? Or should this
        //       be limited to a page? Probably needs to include the config
        //       as well (let config have a default impl) since we would have
        //       no idea what some of the above items like title would be
        //       without a file name
        content
    }
}
