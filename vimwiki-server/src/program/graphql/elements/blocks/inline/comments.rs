use super::Region;
use vimwiki::{elements, Located};

/// Represents a single document comment
#[derive(async_graphql::Union, Debug)]
pub enum Comment {
    Line(LineComment),
    MultiLine(MultiLineComment),
}

#[derive(Debug)]
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

#[derive(Debug)]
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

impl<'a> From<Located<elements::Comment<'a>>> for Comment {
    fn from(le: Located<elements::Comment<'a>>) -> Self {
        let region = Region::from(le.region());
        match le.into_inner() {
            elements::Comment::Line(x) => Self::from(LineComment {
                region,
                line: x.0.to_string(),
            }),
            elements::Comment::MultiLine(x) => Self::from(MultiLineComment {
                region,
                lines: x.0.iter().map(ToString::to_string).collect(),
            }),
        }
    }
}
