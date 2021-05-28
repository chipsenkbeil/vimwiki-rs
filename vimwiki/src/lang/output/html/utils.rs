use super::{HtmlConfig, HtmlWikiConfig};
use crate::Link;
use chrono::NaiveDate;
use derive_more::{Display, Error};
use relative_path::RelativePathBuf;
use serde::{de, Deserialize};
use std::{
    convert::TryFrom,
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};
use uriparse::{
    Fragment, RelativeReference, RelativeReferenceError, URIReference,
};

/// For use with serde's deserialize_with when deseriaizing to a path that
/// we also want to validate is an absolute path
pub(crate) fn deserialize_absolute_path<'de, D>(
    d: D,
) -> Result<PathBuf, D::Error>
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

    // Attempt to resolve all symlinks and other quirks
    let value = value.canonicalize().map_err(|x| {
        de::Error::invalid_value(
            de::Unexpected::Str(value.to_string_lossy().as_ref()),
            &x.to_string().as_str(),
        )
    })?;

    // Verify that the path given is actually absolute
    if !value.is_absolute() {
        return Err(de::Error::invalid_value(
            de::Unexpected::Str(value.to_string_lossy().as_ref()),
            &"path must be absolute",
        ));
    }

    Ok(value)
}

#[derive(Clone, Debug, PartialEq, Eq, Display, Error)]
pub enum LinkResolutionError {
    /// Represents an error that occurred when evaluating a file in a wiki
    /// identified by index and determining that there is no loaded wiki with
    /// the specified index
    MissingWikiWithIndex {
        #[error(not(source))]
        index: usize,
    },

    /// Represents an error that occurred when evaluating a file in a wiki
    /// identified by name and determining that there is no loaded wiki with
    /// the specified name
    MissingWikiWithName {
        #[error(not(source))]
        name: String,
    },

    /// Represents an error that occurred when trying to construct a
    /// relative reference
    RelativeReference {
        #[error(source)]
        source: RelativeReferenceError,
    },
}

/// Performs link resolution to figure out the resulting URI or relative path
/// based on the file containing the link, the destination wiki, and the
/// outgoing link
pub(crate) fn resolve_link(
    config: &HtmlConfig,
    src_wiki: &HtmlWikiConfig,
    src: &Path,
    target: &Link<'_>,
) -> Result<URIReference<'static>, LinkResolutionError> {
    let ext = "html";
    let src_out = src_wiki.make_output_path(src, ext);

    // We want to figure out if the target uri is a directory to ensure that
    // certain links account for that
    let target_is_dir = is_directory_uri(target.data().uri_ref());

    // First, build our raw uri WITHOUT anchors
    let uri_ref = match target {
        Link::Wiki { data } => {
            if data.is_local() {
                // TODO: Support alternative directory file name
                // NOTE: Don't need to provide extension as will be replaced in
                //       the absolute output path anyway
                let mut path = data.to_path_buf();
                if target_is_dir {
                    path.push("index");
                }

                // Build our output path
                //
                // 1. If absolute (starts with /), then we want to place the
                //    path relative to the root of the current wiki
                // 2. If relative, then we want to place the path relative to
                //    the current file's directory
                let target_out = if data.uri_ref().path().is_absolute() {
                    src_wiki.make_output_path(path.as_path(), ext)
                } else {
                    src_wiki.make_output_path(
                        src.parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_default()
                            .join(path.as_path())
                            .as_path(),
                        ext,
                    )
                };

                let mut uri_ref = make_relative_link(src_out, target_out)
                    .map(URIReference::from)
                    .map_err(|source| {
                        LinkResolutionError::RelativeReference { source }
                    })?;

                if let Some(anchor) = data.to_anchor() {
                    uri_ref.map_fragment(|_| Fragment::try_from(anchor).ok());
                }

                uri_ref
            } else {
                data.uri_ref().clone()
            }
        }
        Link::IndexedInterWiki { index, data } => {
            let index = *index as usize;
            let wiki = config.find_wiki_by_index(index).ok_or({
                LinkResolutionError::MissingWikiWithIndex { index }
            })?;

            // TODO: Support alternative directory file name
            // NOTE: Don't need to provide extension as will be replaced in
            //       the absolute output path anyway
            let mut path = data.to_path_buf();
            if target_is_dir {
                path.push("index");
            }

            // Take the path of the target from the uri reference and make it
            // a relative path as it will always be added to the path of the
            // specified wiki
            let target_out =
                wiki.make_output_path(data.to_path_buf().as_path(), ext);

            let mut uri_ref = make_relative_link(src_out, target_out)
                .map(URIReference::from)
                .map_err(|source| LinkResolutionError::RelativeReference {
                    source,
                })?;

            if let Some(anchor) = data.to_anchor() {
                uri_ref.map_fragment(|_| Fragment::try_from(anchor).ok());
            }

            uri_ref
        }
        Link::NamedInterWiki { name, data } => {
            let wiki = config.find_wiki_by_name(name).ok_or_else(|| {
                LinkResolutionError::MissingWikiWithName {
                    name: name.to_string(),
                }
            })?;

            // TODO: Support alternative directory file name
            // NOTE: Don't need to provide extension as will be replaced in
            //       the absolute output path anyway
            let mut path = data.to_path_buf();
            if target_is_dir {
                path.push("index");
            }

            // Take the path of the target from the uri reference and make it
            // a relative path as it will always be added to the path of the
            // specified wiki
            let target_out =
                wiki.make_output_path(data.to_path_buf().as_path(), ext);

            let mut uri_ref = make_relative_link(src_out, target_out)
                .map(URIReference::from)
                .map_err(|source| LinkResolutionError::RelativeReference {
                    source,
                })?;

            if let Some(anchor) = data.to_anchor() {
                uri_ref.map_fragment(|_| Fragment::try_from(anchor).ok());
            }

            uri_ref
        }
        Link::Diary { date, data } => {
            let diary_out =
                make_diary_absolute_output_path(src_wiki, *date, ext);

            let mut uri_ref = make_relative_link(src_out, diary_out)
                .map(URIReference::from)
                .map_err(|source| LinkResolutionError::RelativeReference {
                    source,
                })?;

            if let Some(anchor) = data.to_anchor() {
                uri_ref.map_fragment(|_| Fragment::try_from(anchor).ok());
            }

            uri_ref
        }
        Link::Raw { data } => data.uri_ref().clone(),
        Link::Transclusion { data } => {
            // If target is a local link, then we need to process it the same
            // as any wiki link
            if data.is_local() {
                let path = data.to_path_buf();

                // We want to reuse the extension if there is one rather than
                // modifying it, so pull out the extension
                let ext =
                    path.extension().and_then(OsStr::to_str).unwrap_or("");

                // Build our output path
                //
                // 1. If absolute (starts with /), then we want to place the
                //    path relative to the root of the current wiki
                // 2. If relative, then we want to place the path relative to
                //    the current file's directory
                let target_out = if data.uri_ref().path().is_absolute() {
                    src_wiki.make_output_path(path.as_path(), ext)
                } else {
                    src_wiki.make_output_path(
                        src.parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_default()
                            .join(path.as_path())
                            .as_path(),
                        ext,
                    )
                };

                make_relative_link(src_out, target_out)
                    .map(URIReference::from)
                    .map_err(|source| {
                        LinkResolutionError::RelativeReference { source }
                    })?

            // Otherwise, we can just pass back the link as-is
            } else {
                data.uri_ref().clone()
            }
        }
    };

    Ok(uri_ref.into_owned())
}

/// Produces an output path for a diary file
fn make_diary_absolute_output_path(
    config: &HtmlWikiConfig,
    date: NaiveDate,
    ext: &str,
) -> PathBuf {
    // Make our input path relative to wiki root
    //
    // {WIKI-ROOT}/{DIARY-REL-PATH}/{DATE}
    //
    // NOTE: The extension of our input doesn't matter (don't even need one)
    //       as we are replacing it with the provided extension
    let input = config
        .path
        .join(config.diary_rel_path.as_path())
        .join(date.format("%Y-%m-%d").to_string());
    config.make_output_path(input.as_path(), ext)
}

/// Given a src and target path, creates a relative reference
#[inline]
fn make_relative_link<P1: AsRef<Path>, P2: AsRef<Path>>(
    src: P1,
    target: P2,
) -> Result<RelativeReference<'static>, RelativeReferenceError> {
    let src_rel = RelativePathBuf::from_path(make_path_relative(src))
        .expect("Impossible: relative path should always succeed");
    let target_rel = RelativePathBuf::from_path(make_path_relative(target))
        .expect("Impossible: relative path should always succeed");

    // NOTE: a relative path of a/b -> a/c would yield ../c, but in the case
    //       of the web, we just want c as referencing from the same directory
    //       is fine; this means that we remove the first .. in the path
    let relative_path = src_rel.relative(target_rel);
    let res = RelativeReference::try_from(
        relative_path
            .strip_prefix("..")
            .unwrap_or(&relative_path)
            .as_str(),
    )
    .map(RelativeReference::into_owned);
    res
}

/// Makes a path relative by stripping it of absolute/root starting elements
pub fn make_path_relative<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref()
        .components()
        .filter(|c| {
            matches!(
                c,
                Component::CurDir | Component::ParentDir | Component::Normal(_)
            )
        })
        .collect()
}

/// Checks if a URI reference's path represents a directory
fn is_directory_uri(uri_ref: &URIReference<'_>) -> bool {
    // NOTE: URI Reference breaks up segments by /, which means that if we
    //       end with a / there is one final segment that is completely
    //       empty
    uri_ref
        .path()
        .segments()
        .last()
        .map_or(false, |s| s.as_str().is_empty())
}
