use super::{Config, ShareableDatabase};
use log::{error, trace};
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
    Error, Event, EventKind, RecommendedWatcher, RecursiveMode,
};
use std::{path::Path, sync::Arc};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

pub struct Watcher {
    watcher: Arc<Mutex<RecommendedWatcher>>,
    handle: JoinHandle<()>,
}

impl Watcher {
    /// Adds a new wiki directory for changes
    pub async fn watch_wiki(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        use notify::Watcher;

        trace!("Watching wiki {:?}", path.as_ref());
        self.watcher
            .lock()
            .await
            .watch(path, RecursiveMode::Recursive)
    }

    /// Adds a new file path to be watched
    pub async fn watch_standalone(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        use notify::Watcher;

        trace!("Watching standalone {:?}", path.as_ref());
        self.watcher
            .lock()
            .await
            .watch(path, RecursiveMode::NonRecursive)
    }

    /// Removes a path (wiki or standalone) from being watched
    pub async fn unwatch(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        use notify::Watcher;

        trace!("Unwatching {:?}", path.as_ref());
        self.watcher.lock().await.unwatch(path)
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
        let handle = Self::spawn_handle(config, Arc::clone(&database), rx);
        let watcher = Self {
            watcher: Arc::new(Mutex::new(internal_watcher)),
            handle,
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
