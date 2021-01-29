mod elements;
pub use elements::*;

use entity::*;
use std::path::PathBuf;

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Wiki {
    index: usize,
    name: Option<String>,
    path: PathBuf,

    #[ent(edge(policy = "deep"))]
    files: Vec<ParsedFile>,
}

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct ParsedFile {
    path: PathBuf,
    checksum: String,

    #[ent(edge(policy = "deep"))]
    page: Page,
}
