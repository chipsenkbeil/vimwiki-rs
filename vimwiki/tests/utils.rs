use vimwiki::{components::BlockComponent, LC};

#[inline]
pub fn blank_line() -> LC<BlockComponent> {
    LC::from(BlockComponent::BlankLine)
}

/// Compares top-level block components from a page against an expected set
pub fn compare_page_components(
    actual: &[LC<BlockComponent>],
    expected: &[LC<BlockComponent>],
) {
    // NOTE: Rather than comparing vecs directly, we iterate through the
    //       page components with a zip so we can get finer-grain details on
    //       what and when there is an issue
    for (i, (ac, ec)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(ac, ec, "Components at index {} are not equal!", i);
        assert_eq!(
            ac.region, ec.region,
            "Component regions at index {} are not equal!",
            i
        );
    }

    // NOTE: Because we are not comparing vecs directly, we need to verify at
    //       the end that their sizes match because a zip will work with
    //       uneven vecs, stopping when the first stops
    assert_eq!(
        actual.len(),
        expected.len(),
        "Varying number of top-level page components!"
    );
}
