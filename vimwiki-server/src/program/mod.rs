mod graphql;
mod server;
mod stdin;
mod watcher;
use watcher::*;
mod database;
use database::*;

use log::error;
use snafu::{ResultExt, Snafu};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

/// Alias for a result with a program error
pub type ProgramResult<T, E = ProgramError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum ProgramError {
    #[snafu(display("Could not load database: {}", source))]
    LoadDatabase { source: DatabaseError },
    #[snafu(display("Could not start file watcher: {}", source))]
    FileWatcher { source: notify::Error },
}

/// Contains the state of the program while it is running
pub struct Program {
    /// Represents the database containing queriable information
    database: ShareableDatabase,

    /// Represents a file & directory watcher to update the database when
    /// changes occur
    watcher: Watcher,
}

impl Program {
    /// Runs our program
    pub async fn run(config: Config) -> ProgramResult<()> {
        // Load our database using the provided configuration and any
        // cached data from a previous run
        let database: ShareableDatabase = Arc::new(Mutex::new(
            Database::load(&config).await.context(LoadDatabase {})?,
        ));

        // Initialize our watcher to update the database based on changes
        // that occur in wikis and standalone files
        let watcher = Watcher::initialize(Arc::clone(&database), &config)
            .await
            .context(FileWatcher {})?;

        let program = Self { database, watcher };
        match config.mode {
            Mode::Stdin => stdin::run(program, config).await,
            Mode::Http => server::run(program, config).await,
        }

        Ok(())
    }

    /// Loads a GraphQL page with the given path and begins to watch it
    /// for changes (if not already being watched)
    pub async fn load_and_watch_graphql_page(
        &self,
        path: impl AsRef<Path>,
        reload: bool,
    ) -> Option<graphql::elements::Page> {
        let mut database = self.database.lock().await;

        if reload {
            if let Err(x) = database.load_file(&path).await {
                error!("{}", x);
            } else if let Err(x) = self.watcher.watch_standalone(&path).await {
                error!("{}", x);
            }
        }

        database.graphql_page(path)
    }

    /// Returns all graphql pages contained in the database
    pub async fn graphql_pages(&self) -> Vec<graphql::elements::Page> {
        self.database.lock().await.graphql_pages()
    }

    /// Returns the wiki at the given index in the database
    pub async fn wiki_by_index(&self, index: usize) -> Option<Wiki> {
        self.database.lock().await.wiki_by_index(index).cloned()
    }

    /// Returns the wiki with the given name in the database
    pub async fn wiki_by_name(&self, name: &str) -> Option<Wiki> {
        self.database.lock().await.wiki_by_name(name).cloned()
    }
}
