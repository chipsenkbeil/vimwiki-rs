use super::FormatArgs;
use proc_macro2::TokenStream;
use quote::quote;
use regex::{Captures, Regex};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::{
    parse::{Error, Result},
    spanned::Spanned,
};

pub struct Formatter {
    args: FormatArgs,
    next_argument: AtomicUsize,
}

impl Formatter {
    pub fn new(args: FormatArgs) -> Self {
        Self {
            args,
            next_argument: AtomicUsize::new(0),
        }
    }

    fn next_argument(&self) -> usize {
        self.next_argument.fetch_add(1, Ordering::Relaxed)
    }

    /// Produces a `proc_macro2::TokenStream` in the form of
    /// `::std::format!(<INPUT STR>[, <RELEVANT ARGS>])` by detecting which
    /// arguments are needed from the available and supplying them. If the
    /// needed positional or named argument is missing, the generated token
    /// stream will yield an error instead
    pub fn quote_format_str(&self, input: &str) -> Result<TokenStream> {
        lazy_static::lazy_static! {
            static ref FORMAT_RE: Regex = Regex::new(r"\{(?P<arg>[^:]*?)(?P<spec>:.*?)?\}").unwrap();
        }

        // First, we replace all {} and {:...} with an actual position while
        // keeping track of the positions and names that are produced
        let mut positions = HashSet::new();
        let mut names = HashSet::new();
        let input = FORMAT_RE.replace_all(input, |caps: &Captures| {
            let arg = caps["arg"].trim();

            // Represents {} or {:<SPEC>}
            if arg.is_empty() {
                let pos = self.next_argument();
                positions.insert(pos);
                if let Some(spec) = caps.name("spec") {
                    format!("{{{}{}}}", pos, spec.as_str())
                } else {
                    format!("{{{}}}", pos)
                }

            // Represents {123} or {123:<SPEC>}
            } else if let Ok(pos) = arg.parse::<usize>() {
                positions.insert(pos);
                caps.get(0).unwrap().as_str().to_string()

            // Represents {name} or {name:<SPEC>}
            } else {
                let name = arg.to_string();
                names.insert(name);
                caps.get(0).unwrap().as_str().to_string()
            }
        });

        // Second, we sort our positions and pair them with their new indexes
        // so we have something like (actual_position, positional_arg_index)
        let mut pos_args = Vec::with_capacity(positions.len());
        let index_and_arg_pos: Vec<(usize, usize)> = {
            let mut tmp: Vec<usize> = positions.into_iter().collect();
            tmp.sort_unstable();
            tmp.into_iter().enumerate().collect()
        };

        // Third, we build up our positional arguments that will be included,
        // erroring out if we encounter one that we don't have available
        for idx in index_and_arg_pos.iter().map(|(_, x)| x).copied() {
            if let Some(expr) = self.args.positional_args.get(idx) {
                pos_args.push(quote!(#expr));
            } else {
                return Err(Error::new(
                    self.args.format_string.span(),
                    format!(
                        "{{{}}} requested, but only {} positional arguments provided",
                        idx,
                        self.args.positional_args.len()
                    ),
                ));
            }
        }

        // Fourth, we determine if we have the needed named arguments to format the string
        let mut name_args = Vec::with_capacity(names.len());
        for name in names.iter() {
            match self.args.named_args.iter().find(|(ident, _)| ident == name) {
                Some((ident, expr)) => name_args.push(quote!(#ident = #expr)),
                None => {
                    return Err(Error::new(
                        self.args.format_string.span(),
                        format!(
                            "{{{}}} requested, but no argumented named {} provided",
                            name, name
                        ),
                    ));
                }
            }
        }

        // Fifth, we go back through our input and replace all of the positional
        // arguments with the proper index (arg index -> fmt index)
        let arg_to_idx = index_and_arg_pos
            .into_iter()
            .map(|(a, b)| (b, a))
            .collect::<HashMap<usize, usize>>();
        let input = FORMAT_RE.replace_all(&input, |caps: &Captures| {
            let arg = caps["arg"].trim();

            // Transform 123 from {123} or {123:<SPEC>} into idx
            if let Ok(pos) = arg.parse::<usize>() {
                let idx = *arg_to_idx.get(&pos).unwrap();
                if let Some(spec) = caps.name("spec") {
                    format!("{{{}{}}}", idx, spec.as_str())
                } else {
                    format!("{{{}}}", idx)
                }
            } else {
                caps.get(0).unwrap().as_str().to_string()
            }
        });

        // Sixth, we build out the format string macro as such
        //
        // format!("..."[, pos1, pos2, ...][, name1 = arg1, name2 = arg2, ...])
        Ok(quote!(
            ::std::format!(#input #(,#pos_args)* #(,#name_args)*)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn quote_format_str_should_support_implicit_positional_arguments() {
        let args: FormatArgs = parse_str(r#"" ", "cool", 123"#).unwrap();
        let formatter = Formatter::new(args);
        let stream = formatter.quote_format_str("{} {}").unwrap();

        assert_eq!(
            stream.to_string(),
            quote!(::std::format!("{0} {1}", "cool", 123)).to_string()
        );
    }

    #[test]
    fn quote_format_str_should_support_explicit_positional_arguments() {
        let args: FormatArgs = parse_str(r#"" ", "cool", 123"#).unwrap();
        let formatter = Formatter::new(args);
        let stream = formatter.quote_format_str("{1} {0}").unwrap();

        assert_eq!(
            stream.to_string(),
            quote!(::std::format!("{1} {0}", "cool", 123)).to_string()
        );
    }

    #[test]
    fn quote_format_str_should_support_selectively_including_positional_arguments(
    ) {
        let args: FormatArgs = parse_str(r#"" ", "cool", 123, 456"#).unwrap();
        let formatter = Formatter::new(args);

        let stream = formatter.quote_format_str("{1}").unwrap();
        assert_eq!(
            stream.to_string(),
            quote!(::std::format!("{0}", 123)).to_string()
        );

        let stream = formatter.quote_format_str("{}").unwrap();
        assert_eq!(
            stream.to_string(),
            quote!(::std::format!("{0}", "cool")).to_string()
        );

        let stream = formatter.quote_format_str("no format text").unwrap();
        assert_eq!(
            stream.to_string(),
            quote!(::std::format!("no format text")).to_string()
        );
    }

    #[test]
    fn quote_format_str_should_support_named_arguments() {
        let args: FormatArgs =
            parse_str(r#"" ", a = "cool", b = 123"#).unwrap();
        let formatter = Formatter::new(args);
        let stream = formatter.quote_format_str("{a} {b}").unwrap();

        // NOTE: The order of named parameters is not guaranteed, so we check
        //       either possibility
        let expected_1 = quote!(::std::format!("{a} {b}", a = "cool", b = 123));
        let expected_2 = quote!(::std::format!("{a} {b}", b = 123, a = "cool"));

        assert!(
            stream.to_string() == expected_1.to_string()
                || stream.to_string() == expected_2.to_string(),
            "{} did not match either variant of {}",
            stream,
            expected_1,
        );
    }

    #[test]
    fn quote_format_str_should_support_selectively_including_named_arguments() {
        let args: FormatArgs =
            parse_str(r#"" ", a = "cool", b = 123"#).unwrap();
        let formatter = Formatter::new(args);

        let stream = formatter.quote_format_str("{a}").unwrap();
        assert_eq!(
            stream.to_string(),
            quote!(::std::format!("{a}", a = "cool")).to_string()
        );

        let stream = formatter.quote_format_str("no format text").unwrap();
        assert_eq!(
            stream.to_string(),
            quote!(::std::format!("no format text")).to_string()
        );
    }

    #[test]
    fn quote_format_str_should_support_mixture_of_arguments() {
        let args: FormatArgs =
            parse_str(r#"" ", "fish", 456, a = "cool", b = 123"#).unwrap();
        let formatter = Formatter::new(args);
        let stream =
            formatter.quote_format_str("{1} {} {b} {0} {a} {}").unwrap();

        // NOTE: The order of named parameters is not guaranteed, so we check
        //       either possibility
        let expected_1 = quote!(::std::format!(
            "{1} {0} {b} {0} {a} {1}",
            "fish",
            456,
            a = "cool",
            b = 123
        ));
        let expected_2 = quote!(::std::format!(
            "{1} {0} {b} {0} {a} {1}",
            "fish",
            456,
            b = 123,
            a = "cool"
        ));

        assert!(
            stream.to_string() == expected_1.to_string()
                || stream.to_string() == expected_2.to_string(),
            "{} did not match either variant of {}",
            stream,
            expected_1,
        );
    }
}
