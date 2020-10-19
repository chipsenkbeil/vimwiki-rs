use super::Region;
use vimwiki::{elements, Located};

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

impl<'a> From<Located<elements::MathBlock<'a>>> for MathBlock {
    fn from(le: Located<elements::MathBlock<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            lines: element.lines.iter().map(ToString::to_string).collect(),
            environment: element.environment.as_ref().map(ToString::to_string),
        }
    }
}
