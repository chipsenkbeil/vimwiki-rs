use vimwiki::{components::BlockComponent, LC};

#[inline]
pub fn blank_line() -> LC<BlockComponent> {
    LC::from(BlockComponent::BlankLine)
}
