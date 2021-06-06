use crate::config::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::*;
use serde::{de, Deserialize};
use std::{
    ffi::OsStr,
    fs, io,
    path::{Component, Path, PathBuf},
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
pub fn walk_and_resolve_paths(path: &Path, ext: &str) -> Vec<PathBuf> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| {
            e.ok()
                .filter(|e| e.file_type().is_file())
                .map(|e| e.into_path())
                .filter(|p| p.extension().and_then(OsStr::to_str) == Some(ext))
                .and_then(|p| fs::canonicalize(p).ok())
        })
        .collect()
}

/// For use with serde's deserialize_with when deseriaizing to a path that
/// we also want to validate is an absolute path
pub fn deserialize_absolute_path<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = PathBuf::deserialize(d)?;

    // Expand any shell content like ~ or $HOME
    let value = PathBuf::from(
        shellexpand::full(&value.to_string_lossy())
            .map_err(|x| {
                de::Error::invalid_value(
                    de::Unexpected::Str(value.to_string_lossy().as_ref()),
                    &x.to_string().as_str(),
                )
            })?
            .to_string(),
    );

    // Resolve .. and . in path (but not symlinks)
    let value = normalize_path(value.as_path());

    // Verify that the path given is actually absolute
    if !value.is_absolute() {
        return Err(de::Error::invalid_value(
            de::Unexpected::Str(value.to_string_lossy().as_ref()),
            &"path must be absolute",
        ));
    }

    Ok(value)
}

/// Normalize a path, removing things like `.` and `..`.
///
/// CAUTION: This does not resolve symlinks (unlike
/// [`std::fs::canonicalize`]). This may cause incorrect or surprising
/// behavior at times. This should be used carefully. Unfortunately,
/// [`std::fs::canonicalize`] can be hard to use correctly, since it can often
/// fail, or on Windows returns annoying device paths. This is a problem Cargo
/// needs to improve on.
///
/// From https://github.com/rust-lang/cargo/blob/070e459c2d8b79c5b2ac5218064e7603329c92ae/crates/cargo-util/src/paths.rs#L81
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret =
        if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
            components.next();
            PathBuf::from(c.as_os_str())
        } else {
            PathBuf::new()
        };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

/// Attempts to load a config from a file, attempting to load wikis from
/// vim/neovim if no wikis are defined or if merge = true
pub fn load_config<'a, I: Into<Option<&'a Path>>>(
    path: I,
    merge: bool,
) -> io::Result<Config> {
    let maybe_path = path.into();
    trace!("load_config(path = {:?}, merge = {})", maybe_path, merge);

    let mut config: Config = if let Some(path) = maybe_path {
        let config_string = std::fs::read_to_string(path)?;
        toml::from_str(config_string.as_str())?
    } else {
        Config::default()
    };

    // Attempt to load wikis from vim if html config has no wikis or if
    // we are explicitly told to merge
    if config.wikis.is_empty() || merge {
        // We attempt to load and parse our wiki content now, and if it fails
        // then we report over stderr and continue
        match load_vimwiki_list() {
            Ok(wikis) => config.wikis.extend(wikis),
            Err(x) => {
                error!("Failed to load vimwiki_list from vim/neovim: {}", x)
            }
        }
    }

    Ok(config)
}

/// Loads g:vimwiki_list from vim/neovim and then attempts to convert it into
/// a structured html wiki config
fn load_vimwiki_list() -> std::io::Result<Vec<WikiConfig>> {
    trace!("load_vimwiki_list()");

    let vimwiki_list_json = vimvar::load_global_var("vimwiki_list", false)?;
    trace!("g:vimwiki_list == {:?}", vimwiki_list_json);

    if let Some(json) = vimwiki_list_json {
        serde_json::from_value(json).map_err(Into::into)
    } else {
        Ok(Vec::new())
    }
}
