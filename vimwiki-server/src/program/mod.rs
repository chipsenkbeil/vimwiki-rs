pub(crate) mod config;
mod graphql;
use config::*;
mod server;
mod stdin;
mod watcher;
use watcher::*;
mod database;
use database::*;

use snafu::{ResultExt, Snafu};
use std::sync::Arc;
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
    pub(crate) database: ShareableDatabase,

    /// Represents a file & directory watcher to update the database when
    /// changes occur
    pub(crate) watcher: Watcher,
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
}
