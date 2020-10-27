use indicatif::{ProgressBar, ProgressStyle};
use snafu::{ResultExt, Snafu};
use std::{
    fs,
    path::{Path, PathBuf},
};
use vimwiki::{elements::Page, Language, ParseError};

/// Builds a new progress bar for n items
pub fn new_progress_bar(n: u64) -> ProgressBar {
    ProgressBar::new(n).with_style(
        ProgressStyle::default_bar().template("{msg} {wide_bar} {pos}/{len}"),
    )
}

/// Walks the provided path if it is a directory, canonicalizing each path and
/// filtering out any invalid paths.
///
/// If the given path is a file, it is returned.
///
/// Filters by the given extension.
pub fn walk_and_resolve_paths<E: AsRef<str>>(
    path: impl AsRef<Path>,
    exts: &[E],
) -> Vec<PathBuf> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| {
            e.ok()
                .filter(|e| e.file_type().is_file())
                .map(|e| e.into_path())
                .filter(|p| {
                    p.extension()
                        .map(|e| exts.iter().any(|ext| ext.as_ref() == e))
                        .unwrap_or_default()
                })
                .and_then(|p| fs::canonicalize(p).ok())
        })
        .collect()
}

/// Loads a vimwiki page by reading the contents from the file at the
/// specified path and then parsing it into our internal representation.
pub async fn load_page(
    path: impl AsRef<Path>,
) -> Result<Page<'static>, LoadPageError> {
    let contents =
        tokio::fs::read_to_string(path.as_ref())
            .await
            .context(ReadFile {
                path: path.as_ref().to_path_buf(),
            })?;

    let page: Page = Language::from_vimwiki_str(&contents).parse().map_err(
        |x: ParseError| LoadPageError::ParseFile {
            path: path.as_ref().to_path_buf(),
            source: x.to_string(),
        },
    )?;

    Ok(page.into_owned())
}

#[derive(Debug, Snafu)]
pub enum LoadPageError {
    #[snafu(display("Could not read contents from {}: {}", path.display(), source))]
    ReadFile {
        path: PathBuf,
        source: tokio::io::Error,
    },
    #[snafu(display("Could not parse {}: {}", path.display(), source))]
    ParseFile {
        path: PathBuf,
        #[snafu(source(false))]
        source: String,
    },
}
