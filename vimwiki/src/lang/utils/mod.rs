use nom_locate::LocatedSpan;

mod lc;
pub use lc::{LocatedComponent, LC};
mod position;
pub use position::Position;
mod region;
pub use region::Region;
mod slc;
pub use slc::{StrictLocatedComponent, SLC};

/// Represents input for the parser
pub type Span<'a> = LocatedSpan<&'a str>;
