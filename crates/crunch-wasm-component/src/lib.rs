mod aot;
mod error;
mod files;
mod materialization;
mod model;
mod octet;
mod pipeline;
mod preflight;
mod process;
mod reporting;
mod stages;
mod toolchain;
mod verification;

pub use error::Error;
pub use files::copy_source_tree;
pub use files::materialize_generated_inputs;
pub use files::sha256_file;
pub use files::validate_relative_path;
pub use files::write_json_new;
pub use materialization::verify_materialization_bundle_files;
pub use model::*;
pub use pipeline::declared_cohort_identity;
pub use pipeline::run_component_pipeline;
pub use process::ToolInvocation;
pub use process::ToolLimits;
pub use process::ToolRun;
pub use process::run_offline_tool;
pub use toolchain::verify_toolchain_manifest;
pub use verification::verify_pipeline_execution_report_files;

#[cfg(all(test, unix))]
mod process_tests;
