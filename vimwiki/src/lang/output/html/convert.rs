use super::{HtmlConfig, HtmlFormatter, Output, OutputError};
use chrono::Local;

pub trait ToHtmlString {
    /// Converts to individual HTML dom string
    fn to_html_string(&self, config: HtmlConfig)
        -> Result<String, OutputError>;
}

impl<T: Output<Formatter = HtmlFormatter>> ToHtmlString for T {
    fn to_html_string(
        &self,
        config: HtmlConfig,
    ) -> Result<String, OutputError> {
        let mut formatter = HtmlFormatter::new(config);
        self.fmt(&mut formatter)?;
        Ok(formatter.into_content())
    }
}

pub trait ToHtmlPage {
    /// Converts to an HTML page string
    fn to_html_page(&self, config: HtmlConfig) -> Result<String, OutputError>;
}

impl<T: Output<Formatter = HtmlFormatter>> ToHtmlPage for T {
    fn to_html_page(&self, config: HtmlConfig) -> Result<String, OutputError> {
        // Build an HTML formatter using the provided config and funnel our
        // output through it
        let mut formatter = HtmlFormatter::new(config);
        self.fmt(&mut formatter)?;

        // Leverage provided title or default to the src path's filename if
        // available, finally defaulting to an empty string
        let title = formatter.take_title().unwrap_or_else(|| {
            formatter
                .config()
                .page
                .get_path()
                .file_stem()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or_else(String::new)
        });

        // Leverage the provided date, falling back to the current, local date
        let date = formatter
            .take_date()
            .unwrap_or_else(|| Local::now().naive_local().date());

        // Attempts to find and load template in {TEMPLATE_DIR}/{TEMPLATE},
        // defaulting to vimwiki's standard template
        let template = formatter
            .take_template()
            .map(|p| formatter.config().template.dir.join(p))
            .map(std::fs::read_to_string)
            .transpose()
            .map_err(OutputError::from)?
            .unwrap_or_else(|| formatter.config().template.text.to_string());

        // Fill in template variables
        // NOTE: Content is filled in last so we don't replace parts of content
        //       with template variable contents as template variables only
        //       apply to the template and not the content itself
        let template = template
            .replace("%title%", &title)
            .replace("%date%", &date.to_string())
            .replace(
                "%root_path%",
                &formatter
                    .config()
                    .page
                    .get_path_to_root()
                    .filter(|path| !path.as_os_str().is_empty())
                    .map(|path| format!("{}/", path.to_string_lossy()))
                    .unwrap_or_else(String::new),
            )
            .replace(
                "%wiki_path%",
                &formatter
                    .config()
                    .page
                    .get_path_within_root()
                    .map(|path| format!("{}", path.to_string_lossy()))
                    .unwrap_or_else(String::new),
            )
            .replace("%css%", formatter.config().page.get_css_name_or_default())
            .replace("%encoding%", "utf-8")
            .replace("%content%", formatter.get_content());

        Ok(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HtmlPageConfig, HtmlTemplateConfig, OutputResult};
    use chrono::NaiveDate;
    use std::path::PathBuf;

    struct TestOutput<F: Fn(&mut HtmlFormatter) -> OutputResult>(F);
    impl<F: Fn(&mut HtmlFormatter) -> OutputResult> Output for TestOutput<F> {
        type Formatter = HtmlFormatter;

        fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
            self.0(f)?;
            Ok(())
        }
    }

    fn _text(
        text: impl Into<String>,
    ) -> impl Fn(&mut HtmlFormatter) -> OutputResult {
        let text = text.into();
        move |f: &mut HtmlFormatter| {
            use std::fmt::Write;
            write!(f, "{}", text.as_str())?;
            Ok(())
        }
    }

    #[test]
    fn to_html_string_should_produce_a_string_representing_only_the_html_of_the_output(
    ) {
        let output = TestOutput(_text("<b>I am some html output</b>"));
        let result = output.to_html_string(HtmlConfig::default()).unwrap();
        assert_eq!(result, "<b>I am some html output</b>");
    }

    #[test]
    fn to_html_page_should_not_replace_placeholders_in_content() {
        let output = TestOutput(_text("some %title% content"));
        let template = HtmlTemplateConfig::from_text("<html>%content%</html>");
        let config = HtmlConfig::build().template(template).finish().unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>some %title% content</html>");
    }

    #[test]
    fn to_html_page_should_replace_title_placeholder_with_provided_title() {
        let output = TestOutput(|f| {
            f.set_title("some title");
            Ok(())
        });
        let template = HtmlTemplateConfig::from_text("<html>%title%</html>");
        let config = HtmlConfig::build().template(template).finish().unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>some title</html>");
    }

    #[test]
    fn to_html_page_should_replace_title_placeholder_with_filename_if_no_provided_title(
    ) {
        let output = TestOutput(_text(""));
        let template = HtmlTemplateConfig::from_text("<html>%title%</html>");
        let config = HtmlConfig::build()
            .template(template)
            .page(
                HtmlPageConfig::build()
                    .path("some/page.wiki")
                    .finish()
                    .unwrap(),
            )
            .finish()
            .unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>page</html>");
    }

    #[test]
    fn to_html_page_should_replace_date_placeholder_with_provided_date() {
        let output = TestOutput(|f| {
            f.set_date(&NaiveDate::from_ymd(2003, 11, 27));
            Ok(())
        });
        let template = HtmlTemplateConfig::from_text("<html>%date%</html>");
        let config = HtmlConfig::build().template(template).finish().unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>2003-11-27</html>");
    }

    #[test]
    fn to_html_page_should_replace_date_placeholder_with_current_date_if_no_provided_date(
    ) {
        let output = TestOutput(_text(""));
        let template = HtmlTemplateConfig::from_text("<html>%date%</html>");
        let config = HtmlConfig::build().template(template).finish().unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(
            result,
            format!("<html>{}</html>", Local::now().naive_local().date())
        );
    }

    #[test]
    fn to_html_page_should_replace_root_path_placeholder_with_path_relative_to_wiki_root(
    ) {
        let output = TestOutput(_text(""));
        let template =
            HtmlTemplateConfig::from_text("<html>%root_path%</html>");

        // When the file is nested in some subdirectory of the wiki
        let config = HtmlConfig::build()
            .template(template.clone())
            .page(
                HtmlPageConfig::build()
                    .wiki_root(["some", "path"].iter().collect::<PathBuf>())
                    .path(
                        ["some", "path", "to", "a", "file.wiki"]
                            .iter()
                            .collect::<PathBuf>(),
                    )
                    .finish()
                    .unwrap(),
            )
            .finish()
            .unwrap();
        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>../../</html>");

        // When the file is one directory deep
        let config = HtmlConfig::build()
            .template(template.clone())
            .page(
                HtmlPageConfig::build()
                    .wiki_root(["some", "path"].iter().collect::<PathBuf>())
                    .path(
                        ["some", "path", "to", "file.wiki"]
                            .iter()
                            .collect::<PathBuf>(),
                    )
                    .finish()
                    .unwrap(),
            )
            .finish()
            .unwrap();
        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>../</html>");

        // When the file is at the root of the wiki
        let config = HtmlConfig::build()
            .template(template)
            .page(
                HtmlPageConfig::build()
                    .wiki_root(["some", "path"].iter().collect::<PathBuf>())
                    .path(
                        ["some", "path", "file.wiki"]
                            .iter()
                            .collect::<PathBuf>(),
                    )
                    .finish()
                    .unwrap(),
            )
            .finish()
            .unwrap();
        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html></html>");
    }

    #[test]
    fn to_html_page_should_replace_wiki_path_placeholder_with_file_path() {
        let output = TestOutput(_text(""));
        let template =
            HtmlTemplateConfig::from_text("<html>%wiki_path%</html>");
        let config = HtmlConfig::build()
            .template(template)
            .page(
                HtmlPageConfig::build()
                    .wiki_root(["some", "path"].iter().collect::<PathBuf>())
                    .path(
                        ["some", "path", "to", "a", "file.wiki"]
                            .iter()
                            .collect::<PathBuf>(),
                    )
                    .finish()
                    .unwrap(),
            )
            .finish()
            .unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>to/a/file.wiki</html>");
    }

    #[test]
    fn to_html_page_should_replace_css_placeholder_with_provided_css_name() {
        let output = TestOutput(_text(""));
        let template = HtmlTemplateConfig::from_text("<html>%css%</html>");
        let config = HtmlConfig::build()
            .template(template)
            .page(
                HtmlPageConfig::build()
                    .path("")
                    .css_name("css_file")
                    .finish()
                    .unwrap(),
            )
            .finish()
            .unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>css_file</html>");
    }

    #[test]
    fn to_html_page_should_replace_css_placeholder_with_default_if_no_provided_css_name(
    ) {
        let output = TestOutput(_text(""));
        let template = HtmlTemplateConfig::from_text("<html>%css%</html>");
        let config = HtmlConfig::build().template(template).finish().unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(
            result,
            format!("<html>{}</html>", HtmlPageConfig::default_css_name())
        );
    }

    #[test]
    fn to_html_page_should_replace_encoding_placeholder_with_utf8() {
        let output = TestOutput(_text(""));
        let template = HtmlTemplateConfig::from_text("<html>%encoding%</html>");
        let config = HtmlConfig::build().template(template).finish().unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>utf-8</html>");
    }

    #[test]
    fn to_html_page_should_replace_content_placeholder_with_output_results() {
        let output = TestOutput(_text("some output content"));
        let template = HtmlTemplateConfig::from_text("<html>%content%</html>");
        let config = HtmlConfig::build().template(template).finish().unwrap();

        let result = output.to_html_page(config).unwrap();
        assert_eq!(result, "<html>some output content</html>");
    }
}
