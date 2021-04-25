// FormatArgs Parse implementation from https://github.com/dtolnay/syn/blob/master/src/mac.rs

use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Expr, Ident, Token};

// The arguments expected by libcore's format_args macro, and as a
// result most other formatting and printing macros like println.
//
//     println!("{} is {number:.prec$}", "x", prec=5, number=0.01)
#[derive(Debug)]
pub struct FormatArgs {
    pub format_string: Expr,
    pub positional_args: Vec<Expr>,
    pub named_args: Vec<(Ident, Expr)>,
}

impl Parse for FormatArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let format_string: Expr;
        let mut positional_args = Vec::new();
        let mut named_args = Vec::new();

        format_string = input.parse()?;
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            if input.peek(Ident::peek_any) && input.peek2(Token![=]) {
                while !input.is_empty() {
                    let name: Ident = input.call(Ident::parse_any)?;
                    input.parse::<Token![=]>()?;
                    let value: Expr = input.parse()?;
                    named_args.push((name, value));
                    if input.is_empty() {
                        break;
                    }
                    input.parse::<Token![,]>()?;
                }
                break;
            }
            positional_args.push(input.parse()?);
        }

        Ok(FormatArgs {
            format_string,
            positional_args,
            named_args,
        })
    }
}
