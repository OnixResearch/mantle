fn leak(value: snix_store::path_info::PathInfo) {
    drop(value);
}
