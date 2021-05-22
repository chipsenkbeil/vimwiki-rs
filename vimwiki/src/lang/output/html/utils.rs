use super::{HtmlLinkConfig, HtmlWikiConfig};
use crate::Link;
use std::{
    convert::TryFrom,
    path::{Component, Path, PathBuf},
};
use uriparse::{
    Path as UriPath, Scheme, Segment, URIReference, URIReferenceError,
};

/// Performs link resolution to figure out the resulting URI or relative path
/// based on the file containing the link, the destination wiki, and the
/// outgoing link
pub fn resolve_link(
    config: &HtmlLinkConfig,
    src: &Path,
    target: &Link<'_>,
    target_wiki: &HtmlWikiConfig,
) -> Result<URIReference<'static>, URIReferenceError> {
    let mut target_uri_ref = target.data().uri_ref().clone().into_owned();

    // If we are given a URI that has a scheme, we want to just use it as is
    // TODO: We might need to handle special schemes like file: and local:
    if target.data().uri_ref().has_scheme() {
        return Ok(target_uri_ref);
    }

    if_dir_make_index_html(&mut target_uri_ref);

    Ok(target_uri_ref)
}

/// Resolves a diary link, which always points to a diary entry within the
/// current wiki
fn resolve_diary_link(
    config: &HtmlLinkConfig,
    src: &Path,
    target: &Link<'_>,
    target_wiki: &HtmlWikiConfig,
) -> Result<URIReference<'static>, OutputError> {
    let date_page_string = format!("{}.html", date);
    let wiki = f.config().to_current_wiki();

    // Diary URI path is empty, so we're going to replace it with
    // an actual path by using our wiki root relative to the
    // current file, adding the diary section, and then the date
    let (_, mut path, _, _) = wiki
        .to_relative_reference()
        .map_err(OutputError::from)?
        .into_parts();

    // Add diary path, which should be relative to the wiki
    for c in wiki.diary_rel_path.components() {
        path.push(c.as_os_str().to_str().ok_or(
            OutputError::FailedToModifyUriPath {
                source: uriparse::PathError::InvalidCharacter,
            },
        )?)
        .map_err(OutputError::from)?;
    }
    path.push(date_page_string.as_str())
        .map_err(OutputError::from)?;
}

/// Makes a URI reference that is a directory into an html link
/// by adding index.html or equivalent
fn if_dir_make_index_html(uri_ref: &mut URIReference<'_>) {
    // Lastly, if target is a directory, we need to add index.html
    if is_directory_uri(uri_ref) {
        // NOTE: We know that target is a directory if it ends in /
        //       Additionally, because of that, there is a dangling segment
        //       in the uri ref path that is an empty string, which we
        //       will replace with index.html
        uri_ref.map_path(|mut path| {
            // TODO: Support an option to configure what index.html might be
            *path.segments_mut().last_mut().unwrap() =
                Segment::try_from("index.html").unwrap();
            path
        });
    }
}

/// Checks if a URI reference represents a directory
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
