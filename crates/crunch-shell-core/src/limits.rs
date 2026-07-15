pub(crate) fn count_with_overflow_marker(count: usize, limit: u32) -> u32 {
    assert!(limit > 0, "collection limit must be positive");
    assert!(limit < u32::MAX, "collection limit must leave room for an overflow marker");

    match u32::try_from(count) {
        Ok(count_u32) => count_u32,
        Err(_) => limit.saturating_add(1),
    }
}
