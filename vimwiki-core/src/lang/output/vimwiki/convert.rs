use super::{Output, VimwikiConfig, VimwikiFormatter, VimwikiOutputError};

pub trait ToVimwikiString {
    fn to_vimwiki_string(
        &self,
        config: VimwikiConfig,
    ) -> Result<String, VimwikiOutputError>;
}

impl<T: Output<VimwikiFormatter>> ToVimwikiString for T {
    fn to_vimwiki_string(
        &self,
        config: VimwikiConfig,
    ) -> Result<String, VimwikiOutputError> {
        let mut formatter = VimwikiFormatter::new(config);
        self.fmt(&mut formatter)?;
        Ok(formatter.into_content())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VimwikiOutputResult;

    struct TestOutput<F: Fn(&mut VimwikiFormatter) -> VimwikiOutputResult>(F);
    impl<F: Fn(&mut VimwikiFormatter) -> VimwikiOutputResult>
        Output<VimwikiFormatter> for TestOutput<F>
    {
        fn fmt(&self, f: &mut VimwikiFormatter) -> VimwikiOutputResult {
            self.0(f)?;
            Ok(())
        }
    }

    fn _text(
        text: impl Into<String>,
    ) -> impl Fn(&mut VimwikiFormatter) -> VimwikiOutputResult {
        let text = text.into();
        move |f: &mut VimwikiFormatter| {
            use std::fmt::Write;
            write!(f, "{}", text.as_str())?;
            Ok(())
        }
    }

    #[test]
    fn to_vimwiki_string_should_produce_a_string_representing_only_the_vimwiki_of_the_output(
    ) {
        let output = TestOutput(_text("<b>I am some vimwiki output</b>"));
        let result =
            output.to_vimwiki_string(VimwikiConfig::default()).unwrap();
        assert_eq!(result, "<b>I am some vimwiki output</b>");
    }
}
