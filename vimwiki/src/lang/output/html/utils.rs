use super::HtmlWikiConfig;
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
    src: &Path,
    target: &Link<'_>,
    target_wiki: &HtmlWikiConfig,
) -> Result<URIReference<'static>, URIReferenceError> {
    // If we have a scheme, use it directly after normalizing the uri
    // TODO: Handle file: and local: specially
    if target_uri_ref.has_scheme() {
        target_uri_ref.normalize();
        return Ok(target_uri_ref.into_owned());
    }

    // If the src & target are in different wikis, we need to do the following:
    //
    // 1. Find our way up from the src path to the root of the entire site
    // 2. Enter into the target wiki
    // 3. Apply the target relative reference as the end of the link
    let mut uri_ref = if let Some(wiki_path) = target_wiki_path {
        let mut segments = Vec::new();
        target_uri_ref

    // Otherwise, if both the src and target paths are within the same wiki,
    // we just use the provided uri reference (relative reference) as is
    } else {
        target_uri_ref
    };

    // Lastly, if target is a directory, we need to add index.html
    if is_directory_uri(&target_uri_ref) {
        // NOTE: We know that target is a directory if it ends in /
        //       Additionally, because of that, there is a dangling segment
        //       in the uri ref path that is an empty string, which we
        //       will replace with index.html
        uri_ref.map_path(|path| {
            // TODO: Support an option to configure what index.html might be
            *path.segments_mut().last_mut().unwrap() =
                Segment::try_from("index.html").unwrap();
            path
        });
    }

    Ok(uri_ref)
}

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
