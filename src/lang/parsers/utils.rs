use super::Span;
use nom::{
    error::{VerboseError, VerboseErrorKind},
    Err, HexDisplay, IResult, Offset,
};
use std::fmt::{Debug, Write};

/// Alias to the type of error to use with vimwiki parsing using nom
pub type VimwikiNomError<'a> = VerboseError<Span<'a>>;

/// Alias to an IResult using VimwikiNomError
pub type VimwikiIResult<'a, I, O> = Result<(I, O), Err<VimwikiNomError<'a>>>;

/// Port of nom's `dbg_dmp` to support a `Span`
#[allow(dead_code)]
pub fn dbg_dmp<'a, F, O, E: Debug>(
    f: F,
    context: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    move |s: Span<'a>| match f(s) {
        Err(e) => {
            println!(
                "{}: Error({:?}) at:\n{}",
                context,
                e,
                (**s.fragment()).to_hex(8)
            );
            Err(e)
        }
        a => a,
    }
}

/// Port of nom's `convert_error` to support a `Span`
#[allow(dead_code)]
pub fn convert_error<'a>(input: Span<'a>, e: VimwikiNomError<'a>) -> String {
    let mut result = String::new();

    for (i, (substring, kind)) in e.errors.iter().enumerate() {
        let offset = input.offset(substring);

        if input.fragment().is_empty() {
            match kind {
                VerboseErrorKind::Char(c) => write!(
                    &mut result,
                    "{}: expected '{}', got empty input\n\n",
                    i, c
                ),
                VerboseErrorKind::Context(s) => {
                    write!(&mut result, "{}: in {}, got empty input\n\n", i, s)
                }
                VerboseErrorKind::Nom(e) => write!(
                    &mut result,
                    "{}: in {:?}, got empty input\n\n",
                    i, e
                ),
            }
        } else {
            let prefix = &input.fragment().as_bytes()[..offset];

            // Count the number of newlines in the first `offset` bytes of input
            let line_number =
                prefix.iter().filter(|&&b| b == b'\n').count() + 1;

            // Find the line that includes the subslice:
            // Find the *last* newline before the substring starts
            let line_begin = prefix
                .iter()
                .rev()
                .position(|&b| b == b'\n')
                .map(|pos| offset - pos)
                .unwrap_or(0);

            // Find the full line after that newline
            let line = input.fragment()[line_begin..]
                .lines()
                .next()
                .unwrap_or(&input.fragment()[line_begin..])
                .trim_end();

            // The (1-indexed) column number is the offset of our substring into that line
            let column_number = line.offset(substring.fragment()) + 1;

            match kind {
                VerboseErrorKind::Char(c) => {
                    if let Some(actual) = substring.fragment().chars().next() {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
               {line}\n\
               {caret:>column$}\n\
               expected '{expected}', found {actual}\n\n",
                            i = i,
                            line_number = line_number,
                            line = line,
                            caret = '^',
                            column = column_number,
                            expected = c,
                            actual = actual,
                        )
                    } else {
                        write!(
                            &mut result,
                            "{i}: at line {line_number}:\n\
               {line}\n\
               {caret:>column$}\n\
               expected '{expected}', got end of input\n\n",
                            i = i,
                            line_number = line_number,
                            line = line,
                            caret = '^',
                            column = column_number,
                            expected = c,
                        )
                    }
                }
                VerboseErrorKind::Context(s) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {context}:\n\
             {line}\n\
             {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    context = s,
                    line = line,
                    caret = '^',
                    column = column_number,
                ),
                VerboseErrorKind::Nom(e) => write!(
                    &mut result,
                    "{i}: at line {line_number}, in {nom_err:?}:\n\
             {line}\n\
             {caret:>column$}\n\n",
                    i = i,
                    line_number = line_number,
                    nom_err = e,
                    line = line,
                    caret = '^',
                    column = column_number,
                ),
            }
        }
        // Because `write!` to a `String` is infallible, this `unwrap` is fine.
        .unwrap();
    }

    result
}
