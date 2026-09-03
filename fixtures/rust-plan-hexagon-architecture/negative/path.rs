fn leak(value: std::path::PathBuf) {
    drop(value);
}
