// Export core at all times
pub use vimwiki_core::*;

// Export macros if specified as a separate module to avoid
// potential name collisions in the future
#[cfg(feature = "macros")]
pub mod macros {
    pub use vimwiki_macros::*;
}
