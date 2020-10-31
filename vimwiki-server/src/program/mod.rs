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

use graphql::elements::Page;
use log::error;
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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

impl Program {
    /// Runs our program
    pub async fn run(config: Config) -> ProgramResult<()> {
        let program = Self::load(&config).await?;

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

    /// Loads the file at the specified path into the program, or maintains
    /// the current file if determined that it has not changed
    pub async fn load_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(), LoadFileError> {
        let c_path = tokio::fs::canonicalize(path.as_ref()).await.context(
            ReadFailed {
                path: path.as_ref().to_path_buf(),
            },
        )?;

        let file = match self.files.remove(c_path.as_path()) {
            Some(f) => f.reload().await?,
            None => ParsedFile::load(c_path).await?,
        };

        self.files.insert(file.path().to_path_buf(), file);
        Ok(())
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
