use super::{
    components::{Definition, DefinitionList, Term},
    utils::{beginning_of_line, position},
    Span, VimwikiIResult, LC,
};

#[inline]
pub fn definition_list(input: Span) -> VimwikiIResult<LC<DefinitionList>> {
    let (input, pos) = position(input)?;

    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;
}
