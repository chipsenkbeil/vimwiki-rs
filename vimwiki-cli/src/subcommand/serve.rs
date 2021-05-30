use crate::{Ast, CommonOpt, ServeSubcommand};
use std::io;
use vimwiki::HtmlConfig;

pub fn serve(
    _cmd: ServeSubcommand,
    _opt: CommonOpt,
    _config: HtmlConfig,
    _ast: Ast,
) -> io::Result<()> {
    Ok(())
}
