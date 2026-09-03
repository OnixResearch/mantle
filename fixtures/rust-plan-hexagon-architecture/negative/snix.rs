fn leak(value: snix_store::pathinfoservice::PathInfo) {
    drop(value);
}
