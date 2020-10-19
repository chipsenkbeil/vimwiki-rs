use super::Region;
use vimwiki::{
    elements::{self, Located},
    vendor::{chrono::NaiveDate, uriparse::URI},
};

#[derive(async_graphql::Union, Debug)]
pub enum Link {
    Wiki(WikiLink),
    IndexedInterWiki(IndexedInterWikiLink),
    NamedInterWiki(NamedInterWikiLink),
    Diary(DiaryLink),
    Raw(RawLink),
    ExternalFile(ExternalFileLink),
    Transclusion(TransclusionLink),
}

impl<'a> From<Located<elements::Link<'a>>> for Link {
    fn from(le: Located<elements::Link<'a>>) -> Self {
        let region = le.region();
        match le.into_inner() {
            elements::Link::Wiki(x) => {
                Self::from(WikiLink::from(Located::new(x, region)))
            }
            elements::Link::InterWiki(elements::InterWikiLink::Indexed(x)) => {
                Self::from(IndexedInterWikiLink::from(Located::new(x, region)))
            }
            elements::Link::InterWiki(elements::InterWikiLink::Named(x)) => {
                Self::from(NamedInterWikiLink::from(Located::new(x, region)))
            }
            elements::Link::Diary(x) => {
                Self::from(DiaryLink::from(Located::new(x, region)))
            }
            elements::Link::Raw(x) => {
                Self::from(RawLink::from(Located::new(x, region)))
            }
            elements::Link::ExternalFile(x) => {
                Self::from(ExternalFileLink::from(Located::new(x, region)))
            }
            elements::Link::Transclusion(x) => {
                Self::from(TransclusionLink::from(Located::new(x, region)))
            }
        }
    }
}

/// Represents a single document wiki link
#[derive(async_graphql::SimpleObject, Debug)]
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

impl<'a> From<Located<elements::WikiLink<'a>>> for WikiLink {
    fn from(le: Located<elements::WikiLink<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            is_dir: element.is_path_dir(),
            is_local_anchor: element.is_local_anchor(),
            path: element.path.to_string_lossy().to_string(),
            description: element.description.map(Description::from),
            anchor: element.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document wiki link within another wiki
/// referenced by index
#[derive(async_graphql::SimpleObject, Debug)]
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

impl<'a> From<Located<elements::IndexedInterWikiLink<'a>>>
    for IndexedInterWikiLink
{
    fn from(le: Located<elements::IndexedInterWikiLink<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            index: element.index as i32,
            is_dir: element.link.is_path_dir(),
            is_local_anchor: element.link.is_local_anchor(),
            path: element.link.path.to_string_lossy().to_string(),
            description: element.link.description.map(Description::from),
            anchor: element.link.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document wiki link within another wiki
/// referenced by name
#[derive(async_graphql::SimpleObject, Debug)]
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

impl<'a> From<Located<elements::NamedInterWikiLink<'a>>>
    for NamedInterWikiLink
{
    fn from(le: Located<elements::NamedInterWikiLink<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            name: element.name.to_string(),
            is_dir: element.link.is_path_dir(),
            is_local_anchor: element.link.is_local_anchor(),
            path: element.link.path.to_string_lossy().to_string(),
            description: element.link.description.map(Description::from),
            anchor: element.link.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document link to a diary entry
#[derive(async_graphql::SimpleObject, Debug)]
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

impl<'a> From<Located<elements::DiaryLink<'a>>> for DiaryLink {
    fn from(le: Located<elements::DiaryLink<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            date: element.date,
            description: element.description.map(Description::from),
            anchor: element.anchor.map(Anchor::from),
        }
    }
}

/// Represents a single document link to an external file
#[derive(async_graphql::SimpleObject, Debug)]
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

impl<'a> From<Located<elements::ExternalFileLink<'a>>> for ExternalFileLink {
    fn from(le: Located<elements::ExternalFileLink<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            scheme: ExternalFileLinkScheme::from(element.scheme),
            path: element.path.to_string_lossy().to_string(),
            description: element.description.map(Description::from),
        }
    }
}

/// Represents the scheme associated with an external file link
#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExternalFileLinkScheme {
    Local,
    File,
    Absolute,
}

impl From<elements::ExternalFileLinkScheme> for ExternalFileLinkScheme {
    fn from(s: elements::ExternalFileLinkScheme) -> Self {
        match s {
            elements::ExternalFileLinkScheme::Local => Self::Local,
            elements::ExternalFileLinkScheme::File => Self::File,
            elements::ExternalFileLinkScheme::Absolute => Self::Absolute,
        }
    }
}

/// Represents a single document link formed from a raw URI
#[derive(async_graphql::SimpleObject, Debug)]
pub struct RawLink {
    /// The segment of the document this link covers
    region: Region,

    /// The URI representing the link
    uri: Uri,
}

impl<'a> From<Located<elements::RawLink<'a>>> for RawLink {
    fn from(le: Located<elements::RawLink<'a>>) -> Self {
        let region = Region::from(le.region());
        Self {
            region,
            uri: Uri::from(le.into_inner().uri),
        }
    }
}

/// Represents a single document transclusion link
#[derive(async_graphql::SimpleObject, Debug)]
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

impl<'a> From<Located<elements::TransclusionLink<'a>>> for TransclusionLink {
    fn from(le: Located<elements::TransclusionLink<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            uri: Uri::from(element.uri),
            description: element.description.map(Description::from),
            properties: element
                .properties
                .into_iter()
                .map(|(key, value)| Property {
                    key: key.to_string(),
                    value: value.to_string(),
                })
                .collect(),
        }
    }
}

#[derive(async_graphql::SimpleObject, Debug)]
pub struct Property {
    key: String,
    value: String,
}

#[derive(Debug)]
pub enum Description {
    Text(String),
    URI(Uri),
}

impl<'a> From<elements::Description<'a>> for Description {
    fn from(d: elements::Description<'a>) -> Self {
        match d {
            elements::Description::Text(x) => Self::Text(x.to_string()),
            elements::Description::URI(x) => Self::URI(Uri::from(x)),
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
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Anchor {
    /// The pieces of an anchor #one#two#three -> ["one", "two", "three"]
    elements: Vec<String>,
}

impl<'a> From<elements::Anchor<'a>> for Anchor {
    fn from(a: elements::Anchor<'a>) -> Self {
        Self {
            elements: a.elements.iter().map(ToString::to_string).collect(),
        }
    }
}

#[derive(Debug)]
pub struct Uri(URI<'static>);

impl<'a> From<URI<'a>> for Uri {
    fn from(uri: URI<'a>) -> Self {
        Self(uri.into_owned())
    }
}

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
