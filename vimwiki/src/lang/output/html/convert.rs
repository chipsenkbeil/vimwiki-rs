use super::{
    HtmlConfig, HtmlFormatter, HtmlWikiPageConfig, Output, OutputError,
};
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
    fn to_html_page(
        &self,
        config: HtmlConfig,
        wiki_page_config: HtmlWikiPageConfig,
    ) -> Result<String, OutputError>;
}

impl<T: Output<Formatter = HtmlFormatter>> ToHtmlPage for T {
    fn to_html_page(
        &self,
        config: HtmlConfig,
        wiki_page_config: HtmlWikiPageConfig,
    ) -> Result<String, OutputError> {
        let mut formatter = HtmlFormatter::new(config);
        self.fmt(&mut formatter)?;

        // Leverage provided title or default to the src path's filename if
        // available, finally defaulting to an empty string
        let title = formatter.take_title().unwrap_or_else(|| {
            wiki_page_config
                .get_page_path()
                .file_name()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or_else(String::new)
        });

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

        println!("TEMPLATE: {}", template);

        // Fill in template variables
        let template = template
            .replace("%title%", &title)
            .replace("%date%", &date.to_string())
            .replace(
                "%root_path%",
                &wiki_page_config
                    .get_page_relative_path_to_root()
                    .to_string_lossy(),
            )
            .replace(
                "%wiki_path%",
                &wiki_page_config.get_page_path().to_string_lossy(),
            )
            .replace("%css%", wiki_page_config.get_css_name_or_default())
            .replace("%encoding%", "utf-8")
            .replace("%content%", formatter.get_content());

        println!("POST VARIABLE TEMPLATE: {}", template);

        Ok(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OutputResult;

    struct TestOutput<'a>(&'a str);
    impl<'a> Output for TestOutput<'a> {
        type Formatter = HtmlFormatter;

        fn fmt(&self, f: &mut Self::Formatter) -> OutputResult {
            use std::fmt::Write;
            write!(f, "{}", self.0)?;
            Ok(())
        }
    }

    #[test]
    fn to_html_string_should_produce_a_clean_html_string() {
        let output = TestOutput("<b><img src='' onerror='alert(\\'hax\\')'>I'm not trying to XSS you</b>");
        let result = output.to_html_string(HtmlConfig::default()).unwrap();
        assert_eq!(result, "<b><img src=\"\">I'm not trying to XSS you</b>");
    }

    #[test]
    fn to_html_page_should_replace_title_placeholder_with_provided_title() {
        let output = TestOutput("<span>abc %title% def</span>");
        let title = "some title";
        let result = output
            .to_html_page(
                HtmlConfig::default(),
                HtmlWikiPageConfig::build().page("").finish().unwrap(),
            )
            .unwrap();
        assert_eq!(result, "<span>abc some title def</span>");
    }

    #[test]
    fn to_html_page_should_replace_title_placeholder_with_filename_if_no_provided_title(
    ) {
        todo!();
    }

    #[test]
    fn to_html_page_should_replace_date_placeholder_with_provided_date() {
        todo!();
    }

    #[test]
    fn to_html_page_should_replace_date_placeholder_with_current_date_if_no_provided_date(
    ) {
        todo!();
    }

    #[test]
    fn to_html_page_should_replace_root_path_placeholder_with_path_relative_to_wiki_root(
    ) {
        todo!();
    }

    #[test]
    fn to_html_page_should_replace_wiki_path_placeholder_with_file_path() {
        todo!();
    }

    #[test]
    fn to_html_page_should_replace_css_placeholder_with_provided_css_name() {
        todo!();
    }

    #[test]
    fn to_html_page_should_replace_css_placeholder_with_default_if_no_provided_css_name(
    ) {
        todo!();
    }

    #[test]
    fn to_html_page_should_replace_encoding_placeholder_with_utf8() {
        todo!();
    }

    #[test]
    fn to_html_page_should_replace_content_placeholder_with_output_results() {
        todo!();
    }
}
