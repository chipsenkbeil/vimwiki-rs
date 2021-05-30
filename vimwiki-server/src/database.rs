use crate::{data::Wiki, utils, Config, Opt};
use async_graphql::ErrorExtensions;
use entity::*;
use entity_inmemory::InmemoryDatabase;
use snafu::{ResultExt, Snafu};
use std::path::PathBuf;

#[derive(Debug, Snafu)]
pub enum VimwikiDatabaseError {
    #[snafu(display("Database unavailable"))]
    DatabaseUnavailable,
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

impl async_graphql::ErrorExtensions for VimwikiDatabaseError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self))
    }
}

/// Provides reference to global GraphQL database, failing if the database
/// is not available
#[inline]
pub fn gql_db() -> async_graphql::Result<DatabaseRc> {
    WeakDatabaseRc::upgrade(&entity::global::db())
        .ok_or_else(|| VimwikiDatabaseError::DatabaseUnavailable.extend())
}

/// Load database state using given opt
pub async fn load(
    opt: &Opt,
    config: &Config,
) -> async_graphql::Result<DatabaseRc> {
    // If we already have a database loaded, just return it
    if let Ok(db) = gql_db() {
        return Ok(db);
    }

    // Load our database from a cache file if it exists, otherwise we
    // start with a clean cache file
    let database = {
        let path = cache_file(opt);
        if path.exists() {
            let contents = tokio::fs::read_to_string(&path)
                .await
                .context(LoadDatabase { path })?;

            // After deserializing our database, we need to update the
            // global id allocator to the previous state
            let db: InmemoryDatabase =
                serde_json::from_str(&contents).context(JsonToDatabase {})?;

            db
        } else {
            InmemoryDatabase::default()
        }
    };

    // Set database to be globally available
    global::set_db(database);

    // Determine the paths of the pre-known wikis we will be parsing and indexing
    let _ = Wiki::load_all_from_config(
        &config,
        |file_cnt| utils::new_progress_bar(file_cnt as u64),
        |tracker, _idx, path| {
            tracker.set_message(&format!("Loaded {}", path.to_string_lossy()));
            tracker.inc(1);
        },
        |tracker| tracker.finish_and_clear(),
    )
    .await?;

    // Store our new database as the cache
    let _ = store(&opt).await?;

    gql_db()
}

/// Write database state to disk using given opt
pub async fn store(opt: &Opt) -> async_graphql::Result<()> {
    let db = gql_db()?;

    let json = serde_json::to_string_pretty(
        db.as_ref().as_database::<InmemoryDatabase>().unwrap(),
    )
    .context(DatabaseToJson {})
    .map_err(|x| x.extend())?;

    let path = cache_file(opt);
    if let Some(path) = path.parent() {
        tokio::fs::create_dir_all(path)
            .await
            .context(MakeDatabaseCacheDirectory { path })
            .map_err(|x| x.extend())?;
    }

    tokio::fs::write(&path, json)
        .await
        .context(StoreDatabase { path })
        .map_err(|x| x.extend())?;

    Ok(())
}

/// Represents the path to the cache file for the database
#[inline]
fn cache_file(opt: &Opt) -> PathBuf {
    opt.cache.join("vimwiki.database")
}
