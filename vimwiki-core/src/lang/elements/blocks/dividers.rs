use crate::StrictEq;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Divider;

impl StrictEq for Divider {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
