use std::{fs, io, path::PathBuf};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum VimwikiFile {
    Issue119,
    Issue120,
    Issue122,
    MiscCommentInDefinitionList,
    MiscDepthCheck,
    MiscWindowsSupport,
    PandocVimwikiReader,
    VimwikiWikiIndex,
    VimwikiWikiTroubleshooting,
    VimwikiWikiTipsAndSnips,
    VimwikiWikiRelatedTools,
}

impl VimwikiFile {
    /// Loads and returns the file represented by the fixture
    pub fn load(&self) -> io::Result<String> {
        fs::read_to_string(self.to_path())
    }

    /// Returns the path associated with the fixture
    pub fn to_path(self) -> PathBuf {
        let head = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/parser/fixtures");
        let tail = match self {
            Self::Issue119 => PathBuf::from("issue/119.wiki"),
            Self::Issue120 => PathBuf::from("issue/120.wiki"),
            Self::Issue122 => PathBuf::from("issue/122.wiki"),
            Self::MiscCommentInDefinitionList => {
                PathBuf::from("misc/comment-in-definition-list.wiki")
            }
            Self::MiscDepthCheck => PathBuf::from("misc/depth-check.wiki"),
            Self::MiscWindowsSupport => {
                PathBuf::from("misc/windows-support.wiki")
            }
            Self::PandocVimwikiReader => {
                PathBuf::from("pandoc/vimwiki-reader.wiki")
            }
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
        };
        head.join(tail)
    }
}
