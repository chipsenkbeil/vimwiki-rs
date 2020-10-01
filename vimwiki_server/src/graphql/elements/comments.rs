use super::Region;
use vimwiki::{elements, LC};

/// Represents a single document comment
#[derive(async_graphql::Union)]
pub enum Comment {
    Line(LineComment),
    MultiLine(MultiLineComment),
}

pub struct LineComment {
    region: Region,
    line: String,
}

/// Represents a comment on a single line of a document
#[async_graphql::Object]
impl LineComment {
    /// The segment of the document this comment covers
    async fn region(&self) -> &Region {
        &self.region
    }

    /// The line of content contained within this comment
    async fn line(&self) -> &str {
        &self.line
    }

    /// Same as `line`
    async fn content(&self) -> &str {
        &self.line
    }
}

pub struct MultiLineComment {
    region: Region,
    lines: Vec<String>,
}

/// Represents a comment that can potentially cross multiple lines of a document
#[async_graphql::Object]
impl MultiLineComment {
    /// The segment of the document this comment covers
    async fn region(&self) -> &Region {
        &self.region
    }

    /// The lines of content contained within this comment
    async fn lines(&self) -> &[String] {
        &self.lines
    }

    /// The lines joined with " " inbetween
    async fn content(&self) -> String {
        self.lines.join(" ")
    }
}

impl From<LC<elements::Comment>> for Comment {
    fn from(lc: LC<elements::Comment>) -> Self {
        let region = Region::from(lc.region);
        match lc.element {
            elements::Comment::Line(x) => {
                Self::from(LineComment { region, line: x.0 })
            }
            elements::Comment::MultiLine(x) => {
                Self::from(MultiLineComment { region, lines: x.0 })
            }
        }
    }
}
