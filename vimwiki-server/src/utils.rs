use crate::Config;
use entity::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs,
    path::{Path, PathBuf},
};

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
