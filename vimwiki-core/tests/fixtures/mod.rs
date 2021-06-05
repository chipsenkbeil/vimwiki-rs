use std::{fs, io, path::PathBuf};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum VimwikiFile {
    VimwikiWikiIndex,
    VimwikiWikiTroubleshooting,
    VimwikiWikiTipsAndSnips,
    VimwikiWikiRelatedTools,
    PandocVimwikiReader,
    MiscCommentInDefinitionList,
    MiscDepthCheck,
}

impl VimwikiFile {
    /// Loads and returns the file represented by the fixture
    pub fn load(&self) -> io::Result<String> {
        fs::read_to_string(self.to_path())
    }

    /// Returns the path associated with the fixture
    pub fn to_path(&self) -> PathBuf {
        let head =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let tail = match self {
            Self::VimwikiWikiIndex => PathBuf::from("vimwikiwiki/index.wiki"),
            Self::VimwikiWikiRelatedTools => {
                PathBuf::from("vimwikiwiki/Related Tools.wiki")
            }
            Self::VimwikiWikiTipsAndSnips => {
                PathBuf::from("vimwikiwiki/Tips and Snips.wiki")
            }
            Self::VimwikiWikiTroubleshooting => {
                PathBuf::from("vimwikiwiki/Troubleshooting.wiki")
            }
            Self::PandocVimwikiReader => {
                PathBuf::from("pandoc/vimwiki-reader.wiki")
            }
            Self::MiscCommentInDefinitionList => {
                PathBuf::from("misc/comment-in-definition-list.wiki")
            }
            Self::MiscDepthCheck => PathBuf::from("misc/depth-check.wiki"),
        };
        head.join(tail)
    }
}
