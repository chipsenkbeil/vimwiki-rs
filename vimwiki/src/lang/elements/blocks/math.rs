use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct MathBlock<'a> {
    pub lines: Vec<Cow<'a, str>>,
    pub environment: Option<Cow<'a, str>>,
}
