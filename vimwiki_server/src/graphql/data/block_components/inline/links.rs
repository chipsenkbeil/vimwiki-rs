use super::Region;
use vimwiki::{
    components,
    vendor::{chrono::NaiveDate, uriparse::URI},
    LC,
};

#[derive(async_graphql::Union)]
pub enum Link {
    Wiki(WikiLink),
    IndexedInterWiki(IndexedInterWikiLink),
    NamedInterWiki(NamedInterWikiLink),
    Diary(DiaryLink),
    Raw(RawLink),
    ExternalFile(ExternalFileLink),
    Transclusion(TransclusionLink),
}

impl From<LC<components::Link>> for Link {
    fn from(lc: LC<components::Link>) -> Self {
        match lc.component {
            components::Link::Wiki(x) => {
                Self::from(WikiLink::from(LC::new(x, lc.region)))
            }
            components::Link::InterWiki(
                components::InterWikiLink::Indexed(x),
            ) => Self::from(IndexedInterWikiLink::from(LC::new(x, lc.region))),
            components::Link::InterWiki(components::InterWikiLink::Named(
                x,
            )) => Self::from(NamedInterWikiLink::from(LC::new(x, lc.region))),
            components::Link::Diary(x) => {
                Self::from(DiaryLink::from(LC::new(x, lc.region)))
            }
            components::Link::Raw(x) => {
                Self::from(RawLink::from(LC::new(x, lc.region)))
            }
            components::Link::ExternalFile(x) => {
                Self::from(ExternalFileLink::from(LC::new(x, lc.region)))
            }
            components::Link::Transclusion(x) => {
                Self::from(TransclusionLink::from(LC::new(x, lc.region)))
            }
        }
    }
}

/// Represents a single document wiki link
#[derive(async_graphql::SimpleObject)]
pub struct WikiLink {
    /// The segment of the document this link covers
    region: Region,

    /// Whether or not the link connects to a directory
    is_dir: bool,

    /// Whether or not the link is just an anchor to a location
    /// within the current document
    is_local_anchor: bool,

    /// The path the link connects to
    path: String,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Optional anchor associated with the link
    anchor: Option<Anchor>,
}

impl From<LC<components::WikiLink>> for WikiLink {
    fn from(lc: LC<components::WikiLink>) -> Self {
        Self {
            region: Region::from(lc.region),
            is_dir: lc.component.is_path_dir(),
            is_local_anchor: lc.component.is_local_anchor(),
            path: lc.component.path.to_string_lossy().to_string(),
            description: lc.component.description.map(Description::from),
            anchor: lc.component.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document wiki link within another wiki
/// referenced by index
#[derive(async_graphql::SimpleObject)]
pub struct IndexedInterWikiLink {
    /// The segment of the document this link covers
    region: Region,

    /// The index of the wiki this link is associated with
    index: i32,

    /// Whether or not the link connects to a directory
    is_dir: bool,

    /// Whether or not the link is just an anchor to a location
    /// within the current document
    is_local_anchor: bool,

    /// The path the link connects to
    path: String,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Optional anchor associated with the link
    anchor: Option<Anchor>,
}

impl From<LC<components::IndexedInterWikiLink>> for IndexedInterWikiLink {
    fn from(lc: LC<components::IndexedInterWikiLink>) -> Self {
        Self {
            region: Region::from(lc.region),
            index: lc.component.index as i32,
            is_dir: lc.component.link.is_path_dir(),
            is_local_anchor: lc.component.link.is_local_anchor(),
            path: lc.component.link.path.to_string_lossy().to_string(),
            description: lc.component.link.description.map(Description::from),
            anchor: lc.component.link.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document wiki link within another wiki
/// referenced by name
#[derive(async_graphql::SimpleObject)]
pub struct NamedInterWikiLink {
    /// The segment of the document this link covers
    region: Region,

    /// The name of the wiki this link is associated with
    name: String,

    /// Whether or not the link connects to a directory
    is_dir: bool,

    /// Whether or not the link is just an anchor to a location
    /// within the current document
    is_local_anchor: bool,

    /// The path the link connects to
    path: String,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Optional anchor associated with the link
    anchor: Option<Anchor>,
}

impl From<LC<components::NamedInterWikiLink>> for NamedInterWikiLink {
    fn from(lc: LC<components::NamedInterWikiLink>) -> Self {
        Self {
            region: Region::from(lc.region),
            name: lc.component.name,
            is_dir: lc.component.link.is_path_dir(),
            is_local_anchor: lc.component.link.is_local_anchor(),
            path: lc.component.link.path.to_string_lossy().to_string(),
            description: lc.component.link.description.map(Description::from),
            anchor: lc.component.link.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document link to a diary entry
#[derive(async_graphql::SimpleObject)]
pub struct DiaryLink {
    /// The segment of the document this link covers
    region: Region,

    /// Date of diary entry
    date: NaiveDate,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Optional anchor associated with the link
    anchor: Option<Anchor>,
}

impl From<LC<components::DiaryLink>> for DiaryLink {
    fn from(lc: LC<components::DiaryLink>) -> Self {
        Self {
            region: Region::from(lc.region),
            date: lc.component.date,
            description: lc.component.description.map(Description::from),
            anchor: lc.component.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document link to an external file
#[derive(async_graphql::SimpleObject)]
pub struct ExternalFileLink {
    /// The segment of the document this link covers
    region: Region,

    /// Scheme associated with the link
    scheme: ExternalFileLinkScheme,

    /// Path to the local file
    path: String,

    /// Optional description associated with the link
    description: Option<Description>,
}

impl From<LC<components::ExternalFileLink>> for ExternalFileLink {
    fn from(lc: LC<components::ExternalFileLink>) -> Self {
        Self {
            region: Region::from(lc.region),
            scheme: ExternalFileLinkScheme::from(lc.component.scheme),
            path: lc.component.path.to_string_lossy().to_string(),
            description: lc.component.description.map(Description::from),
        }
    }
}

/// Represents the scheme associated with an external file link
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum ExternalFileLinkScheme {
    Local,
    File,
    Absolute,
}

impl From<components::ExternalFileLinkScheme> for ExternalFileLinkScheme {
    fn from(s: components::ExternalFileLinkScheme) -> Self {
        match s {
            components::ExternalFileLinkScheme::Local => Self::Local,
            components::ExternalFileLinkScheme::File => Self::File,
            components::ExternalFileLinkScheme::Absolute => Self::Absolute,
        }
    }
}

/// Represents a single document link formed from a raw URI
#[derive(async_graphql::SimpleObject)]
pub struct RawLink {
    /// The segment of the document this link covers
    region: Region,

    /// The URI representing the link
    uri: Uri,
}

impl From<LC<components::RawLink>> for RawLink {
    fn from(lc: LC<components::RawLink>) -> Self {
        Self {
            region: Region::from(lc.region),
            uri: Uri(lc.component.uri),
        }
    }
}

/// Represents a single document transclusion link
#[derive(async_graphql::SimpleObject)]
pub struct TransclusionLink {
    /// The segment of the document this link covers
    region: Region,

    /// The URI representing the link's content to pull in
    uri: Uri,

    /// Optional description associated with the link
    description: Option<Description>,

    /// Additional properties associated with the link
    properties: Vec<Property>,
}

impl From<LC<components::TransclusionLink>> for TransclusionLink {
    fn from(mut lc: LC<components::TransclusionLink>) -> Self {
        Self {
            region: Region::from(lc.region),
            uri: Uri(lc.component.uri),
            description: lc.component.description.map(Description::from),
            properties: lc
                .component
                .properties
                .drain()
                .map(|(key, value)| Property { key, value })
                .collect(),
        }
    }
}

#[derive(async_graphql::SimpleObject)]
pub struct Property {
    key: String,
    value: String,
}

pub enum Description {
    Text(String),
    URI(Uri),
}

impl From<components::Description> for Description {
    fn from(d: components::Description) -> Self {
        match d {
            components::Description::Text(x) => Self::Text(x),
            components::Description::URI(x) => Self::URI(Uri(x)),
        }
    }
}

/// Represents the description of a link
#[async_graphql::Object]
impl Description {
    /// Represents the content of the description if it is text
    async fn text(&self) -> Option<&String> {
        match self {
            Self::Text(ref x) => Some(x),
            _ => None,
        }
    }

    /// Represents the content of the description if it is a URI
    async fn uri(&self) -> Option<&Uri> {
        match self {
            Self::URI(ref x) => Some(x),
            _ => None,
        }
    }

    /// Represents the content of the description
    async fn content(&self) -> String {
        match self {
            Self::Text(ref x) => x.to_string(),
            Self::URI(ref x) => x.0.to_string(),
        }
    }
}

/// Represents anchor for a link
#[derive(async_graphql::SimpleObject)]
pub struct Anchor {
    /// The pieces of an anchor #one#two#three -> ["one", "two", "three"]
    components: Vec<String>,
}

impl From<components::Anchor> for Anchor {
    fn from(a: components::Anchor) -> Self {
        Self {
            components: a.components,
        }
    }
}

pub struct Uri(URI<'static>);

/// Represents a traditional URI
#[async_graphql::Object]
impl Uri {
    /// The authority portion of the URI, if it exists
    async fn authority(&self) -> Option<String> {
        self.0.authority().map(|x| x.to_string())
    }

    /// The fragment portion of the URI, if it exists
    async fn fragment(&self) -> Option<String> {
        self.0.fragment().map(|x| x.to_string())
    }

    /// The host portion of the URI, if it exists
    async fn host(&self) -> Option<String> {
        self.0.host().map(|x| x.to_string())
    }

    /// The password portion of the URI, if it exists
    async fn password(&self) -> Option<String> {
        self.0.password().map(|x| x.to_string())
    }

    /// The path of the URI
    async fn path(&self) -> String {
        self.0.path().to_string()
    }

    /// The port portion of the URI, if it exists
    async fn port(&self) -> Option<i32> {
        self.0.port().map(|x| x as i32)
    }

    /// The query portion of the URI, if it exists
    async fn query(&self) -> Option<String> {
        self.0.query().map(|x| x.to_string())
    }

    /// The scheme of the URI
    async fn scheme(&self) -> String {
        self.0.scheme().to_string()
    }

    /// The username portion of the URI, if it exists
    async fn username(&self) -> Option<String> {
        self.0.username().map(|x| x.to_string())
    }

    /// The entire URI
    async fn text(&self) -> String {
        self.0.to_string()
    }
}
