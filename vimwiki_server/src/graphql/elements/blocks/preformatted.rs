use super::Region;
use std::collections::HashMap;
use vimwiki::{elements, LC};

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

impl From<LC<elements::PreformattedText>> for PreformattedText {
    fn from(lc: LC<elements::PreformattedText>) -> Self {
        Self {
            region: Region::from(lc.region),
            language: lc.element.lang,
            metadata: lc.element.metadata,
            lines: lc.element.lines,
        }
    }
}

#[derive(async_graphql::SimpleObject)]
pub struct Metadata {
    key: String,
    value: String,
}
