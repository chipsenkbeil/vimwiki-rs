use super::{Config, ShareableProgram};
use log::{error, trace};
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
    Error, EventKind, RecommendedWatcher, RecursiveMode,
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
    /// Attempts to initialize a file/directory watcher using the given program
    pub async fn initialize(
        program: ShareableProgram,
        config: &Config,
    ) -> Result<Self, Error> {
        use notify::Watcher;

        let (tx, mut rx) = mpsc::unbounded_channel::<notify::Event>();
        let watcher: RecommendedWatcher =
            Watcher::new_immediate(move |res| match res {
                Ok(event) => {
                    if let Err(x) = tx.send(event) {
                        error!("Failed to queue event: {}", x);
                    }
                }
                Err(e) => error!("Encountered watch error: {:?}", e),
            })?;

        let exts = config.exts.clone();
        let handle = tokio::spawn(async move {
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
                                    program.lock().await.load_file(path).await
                                {
                                    error!("{}", x);
                                }
                            } else {
                                program.lock().await.remove_file(path);
                            }
                        }
                    }
                    EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                        if event.paths.len() == 2 {
                            if let (Some(from), Some(to)) =
                                (event.paths.first(), event.paths.last())
                            {
                                program.lock().await.rename_file(from, to)
                            }
                        } else {
                            error!("Unexpected total paths for a file rename");
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(Self {
            watcher: Arc::new(Mutex::new(watcher)),
            handle,
        })
    }
}
