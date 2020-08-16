use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    // TODO: Support a line struct that can comprise one or more inline elements
    //       including plain text, typefaes, links, and more
    //
    //       Elements like headers, tables, other decorations, etc. should
    //       not be included
    lines: Vec<String>,
}
