use super::{Config, WikiConfig};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use log::{debug, error, info, trace};
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap, convert::TryInto, path::PathBuf, time::Instant,
};
use vimwiki::{elements::Page, RawStr, LE};

/// Contains the state of the program while it is running
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Program {
    wikis: HashMap<u32, Wiki>,
    name_to_index: HashMap<String, u32>,
}

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

pub type ProgramResult<T, E = ProgramError> = std::result::Result<T, E>;

impl Program {
    // Load program state using given config
    pub async fn load(config: &Config) -> ProgramResult<Self> {
        // TODO: Load program from cached file instead of default, then
        //       we need to determine which parts of the program are invalid
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

        // Determine the paths of the wiki files we will be parsing and indexing
        // TODO: Provide caching of directories that haven't changed?
        let mut paths = load_paths(&config.wikis, &config.exts);

        // Process all of the files and add them to our program state
        // TODO: Provide caching of wikis that haven't changed?
        if !paths.is_empty() {
            for (w, paths) in paths.drain() {
                let started = Instant::now();
                let wiki = build_wiki(w.clone(), paths).await;
                debug!("Parsed {} in {}", w, HumanDuration(started.elapsed()));
                if let Some(name) = wiki.name.as_ref() {
                    program.name_to_index.insert(name.to_string(), wiki.index);
                }
                program.wikis.insert(wiki.index, wiki);
            }
        }

        // Store our new program as the cache, logging any errors
        if let Err(x) = program.store(&config).await {
            error!("Failed to update program cache: {}", x);
        }

        Ok(program)
    }

    // Write program state to disk using given config
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

    pub fn wiki_by_name(&self, name: &str) -> Option<&Wiki> {
        self.name_to_index
            .get(name)
            .and_then(|index| self.wikis.get(index))
    }

    pub fn wiki_by_index(&self, index: u32) -> Option<&Wiki> {
        self.wikis.get(&index)
    }

    fn cache_file(config: &Config) -> PathBuf {
        config.cache_dir.join("vimwiki.program")
    }
}

/// Represents a wiki and its associated files
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Wiki {
    index: u32,
    name: Option<String>,
    path: PathBuf,
    files: HashMap<PathBuf, LE<Page>>,
}

#[async_graphql::Object]
impl Wiki {
    async fn index(&self) -> u32 {
        self.index
    }

    async fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    async fn path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    async fn page(
        &self,
        path: String,
    ) -> Option<super::graphql::elements::Page> {
        self.files
            .get(&self.path.join(path))
            .map(|x| super::graphql::elements::Page::from(x.clone()))
    }
}

fn load_paths(
    wikis: &[WikiConfig],
    exts: &[String],
) -> HashMap<WikiConfig, Vec<PathBuf>> {
    info!("Scanning wiki directories for {:?}", exts);
    let mut paths = HashMap::new();

    for wiki in wikis.iter() {
        debug!("Checking wiki {}", wiki);
        let files = walkdir::WalkDir::new(&wiki.path)
            .into_iter()
            .filter_map(|e| {
                e.ok()
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.into_path())
                    .filter(|p| {
                        trace!("Checking file: {:?}", p);
                        p.extension()
                            .map(|ext| exts.iter().any(|e| ext == e.as_str()))
                            .unwrap_or_default()
                    })
            })
            .collect();

        paths.insert(wiki.clone(), files);
    }

    paths
}

async fn build_wiki(wiki_config: WikiConfig, mut paths: Vec<PathBuf>) -> Wiki {
    let mut wiki = Wiki {
        index: wiki_config.index,
        name: wiki_config.name,
        path: wiki_config.path,
        files: HashMap::new(),
    };

    // Because this can take awhile, we will be presenting a progress bar
    // TODO: Parallelize this effort
    // TODO: Cache LE<Page> instances for paths that haven't changed?
    let progress = ProgressBar::new(paths.len() as u64).with_style(
        ProgressStyle::default_bar().template("{msg} {wide_bar} {pos}/{len}"),
    );
    for path in paths.drain(..) {
        progress.set_message(&format!(
            "Parsing {:?}",
            path.file_name().unwrap_or_default()
        ));
        let contents = match tokio::fs::read_to_string(&path).await {
            Ok(x) => x,
            Err(x) => {
                error!("Failed to load {}: {}", &path.to_string_lossy(), x);
                progress.inc(1);
                continue;
            }
        };

        let page: LE<Page> = match RawStr::Vimwiki(&contents).try_into() {
            Ok(x) => x,
            Err(x) => {
                error!("Failed to parse {}: {}", path.to_string_lossy(), x);
                progress.inc(1);
                continue;
            }
        };

        wiki.files.insert(path, page);
        progress.inc(1);
    }
    progress.finish_and_clear();
    wiki
}
