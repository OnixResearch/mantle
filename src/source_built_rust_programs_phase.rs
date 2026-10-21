//! Source-only compiler phase modules. These strings are admitted as source,
//! compiled by the native provider, and applied to original authenticated Rust
//! trees by the separately produced processless source transformer.
//!
//! They are not replacement configured/compiler inputs and are never executed
//! by preparation. The normal source recipe/executor remains the authority.

pub(crate) const TRANSFORMER_PATH: &str = "cairn/rust/compiler-source-phase.c";
pub(crate) const LINK_MODULE_PATH: &str = "cairn/rust/compiler-native-link.rs";
pub(crate) const ENV_MODULE_PATH: &str = "cairn/rust/compiler-environment.rs";
pub(crate) const OVERLAY_MODULE_PATH: &str = "cairn/rust/compiler-source-overlay.rs";

pub(crate) const TRANSFORMER_SOURCE: &str = include_str!("source_built_rust_programs_phase.c");
pub(crate) const LINK_MODULE_SOURCE: &str = include_str!("source_built_rust_programs_link.rs.in");
pub(crate) const ENV_MODULE_SOURCE: &str = include_str!("source_built_rust_programs_env.rs.in");
pub(crate) const OVERLAY_MODULE_SOURCE: &str = include_str!("source_built_rust_programs_overlay.rs.in");

pub(crate) fn source_files() -> std::collections::BTreeMap<String, String> {
    [
        (TRANSFORMER_PATH, TRANSFORMER_SOURCE),
        (LINK_MODULE_PATH, LINK_MODULE_SOURCE),
        (ENV_MODULE_PATH, ENV_MODULE_SOURCE),
        (OVERLAY_MODULE_PATH, OVERLAY_MODULE_SOURCE),
    ]
    .into_iter()
    .map(|(path, source)| (path.to_string(), source.to_string()))
    .collect()
}
