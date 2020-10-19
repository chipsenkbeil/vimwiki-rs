use super::Region;
use vimwiki::elements::{self, Located};

#[derive(Debug)]
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

impl<'a> From<Located<elements::Blockquote<'a>>> for Blockquote {
    fn from(le: Located<elements::Blockquote<'a>>) -> Self {
        let region = Region::from(le.region());
        Self {
            region,
            lines: le
                .into_inner()
                .lines
                .iter()
                .map(ToString::to_string)
                .collect(),
        }
    }
}
