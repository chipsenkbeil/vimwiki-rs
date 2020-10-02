use super::{Config, WikiConfig};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info, trace};
use std::{collections::HashMap, convert::TryInto, path::PathBuf};
use vimwiki::{elements::Page, RawStr, LC};

/// Contains the state of the program while it is running
#[derive(Debug, Default)]
pub struct Program {
    wikis: HashMap<u32, Wiki>,
    name_to_index: HashMap<String, u32>,
}

impl Program {
    pub async fn load(config: &Config) -> Self {
        let mut program = Program::default();

        // Determine the paths of the wiki files we will be parsing and indexing
        // TODO: Provide caching of directories that haven't changed?
        let mut paths = load_paths(&config.wikis, &config.exts);

        // Process all of the files and add them to our program state
        // TODO: Provide caching of wikis that haven't changed?
        if !paths.is_empty() {
            for (w, paths) in paths.drain() {
                let wiki = build_wiki(w, paths).await;
                program.wikis.insert(wiki.index, wiki);
            }
        }

        program
    }
}

/// Represents a wiki and its associated files
#[derive(Debug, Default)]
pub struct Wiki {
    index: u32,
    name: Option<String>,
    files: HashMap<PathBuf, LC<Page>>,
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
        files: HashMap::new(),
    };

    // Because this can take awhile, we will be presenting a progress bar
    // TODO: Parallelize this effort
    // TODO: Cache LC<Page> instances for paths that haven't changed?
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

        let page: LC<Page> = match RawStr::Vimwiki(&contents).try_into() {
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
