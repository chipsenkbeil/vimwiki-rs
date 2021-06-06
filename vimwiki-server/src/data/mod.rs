#![allow(clippy::large_enum_variant)]

use crate::{database::gql_db, utils, Config};
use entity::{TypedPredicate as P, *};
use entity_async_graphql::*;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use vimwiki::{self as v, Language, ParseError};

mod errors;
pub use errors::*;

mod elements;
pub use elements::*;

#[gql_ent]
pub struct Wiki {
    index: usize,
    name: Option<String>,
    path: String,

    #[ent(edge(policy = "deep"))]
    files: Vec<ParsedFile>,
}

impl Wiki {
    pub async fn load_all_from_config<F1, F2, F3, R1>(
        config: &Config,
        before_loading_files: F1,
        on_file_loaded: F2,
        after_loading_files: F3,
    ) -> async_graphql::Result<Vec<Wiki>>
    where
        F1: Copy + Fn(usize) -> R1,
        F2: Copy + Fn(&R1, usize, &Path),
        F3: Copy + Fn(R1),
    {
        let mut wikis = Vec::new();

        for (i, wc) in config.wikis.iter().enumerate() {
            wikis.push(
                Self::load(
                    i,
                    &wc.path,
                    wc.name.as_ref(),
                    wc.ext.as_str(),
                    before_loading_files,
                    on_file_loaded,
                    after_loading_files,
                )
                .await?,
            );
        }
        Ok(wikis)
    }

    pub async fn load<
        N: AsRef<str>,
        F1: Fn(usize) -> R1,
        F2: Fn(&R1, usize, &Path),
        F3: Fn(R1),
        R1,
    >(
        index: usize,
        path: impl AsRef<Path>,
        name: Option<N>,
        ext: &str,
        before_loading_files: F1,
        on_file_loaded: F2,
        after_loading_files: F3,
    ) -> async_graphql::Result<Self> {
        let c_path: PathBuf = tokio::fs::canonicalize(path.as_ref())
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        let paths = utils::walk_and_resolve_paths(c_path.as_path(), ext);
        let tracker = before_loading_files(paths.len());

        // Check if the wiki already exists, otherwise create a new one
        // with no files
        let existing_wiki = gql_db()?
            .find_all_typed::<Wiki>(
                Wiki::query()
                    .where_path(P::equals(c_path.to_string_lossy().to_string()))
                    .into(),
            )
            .map_err(|x| async_graphql::Error::new(x.to_string()))?
            .into_iter()
            .next();

        let mut wiki = if let Some(wiki) = existing_wiki {
            wiki
        } else {
            GraphqlDatabaseError::wrap(
                Self::build()
                    .index(index)
                    .name(name.map(|x| x.as_ref().to_string()))
                    .path(c_path.to_string_lossy().to_string())
                    .files(Vec::new())
                    .finish_and_commit(),
            )
            .map_err(|x| async_graphql::Error::new(x.to_string()))?
        };

        let mut file_ids = Vec::new();
        for (i, path) in paths.into_iter().enumerate() {
            file_ids.push(
                ParsedFile::load(Some(wiki.id()), path.as_path())
                    .await?
                    .id(),
            );
            on_file_loaded(&tracker, i, path.as_path());
        }
        after_loading_files(tracker);

        // Update the wiki's files edge
        wiki.set_files_ids(file_ids);
        let _ = wiki
            .commit()
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        Ok(wiki)
    }
}

#[gql_ent]
pub struct ParsedFile {
    #[ent(field(mutable))]
    path: String,
    checksum: String,

    #[ent(edge)]
    wiki: Option<Wiki>,

    #[ent(edge(policy = "deep"))]
    page: Page,
}

impl ParsedFile {
    pub async fn create(
        wiki_id: impl Into<Option<Id>>,
        path: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
        overwrite: bool,
    ) -> async_graphql::Result<Self> {
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .create_new(!overwrite)
            .open(path.as_ref())
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        let _ = file
            .write_all(contents.as_ref())
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        Self::load(wiki_id, path).await
    }

    pub async fn load_all<P: AsRef<Path>>(
        wiki_id: Option<Id>,
        paths: &[P],
    ) -> async_graphql::Result<Vec<Self>> {
        let mut files = Vec::new();
        for p in paths {
            files.push(Self::load(wiki_id, p).await?);
        }
        Ok(files)
    }

    pub async fn load(
        wiki_id: impl Into<Option<Id>>,
        path: impl AsRef<Path>,
    ) -> async_graphql::Result<Self> {
        let c_path: PathBuf = tokio::fs::canonicalize(path)
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
        let text = tokio::fs::read_to_string(c_path.as_path())
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;
        let checksum = format!("{:x}", Sha1::digest(text.as_bytes()));

        // Third, determine if the content has changed from what we know. If it
        // hasn't, we return the current ent; otherwise, we continue with the
        // intention of replacing the ent by returning its old wiki and removing
        // it from the database
        let old_wiki_id = if let Some(ent) = maybe_ent {
            if ent.checksum() == &checksum {
                return Ok(ent);
            } else {
                let id = ent.wiki_id();
                let _ = ent.remove()?;
                id
            }
        } else {
            None
        };

        // Fourth, convert file contents into a vimwiki page
        let page: v::Page = Language::from_vimwiki_str(&text).parse().map_err(
            |x: ParseError| async_graphql::Error::new(x.to_string()),
        )?;

        // Fifth, save the parsed file with a temporary page id
        let mut parsed_file = GraphqlDatabaseError::wrap(
            Self::build()
                .path(c_path.to_string_lossy().to_string())
                .checksum(checksum)
                .wiki(wiki_id.into().or(old_wiki_id))
                .page(EPHEMERAL_ID)
                .finish_and_commit(),
        )
        .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        // Sixth, save the vimwiki page as a graphql page
        let page_id = Page::create_from_vimwiki(parsed_file.id(), page)?.id();

        // Seventh, update the parsed file's page id
        parsed_file.set_page_id(page_id);
        parsed_file.commit()?;

        Ok(parsed_file)
    }

    pub async fn rename<P1: AsRef<Path>, P2: AsRef<Path>>(
        from_path: P1,
        to_path: P2,
    ) -> async_graphql::Result<()> {
        let c_from_path = tokio::fs::canonicalize(from_path)
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        let c_to_path = tokio::fs::canonicalize(to_path)
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        let ents = gql_db()?
            .find_all_typed::<ParsedFile>(
                ParsedFile::query()
                    .where_path(P::equals(
                        c_from_path.to_string_lossy().to_string(),
                    ))
                    .into(),
            )
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        for mut ent in ents {
            ent.set_path(c_to_path.to_string_lossy().to_string());
            ent.commit()
                .map_err(|x| async_graphql::Error::new(x.to_string()))?;
        }

        Ok(())
    }

    pub async fn remove_all<P: AsRef<Path>>(
        paths: &[P],
    ) -> async_graphql::Result<()> {
        for p in paths {
            Self::remove(p).await?;
        }

        Ok(())
    }

    pub async fn remove(path: impl AsRef<Path>) -> async_graphql::Result<()> {
        let c_path = tokio::fs::canonicalize(path)
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        let ents = gql_db()?
            .find_all_typed::<ParsedFile>(
                ParsedFile::query()
                    .where_path(P::equals(c_path.to_string_lossy().to_string()))
                    .into(),
            )
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        for ent in ents {
            ent.remove()
                .map_err(|x| async_graphql::Error::new(x.to_string()))?;
        }

        Ok(())
    }
}
