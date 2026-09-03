fn leak(value: crunch_rust_cache::RustCache) {
    drop(value);
}
