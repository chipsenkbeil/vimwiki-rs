use crate::{CommonOpt, ServeSubcommand};
use std::io;

pub fn serve(_cmd: ServeSubcommand, _opt: CommonOpt) -> io::Result<()> {
    Ok(())
}
