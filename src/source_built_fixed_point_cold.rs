//! Source-only cold route and child reconstruction. The legacy shell remains
//! exclusively a development route; no legacy observation authorizes this path.
use super::*;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipePaths;
use crate::source_built_recipe::{self as recipe};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ColdNativeSource {
    record_identity: String,
    recipe_member: String,
    index_member: String,
}

/// Instruction source, never action/event envelopes or imported output pins.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ColdSourceRoute {
    schema: String,
    stagex_transition: ArtifactRef,
    stagex_publication: ArtifactRef,
    native: ColdNativeSource,
    rust_provider: ArtifactRef,
    mantle_recipes: [ArtifactRef; 2],
}

struct PreparedColdActions {
    stagex: crate::source_built_fixed_point_stagex_actions::PreparedStagexActions,
    native: crate::source_built_fixed_point_native_actions::PreparedNativeActions,
    rust: crate::source_built_fixed_point_rust_actions::PreparedRustActions,
    mantle: crate::source_built_fixed_point_mantle_actions::PreparedMantleActions,
    action_plan: ActionTrustPlan,
}

fn sole_cold_source_file(root: &Path) -> Result<PathBuf, RunError> {
    fn visit(root: &Path, files: &mut Vec<PathBuf>, count: &mut usize) -> Result<(), RunError> {
        let metadata = fs::symlink_metadata(root)
            .map_err(|error| proof_error(format!("reading admitted source {}: {error}", root.display())))?;
        *count += 1;
        if *count > 65_536 {
            return Err(proof_error("single-file source entry bound exceeded".into()));
        }
        if metadata.is_file() {
            files.push(root.to_path_buf());
            if files.len() > 1 {
                return Err(proof_error("single-file source authority has multiple members".into()));
            }
            return Ok(());
        }
        if !metadata.is_dir() {
            return Err(proof_error("source authority contains a symlink or special entry".into()));
        }
        for entry in fs::read_dir(root).map_err(|error| proof_error(error.to_string()))? {
            visit(&entry.map_err(|error| proof_error(error.to_string()))?.path(), files, count)?;
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(root, &mut files, &mut 0)?;
    files
        .pop()
        .ok_or_else(|| proof_error(format!("missing single-file source authority {}", root.display())))
}

pub(super) fn cold_source_roots(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
) -> Result<BTreeMap<SourceAuthorityRole, PathBuf>, RunError> {
    let inputs = root.join(INPUTS_DIR);
    let roots = BTreeMap::from([
        (SourceAuthorityRole::StagexSeed, sole_cold_source_file(&inputs.join(STAGEX_SEED_DIR))?),
        (SourceAuthorityRole::StagexLineage, sole_cold_source_file(&inputs.join(STAGEX_LINEAGE_DIR))?),
        (
            SourceAuthorityRole::StagexSourceBundle,
            sole_cold_source_file(&inputs.join(STAGEX_SOURCE_BUNDLE_DIR))?,
        ),
        (
            SourceAuthorityRole::NativeSourceBundle,
            sole_cold_source_file(&inputs.join(NATIVE_SOURCE_AUTHORITY_DIR))?,
        ),
        (SourceAuthorityRole::RustSourceArchiveSet, inputs.join(RUST_SOURCE_DIR)),
        (SourceAuthorityRole::MantleSource, inputs.join(SOURCE_ROOT_DIR)),
        (SourceAuthorityRole::VendorInputs, inputs.join(VENDOR_RELATIVE_PATH)),
    ]);
    for source in &proof.source_inputs {
        let path = roots
            .get(&source.role)
            .ok_or_else(|| proof_error("cold source role is not reconstructible".into()))?;
        if recipe::digest(path)?.1 != source.digest_blake3 {
            return Err(proof_error(format!("cold source identity differs from admitted {:?}", source.role)));
        }
    }
    Ok(roots)
}

fn read_cold_source_route(
    proof: &SourceBuiltFixedPointPlan,
    roots: &BTreeMap<SourceAuthorityRole, PathBuf>,
) -> Result<ColdSourceRoute, RunError> {
    let cache = tempfile::tempdir().map_err(|error| proof_error(error.to_string()))?;
    let selector = ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::NativeSourceBundle,
        record_identity: crate::source_built_mantle_programs::PROGRAM_RECORD.into(),
        member: crate::source_built_mantle_programs::COLD_ROUTE_MEMBER.into(),
    };
    let path = recipe::hydrate_reference(proof, roots, cache.path(), &selector)
        .map_err(|error| proof_error(format!("missing authenticated cold source program selector: {error}")))?;
    let metadata = fs::symlink_metadata(&path).map_err(|error| proof_error(error.to_string()))?;
    if !metadata.is_file() || metadata.len() > 1024 * 1024 {
        return Err(proof_error("cold source selector is not bounded regular source text".into()));
    }
    let route: ColdSourceRoute =
        serde_json::from_slice(&fs::read(&path).map_err(|error| proof_error(error.to_string()))?)
            .map_err(|error| proof_error(format!("cold source selector syntax: {error}")))?;
    if route.schema != "mantle-source-built-cold-route-v2" {
        return Err(proof_error("cold source selector schema differs".into()));
    }
    for (reference, expected_member) in route.mantle_recipes.iter().zip([
        "bootstrap/source-built/mantle-stage1.json",
        "bootstrap/source-built/mantle-stage2.json",
    ]) {
        let ArtifactRef::ManifestSource {
            role: SourceAuthorityRole::NativeSourceBundle,
            record_identity,
            member,
        } = reference
        else {
            return Err(proof_error("Mantle recipe must be independently authenticated program source".into()));
        };
        if record_identity != crate::source_built_mantle_programs::PROGRAM_RECORD {
            return Err(proof_error("Mantle recipe selects a foreign program source record".into()));
        }
        if member != expected_member {
            return Err(proof_error("Mantle recipe selects a different source member".into()));
        }
    }
    debug_assert_eq!(route.schema, "mantle-source-built-cold-route-v2");
    debug_assert!(route.mantle_recipes.iter().all(|reference| matches!(reference, ArtifactRef::ManifestSource { .. })));
    Ok(route)
}

fn prepare_cold_actions(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    bwrap: &Path,
    host_digest: &str,
) -> Result<PreparedColdActions, RunError> {
    let roots = cold_source_roots(root, proof)?;
    let source = &roots[&SourceAuthorityRole::MantleSource];
    let route = read_cold_source_route(proof, &roots)?;
    let runtime = root.join("cold-runtime");
    let transition_root = runtime.join("stagex-transition");
    let publication_root = runtime.join("stagex-provider-publication");
    let stagex_paths = [
        RecipePaths {
            source_roots: &roots,
            runtime_root: &transition_root,
            sandbox_binary: bwrap,
        },
        RecipePaths {
            source_roots: &roots,
            runtime_root: &publication_root,
            sandbox_binary: bwrap,
        },
    ];
    let stagex = crate::source_built_fixed_point_stagex_actions::prepare_actions(
        proof,
        &route.stagex_transition,
        &route.stagex_publication,
        &stagex_paths,
        &[],
        host_digest,
    )?;
    let stagex_recipes = [stagex.transition_recipe(), stagex.publication_recipe()];
    let native_root = runtime.join("full-source-native-provider");
    let native_publication = root.join("native-source-publication.json");
    let native_paths = RecipePaths {
        source_roots: &roots,
        runtime_root: &native_root,
        sandbox_binary: bwrap,
    };
    let native = crate::source_built_fixed_point_native_actions::prepare_actions(
        proof,
        source,
        &crate::source_built_fixed_point_native_actions::AdapterPaths {
            recipe_paths: &native_paths,
            record_identity: &route.native.record_identity,
            recipe_member: &route.native.recipe_member,
            index_member: &route.native.index_member,
            stagex_recipes: &stagex_recipes,
            publication_path: &native_publication,
            proof_root: root,
        },
        host_digest,
    )?;
    let rust_root = runtime.join("full-source-rust-provider");
    let rust_paths = RecipePaths {
        source_roots: &roots,
        runtime_root: &rust_root,
        sandbox_binary: bwrap,
    };
    let prior = [stagex.transition_recipe(), stagex.publication_recipe(), native.recipe()];
    let rust = crate::source_built_fixed_point_rust_actions::prepare_actions(
        proof,
        &route.rust_provider,
        &rust_paths,
        &prior,
        host_digest,
    )?;
    let fixed_point = root.join(FIXED_POINT_DIR);
    let first_root = fixed_point.join("stage1").join("recipe");
    let second_root = fixed_point.join("stage2").join("recipe");
    let targets = [TARGET_TRIPLE.to_string()];
    let mantle_paths = crate::source_built_fixed_point_mantle_actions::AdapterPaths {
        stage_paths: [
            RecipePaths {
                source_roots: &roots,
                runtime_root: &first_root,
                sandbox_binary: bwrap,
            },
            RecipePaths {
                source_roots: &roots,
                runtime_root: &second_root,
                sandbox_binary: bwrap,
            },
        ],
        recipe_sources: [&route.mantle_recipes[0], &route.mantle_recipes[1]],
        future_rustc: &rust.recipe().export("rustc")?.path,
        future_rustc_host: rust.compiler_host(),
        targets: &targets,
    };
    let prior = [
        stagex.transition_recipe(),
        stagex.publication_recipe(),
        native.recipe(),
        rust.recipe(),
    ];
    let mantle = crate::source_built_fixed_point_mantle_actions::plan_actions(
        proof,
        source,
        &mantle_paths,
        &prior,
        host_digest,
    )?;
    let action_plan = crate::source_built_fixed_point_planning::plan_actions(proof, vec![
        stagex.transition_adapter().clone(),
        stagex.publication_adapter().clone(),
        native.adapter().clone(),
        rust.adapter().clone(),
        mantle.adapters()[0].clone(),
        mantle.adapters()[1].clone(),
    ])
    .map_err(proof_error)?;
    Ok(PreparedColdActions {
        stagex,
        native,
        rust,
        mantle,
        action_plan,
    })
}

pub(super) fn run_cold_attempt(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
) -> Result<bool, RunError> {
    use crate::source_built_fixed_point_receipt::ColdProviderObservation;
    use crate::source_built_fixed_point_receipt::ColdStageObservation;
    let host_digest = recipe::digest(&std::env::current_exe().map_err(|error| proof_error(error.to_string()))?)?.1;
    // All six complete prepared adapters exist before the first source action.
    // A missing source product is a blocker, never a legacy-route fallback.
    let actions = prepare_cold_actions(&prepared.staging_dir, &prepared.plan, options.bwrap, &host_digest)?;
    let recipes = [
        actions.stagex.transition_recipe(),
        actions.stagex.publication_recipe(),
        actions.native.recipe(),
        actions.rust.recipe(),
        &actions.mantle.recipes()[0],
        &actions.mantle.recipes()[1],
    ];
    for recipe in recipes {
        if recipe.runtime_root.try_exists().map_err(|error| proof_error(error.to_string()))? {
            return Err(proof_error(format!("cold execution root is not fresh: {}", recipe.runtime_root.display())));
        }
    }
    for path in [
        prepared.staging_dir.join("cold-runtime"),
        prepared.staging_dir.join(FIXED_POINT_DIR),
    ] {
        fs::create_dir(&path).map_err(|error| {
            proof_error(format!("creating fresh cold execution parent {}: {error}", path.display()))
        })?;
    }
    let mut recorder = ActionObservationRecorder::create(&prepared.staging_dir, &prepared.plan, &actions.action_plan)?;
    let mut emit = |event| recorder.record(event);
    crate::source_built_fixed_point_stagex_actions::run_transition(&actions.stagex, &mut emit)?;
    validate_runtime_bounds(options, prepared)?;
    crate::source_built_fixed_point_stagex_actions::run_publication(&actions.stagex, &mut emit)?;
    validate_runtime_bounds(options, prepared)?;
    let native =
        crate::source_built_fixed_point_native_actions::run_with_action_observations(&actions.native, &mut emit)?;
    validate_runtime_bounds(options, prepared)?;
    crate::source_built_fixed_point_rust_actions::run_with_action_observations(&actions.rust, &mut emit)?;
    validate_runtime_bounds(options, prepared)?;
    crate::source_built_fixed_point_mantle_actions::run_with_action_observations(
        &actions.mantle,
        &prepared.staging_dir,
        &actions.action_plan.digest_blake3,
        options.bwrap,
        &mut emit,
    )?;
    recorder.finish(&prepared.plan)?;
    validate_runtime_bounds(options, prepared)?;
    let export_roles = [
        "stagex-transition",
        "stagex-provider",
        "native.provider",
        "full-source-rust-provider",
        "mantle-stage1",
        "mantle-stage2",
    ];
    let mut stages = Vec::with_capacity(EXPECTED_STAGE_COUNT);
    for (index, (recipe, role)) in recipes.into_iter().zip(export_roles).enumerate() {
        let output = recipe.export(role)?;
        let ArtifactRef::Produced { action_id, output_id } = &output.reference else {
            return Err(proof_error("canonical cold output is not produced".into()));
        };
        stages.push(ColdStageObservation {
            stage_id: recipe.adapter().stage_id.clone(),
            output_path: output.path.clone(),
            producer_action_id: action_id.clone(),
            output_id: output_id.clone(),
            execution_journal_path: recipe.runtime_root.join("execution.jsonl"),
            native_publication_audit_path: if index == 2 { Some(native.clone()) } else { None },
        });
    }
    let observation = ColdProviderObservation {
        stages,
        native_provider_path: actions.native.export("native.provider")?.path.clone(),
        rust_provider_path: actions.rust.recipe().export("full-source-rust-provider")?.path.clone(),
        rustc_path: actions.rust.recipe().export("rustc")?.path.clone(),
        toolchain_closure_path: actions.rust.recipe().export("toolchain-closure")?.path.clone(),
    };
    crate::source_built_fixed_point_receipt::write_cold_source_built_fixed_point_receipt(
        &prepared.staging_dir,
        &prepared.plan,
        &observation,
    )?;
    validate_runtime_bounds(options, prepared)?;
    Ok(false)
}

/// Private child: canonical source reconstruction, never externally supplied
/// action/event evidence. Parent admits every live event before continuation.
pub(crate) fn cmd_source_built_replay(
    proof_root: &Path,
    stage_id: &str,
    capture_fd: i32,
    expected_proof_digest: &str,
    expected_action_plan_digest: &str,
    host_mantle_digest: &str,
    bwrap: &Path,
) -> Result<(), RunError> {
    use std::os::fd::FromRawFd;

    use mantle_application_contract::bootstrap_effects::BootstrapEffect::ReadEnvironment;
    use mantle_application_contract::bootstrap_effects::BootstrapEffect::ReadInput;
    use mantle_application_contract::bootstrap_effects::BootstrapEffect::ReplayControl;

    let mut port =
        crate::bootstrap_effect_port::BootstrapPort::new(&[(ReadEnvironment, 1), (ReadInput, 3), (ReplayControl, 5)])?;
    let result = (|| {
        port.call(ReadEnvironment, || validate_cold_compiler_environment(std::env::vars_os().map(|(name, _)| name)))?;
        if capture_fd < 3 || !matches!(stage_id, "mantle-stage1" | "mantle-stage2") {
            return Err(proof_error("invalid private replay stage/socket descriptor".into()));
        }
        for (label, digest) in [
            ("proof", expected_proof_digest),
            ("action plan", expected_action_plan_digest),
            ("host Mantle", host_mantle_digest),
        ] {
            validate_expected_digest(label, digest)?;
        }
        if !proof_root.is_absolute()
            || port
                .call(ReadInput, || fs::canonicalize(proof_root))
                .map_err(|error| proof_error(error.to_string()))?
                != proof_root
        {
            return Err(proof_error("replay proof root must be absolute and nonsymlinked".into()));
        }
        let mut socket_type: libc::c_int = 0;
        let mut length = std::mem::size_of_val(&socket_type) as libc::socklen_t;
        port.call(ReplayControl, || {
            if unsafe {
                libc::getsockopt(
                    capture_fd,
                    libc::SOL_SOCKET,
                    libc::SO_TYPE,
                    (&mut socket_type as *mut libc::c_int).cast(),
                    &mut length,
                )
            } != 0
                || socket_type != libc::SOCK_STREAM
            {
                return Err(proof_error("replay capture descriptor must be an inherited live stream socket".into()));
            }
            Ok(())
        })?;
        let descriptor_flags = port.call(ReplayControl, || {
            let flags = unsafe { libc::fcntl(capture_fd, libc::F_GETFD) };
            if flags < 0 {
                return Err(proof_error("could not inspect replay control descriptor".into()));
            }
            Ok(flags)
        })?;
        port.call(ReplayControl, || {
            if unsafe { libc::fcntl(capture_fd, libc::F_SETFD, descriptor_flags | libc::FD_CLOEXEC) } < 0 {
                return Err(proof_error("could not isolate replay control descriptor from leaf children".into()));
            }
            Ok(())
        })?;
        let path = proof_root.join(PLAN_FILE);
        let metadata = port
            .call(ReadInput, || fs::symlink_metadata(&path))
            .map_err(|error| proof_error(error.to_string()))?;
        if !metadata.is_file() || metadata.len() > 1024 * 1024 {
            return Err(proof_error("replay proof plan is not bounded regular source admission data".into()));
        }
        let proof: SourceBuiltFixedPointPlan = serde_json::from_slice(
            &port.call(ReadInput, || fs::read(&path)).map_err(|error| proof_error(error.to_string()))?,
        )
        .map_err(|error| proof_error(error.to_string()))?;
        crate::source_built_fixed_point::validate_source_built_fixed_point_plan(&proof)
            .map_err(|error| proof_error(error.to_string()))?;
        if proof.plan_digest_blake3 != expected_proof_digest {
            return Err(proof_error("replay proof differs from parent's immutable source root".into()));
        }
        let actions = prepare_cold_actions(proof_root, &proof, bwrap, host_mantle_digest)?;
        if actions.action_plan.digest_blake3 != expected_action_plan_digest {
            return Err(proof_error(
                "reconstructed cold action plan differs from parent's frozen in-memory plan".into(),
            ));
        }
        let mut stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(capture_fd) };
        let timeout = Some(Duration::from_secs(proof.resource_bounds.elapsed_seconds_max));
        port.call(ReplayControl, || stream.set_read_timeout(timeout))
            .map_err(|error| proof_error(error.to_string()))?;
        port.call(ReplayControl, || stream.set_write_timeout(timeout))
            .map_err(|error| proof_error(error.to_string()))?;
        crate::source_built_fixed_point_mantle_actions::replay_stage(&actions.mantle, stage_id, &mut stream)
    })();
    port.finish(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
    use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
    use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;

    fn route() -> serde_json::Value {
        let member = |path| {
            serde_json::json!({
                "kind": "manifest-source", "role": "native-source-bundle",
                "record_identity": crate::source_built_mantle_programs::PROGRAM_RECORD, "member": path,
            })
        };
        serde_json::json!({
            "schema": "mantle-source-built-cold-route-v2",
            "stagex_transition": member("stagex-transition.json"),
            "stagex_publication": member("stagex-provider-publication.json"),
            "native": {"record_identity": "native-program", "recipe_member": "native.json", "index_member": "native.expansion.json"},
            "rust_provider": {"kind": "source", "role": "rust-source-archive-set", "path": "rust.json"},
            "mantle_recipes": [member("bootstrap/source-built/mantle-stage1.json"), member("bootstrap/source-built/mantle-stage2.json")],
        })
    }

    fn selector_fixture(
        root: &Path,
        route: &serde_json::Value,
    ) -> (SourceBuiltFixedPointPlan, BTreeMap<SourceAuthorityRole, PathBuf>) {
        let programs = root.join("programs");
        let member = programs.join(crate::source_built_mantle_programs::COLD_ROUTE_MEMBER);
        fs::create_dir_all(member.parent().unwrap()).unwrap();
        fs::write(&member, serde_json::to_vec_pretty(route).unwrap()).unwrap();
        let bundle = crate::source_bundle::plan_source_bundle(
            &[crate::source_bundle::SourceSpec {
                kind: crate::source_bundle::SourceRecordKind::LocalPath,
                identity: crate::source_built_mantle_programs::PROGRAM_RECORD.into(),
                path: programs,
                adapter: None,
            }],
            "/mantle/store",
        )
        .unwrap();
        let manifest = root.join("native-sources.json");
        crate::source_bundle::write_source_bundle(&manifest, &bundle).unwrap();
        let mut original = crate::source_built_fixed_point_action_trust::tests::fixture().0;
        let native = original
            .source_inputs
            .iter_mut()
            .find(|input| input.role == SourceAuthorityRole::NativeSourceBundle)
            .unwrap();
        (native.size_bytes, native.digest_blake3) = recipe::digest(&manifest).unwrap();
        let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
            proof_id: original.proof_id,
            logical_store_prefix: original.logical_store_prefix,
            source_inputs: original.source_inputs,
            initial_output_authority: InitialOutputAuthorityState {
                stagex_transition_entries: 0,
                native_provider_entries: 0,
                rust_provider_entries: 0,
                mantle_output_entries: 0,
            },
            policies: original.policies,
            resource_bounds: original.resource_bounds,
        })
        .unwrap();
        (proof, BTreeMap::from([(SourceAuthorityRole::NativeSourceBundle, manifest)]))
    }

    #[test]
    fn cold_selector_uses_program_record_not_compiler_source_or_legacy_route() {
        let directory = tempfile::tempdir().unwrap();
        let (proof, mut roots) = selector_fixture(directory.path(), &route());
        let source = directory.path().join("compiler-source");
        let legacy = source.join(crate::source_built_mantle_programs::COLD_ROUTE_MEMBER);
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(&legacy, "invalid legacy selector must never be consulted").unwrap();
        roots.insert(SourceAuthorityRole::MantleSource, source);
        let selected = read_cold_source_route(&proof, &roots).unwrap();
        assert_eq!(selected.schema, "mantle-source-built-cold-route-v2");
        assert!(matches!(&selected.mantle_recipes[0], ArtifactRef::ManifestSource { record_identity, member, .. }
            if record_identity == crate::source_built_mantle_programs::PROGRAM_RECORD && member == "bootstrap/source-built/mantle-stage1.json"));
        roots.remove(&SourceAuthorityRole::NativeSourceBundle);
        assert!(read_cold_source_route(&proof, &roots).is_err());
        assert_eq!(fs::read_to_string(legacy).unwrap(), "invalid legacy selector must never be consulted");
    }

    #[test]
    fn cold_selector_rejects_old_schema_and_foreign_or_unqualified_mantle_recipes() {
        let directory = tempfile::tempdir().unwrap();
        let mut old = route();
        old["schema"] = "mantle-source-built-cold-route-v1".into();
        let mut foreign = route();
        foreign["mantle_recipes"][0]["record_identity"] = "foreign-program".into();
        let mut unqualified = route();
        unqualified["mantle_recipes"][0] = serde_json::json!({"kind": "source", "role": "mantle-source", "path": "bootstrap/source-built/mantle-stage1.json"});
        let mut wrong_member = route();
        wrong_member["mantle_recipes"][1]["member"] = "bootstrap/source-built/mantle-stage1.json".into();
        for (index, invalid) in [old, foreign, unqualified, wrong_member].iter().enumerate() {
            let (proof, roots) = selector_fixture(&directory.path().join(index.to_string()), invalid);
            assert!(read_cold_source_route(&proof, &roots).is_err());
        }
    }
}
