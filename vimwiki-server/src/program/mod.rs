pub(crate) mod config;
mod graphql;
use config::*;
mod wiki;
use wiki::*;
mod file;
use file::*;
mod server;
mod stdin;
mod utils;
mod watcher;
use watcher::*;

use graphql::elements::Page;
use log::error;
use log::trace;
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

/// Alias for a result with a program error
pub type ProgramResult<T, E = ProgramError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum ProgramError {
    #[snafu(display("Could not load program from {}: {}", path.display(), source))]
    LoadProgram {
        path: PathBuf,
        source: tokio::io::Error,
    },
    #[snafu(display("Could not deserialize json to program: {}", source))]
    JsonToProgram { source: serde_json::Error },
    #[snafu(display("Could not serialize program to json: {}", source))]
    ProgramToJson { source: serde_json::Error },
    #[snafu(display("Could not create cache directory {}: {}", path.display(), source))]
    MakeProgramCacheDirectory {
        path: PathBuf,
        source: tokio::io::Error,
    },
    #[snafu(display("Could not store program to {}: {}", path.display(), source))]
    StoreProgram {
        path: PathBuf,
        source: tokio::io::Error,
    },
    #[snafu(display("Could not start file watcher: {}", source))]
    FileWatcher { source: notify::Error },
}

/// Contains the state of the program while it is running
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Program {
    /// Represents the files loaded into the program
    files: HashMap<PathBuf, ParsedFile>,

    /// Represents the information associated with each wiki; the ordering
    /// is significant here as it matches the order as defined by the user
    /// and is also the ordering here (wiki index 0 is index 0 in the vec)
    wikis: Vec<Wiki>,
}

/// Represents a program that can be shared and modified across threads
pub type ShareableProgram = Arc<Mutex<Program>>;

impl Program {
    /// Runs our program
    pub async fn run(config: Config) -> ProgramResult<()> {
        let program: ShareableProgram =
            Arc::new(Mutex::new(Self::load(&config).await?));

        // Create our watcher, which will persist for the lifetime of this
        // run, monitor file changes, and reload files
        let watcher = Watcher::initialize(Arc::clone(&program), &config)
            .await
            .context(FileWatcher {})?;

        for path in program.lock().await.wiki_paths() {
            if path.exists() {
                if let Err(x) = watcher.watch_wiki(path).await {
                    error!("Failed to watch {:?}: {}", path, x);
                }
            }
        }

        for path in program.lock().await.loaded_standalone_file_paths() {
            if path.exists() {
                if let Err(x) = watcher.watch_standalone(path).await {
                    error!("Failed to watch {:?}: {}", path, x);
                }
            }
        }

        match config.mode {
            Mode::Stdin => stdin::run(program, config).await,
            Mode::Http => server::run(program, config).await,
        }

        Ok(())
    }
}

impl Program {
    /// Load program state using given config
    pub async fn load(config: &Config) -> ProgramResult<Self> {
        // Load our program from a cache file if it exists, otherwise we
        // start with a clean cache file
        let mut program = {
            let path = Self::cache_file(config);
            if path.exists() {
                let contents = tokio::fs::read_to_string(&path)
                    .await
                    .context(LoadProgram { path })?;
                serde_json::from_str(&contents).context(JsonToProgram {})?
            } else {
                Program::default()
            }
        };

        // Determine the paths of the pre-known wikis we will be parsing and indexing
        program.preload_wikis(config);

        // Process all of the files and add them to our program state
        program
            .preload_files(
                program
                    .wikis
                    .iter()
                    .flat_map(Wiki::files)
                    .map(Path::to_path_buf)
                    .collect(),
            )
            .await;

        // Store our new program as the cache, logging any errors
        if let Err(x) = program.store(&config).await {
            error!("Failed to update program cache: {}", x);
        }

        Ok(program)
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

    /// Write program state to disk using given config
    pub async fn store(&self, config: &Config) -> ProgramResult<()> {
        let json =
            serde_json::to_string_pretty(&self).context(ProgramToJson {})?;

        let path = Self::cache_file(config);
        if let Some(path) = path.parent() {
            tokio::fs::create_dir_all(path)
                .await
                .context(MakeProgramCacheDirectory { path })?;
        }

        tokio::fs::write(&path, json)
            .await
            .context(StoreProgram { path })
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
    pub fn loaded_wiki_file_paths(&self) -> impl Iterator<Item = &Path> {
        self.loaded_file_paths()
            .filter(move |p| self.wiki_paths().any(|w| p.starts_with(w)))
    }

    /// Loads the file at the specified path into the program, or maintains
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

    /// Renames the specified path within the program, moving the internal
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
    /// Note that the file paths are stored in the program aftering being
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

    /// Represents the path to the cache file for the program
    fn cache_file(config: &Config) -> PathBuf {
        config.cache_dir.join("vimwiki.program")
    }
}
