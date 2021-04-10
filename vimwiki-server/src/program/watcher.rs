use crate::{
    data::{ParsedFile, Wiki},
    Config,
};
use entity::{TypedPredicate as P, *};
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
        config: &Config,
        database: DatabaseRc,
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

    async fn watch_from_database(&self, database: DatabaseRc) {
        let wiki_paths: Vec<PathBuf> = database
            .find_all_typed::<Wiki>(Wiki::query().into())
            .expect("Database failed to query for wikis")
            .into_iter()
            .map(|x| PathBuf::from(x.path()))
            .collect();

        for path in wiki_paths.iter() {
            if path.exists() {
                if let Err(x) = self.watch_wiki(path).await {
                    error!("Failed to watch {:?}: {}", path, x);
                }
            }
        }

        let standalone_file_paths: Vec<PathBuf> = database
            .find_all_typed::<ParsedFile>(
                ParsedFile::query()
                    .where_path(P::lambda(move |path_str: String| {
                        !wiki_paths
                            .iter()
                            .any(|p| p.ends_with(path_str.as_str()))
                    }))
                    .into(),
            )
            .expect("Database failed to query for standalone files")
            .into_iter()
            .map(|x| PathBuf::from(x.path()))
            .collect();

        for path in standalone_file_paths.iter() {
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
        _database: DatabaseRc,
        mut rx: mpsc::UnboundedReceiver<Event>,
    ) -> JoinHandle<()> {
        let exts = config.exts.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Ensure that the event we receive is for a supported
                // file extension
                let not_for_valid_file_exts = event.paths.iter().any(|p| {
                    p.extension()
                        .map(|ex| !exts.iter().any(|ext| ext.as_str() == ex))
                        .unwrap_or(true)
                });
                if not_for_valid_file_exts {
                    continue;
                }

                trace!(
                    "Got event {:?} for paths {:?}",
                    event.kind,
                    event.paths
                );

                match event.kind {
                    EventKind::Create(CreateKind::File)
                    | EventKind::Modify(ModifyKind::Data(_)) => {
                        if let Err(x) = ParsedFile::load_all(&event.paths).await
                        {
                            error!("{}", x.into_server_error());
                        }
                    }
                    EventKind::Remove(RemoveKind::File) => {
                        if let Err(x) =
                            ParsedFile::remove_all(&event.paths).await
                        {
                            error!("{}", x.into_server_error());
                        }
                    }
                    EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                        if event.paths.len() == 2 {
                            if let (Some(from), Some(to)) =
                                (event.paths.first(), event.paths.last())
                            {
                                if let Err(x) =
                                    ParsedFile::rename(from, to).await
                                {
                                    error!("{}", x.into_server_error());
                                }
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
