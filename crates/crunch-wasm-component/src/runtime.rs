use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use crate::CompositionDependencySource;
use crate::Error;
use crate::RuntimeComposition;
use crate::files::write_new;
use crate::preflight::PreparedPipeline;
use crate::reporting::ExecutionState;
use crate::stages::StageInvocation;
use crate::stages::StageWorkspace;
use crate::stages::invoke;
use crate::stages::publish_evidence_artifact;
use crate::toolchain::hash_file_bounded;

const MAX_RUNTIME_DEPENDENCIES: usize = 32;
const MAX_RUNTIME_WAC_BYTES: usize = 65_536;

// r[impl materialize_wrpc_producer.runtime_binding]
pub(crate) fn compose_runtime(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    artifact: &Path,
) -> Result<PathBuf, Error> {
    let Some(plan) = &prepared.request.runtime_composition else {
        return Ok(artifact.to_path_buf());
    };
    admit_plan_shape(plan)?;
    let source = workspace.root.join("runtime-composition.wac");
    write_new(&source, plan.source_wac.as_bytes())?;
    let object = publish_evidence_artifact(&source, "runtime-composition.wac", workspace)?;
    state.add_artifact("runtime-composition-source", &object)?;
    let output = workspace.root.join("runtime-composed.wasm");
    let mut args = vec!["compose".to_string(), source.display().to_string()];
    args.extend(dependency_args(plan, artifact)?);
    args.extend(["--output".to_string(), output.display().to_string()]);
    debug_assert!(source.is_file());
    debug_assert!(!output.exists());
    execute_composition(prepared, workspace, state, args, output)
}

fn admit_plan_shape(plan: &RuntimeComposition) -> Result<(), Error> {
    let shape_error = || Error::Invalid("runtime composition exceeds bounded source/dependency shape".to_string());
    if plan.source_wac.is_empty() {
        return Err(shape_error());
    }
    if plan.source_wac.len() > MAX_RUNTIME_WAC_BYTES {
        return Err(shape_error());
    }
    if plan.dependencies.is_empty() {
        return Err(shape_error());
    }
    if plan.dependencies.len() > MAX_RUNTIME_DEPENDENCIES {
        return Err(shape_error());
    }
    debug_assert!(plan.source_wac.len() <= MAX_RUNTIME_WAC_BYTES);
    debug_assert!(!plan.dependencies.is_empty());
    Ok(())
}

/// Binds the exact final component once and identity-checks every provider.
fn dependency_args(plan: &RuntimeComposition, artifact: &Path) -> Result<Vec<String>, Error> {
    let mut args = Vec::with_capacity(plan.dependencies.len().saturating_mul(2));
    let mut names = BTreeSet::new();
    let mut final_bindings = 0_u32;
    for dependency in &plan.dependencies {
        if dependency.package.is_empty() || !names.insert(&dependency.package) {
            return Err(Error::Invalid("runtime composition has empty or duplicate package".to_string()));
        }
        let path = match &dependency.source {
            CompositionDependencySource::CompiledComponent => {
                final_bindings = final_bindings.saturating_add(1);
                artifact.to_path_buf()
            }
            CompositionDependencySource::StoreObject { path, digest_blake3 } => {
                let path = PathBuf::from(path);
                if !path.is_absolute() || hash_file_bounded(&path)? != *digest_blake3 {
                    return Err(Error::Invalid("runtime composition dependency identity mismatch".to_string()));
                }
                path
            }
        };
        args.extend([
            "--dep".to_string(),
            format!("{}={}", dependency.package, path.display()),
        ]);
    }
    if final_bindings != 1 {
        return Err(Error::Invalid("runtime composition must bind the exact final component once".to_string()));
    }
    debug_assert_eq!(names.len(), plan.dependencies.len());
    debug_assert_eq!(args.len(), plan.dependencies.len().saturating_mul(2));
    Ok(args)
}

fn execute_composition(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    args: Vec<String>,
    output: PathBuf,
) -> Result<PathBuf, Error> {
    let run = invoke(prepared, workspace, StageInvocation {
        stage_key: "runtime-composition",
        tool_name: "wac",
        args,
        environment_overrides: BTreeMap::new(),
        tool_dependencies: Vec::new(),
        output_path: Some(output.clone()),
    })?;
    state.add_receipt(run.receipt)?;
    if !run.success {
        return Err(Error::Tool("runtime-only WAC composition failed".to_string()));
    }
    let object = publish_evidence_artifact(&output, "runtime-composed.wasm", workspace)?;
    state.add_artifact("runtime-composed-component", &object)?;
    debug_assert!(output.is_file());
    debug_assert!(object.size_bytes > 0);
    Ok(output)
}
