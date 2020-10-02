use super::Region;
use vimwiki::{elements, LC};

#[derive(Debug)]
pub struct MathBlock {
    region: Region,
    lines: Vec<String>,
    environment: Option<String>,
}

/// Represents a single document multi-line math formula
#[async_graphql::Object]
impl MathBlock {
    /// The segment of the document this math block covers
    async fn region(&self) -> &Region {
        &self.region
    }

    /// The lines of content contained within this math block
    async fn lines(&self) -> &[String] {
        &self.lines
    }

    /// The lines joined with " " inbetween
    async fn content(&self) -> String {
        self.lines.join(" ")
    }

    /// The environment associated with this math block
    async fn environment(&self) -> Option<&String> {
        self.environment.as_ref()
    }
}

impl From<LC<elements::MathBlock>> for MathBlock {
    fn from(lc: LC<elements::MathBlock>) -> Self {
        Self {
            region: Region::from(lc.region),
            lines: lc.element.lines,
            environment: lc.element.environment,
        }
    }
}
