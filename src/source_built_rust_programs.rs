//! Source-only authoring of the original selected Rust provider route.
//! The route is read from the admitted Mantle source, never inferred from the
//! preparation host or from a future compiler's --version/--print output.
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;

use sha2::Digest;
use sha2::Sha256;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_toolchain_closure::RustSourceProviderBootstrapPlan;

#[path = "source_built_rust_programs_llvm.rs"]
pub(crate) mod llvm;
#[path = "source_built_rust_programs_minicargo.rs"]
mod minicargo;
#[path = "source_built_rust_programs_mrustc.rs"]
mod mrustc;
#[path = "source_built_rust_programs_patches.rs"]
mod patches;
#[path = "source_built_rust_programs_phase.rs"]
mod phase;
#[path = "source_built_rust_resolve.rs"]
pub(crate) mod resolve;
#[path = "source_built_rust_programs_runtime.rs"]
mod runtime;
#[path = "source_built_rust_programs_successors.rs"]
mod successors;
#[path = "source_built_rust_programs_translation.rs"]
pub(crate) mod translation;

pub(crate) const STAGE: &str = "full-source-rust-provider";
pub(crate) const RECORD_IDENTITY: &str = "fixed-point-rust-source-program";
pub(crate) const ROUTE_SOURCE_PATH: &str = "bootstrap/rust-source-musl-host-plan.ncl";
pub(crate) const PACKAGED_ROUTE_PATH: &str = "cairn/rust/rust-source-musl-host-plan.ncl";
pub(crate) const RECIPE_MEMBER: &str = "source-built/rust/program.json";
pub(crate) const INDEX_MEMBER: &str = "source-built/rust/program.json.expansion.json";
const TRANSLATION_SOURCE_PATH: &str = "cairn/rust/mrustc-native-translation.c";

fn source_files(
    request: &crate::source_built_program_authoring::AuthoringRequest<'_>,
) -> Result<BTreeMap<String, String>, RunError> {
    let mut files = phase::source_files();
    files.extend(runtime::source_files(request)?);
    files.extend([
        (patches::HELPER_PATH.into(), patches::HELPER_SOURCE.into()),
        (patches::METADATA_PATH.into(), patches::METADATA_SOURCE.into()),
        (patches::PROC_MACRO_PATH.into(), patches::PROC_MACRO_SOURCE.into()),
        (TRANSLATION_SOURCE_PATH.into(), translation::TRANSLATION_HELPER_SOURCE.into()),
        (minicargo::HELPER_PATH.into(), minicargo::HELPER_SOURCE.into()),
        (minicargo::STD_PROCESS_DESCRIPTOR_PATH.into(), successors::std_process_patch_descriptor()),
        (minicargo::STD_PROCESS_MODULE_PATH.into(), successors::std_process_module_source()),
    ]);
    let source = request
        .source_roots
        .get(&SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("missing original Mantle source authority"))?;
    files.insert(
        PACKAGED_ROUTE_PATH.into(),
        std::fs::read_to_string(source.join(ROUTE_SOURCE_PATH))
            .map_err(|failure| error(format!("original route source: {failure}")))?,
    );
    Ok(files)
}

pub(super) fn helper_source_ref(path: &str) -> crate::source_built_recipe::ArtifactRef {
    crate::source_built_recipe::ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::NativeSourceBundle,
        record_identity: RECORD_IDENTITY.into(),
        member: path.into(),
    }
}

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("Rust source provider authoring: {}", message.into()))
}

pub(crate) fn source_route(
    source_roots: &BTreeMap<SourceAuthorityRole, PathBuf>,
) -> Result<RustSourceProviderBootstrapPlan, RunError> {
    let source = source_roots
        .get(&SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("missing admitted original Mantle source authority"))?;
    let path = source.join(ROUTE_SOURCE_PATH);
    let metadata =
        std::fs::symlink_metadata(&path).map_err(|e| error(format!("original route {}: {e}", path.display())))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > 1024 * 1024 {
        return Err(error("original route must be a bounded regular source file"));
    }
    crate::source_built_recipe::validate_source_imports(source, &path, &[])?;
    let route: RustSourceProviderBootstrapPlan = crunch_eval::evaluate_and_deserialize(&path, &[])
        .map_err(|e| error(format!("original route {}: {e}", path.display())))?;
    crate::source_toolchain_closure::validate_rust_source_provider_bootstrap_plan(&route)
        .map_err(|e| error(e.message()))?;
    if route.host_triple != "x86_64-unknown-linux-musl" || route.target_triple != "x86_64-unknown-linux-musl" {
        return Err(error("selected original source route requires the musl compiler host and target"));
    }
    Ok(route)
}

/// Resolve an original archive by the admitted route's content pin, not by
/// names in a preparation-machine directory or provider-generated metadata.
pub(crate) fn source_archive(
    source_roots: &BTreeMap<SourceAuthorityRole, PathBuf>,
    id: &str,
) -> Result<PathBuf, RunError> {
    let route = source_route(source_roots)?;
    let expected = route
        .sources
        .iter()
        .find(|source| source.id == id)
        .ok_or_else(|| error(format!("original route lacks source {id}")))?;
    let root = source_roots
        .get(&SourceAuthorityRole::RustSourceArchiveSet)
        .ok_or_else(|| error("missing original Rust source archive authority"))?;
    let mut pending = vec![root.clone()];
    let mut found = None;
    let mut entries = 0u32;
    let mut buffer = [0u8; 65536];
    while let Some(path) = pending.pop() {
        entries = entries.checked_add(1).ok_or_else(|| error("archive entry count overflow"))?;
        if entries > 4096 {
            return Err(error("original archive authority exceeds entry bound"));
        }
        let metadata = std::fs::symlink_metadata(&path).map_err(|e| error(e.to_string()))?;
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path).map_err(|e| error(e.to_string()))? {
                pending.push(entry.map_err(|e| error(e.to_string()))?.path());
            }
        } else if metadata.is_file() {
            let mut source = std::fs::File::open(&path).map_err(|e| error(e.to_string()))?;
            let mut digest = Sha256::new();
            loop {
                let bytes = source.read(&mut buffer).map_err(|e| error(e.to_string()))?;
                if bytes == 0 {
                    break;
                }
                digest.update(&buffer[..bytes]);
            }
            if format!("{:x}", digest.finalize()) == expected.sha256_hex && found.replace(path).is_some() {
                return Err(error(format!("duplicate original archive identity {id}")));
            }
        }
    }
    found.ok_or_else(|| error(format!("original archive authority lacks {id} SHA256 {}", expected.sha256_hex)))
}

/// Expand the complete selected route into immutable source-owned leaf recipes.
/// Reservations are authenticated policy, not predicted successful outcomes.
pub(crate) fn author(
    request: &crate::source_built_program_authoring::AuthoringRequest<'_>,
    reservations: &crate::source_built_program_authoring::ContinuationReservationPolicy,
) -> Result<crate::source_built_program_authoring::AuthoredPrograms, RunError> {
    use crate::source_built_recipe::ArtifactKind;
    use crate::source_built_recipe::ArtifactRef;
    use crate::source_built_recipe::RecipeArchiveFormat;
    use crate::source_built_recipe::RecipeExitPolicy;
    use crate::source_built_recipe::RecipeOperation;
    use crate::source_built_recipe::RecipeOutput;
    use crate::source_built_recipe::RecipePreparation;
    use crate::source_built_recipe::RecipeStep;
    use crate::source_built_recipe::RecipeWord;
    use crate::source_built_recipe::SourceRecipe;
    use crate::source_built_recipe::WordPart;
    use crate::source_toolchain_closure::RustProviderRole;
    let route = source_route(request.source_roots)?;
    let raw_root = request
        .source_roots
        .get(&SourceAuthorityRole::RustSourceArchiveSet)
        .ok_or_else(|| error("missing original raw Rust archive authority"))?;
    let mut files = source_files(request)?;
    let mut program = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "original-rust-provider-source-author".into(),
            provenance: ROUTE_SOURCE_PATH.into(),
        },
        steps: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
        timeout_groups: Vec::new(),
    };
    let produced = |id: &str, output: &str| ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: output.into(),
    };
    let spec = |kind| RecipeOutput {
        kind,
        bytes_max: request.proof.resource_bounds.disk_bytes_max,
    };
    let input = |name: &str, path: &str| RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    };
    let output = |name: &str| RecipeWord {
        parts: vec![WordPart::Output {
            name: name.into(),
            path: String::new(),
        }],
    };
    let mut archives = BTreeMap::new();
    for source in &route.sources {
        let path = source_archive(request.source_roots, &source.id)?;
        let path = path.strip_prefix(raw_root).map_err(|_| error("original archive escaped raw authority"))?;
        let path = path.to_str().ok_or_else(|| error("original archive path is not UTF-8"))?.to_string();
        crate::source_built_recipe::relative(&path, false)?;
        archives.insert(source.id.clone(), ArtifactRef::Source {
            role: SourceAuthorityRole::RustSourceArchiveSet,
            path,
        });
    }
    let mut stages = BTreeMap::new();
    let mut original_mrustc = None;
    let mut first_compilers = None;
    let mut runtime_members = BTreeMap::new();
    let mut native_helper = None;
    let mut source_utility = None;
    let mut patcher = None;
    let mut phase_helper = None;
    let mut translation_helper = None;
    let mut previous: BTreeMap<String, ArtifactRef> = BTreeMap::new();
    let mut previous_provider = None;
    let mut native_context = BTreeMap::new();
    let cmake_phase = request
        .predecessor_exports
        .get("full-source-native-provider:host.cmake-phase")
        .ok_or_else(|| error("missing original source-built CMake phase executable"))?
        .clone();
    let cmake_phase_root = request
        .predecessor_exports
        .get("full-source-native-provider:host.cmake-phase-root")
        .ok_or_else(|| error("missing original source-built CMake installed context"))?
        .clone();
    for (stage_index, stage) in route.stages.iter().enumerate() {
        let first_step = program.steps.len();
        let mut extracted = BTreeMap::new();
        for source_id in &stage.source_ids {
            let source = route
                .sources
                .iter()
                .find(|source| &source.id == source_id)
                .ok_or_else(|| error("original stage names an absent archive"))?;
            let format = if source.url.ends_with(".tar.gz") || source.url.ends_with(".tgz") {
                RecipeArchiveFormat::TarGz
            } else if source.url.ends_with(".tar.xz") {
                RecipeArchiveFormat::TarXz
            } else if source.url.ends_with(".tar.bz2") {
                RecipeArchiveFormat::TarBz2
            } else if source.url.ends_with(".tar.zst") {
                RecipeArchiveFormat::TarZst
            } else if source.url.ends_with(".tar.lzma") {
                RecipeArchiveFormat::TarLzma
            } else if source.url.ends_with(".tar") {
                RecipeArchiveFormat::Tar
            } else {
                return Err(error(format!("unsupported original archive format: {}", source.url)));
            };
            let id = format!("archive-{source_id}");
            program.steps.push(RecipeStep {
                id: id.clone(),
                inputs: BTreeMap::from([("archive".into(), archives[source_id].clone())]),
                outputs: BTreeMap::from([("source".into(), spec(ArtifactKind::Tree))]),
                operation: RecipeOperation::ExtractArchive {
                    input: "archive".into(),
                    output: "source".into(),
                    format,
                },
            });
            extracted.insert(source_id.clone(), produced(&id, "source"));
        }
        if stage_index == 0 {
            let selector = translation::append_c_helper(
                request,
                &mut program,
                "rust-source-phase-helper",
                helper_source_ref(phase::TRANSFORMER_PATH),
                BTreeMap::new(),
            )?;
            runtime_members = runtime::append(request, &mut program, selector.clone())?;
            let source_patcher = translation::append_c_helper(
                request,
                &mut program,
                "rust-source-patch-helper",
                helper_source_ref(patches::HELPER_PATH),
                BTreeMap::new(),
            )?;
            let translator = translation::append_c_helper(
                request,
                &mut program,
                "rust-native-translation-helper",
                helper_source_ref(TRANSLATION_SOURCE_PATH),
                BTreeMap::new(),
            )?;
            let graph_helper = translation::append_c_helper(
                request,
                &mut program,
                "rust-minicargo-graph-helper",
                helper_source_ref(minicargo::HELPER_PATH),
                BTreeMap::new(),
            )?;
            let original = extracted
                .get("mrustc-0.12.0")
                .ok_or_else(|| error("original first stage omits mrustc source"))?
                .clone();
            original_mrustc = Some(original.clone());
            let patched =
                patches::author(request, original, source_patcher.clone(), patches::PatchSelection::Mrustc012 {
                    runtime: runtime_members["tree"].clone(),
                })?;
            let source = patched.exports["patched.source"].clone();
            program.steps.extend(patched.steps);
            files.extend(patched.files);
            let compilers = mrustc::author(request, source)?;
            program.steps.extend(compilers.steps);
            files.extend(compilers.files);
            first_compilers = Some(compilers.exports);
            source_utility = Some(graph_helper);
            patcher = Some(source_patcher);
            phase_helper = Some(selector);
            translation_helper = Some(translator);
        }
        let rust_id = format!("rust-{}", stage.rust_version);
        let original = extracted
            .get(&rust_id)
            .ok_or_else(|| error(format!("original stage {} omits {rust_id}", stage.id)))?
            .clone();
        let selection = if stage_index == 0 {
            patches::PatchSelection::FirstRust190 {
                mrustc_source: original_mrustc.clone().unwrap(),
            }
        } else {
            patches::PatchSelection::Successor {
                version: &stage.rust_version,
            }
        };
        let patched = patches::author(request, original, patcher.clone().unwrap(), selection)?;
        let patched_source = patched.exports["patched.source"].clone();
        program.steps.extend(patched.steps);
        files.extend(patched.files);
        let top = format!("rustc-{}-src", stage.rust_version);
        let phase_id = format!("compiler-source-phase-{}", stage.rust_version);
        let mut declared = BTreeMap::from([("source".into(), spec(ArtifactKind::Tree))]);
        for name in ["stdout", "stderr"] {
            declared.insert(name.into(), RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max: 64 * 1024 * 1024,
            });
        }
        program.steps.push(RecipeStep {
            id: phase_id.clone(),
            inputs: BTreeMap::from([
                ("executable".into(), phase_helper.clone().unwrap()),
                ("original".into(), patched_source),
                ("link-module".into(), helper_source_ref(phase::LINK_MODULE_PATH)),
                ("env-module".into(), helper_source_ref(phase::ENV_MODULE_PATH)),
                ("overlay-module".into(), helper_source_ref(phase::OVERLAY_MODULE_PATH)),
            ]),
            outputs: declared,
            operation: RecipeOperation::Exec {
                executable_input: "executable".into(),
                invocation_input: None,
                arguments: vec![
                    input("original", &top),
                    input("link-module", ""),
                    input("env-module", ""),
                    output("source"),
                    input("overlay-module", ""),
                ],
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                output_seeds: BTreeMap::new(),
                command_state: None,
                guard: None,
                merge_stderr: false,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        });
        let wrapped = format!("{phase_id}.archive-root");
        program.steps.push(RecipeStep {
            id: wrapped.clone(),
            inputs: BTreeMap::from([("source".into(), produced(&phase_id, "source"))]),
            outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries: BTreeMap::from([(top, "source".into())]),
            },
        });
        let source_tree = produced(&wrapped, "tree");
        let llvm_capacity = crate::source_built_recipe::STEP_COUNT_MAX
            .checked_sub(program.steps.len())
            .ok_or_else(|| error("source recipe capacity exhausted before original LLVM graph"))?;
        let llvm = llvm::author(
            request,
            &stage.rust_version,
            source_tree.clone(),
            original_mrustc.clone().ok_or_else(|| error("missing original mrustc source for LLVM policy"))?,
            &runtime_members,
            reservations,
            cmake_phase.clone(),
            cmake_phase_root.clone(),
            llvm_capacity,
        )?;
        program.steps.extend(llvm.steps);
        files.extend(llvm.files);
        if stage_index == 0 {
            let first = first_compilers.as_ref().unwrap();
            let bootstrap = minicargo::author(
                request,
                source_tree.clone(),
                original_mrustc.clone().unwrap(),
                first["mrustc"].clone(),
                first["minicargo"].clone(),
                translation_helper.clone().unwrap(),
                source_utility.clone().unwrap(),
                &llvm.exports,
                &runtime_members,
                reservations,
            )?;
            native_helper = Some(bootstrap.exports["minicargo.native-helper.artifact"].clone());
            previous_provider = Some(bootstrap.exports["bootstrap.provider"].clone());
            previous.insert("rustc".into(), bootstrap.exports["bootstrap.rustc"].clone());
            program.steps.extend(bootstrap.steps);
            files.extend(bootstrap.files);
        } else {
            // Reassemble actual original installed products, consumed by the
            // next source compiler, without executing predecessor Cargo.
            let bootstrap_id = format!("bootstrap-products-{}", stage.rust_version);
            let mut inputs = BTreeMap::new();
            let mut entries = BTreeMap::new();
            for (name, reference) in &previous {
                let path = match name.as_str() {
                    "rustc" | "cargo" | "rustdoc" => format!("bin/{name}"),
                    "host-rustlib" => format!("lib/rustlib/{}/lib", route.host_triple),
                    "target-rustlib" if route.target_triple == route.host_triple => {
                        if previous.get("host-rustlib") != Some(reference) {
                            return Err(error(
                                "identical original host/target triple must share its actual library product",
                            ));
                        }
                        continue;
                    }
                    "target-rustlib" => format!("lib/rustlib/{}/lib", route.target_triple),
                    other => return Err(error(format!("unexpected original bootstrap product {other}"))),
                };
                inputs.insert(name.clone(), reference.clone());
                entries.insert(path, name.clone());
            }
            program.steps.push(RecipeStep {
                id: bootstrap_id.clone(),
                inputs,
                outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
                operation: RecipeOperation::AssembleTree {
                    output: "tree".into(),
                    entries,
                },
            });
            let merge = format!("{bootstrap_id}.merge");
            let mut outputs = BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]);
            for name in ["stdout", "stderr"] {
                outputs.insert(name.into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 64 * 1024 * 1024,
                });
            }
            program.steps.push(RecipeStep {
                id: merge.clone(),
                inputs: BTreeMap::from([
                    ("executable".into(), native_helper.clone().unwrap()),
                    ("original".into(), previous_provider.clone().unwrap()),
                    ("products".into(), produced(&bootstrap_id, "tree")),
                    ("runtime".into(), runtime_members["tree"].clone()),
                ]),
                outputs,
                operation: RecipeOperation::Exec {
                    executable_input: "executable".into(),
                    invocation_input: None,
                    arguments: vec![
                        RecipeWord {
                            parts: vec![WordPart::Literal {
                                text: "--merge-trees".into(),
                            }],
                        },
                        output("tree"),
                        input("original", ""),
                        input("products", ""),
                    ],
                    environment: BTreeMap::new(),
                    runtime_mounts: BTreeMap::from([("/mantle-rust-runtime".into(), "runtime".into())]),
                    cwd_input: None,
                    cwd_output: None,
                    stdin_input: None,
                    output_seeds: BTreeMap::new(),
                    command_state: None,
                    guard: None,
                    merge_stderr: false,
                    exit_policy: RecipeExitPolicy::Exact { code: 0 },
                },
            });
            previous_provider = Some(produced(&merge, "tree"));
        }
        let built = successors::author_version(
            request,
            &stage.rust_version,
            source_tree,
            previous["rustc"].clone(),
            previous_provider.clone().unwrap(),
            &llvm.exports,
            &runtime_members,
            source_utility.clone().unwrap(),
            native_helper.clone().unwrap(),
            original_mrustc.clone().unwrap(),
            reservations,
            translation_helper.clone().unwrap(),
        )?;
        let outputs = stage
            .outputs
            .iter()
            .map(|role| {
                let name = match role {
                    RustProviderRole::Rustc => "rustc",
                    RustProviderRole::Cargo => "cargo",
                    RustProviderRole::Rustdoc => "rustdoc",
                    RustProviderRole::HostRustlib => "host-rustlib",
                    RustProviderRole::TargetRustlib => "target-rustlib",
                    RustProviderRole::ProviderReceipt => "provider-receipt",
                };
                built
                    .exports
                    .get(name)
                    .cloned()
                    .map(|reference| (name.to_string(), reference))
                    .ok_or_else(|| error(format!("complete original stage {} omitted {name}", stage.id)))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        previous_provider = Some(built.exports["provider"].clone());
        if let Some(helper) = built.exports.get("source-utility") {
            source_utility = Some(helper.clone());
        }
        for name in [
            "native-controller",
            "native-source-boundary",
            "native-target-wrappers",
            "native-package",
        ] {
            native_context.insert(
                name.to_string(),
                built
                    .exports
                    .get(name)
                    .ok_or_else(|| error(format!("original stage {} omitted its actual {name}", stage.id)))?
                    .clone(),
            );
        }
        program.steps.extend(built.steps);
        files.extend(built.files);
        stages.insert(stage.id.clone(), serde_json::json!({ "step_ids": program.steps[first_step..].iter().map(|step| &step.id).collect::<Vec<_>>(), "outputs": outputs }));
        previous = outputs;
    }
    let final_provider = previous_provider.ok_or_else(|| error("original route produced no provider"))?;
    program.exports = previous;
    program.exports.extend(native_context);
    program.exports.insert(STAGE.into(), final_provider.clone());
    let receipt = program
        .exports
        .get("provider-receipt")
        .ok_or_else(|| error("final original provider lacks its actual compiler receipt"))?
        .clone();
    program.exports.insert("toolchain-closure".into(), receipt);
    program.exports.insert("rust-runtime".into(), runtime_members["tree"].clone());
    program.roots = program.exports.values().cloned().collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    for step in &mut program.steps {
        if matches!(step.operation, RecipeOperation::Exec { .. }) {
            step.outputs.entry("exit-status".into()).or_insert(RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max: 4,
            });
        }
    }
    program.semantic_roots.insert(STAGE.into(), vec![final_provider]);
    crate::source_built_recipe::validate_source_recipe(&program)?;
    let index = serde_json::json!({ "schema": "mantle-rust-source-expansion-v1", "recipe_path": RECIPE_MEMBER, "route_path": PACKAGED_ROUTE_PATH, "archives": archives, "stages": stages });
    files.insert(RECIPE_MEMBER.into(), serde_json::to_string(&program).map_err(|failure| error(failure.to_string()))?);
    files.insert(INDEX_MEMBER.into(), serde_json::to_string(&index).map_err(|failure| error(failure.to_string()))?);
    let exports = program
        .exports
        .into_iter()
        .map(|(name, reference)| (format!("{STAGE}:{name}"), reference))
        .collect();
    Ok(crate::source_built_program_authoring::AuthoredPrograms { files, exports })
}
