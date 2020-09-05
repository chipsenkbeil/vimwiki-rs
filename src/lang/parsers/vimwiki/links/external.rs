use super::{
    components::{ExternalLink, ExternalLinkScheme},
    utils::position,
    Span, VimwikiIResult, LC,
};
use nom::{branch::alt, combinator::map, error::context};

#[inline]
pub fn external_link(input: Span) -> VimwikiIResult<LC<ExternalLink>> {
    let (input, pos) = position(input)?;
    // delimited(tag("[["), anychar, tag("]]")),
    panic!("TODO: Implement");
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
