use super::Region;
use vimwiki::{components, LC};

pub struct Blockquote {
    region: Region,
    lines: Vec<String>,
}

/// Represents a single document blockquote
#[async_graphql::Object]
impl Blockquote {
    /// The segment of the document this blockquote covers
    async fn region(&self) -> &Region {
        &self.region
    }

    /// The lines of content contained within this blockquote
    async fn lines(&self) -> &[String] {
        &self.lines
    }

    /// The lines joined with " " inbetween
    async fn content(&self) -> String {
        self.lines.join(" ")
    }
}

impl From<LC<components::Blockquote>> for Blockquote {
    fn from(lc: LC<components::Blockquote>) -> Self {
        let region = Region::from(lc.region);
        Self {
            region,
            lines: lc.component.lines,
        }
    }
}
