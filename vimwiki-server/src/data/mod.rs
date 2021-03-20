use crate::utils::gql_db;
use entity::{TypedPredicate as P, *};
use sha1::{Digest, Sha1};
use std::path::Path;
use vimwiki::{elements as v, Language, ParseError};

mod errors;
pub use errors::*;

mod elements;
pub use elements::*;

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

impl ParsedFile {
    pub async fn read_to_file(
        path: impl AsRef<Path>,
    ) -> async_graphql::Result<Self> {
        let c_path = tokio::fs::canonicalize(path)
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        // First, search for an existing ent at the specified path
        let maybe_ent = gql_db()?
            .find_all_typed::<ParsedFile>(
                ParsedFile::query()
                    .where_path(P::equals(c_path.to_string_lossy().to_string()))
                    .into(),
            )
            .map_err(|x| async_graphql::Error::new(x.to_string()))?
            .into_iter()
            .next();

        // Second, load the contents of the file into memory
        let text = tokio::fs::read_to_string(c_path.as_ref())
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;
        let checksum = format!("{:x}", Sha1::digest(text.as_bytes()));

        // Third, determine if the content has changed from what we know. If it
        // has, we remove the old ent in preparation for creating a new one. If
        // it hasn't, we return the current ent.
        if let Some(ent) = maybe_ent {
            if ent.checksum() == &checksum {
                return Ok(ent);
            } else {
                // TODO: Don't remove, just update page ent id and commit
                let _ = ent.remove()?;
            }
        }

        // Fourth, convert file contents into a parsed file
        let page: v::Page = Language::from_vimwiki_str(&text).parse().map_err(
            |x: ParseError| async_graphql::Error::new(x.to_string()),
        )?;

        // Fifth, save the
    }
}
