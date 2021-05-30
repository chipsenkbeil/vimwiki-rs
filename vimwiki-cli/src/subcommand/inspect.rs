use crate::{Ast, CommonOpt, InspectSubcommand};
use jsonpath_lib as jsonpath;
use std::{fs, io};
use vimwiki::HtmlConfig;

pub fn inspect(
    cmd: InspectSubcommand,
    _opt: CommonOpt,
    _config: HtmlConfig,
    ast: Ast,
) -> io::Result<()> {
    let InspectSubcommand { output, json_path } = cmd;

    let ast_json = serde_json::to_value(ast).map_err(io::Error::from)?;
    let values =
        jsonpath::select(&ast_json, json_path.as_str()).map_err(|x| {
            io::Error::new(io::ErrorKind::InvalidData, x.to_string())
        })?;

    if let Some(path) = output {
        serde_json::to_writer_pretty(fs::File::create(path)?, &values)
            .map_err(io::Error::from)
    } else {
        serde_json::to_writer_pretty(io::stdout(), &values)
            .map_err(io::Error::from)
    }
}
