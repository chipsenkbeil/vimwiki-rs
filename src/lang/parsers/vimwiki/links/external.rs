use super::{
    components::{ExternalLink, ExternalLinkScheme},
    utils::new_nom_error,
    wiki::wiki_link,
    Span, VimwikiIResult, LC,
};
use url::Url;

#[inline]
pub fn external_link(input: Span) -> VimwikiIResult<LC<ExternalLink>> {
    // First, parse as a standard wiki link, which should stash the potential
    // URL as the path
    let (input, link) = wiki_link(input)?;

    // Second, check
    //
    // 1. if the path is actually a url
    // 2. if it's one of the expected schemes for an external link
    // 3. if we can form a local, external path for it
    let maybe_url = link.path.to_str().map(|x| Url::parse(x).ok()).flatten();
    let maybe_scheme = maybe_url.as_ref().map(url_to_external_scheme).flatten();
    let maybe_path = maybe_url.map(|x| x.to_file_path().ok()).flatten();
    match (maybe_path, maybe_scheme) {
        (Some(path), Some(scheme)) => Ok((
            input,
            // TODO: Do we need to adjust the path based on the prefix?
            link.map(|c| ExternalLink::new(scheme, path, c.description)),
        )),
        _ => Err(nom::Err::Error(new_nom_error(input, "Not external link"))),
    }
}

fn url_to_external_scheme(url: &Url) -> Option<ExternalLinkScheme> {
    if url.scheme() == "local" {
        Some(ExternalLinkScheme::Local)
    } else if url.scheme() == "file" {
        Some(ExternalLinkScheme::File)
    } else if url.cannot_be_a_base() && url.as_str().starts_with("//") {
        Some(ExternalLinkScheme::Absolute)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_link_should_support_absolute_path_with_no_scheme() {
        // [[//absolute_path]]
        // [[///tmp/in_root_tmp]]
        // [[//~/in_home_dir]]
        // [[//$HOME/in_home_dir]]
        todo!();
    }

    #[test]
    fn external_link_should_support_file_scheme() {
        // [[file:/home/somebody/a/b/c/music.mp3]]
        // [[file:C:/Users/somebody/d/e/f/music.mp3]]
        // [[file:~/a/b/c/music.mp3]]
        // [[file:../assets/data.csv|Important Data]]
        // [[file:/home/user/documents/|Link to a directory]]
        todo!();
    }

    #[test]
    fn external_link_should_support_local_scheme() {
        // [[local:C:/Users/somebody/d/e/f/music.mp3]]
        todo!();
    }
}
