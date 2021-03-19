mod elements;
pub use elements::*;

use entity::*;

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Wiki {
    index: usize,
    name: Option<String>,
    path: String,

    #[ent(edge(policy = "deep"))]
    files: Vec<ParsedFile>,
}

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct ParsedFile {
    path: String,
    checksum: String,

    #[ent(edge(policy = "deep"))]
    page: Page,
}
