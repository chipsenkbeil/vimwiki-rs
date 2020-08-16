use derive_more::From;
use serde::{Deserialize, Serialize};

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

/// Represents support for a description
pub trait WithDescription {
    fn with_description(&mut self, description: String) -> &mut Self;
    fn description(&self) -> Option<&str>;
}

/// Represents support for an anchor
pub trait WithAnchor {
    fn with_anchor(&mut self, anchor: String) -> &mut Self;
    fn anchor(&self) -> Option<&str>;
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Link {
    Wiki(WikiLink),
    InterWiki(InterWikiLink),
    Diary(DiaryLink),
    Raw(RawLink),
    External(ExternalLink),
    Transclusion(TransclusionLink),
}
