use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component_core::BoundedComponentClaim;
use crunch_wasm_component_core::BuildValidationBinding;
use crunch_wasm_component_core::ComponentStageKind;
use crunch_wasm_component_core::ComponentStageStatus;
use crunch_wasm_component_core::StoreObject;
use crunch_wasm_component_core::ValidationDecision;
use crunch_wasm_component_core::VirtualizationAction;
use crunch_wasm_component_core::VirtualizationPlan;
use crunch_wasm_component_core::WasiSubsystem;
use crunch_wasm_component_core::plan_virtualization;
use crunch_wasm_component_core::validate_remaining_imports;

use crate::CompositionDependencySource;
use crate::Error;
use crate::ToolInvocation;
use crate::VerifiedToolchain;
use crate::copy_source_tree;
use crate::files::copy_regular_file_new;
use crate::files::read_source_file_bounded;
use crate::materialize_generated_inputs;
use crate::preflight::PreparedPipeline;
use crate::preflight::core_blockers;
use crate::reporting::ExecutionState;
use crate::run_offline_tool;
use crate::toolchain::hash_file_bounded;

const GENERATED_WKG_CONFIG: &str = ".mantle/wasm-component/wkg-config.toml";
const GENERATED_WAC_SOURCE: &str = ".mantle/wasm-component/composition.wac";
const MAX_EXPECTED_RUNTIME_STDOUT_BYTES: usize = 1024 * 1024;
const MAX_WKG_LOCK_BYTES: u64 = 1024 * 1024;

pub(crate) struct StageWorkspace {
    pub root: PathBuf,
    pub source: PathBuf,
    pub publication_root: PathBuf,
    pub final_output: PathBuf,
}

pub(crate) fn create_workspace(
    root: PathBuf,
    publication_root: PathBuf,
    final_output: PathBuf,
) -> Result<StageWorkspace, Error> {
    let source = root.join("source");
    fs::create_dir_all(publication_root.join("artifacts")).map_err(|error| {
        Error::io("creating component publication artifact directory", &publication_root.join("artifacts"), error)
    })?;
    fs::create_dir_all(root.join("home"))
        .map_err(|error| Error::io("creating component home", &root.join("home"), error))?;
    fs::create_dir_all(root.join("cargo-home"))
        .map_err(|error| Error::io("creating component cargo home", &root.join("cargo-home"), error))?;
    fs::create_dir_all(root.join("target"))
        .map_err(|error| Error::io("creating component target", &root.join("target"), error))?;
    debug_assert!(root.is_absolute());
    debug_assert!(final_output.is_absolute());
    Ok(StageWorkspace {
        root,
        source,
        publication_root,
        final_output,
    })
}

pub(crate) fn stage_source_and_inputs(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
) -> Result<(), Error> {
    let measured_source = copy_source_tree(Path::new(&prepared.request.source_root), &workspace.source)?;
    if measured_source.digest_blake3 != prepared.request.manifest.implementation.source.digest_blake3
        || measured_source.size_bytes != prepared.request.manifest.implementation.source.size_bytes
    {
        return Err(Error::Invalid("implementation source closure identity drifted before execution".to_string()));
    }
    materialize_generated_inputs(&workspace.source, &prepared.generated_plan)?;
    let staged_lock = workspace.source.join(&prepared.request.manifest.package_resolution.lock_path);
    let lock_object = publish_evidence_artifact(&staged_lock, "wkg.lock", workspace)?;
    state.add_artifact("checked-wkg-lock", &lock_object)?;
    state.push_stage(
        "checked-lock",
        ComponentStageKind::Lock,
        ComponentStageStatus::Succeeded,
        Some(lock_object),
        None,
        Some(prepared.source_plan.plan_identity_blake3.clone()),
        vec![BoundedComponentClaim::ExactInputIdentities],
    )?;
    debug_assert!(workspace.source.is_dir());
    debug_assert!(staged_lock.is_file());
    Ok(())
}

pub(crate) fn run_wkg_stage(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
) -> Result<bool, Error> {
    let cache = workspace.root.join("wkg-cache");
    fs::create_dir(&cache).map_err(|error| Error::io("creating wkg cache", &cache, error))?;
    let args = vec![
        "wit".to_string(),
        "fetch".to_string(),
        "--config".to_string(),
        workspace.source.join(GENERATED_WKG_CONFIG).display().to_string(),
        "--cache".to_string(),
        cache.display().to_string(),
        "--wit-dir".to_string(),
        workspace.source.join(&prepared.request.wit_relative_path).display().to_string(),
        "--type".to_string(),
        "wasm".to_string(),
    ];
    let run = invoke(prepared, workspace, "package-resolution", "wkg", args, None)?;
    state.add_receipt(run.receipt.clone())?;
    let status = stage_status(run.success);
    state.push_stage(
        "package-resolution",
        ComponentStageKind::PackageResolution,
        status,
        prepared.request.package_materializations.first().map(|item| item.object.clone()),
        Some(prepared.toolchain.tool_digest("wkg")?),
        Some(prepared.source_plan.plan_identity_blake3.clone()),
        vec![BoundedComponentClaim::ExactInputIdentities],
    )?;
    if !run.success {
        state.block("wkg-execution-failed", "package-resolution", stderr_summary(&run.stderr));
        return Ok(false);
    }
    let staged_lock = workspace.source.join(&prepared.request.manifest.package_resolution.lock_path);
    let observed = read_source_file_bounded(&staged_lock, MAX_WKG_LOCK_BYTES, "staged wkg.lock")?;
    if observed != prepared.lock_bytes {
        state.block("wkg-lock-drift", "package-resolution", "wkg changed the checked lock during resolution");
        return Ok(false);
    }
    verify_wkg_outputs(prepared, workspace, state)
}

pub(crate) fn run_binding_stage(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
) -> Result<bool, Error> {
    let output = workspace.root.join("generated-bindings");
    fs::create_dir(&output).map_err(|error| Error::io("creating binding output", &output, error))?;
    let args = vec![
        "rust".to_string(),
        "--world".to_string(),
        prepared.request.manifest.wit.world.clone(),
        "--generate-all".to_string(),
        "--out-dir".to_string(),
        output.display().to_string(),
        workspace.source.join(&prepared.request.wit_relative_path).display().to_string(),
    ];
    let run = invoke(prepared, workspace, "binding-generation", "wit-bindgen", args, None)?;
    state.add_receipt(run.receipt.clone())?;
    state.push_stage(
        "binding-generation",
        ComponentStageKind::BindingGeneration,
        stage_status(run.success),
        None,
        Some(prepared.toolchain.tool_digest("wit-bindgen")?),
        None,
        vec![BoundedComponentClaim::ExactInputIdentities],
    )?;
    if !run.success {
        state.block("wit-bindgen-failed", "binding-generation", stderr_summary(&run.stderr));
    }
    debug_assert!(!run.success || output.is_dir());
    debug_assert!(!run.success || fs::read_dir(&output).is_ok());
    Ok(run.success)
}

pub(crate) fn run_compilation_stage(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
) -> Result<Option<(PathBuf, StoreObject)>, Error> {
    let mut args = vec!["build".to_string(), "--locked".to_string(), "--offline".to_string()];
    if prepared.request.manifest.implementation.profile == crunch_wasm_component_core::RustProfile::Release {
        args.push("--release".to_string());
    }
    args.extend([
        "--target".to_string(),
        "wasm32-wasip2".to_string(),
        "--package".to_string(),
    ]);
    args.push(prepared.request.manifest.implementation.package.clone());
    if !prepared.request.manifest.implementation.features.is_empty() {
        args.extend([
            "--features".to_string(),
            prepared.request.manifest.implementation.features.join(","),
        ]);
    }
    let component = workspace.root.join("target").join(&prepared.request.cargo_component_relative_path);
    let component_parent = component
        .parent()
        .ok_or_else(|| Error::Invalid("compiled component output has no parent directory".to_string()))?;
    fs::create_dir_all(component_parent)
        .map_err(|error| Error::io("creating compiled component output parent", component_parent, error))?;
    let run = invoke(prepared, workspace, "compilation", "cargo", args, Some(component.clone()))?;
    state.add_receipt(run.receipt.clone())?;
    if !run.success {
        state.push_stage(
            "compilation",
            ComponentStageKind::Compilation,
            ComponentStageStatus::Failed,
            None,
            Some(prepared.toolchain.tool_digest("rustc")?),
            None,
            vec![BoundedComponentClaim::ExactInputIdentities],
        )?;
        state.block("component-compilation-failed", "compilation", stderr_summary(&run.stderr));
        return Ok(None);
    }
    let object = measure_artifact(&component, "compiled.wasm", workspace)?;
    state.push_stage(
        "compilation",
        ComponentStageKind::Compilation,
        ComponentStageStatus::Succeeded,
        Some(object.clone()),
        Some(prepared.toolchain.tool_digest("rustc")?),
        None,
        vec![BoundedComponentClaim::ExactInputIdentities],
    )?;
    Ok(Some((component, object)))
}

pub(crate) fn run_composition_stage(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    compiled_path: &Path,
) -> Result<Option<(PathBuf, StoreObject)>, Error> {
    let dependencies = composition_dependencies(prepared, compiled_path)?;
    let output = workspace.root.join("composed.wasm");
    let mut args = vec![
        "compose".to_string(),
        workspace.source.join(GENERATED_WAC_SOURCE).display().to_string(),
    ];
    for (package, path) in dependencies {
        args.extend(["--dep".to_string(), format!("{package}={}", path.display())]);
    }
    args.extend(["--output".to_string(), output.display().to_string()]);
    let run = invoke(prepared, workspace, "composition", "wac", args, Some(output.clone()))?;
    state.add_receipt(run.receipt.clone())?;
    if !run.success {
        state.push_stage(
            "composition",
            ComponentStageKind::Composition,
            ComponentStageStatus::Failed,
            None,
            Some(prepared.toolchain.tool_digest("wac")?),
            None,
            vec![BoundedComponentClaim::ExactInputIdentities],
        )?;
        state.block("wac-composition-failed", "composition", stderr_summary(&run.stderr));
        return Ok(None);
    }
    let object = measure_artifact(&output, "composed.wasm", workspace)?;
    state.push_stage(
        "composition",
        ComponentStageKind::Composition,
        ComponentStageStatus::Succeeded,
        Some(object.clone()),
        Some(prepared.toolchain.tool_digest("wac")?),
        None,
        vec![BoundedComponentClaim::ExactInputIdentities],
    )?;
    Ok(Some((output, object)))
}

pub(crate) fn run_validation_stage(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    stage_key: &str,
    artifact_path: &Path,
    artifact: &StoreObject,
) -> Result<Option<BuildValidationBinding>, Error> {
    let args = vec!["validate".to_string(), artifact_path.display().to_string()];
    let run = invoke(prepared, workspace, stage_key, "wasm-tools", args, None)?;
    let report_blake3 = run.receipt.receipt_blake3.clone();
    state.add_receipt(run.receipt.clone())?;
    state.push_stage(
        stage_key,
        ComponentStageKind::BuildValidation,
        stage_status(run.success),
        Some(artifact.clone()),
        Some(prepared.toolchain.tool_digest("wasm-tools")?),
        None,
        if run.success {
            vec![BoundedComponentClaim::PortableBytesValidated]
        } else {
            Vec::new()
        },
    )?;
    if !run.success {
        state.block("wasm-tools-validation-failed", stage_key, stderr_summary(&run.stderr));
        return Ok(None);
    }
    let binding = BuildValidationBinding {
        artifact_blake3: artifact.digest_blake3.clone(),
        cohort_blake3: prepared.toolchain.manifest.cohort_identity_blake3.clone(),
        report_blake3,
        decision: ValidationDecision::Pass,
    };
    debug_assert_eq!(binding.artifact_blake3, artifact.digest_blake3);
    debug_assert_eq!(binding.decision, ValidationDecision::Pass);
    Ok(Some(binding))
}

pub(crate) fn run_virtualization_stage(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    composed_path: &Path,
) -> Result<Option<(PathBuf, StoreObject)>, Error> {
    let plan_result = plan_virtualization(prepared.request.manifest.virtualization.clone());
    let Some(plan) = plan_result.plan else {
        return Err(core_blockers("virtualization", &plan_result.blockers));
    };
    let output = workspace.root.join("virtualized.wasm");
    let Some(input_imports) =
        read_component_imports(prepared, workspace, state, "virtualization-input-inspection", composed_path)?
    else {
        return Ok(None);
    };
    if input_imports.is_empty() {
        let validation = validate_remaining_imports(plan.clone(), input_imports);
        if !validation.matches_plan {
            for blocker in validation.blockers {
                state.block(&blocker.code, "virtualization", blocker.message);
            }
            return Ok(None);
        }
        copy_regular_file_new(composed_path, &output, "import-free virtualization output")?;
        let object = measure_artifact(&output, "virtualized.wasm", workspace)?;
        state.push_stage(
            "virtualization",
            ComponentStageKind::Virtualization,
            ComponentStageStatus::Succeeded,
            Some(object.clone()),
            None,
            Some(plan.identity_blake3),
            vec![BoundedComponentClaim::DenyAllVirtualizationPlanned],
        )?;
        debug_assert!(output.is_file());
        debug_assert_eq!(object.digest_blake3, hash_file_bounded(composed_path)?);
        return Ok(Some((output, object)));
    }
    let args = virtualization_args(&plan, composed_path, &output)?;
    let run = invoke(prepared, workspace, "virtualization", "wasi-virt", args, Some(output.clone()))?;
    state.add_receipt(run.receipt.clone())?;
    if !run.success {
        state.push_stage(
            "virtualization",
            ComponentStageKind::Virtualization,
            ComponentStageStatus::Failed,
            None,
            Some(prepared.toolchain.tool_digest("wasi-virt")?),
            Some(plan.identity_blake3),
            vec![BoundedComponentClaim::DenyAllVirtualizationPlanned],
        )?;
        state.block("wasi-virt-failed", "virtualization", stderr_summary(&run.stderr));
        return Ok(None);
    }
    let object = measure_artifact(&output, "virtualized.wasm", workspace)?;
    if !inspect_remaining_imports(prepared, workspace, state, &plan, &output)? {
        return Ok(None);
    }
    state.push_stage(
        "virtualization",
        ComponentStageKind::Virtualization,
        ComponentStageStatus::Succeeded,
        Some(object.clone()),
        Some(prepared.toolchain.tool_digest("wasi-virt")?),
        Some(plan.identity_blake3),
        vec![BoundedComponentClaim::DenyAllVirtualizationPlanned],
    )?;
    Ok(Some((output, object)))
}

pub(crate) fn run_metadata_normalization_stage(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    artifact_path: &Path,
) -> Result<Option<(PathBuf, StoreObject)>, Error> {
    let output = workspace.root.join("normalized.wasm");
    let args = vec![
        "strip".to_string(),
        artifact_path.display().to_string(),
        "-o".to_string(),
        output.display().to_string(),
    ];
    let run = invoke(prepared, workspace, "metadata-normalization", "wasm-tools", args, Some(output.clone()))?;
    state.add_receipt(run.receipt)?;
    if !run.success {
        state.push_stage(
            "metadata-normalization",
            ComponentStageKind::MetadataNormalization,
            ComponentStageStatus::Failed,
            None,
            Some(prepared.toolchain.tool_digest("wasm-tools")?),
            None,
            Vec::new(),
        )?;
        state.block(
            "wasm-metadata-normalization-failed",
            "metadata-normalization",
            "pinned wasm-tools did not produce the profile-normalized component",
        );
        return Ok(None);
    }
    let object = measure_artifact(&output, "normalized.wasm", workspace)?;
    state.push_stage(
        "metadata-normalization",
        ComponentStageKind::MetadataNormalization,
        ComponentStageStatus::Succeeded,
        Some(object.clone()),
        Some(prepared.toolchain.tool_digest("wasm-tools")?),
        None,
        vec![BoundedComponentClaim::ExactInputIdentities],
    )?;
    debug_assert!(output.is_file());
    debug_assert!(object.size_bytes > 0);
    Ok(Some((output, object)))
}

pub(crate) fn run_runtime_smoke(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    artifact_path: &Path,
    artifact: &StoreObject,
) -> Result<bool, Error> {
    let mut args = vec!["run".to_string()];
    if let Some(runtime_invoke) = &prepared.request.runtime_invoke {
        args.extend(["--invoke".to_string(), runtime_invoke.clone()]);
    }
    args.push(artifact_path.display().to_string());
    let run = invoke(prepared, workspace, "runtime-smoke", "wasmtime", args, None)?;
    state.add_receipt(run.receipt.clone())?;
    let expected = prepared.request.expected_runtime_stdout.as_bytes();
    let matches = run.success && run.stdout == expected;
    state.push_stage(
        "runtime-smoke",
        ComponentStageKind::BuildValidation,
        if matches {
            ComponentStageStatus::Succeeded
        } else {
            ComponentStageStatus::Failed
        },
        Some(artifact.clone()),
        Some(prepared.toolchain.tool_digest("wasmtime")?),
        Some(prepared.request.manifest.validation_profiles.expected_runtime_profile_identity_blake3.clone()),
        vec![BoundedComponentClaim::ExactInputIdentities],
    )?;
    if !run.success {
        state.block("wasmtime-smoke-failed", "runtime-smoke", stderr_summary(&run.stderr));
    } else if run.stdout != expected {
        state.block(
            "wasmtime-stdout-mismatch",
            "runtime-smoke",
            format!("runtime stdout digest differed; observed {} bytes", run.stdout.len()),
        );
    }
    debug_assert!(expected.len() <= MAX_EXPECTED_RUNTIME_STDOUT_BYTES);
    debug_assert_eq!(matches, run.success && run.stdout == expected);
    Ok(matches)
}

fn inspect_remaining_imports(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    plan: &VirtualizationPlan,
    output: &Path,
) -> Result<bool, Error> {
    let Some(observed) =
        read_component_imports(prepared, workspace, state, "virtualization-import-inspection", output)?
    else {
        return Ok(false);
    };
    let validation = validate_remaining_imports(plan.clone(), observed);
    if !validation.matches_plan {
        for blocker in validation.blockers {
            state.block(&blocker.code, "virtualization", blocker.message);
        }
    }
    Ok(validation.matches_plan)
}

fn read_component_imports(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    stage_key: &str,
    component: &Path,
) -> Result<Option<Vec<String>>, Error> {
    let args = vec![
        "component".to_string(),
        "wit".to_string(),
        component.display().to_string(),
    ];
    let run = invoke(prepared, workspace, stage_key, "wasm-tools", args, None)?;
    state.add_receipt(run.receipt.clone())?;
    if !run.success {
        state.block("virtualization-inspection-failed", "virtualization", stderr_summary(&run.stderr));
        return Ok(None);
    }
    let wit = String::from_utf8(run.stdout)
        .map_err(|error| Error::Tool(format!("wasm-tools WIT inspection was not UTF-8: {error}")))?;
    let imports = wit.lines().filter_map(parse_import_line).collect();
    debug_assert!(stage_key.starts_with("virtualization-"));
    debug_assert!(component.is_absolute());
    Ok(Some(imports))
}

fn parse_import_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let value = trimmed.strip_prefix("import ")?.strip_suffix(';')?;
    if value.is_empty() || value.contains(char::is_whitespace) {
        return None;
    }
    Some(value.to_string())
}

fn virtualization_args(plan: &VirtualizationPlan, input: &Path, output: &Path) -> Result<Vec<String>, Error> {
    let mut args = vec!["--wasi-version".to_string(), "0.2.3".to_string()];
    for entry in &plan.entries {
        append_virtualization_action(&mut args, entry.subsystem, &entry.action)?;
    }
    args.extend([
        "--out".to_string(),
        output.display().to_string(),
        input.display().to_string(),
    ]);
    debug_assert!(!args.iter().any(|arg| arg == "--allow-all"));
    debug_assert!(args.iter().any(|arg| arg == "--wasi-version"));
    Ok(args)
}

fn append_virtualization_action(
    args: &mut Vec<String>,
    subsystem: WasiSubsystem,
    action: &VirtualizationAction,
) -> Result<(), Error> {
    match action {
        VirtualizationAction::Deny => Ok(()),
        VirtualizationAction::Allow { .. } | VirtualizationAction::Passthrough { .. } => {
            args.push(allow_flag(subsystem)?.to_string());
            Ok(())
        }
        VirtualizationAction::Ignore { .. } if subsystem == WasiSubsystem::Stdio => {
            args.push("--stdio=ignore".to_string());
            Ok(())
        }
        VirtualizationAction::FixedValue { value, .. } if subsystem == WasiSubsystem::Environment => {
            args.extend(["--env".to_string(), value.clone()]);
            Ok(())
        }
        VirtualizationAction::VirtualMount { guest_path, input } if subsystem == WasiSubsystem::Filesystem => {
            args.extend(["--mount".to_string(), format!("{guest_path}={}", input.logical_path)]);
            Ok(())
        }
        _ => Err(Error::Blocked(format!(
            "WASI-Virt CLI cannot represent declared {:?} action {:?} in the pinned cohort",
            subsystem, action
        ))),
    }
}

fn allow_flag(subsystem: WasiSubsystem) -> Result<&'static str, Error> {
    match subsystem {
        WasiSubsystem::Cli => Ok("--allow-exit=true"),
        WasiSubsystem::Clocks => Ok("--allow-clocks=true"),
        WasiSubsystem::Environment => Ok("--allow-env"),
        WasiSubsystem::Filesystem => Ok("--allow-fs=true"),
        WasiSubsystem::Http => Ok("--allow-http=true"),
        WasiSubsystem::Random => Ok("--allow-random=true"),
        WasiSubsystem::Sockets | WasiSubsystem::Network => Ok("--allow-sockets=true"),
        WasiSubsystem::Stdio => Ok("--allow-stdio=true"),
        WasiSubsystem::Poll => {
            Err(Error::Blocked("pinned WASI-Virt CLI cannot independently admit the poll subsystem".to_string()))
        }
    }
}

fn composition_dependencies(prepared: &PreparedPipeline, compiled: &Path) -> Result<BTreeMap<String, PathBuf>, Error> {
    let mut dependencies = BTreeMap::new();
    for dependency in &prepared.request.composition_dependencies {
        let path = match &dependency.source {
            CompositionDependencySource::CompiledComponent => compiled.to_path_buf(),
            CompositionDependencySource::StoreObject { path, digest_blake3 } => {
                let path = PathBuf::from(path);
                if hash_file_bounded(&path)? != *digest_blake3 {
                    return Err(Error::Invalid(format!(
                        "composition dependency `{}` digest drifted",
                        dependency.package
                    )));
                }
                path
            }
        };
        if dependencies.insert(dependency.package.clone(), path).is_some() {
            return Err(Error::Invalid(format!("composition dependency `{}` is duplicated", dependency.package)));
        }
    }
    let expected: BTreeSet<String> =
        prepared.request.manifest.composition.nodes.iter().map(|node| node.package.clone()).collect();
    let actual: BTreeSet<String> = dependencies.keys().cloned().collect();
    if actual != expected {
        return Err(Error::Invalid(
            "composition dependency bindings do not exactly match the validated WAC graph".to_string(),
        ));
    }
    Ok(dependencies)
}

fn verify_wkg_outputs(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
) -> Result<bool, Error> {
    for materialization in &prepared.request.package_materializations {
        let file_name = format!("{}-{}.wasm", materialization.package.replace(':', "-"), materialization.version);
        let resolved = workspace.source.join(&prepared.request.wit_relative_path).join("deps").join(file_name);
        let digest = hash_file_bounded(&resolved)?;
        if digest != materialization.object.digest_blake3 {
            state.block(
                "wkg-materialization-drift",
                "package-resolution",
                format!("wkg output for `{}` differs from the checked immutable package", materialization.package),
            );
            return Ok(false);
        }
    }
    debug_assert!(!prepared.request.package_materializations.is_empty());
    debug_assert!(state.blockers.is_empty());
    Ok(true)
}

pub(crate) fn invoke(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    stage_key: &str,
    tool_name: &str,
    args: Vec<String>,
    output_path: Option<PathBuf>,
) -> Result<crate::ToolRun, Error> {
    run_offline_tool(&prepared.toolchain, ToolInvocation {
        stage_key: stage_key.to_string(),
        tool_name: tool_name.to_string(),
        args,
        cwd: workspace.source.clone(),
        work_root: workspace.root.clone(),
        env: tool_environment(&prepared.toolchain, workspace)?,
        read_only_inputs: read_only_inputs(prepared),
        output_path,
        limits: crate::ToolLimits::default(),
    })
}

fn read_only_inputs(prepared: &PreparedPipeline) -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    for registry in &prepared.request.manifest.package_resolution.registries {
        if let Some(root) = &registry.local_root {
            paths.insert(PathBuf::from(root));
        }
    }
    for dependency in &prepared.request.composition_dependencies {
        if let CompositionDependencySource::StoreObject { path, .. } = &dependency.source {
            paths.insert(PathBuf::from(path));
        }
    }
    for rule in &prepared.request.manifest.virtualization.rules {
        if let Some(input) = &rule.input {
            paths.insert(PathBuf::from(&input.logical_path));
        }
    }
    let paths: Vec<PathBuf> = paths.into_iter().collect();
    paths
        .iter()
        .filter(|candidate| !paths.iter().any(|parent| *candidate != parent && candidate.starts_with(parent)))
        .cloned()
        .collect()
}

fn tool_environment(
    toolchain: &VerifiedToolchain,
    workspace: &StageWorkspace,
) -> Result<BTreeMap<String, String>, Error> {
    Ok(BTreeMap::from([
        ("HOME".to_string(), workspace.root.join("home").display().to_string()),
        ("CARGO_HOME".to_string(), workspace.root.join("cargo-home").display().to_string()),
        ("CARGO_TARGET_DIR".to_string(), workspace.root.join("target").display().to_string()),
        ("CARGO_NET_OFFLINE".to_string(), "true".to_string()),
        ("RUSTC".to_string(), toolchain.tool_path("rustc")?.display().to_string()),
        ("WKG_CONFIG_FILE".to_string(), workspace.source.join(GENERATED_WKG_CONFIG).display().to_string()),
        ("WKG_CACHE_DIR".to_string(), workspace.root.join("wkg-cache").display().to_string()),
    ]))
}

pub(crate) fn measure_artifact(source: &Path, name: &str, workspace: &StageWorkspace) -> Result<StoreObject, Error> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| Error::io("reading component artifact metadata", source, error))?;
    if !metadata.file_type().is_file() || metadata.len() == 0 {
        return Err(Error::Invalid(format!(
            "component artifact is not a non-empty regular file: {}",
            source.display()
        )));
    }
    let final_path = workspace.final_output.join("artifacts").join(name);
    let object = StoreObject {
        logical_path: final_path.display().to_string(),
        digest_blake3: hash_file_bounded(source)?,
        size_bytes: metadata.len(),
    };
    debug_assert!(object.size_bytes > 0);
    debug_assert!(object.logical_path.starts_with('/'));
    Ok(object)
}

pub(crate) fn publish_evidence_artifact(
    source: &Path,
    name: &str,
    workspace: &StageWorkspace,
) -> Result<StoreObject, Error> {
    let object = measure_artifact(source, name, workspace)?;
    publish_validated_artifact(source, name, &object, workspace)?;
    Ok(object)
}

pub(crate) fn publish_validated_artifact(
    source: &Path,
    name: &str,
    expected: &StoreObject,
    workspace: &StageWorkspace,
) -> Result<(), Error> {
    let measured = measure_artifact(source, name, workspace)?;
    if measured != *expected {
        return Err(Error::Invalid(format!("validated component artifact drifted before publication: {name}")));
    }
    let destination = workspace.publication_root.join("artifacts").join(name);
    let copied = copy_regular_file_new(source, &destination, "validated component artifact")?;
    let published = measure_artifact(&destination, name, workspace)?;
    if copied != expected.size_bytes || published != *expected {
        fs::remove_file(&destination)
            .map_err(|error| Error::io("removing mismatched published component artifact", &destination, error))?;
        return Err(Error::Invalid(format!("published component artifact differs from validated bytes: {name}")));
    }
    debug_assert!(destination.is_file());
    debug_assert_eq!(published, *expected);
    Ok(())
}

fn stage_status(success: bool) -> ComponentStageStatus {
    if success {
        ComponentStageStatus::Succeeded
    } else {
        ComponentStageStatus::Failed
    }
}

fn stderr_summary(stderr: &[u8]) -> String {
    let summary = String::from_utf8_lossy(stderr).trim().to_string();
    if summary.is_empty() {
        "tool exited unsuccessfully without stderr".to_string()
    } else {
        summary
    }
}
