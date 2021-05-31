use crate::{Ast, CommonOpt, InspectSubcommand};
use jsonpath_lib as jsonpath;
use std::{
    fs,
    io::{self, Write},
};
use vimwiki::HtmlConfig;

pub fn inspect(
    cmd: InspectSubcommand,
    _opt: CommonOpt,
    _config: HtmlConfig,
    ast: Ast,
) -> io::Result<()> {
    let InspectSubcommand {
        output, json_path, ..
    } = cmd;

    let ast_json = serde_json::to_value(ast).map_err(io::Error::from)?;
    let values =
        jsonpath::select(&ast_json, json_path.as_str()).map_err(|x| {
            io::Error::new(io::ErrorKind::InvalidData, x.to_string())
        })?;

    if let Some(path) = output {
        let file = fs::File::create(path)?;
        let mut writer = io::BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &values)
            .map_err(io::Error::from)?;
        writer.flush()?;
        Ok(())
    } else {
        let stdout = io::stdout();
        serde_json::to_writer_pretty(stdout, &values).map_err(io::Error::from)
    }
}
