use super::{Config, ShareableDatabase};
use log::{error, trace};
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
    Error, Event, EventKind, RecommendedWatcher, RecursiveMode,
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

pub struct Watcher {
    watcher: Arc<Mutex<RecommendedWatcher>>,
    _handle: JoinHandle<()>,
    wikis: Mutex<Vec<PathBuf>>,
    standalone: Mutex<HashSet<PathBuf>>,
}

impl Watcher {
    /// Adds a new wiki directory for changes
    pub async fn watch_wiki(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        use notify::Watcher;

        trace!("Watching wiki {:?}", path.as_ref());
        if !self.is_watched(path.as_ref()).await {
            let result = self
                .watcher
                .lock()
                .await
                .watch(path.as_ref(), RecursiveMode::Recursive);

            if result.is_ok() {
                self.wikis.lock().await.push(path.as_ref().to_path_buf());
            }

            result
        } else {
            trace!("{:?} already being watched", path.as_ref());
            Ok(())
        }
    }

    /// Adds a new file path to be watched
    pub async fn watch_standalone(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        use notify::Watcher;

        trace!("Watching standalone {:?}", path.as_ref());
        if !self.is_watched(path.as_ref()).await {
            let result = self
                .watcher
                .lock()
                .await
                .watch(path.as_ref(), RecursiveMode::NonRecursive);

            if result.is_ok() {
                self.standalone
                    .lock()
                    .await
                    .insert(path.as_ref().to_path_buf());
            }

            result
        } else {
            trace!("{:?} already being watched", path.as_ref());
            Ok(())
        }
    }

    /// Removes a path (wiki or standalone) from being watched
    #[allow(dead_code)]
    pub async fn unwatch(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        use notify::Watcher;

        trace!("Unwatching {:?}", path.as_ref());
        if self.wikis.lock().await.iter().any(|w| w == path.as_ref())
            || self.standalone.lock().await.contains(path.as_ref())
        {
            self.watcher.lock().await.unwatch(path)
        } else {
            trace!("{:?} is not being watched", path.as_ref());
            Ok(())
        }
    }

    /// Whether or not the given path is already being watched, either as
    /// part of a wiki directory or as an individual, standalone file
    pub async fn is_watched(&self, path: impl AsRef<Path>) -> bool {
        let path_ref = path.as_ref();
        self.wikis
            .lock()
            .await
            .iter()
            .any(|w| path_ref.starts_with(w))
            || self.standalone.lock().await.contains(path_ref)
    }

    /// Clears all watched wikis and standalone files from this watcher
    #[allow(dead_code)]
    pub async fn clear(&self) {
        for path in self.wikis.lock().await.drain(..) {
            if let Err(x) = self.unwatch(path.as_path()).await {
                error!("Error clearing {:?}: {}", path, x);
            }
        }

        for path in self.standalone.lock().await.drain() {
            if let Err(x) = self.unwatch(path.as_path()).await {
                error!("Error clearing {:?}: {}", path, x);
            }
        }
    }
}

impl Watcher {
    /// Attempts to initialize a file/directory watcher using the given database
    pub async fn initialize(
        database: ShareableDatabase,
        config: &Config,
    ) -> Result<Self, Error> {
        let (tx, rx) = mpsc::unbounded_channel::<notify::Event>();
        let internal_watcher = Self::new_internal_watcher(tx)?;
        let _handle = Self::spawn_handle(config, Arc::clone(&database), rx);
        let watcher = Self {
            watcher: Arc::new(Mutex::new(internal_watcher)),
            _handle,
            wikis: Mutex::new(Vec::new()),
            standalone: Mutex::new(HashSet::new()),
        };

        watcher.watch_from_database(database).await;

        Ok(watcher)
    }

    async fn watch_from_database(&self, database: ShareableDatabase) {
        for path in database.lock().await.wiki_paths() {
            if path.exists() {
                if let Err(x) = self.watch_wiki(path).await {
                    error!("Failed to watch {:?}: {}", path, x);
                }
            }
        }

        for path in database.lock().await.loaded_standalone_file_paths() {
            if path.exists() {
                if let Err(x) = self.watch_standalone(path).await {
                    error!("Failed to watch {:?}: {}", path, x);
                }
            }
        }
    }

    fn new_internal_watcher(
        tx: mpsc::UnboundedSender<Event>,
    ) -> Result<RecommendedWatcher, Error> {
        use notify::Watcher;

        Watcher::new_immediate(move |res| match res {
            Ok(event) => {
                if let Err(x) = tx.send(event) {
                    error!("Failed to queue event: {}", x);
                }
            }
            Err(e) => error!("Encountered watch error: {:?}", e),
        })
    }

    fn spawn_handle(
        config: &Config,
        database: ShareableDatabase,
        mut rx: mpsc::UnboundedReceiver<Event>,
    ) -> JoinHandle<()> {
        let exts = config.exts.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Ensure that the event we receive is for a supported
                // file extension
                if event.paths.iter().any(|p| {
                    p.extension()
                        .map(|ex| !exts.iter().any(|ext| ext.as_str() == ex))
                        .unwrap_or(true)
                }) {
                    continue;
                }

                trace!(
                    "Got event {:?} for paths {:?}",
                    event.kind,
                    event.paths
                );

                match event.kind {
                    EventKind::Create(CreateKind::File)
                    | EventKind::Modify(ModifyKind::Data(_))
                    | EventKind::Remove(RemoveKind::File) => {
                        for path in event.paths {
                            // TODO: Can we provide an async option here?
                            // TODO: Do we need to detect directories and
                            //       do something special here?
                            if path.exists() {
                                if let Err(x) =
                                    database.lock().await.load_file(path).await
                                {
                                    error!("{}", x);
                                }
                            } else {
                                database.lock().await.remove_file(path);
                            }
                        }
                    }
                    EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                        if event.paths.len() == 2 {
                            if let (Some(from), Some(to)) =
                                (event.paths.first(), event.paths.last())
                            {
                                database.lock().await.rename_file(from, to)
                            }
                        } else {
                            error!("Unexpected total paths for a file rename");
                        }
                    }
                    _ => {}
                }
            }
        })
    }
}
