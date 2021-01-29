use super::{ParsedFile, ParsedFileQuery};
use entity::*;
use std::path::PathBuf;

#[simple_ent]
pub struct Wiki {
    index: usize,
    name: Option<String>,
    path: PathBuf,

    #[ent(edge)]
    files: Vec<ParsedFile>,
}
