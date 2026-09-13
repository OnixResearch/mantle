//! Adapter: materialize one source-root provider from an admitted manifest.
//!
//! The flow owns the scratch directory, the materialization call, the store-name
//! decision, the publication step, and the seed file; the decisions it depends
//! on come from the manifest, materializer, and publication adapters.

use std::path::Path;

use crate::RunError;
use crate::presentation::source_root_provider::SourceRootProviderReport;

/// Materialize one source-root provider and write its seed file.
pub(crate) fn materialize_source_root_provider(
    output: &Path,
    manifest_path: &Path,
    store_dir: &Path,
    verbose: bool,
) -> Result<(), RunError> {
    debug_assert!(!output.as_os_str().is_empty());
    debug_assert!(!manifest_path.as_os_str().is_empty());
    if !store_dir.exists() {
        return Err(RunError::Internal(format!(
            "store directory {} does not exist.\nCreate it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            store_dir.display()
        )));
    }
    let checked = crate::source_root_manifest::read_source_root_manifest(manifest_path)?;
    let scratch = tempfile::Builder::new()
        .prefix("mantle-source-root-")
        .tempdir_in(store_dir)
        .map_err(|err| RunError::Internal(format!("creating source-root scratch in {}: {err}", store_dir.display())))?;
    let provisional_output = scratch.path().join("provider-output");
    let materialized = crate::source_root_provider::materialize_source_root_provider(
        &checked.manifest,
        &checked.manifest_bytes,
        &provisional_output,
        scratch.path(),
        verbose,
    )
    .map_err(|err| RunError::Build(format!("source-root provider materialization failed: {err}")))?;
    if materialized.manifest_digest != checked.manifest_digest {
        return Err(RunError::Internal(
            "source-root provider manifest digest drifted during materialization".to_string(),
        ));
    }
    let store_name = crate::source_root_provider_store_name(&materialized.output_digest)?;
    let final_output = store_dir.join(store_name);
    crate::provider_output_publication::publish_provider_output(&materialized.output_path, &final_output)?;

    let logical_path = final_output.display().to_string();
    let seed_ncl = crate::bootstrap::generate_source_root_seed_ncl(
        &logical_path,
        &checked.manifest_digest,
        &materialized.output_digest,
    );
    crate::generated_file_write::write_generated_text(output, &seed_ncl)?;
    crate::presentation::source_root_provider::emit_source_root_provider_report(&SourceRootProviderReport {
        final_output: &final_output,
        output_path: output,
        manifest_digest: &checked.manifest_digest,
        output_digest: &materialized.output_digest,
        expected_output_role_count: checked.expected_output_role_count,
        dependency_trace_url_count: u32::try_from(materialized.dependency_trace.urls.len())
            .map_err(|_| RunError::Internal("dependency trace url count exceeds u32".to_string()))?,
    })
}
