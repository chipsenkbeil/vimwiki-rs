mod graphql;
mod opt;
use opt::*;
mod server;
mod stdin;

use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info, trace};
use std::{convert::TryInto, path::PathBuf};
use vimwiki::{elements::Page, RawStr, LC};

/// Contains the state of the program while it is running
#[derive(Debug, Default)]
pub struct ProgramState {
    pages: Vec<LC<Page>>,
}

fn load_paths(wikis: &[WikiOpt], exts: &[String]) -> Vec<PathBuf> {
    info!("Scanning wiki directories for {:?}", exts);
    let mut paths = Vec::new();

    for wiki in wikis.iter() {
        debug!("Checking wiki {}", wiki);
        let mut files = walkdir::WalkDir::new(&wiki.path)
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

        paths.append(&mut files);
    }

    paths
}

async fn process_wiki_files(program: &mut ProgramState, files: &[PathBuf]) {
    // Because this can take awhile, we will be presenting a progress bar
    // TODO: Parallelize this effort

    let progress = ProgressBar::new(files.len() as u64).with_style(
        ProgressStyle::default_bar().template("{msg} {wide_bar} {pos}/{len}"),
    );
    for path in files.iter() {
        progress.set_message(&format!(
            "Parsing {:?}",
            path.file_name().unwrap_or_default()
        ));
        let contents = match tokio::fs::read_to_string(path).await {
            Ok(x) => x,
            Err(x) => {
                error!("Failed to load {}: {}", path.to_string_lossy(), x);
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

        program.pages.push(page);
        progress.inc(1);
    }
    progress.finish_and_clear();
}

pub async fn run() {
    let mut program = ProgramState::default();

    use clap::Clap;
    let opt = Opt::parse();

    // Determine the paths of the wiki files we will be parsing and indexing
    let paths = load_paths(&opt.wikis, &opt.exts);

    // Process all of the files and add them to our program state
    if !paths.is_empty() {
        process_wiki_files(&mut program, &paths).await;
    }

    match opt.mode {
        ModeOpt::Stdin => stdin::run(opt).await,
        ModeOpt::Http => server::run(opt).await,
    }
}
