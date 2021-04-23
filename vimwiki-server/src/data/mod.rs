#![allow(clippy::large_enum_variant)]

use crate::{database::gql_db, utils, Config};
use entity::{TypedPredicate as P, *};
use entity_async_graphql::*;
use sha1::{Digest, Sha1};
use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
};
use vimwiki::{elements as v, Language, ParseError};

mod errors;
pub use errors::*;

mod elements;
pub use elements::*;

#[simple_ent]
#[derive(EntObject, EntFilter)]
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
                    &config.exts,
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
        E: AsRef<str>,
        F1: Fn(usize) -> R1,
        F2: Fn(&R1, usize, &Path),
        F3: Fn(R1),
        R1,
    >(
        index: usize,
        path: impl AsRef<Path>,
        name: Option<N>,
        exts: &[E],
        before_loading_files: F1,
        on_file_loaded: F2,
        after_loading_files: F3,
    ) -> async_graphql::Result<Self> {
        let c_path: PathBuf = tokio::fs::canonicalize(path.as_ref())
            .await
            .map_err(|x| async_graphql::Error::new(x.to_string()))?;

        let paths = utils::walk_and_resolve_paths(c_path.as_path(), exts);
        let tracker = before_loading_files(paths.len());

        let mut file_ids = Vec::new();
        for (i, path) in paths.into_iter().enumerate() {
            file_ids.push(ParsedFile::load(path.as_path()).await?.id());
            on_file_loaded(&tracker, i, path.as_path());
        }
        after_loading_files(tracker);

        // Check if the wiki already exists, and if so update its files
        let maybe_wiki = gql_db()?
            .find_all_typed::<Wiki>(
                Wiki::query()
                    .where_path(P::equals(c_path.to_string_lossy().to_string()))
                    .into(),
            )
            .map_err(|x| async_graphql::Error::new(x.to_string()))?
            .into_iter()
            .next();

        if let Some(mut wiki) = maybe_wiki {
            wiki.set_files_ids(file_ids);
            let _ = wiki
                .commit()
                .map_err(|x| async_graphql::Error::new(x.to_string()))?;
            Ok(wiki)
        } else {
            GraphqlDatabaseError::wrap(
                Self::build()
                    .index(index)
                    .name(name.map(|x| x.as_ref().to_string()))
                    .path(c_path.to_string_lossy().to_string())
                    .files(file_ids)
                    .finish_and_commit(),
            )
            .map_err(|x| async_graphql::Error::new(x.to_string()))
        }
    }
}

#[simple_ent]
#[derive(EntObject, EntFilter)]
pub struct ParsedFile {
    #[ent(field(mutable))]
    path: String,
    checksum: String,

    #[ent(edge(policy = "deep"))]
    page: Page,
}

impl ParsedFile {
    pub async fn create(
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

        Self::load(path).await
    }

    pub async fn load_all<P: AsRef<Path>>(
        paths: &[P],
    ) -> async_graphql::Result<Vec<Self>> {
        let mut files = Vec::new();
        for p in paths {
            files.push(Self::load(p).await?);
        }
        Ok(files)
    }

    pub async fn load(path: impl AsRef<Path>) -> async_graphql::Result<Self> {
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
        // has, we remove the old ent in preparation for creating a new one. If
        // it hasn't, we return the current ent.
        if let Some(ent) = maybe_ent {
            if ent.checksum() == &checksum {
                return Ok(ent);
            } else {
                let _ = ent.remove()?;
            }
        }

        // Fourth, convert file contents into a vimwiki page
        let page: v::Page = Language::from_vimwiki_str(&text).parse().map_err(
            |x: ParseError| async_graphql::Error::new(x.to_string()),
        )?;

        // Fifth, save the vimwiki page as a graphql page
        let page_id = Page::try_from(page)?.id();

        // Sixth, save the parsed file and return it
        GraphqlDatabaseError::wrap(
            Self::build()
                .path(c_path.to_string_lossy().to_string())
                .checksum(checksum)
                .page(page_id)
                .finish_and_commit(),
        )
        .map_err(|x| async_graphql::Error::new(x.to_string()))
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
