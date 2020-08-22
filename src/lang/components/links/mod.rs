use derive_more::{
    Constructor, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator,
    TryInto,
};
use serde::{Deserialize, Serialize};
use url::Url;

mod diary;
pub use diary::DiaryLink;

mod external;
pub use external::{ExternalLink, ExternalLinkScheme};

mod interwiki;
pub use interwiki::{IndexedInterWikiLink, InterWikiLink, NamedInterWikiLink};

mod raw;
pub use raw::RawLink;

mod transclusion;
pub use transclusion::TransclusionLink;

mod wiki;
pub use wiki::WikiLink;

/// Represents a description for a link
#[derive(
    Clone, Debug, From, TryInto, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub enum Description {
    Text(String),
    URL(Url),
}

/// Represents an anchor
#[derive(
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    From,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct Anchor {
    components: Vec<String>,
}

/// Inspecting vimwiki source code, there are a couple of link utils
///
/// 1. s:is_web_link = https | http | www | ftp | file | mailto
/// 2. s:is_img_link = .png | .jpg | .gif | .jpeg
///
/// TRANSCLUSIONS
/// NOTE: Can include additional attributes beyond description
///
/// {{imgurl|arg1|arg2}}         -> ???
/// {{imgurl}}                   -> <img src="imgurl"/>
/// {{imgurl|descr|style="A"}}   -> <img src="imgurl" alt="descr" style="A" />
/// {{imgurl|descr|class="B"}}   -> <img src="imgurl" alt="descr" class="B" />
///
/// WIKILINKS
/// NOTE: According to below, only need to worry about transclusion other than
///       string in a description
///
/// [url]]                       -> <a href="url.html">url</a>
/// [[url|descr]]                -> <a href="url.html">descr</a>
/// [[url|{{...}}]]              -> <a href="url.html"> ... </a>
/// [[fileurl.ext|descr]]        -> <a href="fileurl.ext">descr</a>
/// [[dirurl/|descr]]            -> <a href="dirurl/index.html">descr</a>
/// [[url#a1#a2]]                -> <a href="url.html#a1-a2">url#a1#a2</a>
/// [[#a1#a2]]                   -> <a href="#a1-a2">#a1#a2</a>
///
#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Link {
    Wiki(WikiLink),
    InterWiki(InterWikiLink),
    Diary(DiaryLink),
    Raw(RawLink),
    External(ExternalLink),
    Transclusion(TransclusionLink),
}

impl Link {
    pub fn description(&self) -> Option<&Description> {
        match self {
            Self::Wiki(x) => x.description.as_ref(),
            Self::InterWiki(x) => x.link().description.as_ref(),
            Self::Diary(x) => x.description.as_ref(),
            Self::Raw(_) => None,
            Self::External(x) => x.description.as_ref(),
            Self::Transclusion(x) => x.description.as_ref(),
        }
    }

    pub fn anchor(&self) -> Option<&Anchor> {
        match self {
            Self::Wiki(x) => x.anchor.as_ref(),
            Self::InterWiki(x) => x.link().anchor.as_ref(),
            Self::Diary(x) => x.anchor.as_ref(),
            Self::Raw(_) => None,
            Self::External(_) => None,
            Self::Transclusion(_) => None,
        }
    }
}
