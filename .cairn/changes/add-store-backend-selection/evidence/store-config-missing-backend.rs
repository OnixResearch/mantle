// Negative fixture: compile with crunch-store as an external dependency.
// Rust must reject this initializer specifically for the absent backend field.
use crunch_store::StoreConfig;

fn main() {
    let _ = StoreConfig {
        state_dir: std::path::PathBuf::new(),
        output_dir: std::path::PathBuf::new(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: "/mantle/store".into(),
        base_state_dirs: Vec::new(),
    };
}
