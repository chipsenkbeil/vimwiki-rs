/// Represents a segment of a document marked by [start, end]
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Region {
    /// The byte offset within a file where this element begins
    offset: i32,

    /// The byte length of this element within a file
    len: i32,
}

impl From<vimwiki::elements::Region> for Region {
    fn from(region: vimwiki::elements::Region) -> Self {
        Self {
            offset: region.offset() as i32,
            len: region.len() as i32,
        }
    }
}
