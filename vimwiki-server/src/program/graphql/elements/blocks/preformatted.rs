use super::Region;
use std::collections::HashMap;
use vimwiki::{elements, Located};

#[derive(Debug)]
pub struct PreformattedText {
    region: Region,
    language: Option<String>,
    metadata: HashMap<String, String>,
    lines: Vec<String>,
}

/// Represents a single document block of preformatted text (aka code block)
#[async_graphql::Object]
impl PreformattedText {
    /// The segment of the document this preformatted text covers
    async fn region(&self) -> &Region {
        &self.region
    }

    /// The lines of content contained within this preformatted text
    async fn lines(&self) -> &[String] {
        &self.lines
    }

    /// The lines joined with " " inbetween
    async fn content(&self) -> String {
        self.lines.join(" ")
    }

    /// The language associated with this preformatted text
    async fn language(&self) -> Option<String> {
        self.language
            .as_ref()
            .map(|x| x.as_str())
            .or_else(|| {
                self.metadata
                    .get("class")
                    .and_then(|x| x.strip_prefix("brush:"))
            })
            .map(|x| x.trim().to_string())
    }

    /// The metadata associated with some key
    async fn metadata_for_key(&self, name: String) -> Option<&String> {
        self.metadata.get(&name)
    }

    /// All metadata associated with the preformatted text
    async fn metadata(&self) -> Vec<Metadata> {
        self.metadata
            .iter()
            .map(|(k, v)| Metadata {
                key: k.to_owned(),
                value: v.to_owned(),
            })
            .collect()
    }
}

impl<'a> From<Located<elements::PreformattedText<'a>>> for PreformattedText {
    fn from(le: Located<elements::PreformattedText<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            language: element.lang.as_ref().map(ToString::to_string),
            metadata: element
                .metadata
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            lines: element.lines.iter().map(ToString::to_string).collect(),
        }
    }
}

#[derive(async_graphql::SimpleObject)]
pub struct Metadata {
    key: String,
    value: String,
}
