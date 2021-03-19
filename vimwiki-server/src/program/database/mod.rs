mod utils;

use crate::{data::Wiki, Config};
use entity::*;
use log::error;
use log::trace;
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Alias for a result with a database error
pub type VimwikiDatabaseResult<T, E = VimwikiDatabaseError> =
    std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum VimwikiDatabaseError {
    #[snafu(display("Could not load database from {}: {}", path.display(), source))]
    LoadDatabase {
        path: PathBuf,
        source: tokio::io::Error,
    },
    #[snafu(display("Could not deserialize json to database: {}", source))]
    JsonToDatabase { source: serde_json::Error },
    #[snafu(display("Could not serialize database to json: {}", source))]
    DatabaseToJson { source: serde_json::Error },
    #[snafu(display("Could not create cache directory {}: {}", path.display(), source))]
    MakeDatabaseCacheDirectory {
        path: PathBuf,
        source: tokio::io::Error,
    },
    #[snafu(display("Could not store database to {}: {}", path.display(), source))]
    StoreDatabase {
        path: PathBuf,
        source: tokio::io::Error,
    },
    #[snafu(display("Could not start file watcher: {}", source))]
    FileWatcher { source: notify::Error },
}

/// Represents primary wiki database
pub struct VimwikiDatabase(DatabaseRc);

impl Default for VimwikiDatabase {
    fn default() -> Self {
        Self(DatabaseRc::new(Box::new(InmemoryDatabase::default())))
    }
}

impl VimwikiDatabase {
    /// Load database state using given config
    pub async fn load(config: &Config) -> VimwikiDatabaseResult<Self> {
        // Load our database from a cache file if it exists, otherwise we
        // start with a clean cache file
        let mut database = {
            let path = Self::cache_file(config);
            if path.exists() {
                let contents = tokio::fs::read_to_string(&path)
                    .await
                    .context(LoadDatabase { path })?;

                // After deserializing our database, we need to update the
                // global id allocator to the previous state
                let db: InmemoryDatabase = serde_json::from_str(&contents)
                    .context(JsonToDatabase {})?;

                db
            } else {
                Database::default()
            }
        };

        // Determine the paths of the pre-known wikis we will be parsing and indexing
        database.preload_wikis(config);

        // Process all of the files and add them to our database state
        database
            .preload_files(
                database
                    .wikis
                    .iter()
                    .flat_map(Wiki::files)
                    .map(Path::to_path_buf)
                    .collect(),
            )
            .await;

        // Store our new database as the cache, logging any errors
        if let Err(x) = database.store(&config).await {
            error!("Failed to update database cache: {}", x);
        }

        Ok(database)
    }

    fn preload_wikis(&mut self, config: &Config) {
        self.wikis = config
            .wikis
            .iter()
            .enumerate()
            .map(|(index, w)| Wiki {
                index,
                name: w.name.clone(),
                path: w.path.to_path_buf(),
                files: utils::walk_and_resolve_paths(&w.path, &config.exts)
                    .into_iter()
                    .collect(),
            })
            .collect();
    }

    async fn preload_files(&mut self, paths: Vec<PathBuf>) {
        let progress = utils::new_progress_bar(paths.len() as u64);
        for path in paths {
            progress
                .set_message(&format!("Loading {}", path.to_string_lossy()));
            if let Err(x) = self.load_file(path).await {
                error!("{}", x);
            }
            progress.inc(1);
        }
        progress.finish_and_clear();
    }

    /// Write database state to disk using given config
    pub async fn store(&self, config: &Config) -> VimwikiDatabaseResult<()> {
        let json =
            serde_json::to_string_pretty(&self).context(DatabaseToJson {})?;

        let path = Self::cache_file(config);
        if let Some(path) = path.parent() {
            tokio::fs::create_dir_all(path)
                .await
                .context(MakeDatabaseCacheDirectory { path })?;
        }

        tokio::fs::write(&path, json)
            .await
            .context(StoreDatabase { path })
    }

    /// Iterator over all of the loaded file paths
    pub fn loaded_file_paths(&self) -> impl Iterator<Item = &Path> {
        self.files.keys().map(PathBuf::as_path)
    }

    /// Iterator over all of the configured wiki paths
    pub fn wiki_paths(&self) -> impl Iterator<Item = &Path> {
        self.wikis.iter().map(Wiki::as_path)
    }

    /// Iterator over all loaded file paths NOT in a configured wiki
    pub fn loaded_standalone_file_paths(&self) -> impl Iterator<Item = &Path> {
        self.loaded_file_paths()
            .filter(move |p| !self.wiki_paths().any(|w| p.starts_with(w)))
    }

    /// Iterator over all loaded file paths in a configured wiki
    #[allow(dead_code)]
    pub fn loaded_wiki_file_paths(&self) -> impl Iterator<Item = &Path> {
        self.loaded_file_paths()
            .filter(move |p| self.wiki_paths().any(|w| p.starts_with(w)))
    }

    /// Returns the wiki containing the given path, if the path resides
    /// within the wiki. Note that this does not evaluate if the path leads
    /// to a real file/directory/symlink, etc. It only checks if the path
    /// starts with any of the given wikis.
    #[allow(dead_code)]
    pub fn find_wiki_containing_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Option<&Wiki> {
        self.wikis
            .iter()
            .find(|w| path.as_ref().starts_with(w.as_path()))
    }

    /// Loads the file at the specified path into the database, or maintains
    /// the current file if determined that it has not changed
    ///
    /// If the underlying file at the specified path is gone, this will
    /// remove our reference to the file and forest internally
    pub async fn load_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(), LoadFileError> {
        trace!("load_file({:?})", path.as_ref());

        let c_path = tokio::fs::canonicalize(path.as_ref()).await.context(
            ReadFailed {
                path: path.as_ref().to_path_buf(),
            },
        )?;

        let file = match self.remove_file(c_path.as_path()) {
            Some(f) => f.reload().await?,
            None => ParsedFile::load(c_path).await?,
        };

        self.files.insert(file.path().to_path_buf(), file);
        Ok(())
    }

    /// Renames the specified path within the database, moving the internal
    /// parsed file from being keyed by one path to a new path
    pub fn rename_file(
        &mut self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) {
        self.remove_file(from.as_ref()).and_then(|f| {
            trace!(
                "Rename internally {:?} to {:?}",
                from.as_ref(),
                to.as_ref()
            );
            self.files.insert(to.as_ref().to_path_buf(), f)
        });
    }

    /// Removes the file if it matches the provided path
    ///
    /// Note that the file paths are stored in the database aftering being
    /// canonicalized, which means that the given path would also need to
    /// be canonicalized to match
    pub fn remove_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Option<ParsedFile> {
        self.files.remove(path.as_ref())
    }

    /// Retrieves a wiki by its name
    pub fn wiki_by_name(&self, name: &str) -> Option<&Wiki> {
        self.wikis.iter().find(|w| w.as_name() == Some(name))
    }

    /// Retrieves a wiki by its index
    pub fn wiki_by_index(&self, index: usize) -> Option<&Wiki> {
        self.wikis.get(index)
    }

    /// Retrieves a loaded GraphQL page by its path
    pub fn graphql_page(&self, path: impl AsRef<Path>) -> Option<Page> {
        self.files
            .get(path.as_ref())
            .map(ParsedFile::forest)
            .map(Page::new)
    }

    /// Represents all loaded GraphQL pages
    pub fn graphql_pages(&self) -> Vec<Page> {
        self.files
            .values()
            .map(ParsedFile::forest)
            .map(Page::new)
            .collect()
    }

    /// Represents the path to the cache file for the database
    fn cache_file(config: &Config) -> PathBuf {
        config.cache_dir.join("vimwiki.database")
    }
}
