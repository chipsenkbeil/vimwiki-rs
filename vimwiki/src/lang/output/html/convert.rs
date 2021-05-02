use super::{HtmlConfig, HtmlFormatter, Output, OutputError};
use chrono::Local;

static DEFAULT_TEMPLATE_STR: &str = r#"""
<!DOCTYPE html>
<html>
<head>
<link rel="Stylesheet" type="text/css" href="%root_path%%css%">
<title>%title%</title>
<meta http-equiv="Content-Type" content="text/html; charset=%encoding%">
<meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
%content%
</body>
</html>
"""#;

pub trait ToHtmlString {
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

        // TODO: Support file name as default
        let title = formatter.take_title().unwrap_or_else(|| String::new());
        let date = formatter
            .take_date()
            .unwrap_or_else(|| Local::now().naive_local().date());
        let template = formatter
            .take_template()
            .map(std::fs::read_to_string)
            .transpose()
            .map_err(OutputError::from)?
            .unwrap_or_else(|| DEFAULT_TEMPLATE_STR.to_string());

        // Fill in template variables
        // TODO: Support root path variable
        // TODO: Support wiki path variable
        let template = template
            .replace("%title%", &title)
            .replace("%date%", &date.to_string())
            .replace("%root_path%", "")
            .replace("%wiki_path%", "")
            .replace("%encoding%", "utf-8")
            .replace("%contents%", formatter.get_content());
        Ok(ammonia::clean(&template))
    }
}
