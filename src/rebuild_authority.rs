use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::BUILD_EFFECT_POLICY_VERSION;
use crunch_release_core::CONTENT_BOUND_REBUILD_DESCRIPTOR_SCHEMA;
use crunch_release_core::ContentBoundRebuildDescriptor;
use crunch_release_core::RebuildAuthorityInput;
use crunch_release_core::RebuildAuthorityPlan;
use crunch_release_core::RebuildContentIdentity;
use crunch_release_core::RebuildContentKind;
use crunch_release_core::RebuildInputObservation;
use crunch_release_core::RebuildInputRole;
use crunch_release_core::RebuildPolicyIdentities;
use crunch_release_core::RebuildRunRootIdentity;
use crunch_release_core::RebuildRunRootObservation;
use crunch_release_core::ReleaseEvidenceManifest;
use crunch_release_core::content_bound_rebuild_descriptor_digest_blake3;
use crunch_release_core::plan_rebuild_authority;
use crunch_release_core::rebuild_arguments_digest_blake3;
use crunch_release_core::rebuild_authority_plan_digest_blake3;

use crate::errors::RunError;

const HASH_BUFFER_BYTES: usize = 8_192;
const RECIPE_SCAN_BYTES_MAX: u64 = 4 * 1_024 * 1_024;
const FORBIDDEN_BUNDLE_ENV: &[u8] = b"MANTLE_REPRODUCE_BUNDLE_DIR";
const SANDBOX_POLICY_IDENTITY: &str = "exact-declared-read-binds;network=none;bundle=absent";
const NORMALIZATION_POLICY_IDENTITY: &str = "source-date-epoch=1;timezone=UTC;locale=C.UTF-8;umask=0022";
const INPUT_DIRECTORY_NAME: &str = "rebuild-inputs";

#[derive(Debug, Clone)]
pub(crate) struct RebuildRunPaths {
    pub run_id: String,
    pub output_dir: PathBuf,
    pub store_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedRebuildAuthority {
    pub command_path: PathBuf,
    pub command_args: Vec<OsString>,
    pub source_archive_path: PathBuf,
    pub read_only_paths: Vec<PathBuf>,
    pub descriptor: ContentBoundRebuildDescriptor,
    pub descriptor_blake3: String,
    pub authority_plan: RebuildAuthorityPlan,
    pub authority_plan_blake3: String,
}

#[derive(Debug, Clone)]
struct PathObservation {
    observation: RebuildInputObservation,
    original_path: Option<PathBuf>,
}

#[derive(Debug)]
struct RebuildAuthorityMeasurements {
    command_path: PathBuf,
    source_path: PathBuf,
    target_observations: Vec<RebuildInputObservation>,
    path_observations: Vec<PathObservation>,
    run_roots: Vec<RebuildRunRootObservation>,
    descriptor: ContentBoundRebuildDescriptor,
    descriptor_blake3: String,
}

// r[impl mantle.build_correctness.release_determinism.identity_binding]
// r[impl mantle.build_correctness.release_determinism.fixtures.negative.identity_drift]
pub(crate) fn prepare_rebuild_authority(
    manifest: &ReleaseEvidenceManifest,
    bundle_dir: &Path,
    rebuild_command: &Path,
    rebuild_args: &[OsString],
    proof_root: &Path,
    ordinary_output_dir: &Path,
    runs: &[RebuildRunPaths],
) -> Result<PreparedRebuildAuthority, RunError> {
    let measured = measure_rebuild_authority(
        manifest,
        bundle_dir,
        rebuild_command,
        rebuild_args,
        proof_root,
        ordinary_output_dir,
        runs,
    )?;
    let preliminary_plan = authority_plan_for(&measured, &measured.path_observations, proof_root, ordinary_output_dir)?;
    fail_if_authority_blocked(&preliminary_plan)?;

    let materialized = materialize_approved_inputs(proof_root, &measured.path_observations)?;
    let materialized_observations = remap_observations(&measured.path_observations, &materialized)?;
    let authority_plan = authority_plan_for(&measured, &materialized_observations, proof_root, ordinary_output_dir)?;
    fail_if_authority_blocked(&authority_plan)?;
    finalize_prepared_authority(measured, rebuild_args, materialized, authority_plan)
}

fn measure_rebuild_authority(
    manifest: &ReleaseEvidenceManifest,
    bundle_dir: &Path,
    rebuild_command: &Path,
    rebuild_args: &[OsString],
    proof_root: &Path,
    ordinary_output_dir: &Path,
    runs: &[RebuildRunPaths],
) -> Result<RebuildAuthorityMeasurements, RunError> {
    let bundle_dir = canonical_existing(bundle_dir, "release bundle")?;
    let command_path = absolute_existing_no_follow(rebuild_command, "rebuild executable")?;
    let source_path =
        canonical_existing(&bundle_dir.join(&manifest.source_archive.relative_path), "release source archive")?;
    let provider_path =
        canonical_existing(&bundle_dir.join(&manifest.prerequisite_inventory.relative_path), "provider inventory")?;
    let target_observations = measure_targets(manifest, &bundle_dir)?;
    let mut path_observations = measure_declared_paths(
        &bundle_dir,
        &command_path,
        rebuild_args,
        &source_path,
        &provider_path,
        proof_root,
        ordinary_output_dir,
    )?;
    append_policy_observations(&mut path_observations)?;
    append_recipe_authority_observations(&mut path_observations, &target_observations)?;
    let run_roots = run_root_observations(runs)?;
    let descriptor = build_descriptor(manifest, rebuild_args, &path_observations, &run_roots)?;
    let descriptor_blake3 = content_bound_rebuild_descriptor_digest_blake3(descriptor.clone()).map_err(|err| {
        RunError::Internal(format!(
            "deterministic genuine rebuild authority blocked: DescriptorInvalid:descriptor:{err}"
        ))
    })?;
    Ok(RebuildAuthorityMeasurements {
        command_path,
        source_path,
        target_observations,
        path_observations,
        run_roots,
        descriptor,
        descriptor_blake3,
    })
}

fn authority_plan_for(
    measured: &RebuildAuthorityMeasurements,
    observations: &[PathObservation],
    proof_root: &Path,
    ordinary_output_dir: &Path,
) -> Result<RebuildAuthorityPlan, RunError> {
    Ok(plan_rebuild_authority(RebuildAuthorityInput {
        descriptor: measured.descriptor.clone(),
        descriptor_blake3: measured.descriptor_blake3.clone(),
        published_targets: measured.target_observations.clone(),
        candidate_inputs: observations.iter().map(|item| item.observation.clone()).collect(),
        run_roots: measured.run_roots.clone(),
        ordinary_output_path: normalized_path(ordinary_output_dir)?,
        proof_root_path: normalized_path(proof_root)?,
    }))
}

fn finalize_prepared_authority(
    measured: RebuildAuthorityMeasurements,
    rebuild_args: &[OsString],
    materialized: BTreeMap<PathBuf, PathBuf>,
    authority_plan: RebuildAuthorityPlan,
) -> Result<PreparedRebuildAuthority, RunError> {
    let authority_plan_blake3 = rebuild_authority_plan_digest_blake3(authority_plan.clone()).map_err(core_error)?;
    let command_path = required_materialized_path(&materialized, &measured.command_path, "rebuild executable")?;
    let source_archive_path =
        required_materialized_path(&materialized, &measured.source_path, "release source archive")?;
    let command_args = remap_command_args(rebuild_args, &materialized);
    let mut read_only_paths = materialized.values().cloned().collect::<Vec<_>>();
    read_only_paths.sort();
    read_only_paths.dedup();
    if read_only_paths.is_empty() {
        return Err(RunError::Internal("genuine rebuild authority has no materialized read inputs".to_string()));
    }
    debug_assert!(authority_plan.eligible());
    debug_assert!(!read_only_paths.is_empty());
    Ok(PreparedRebuildAuthority {
        command_path,
        command_args,
        source_archive_path,
        read_only_paths,
        descriptor: measured.descriptor,
        descriptor_blake3: measured.descriptor_blake3,
        authority_plan,
        authority_plan_blake3,
    })
}

fn required_materialized_path(
    materialized: &BTreeMap<PathBuf, PathBuf>,
    original: &Path,
    label: &str,
) -> Result<PathBuf, RunError> {
    materialized
        .get(original)
        .cloned()
        .ok_or_else(|| RunError::Internal(format!("materialized {label} is missing")))
}

fn measure_targets(
    manifest: &ReleaseEvidenceManifest,
    bundle_dir: &Path,
) -> Result<Vec<RebuildInputObservation>, RunError> {
    let mut observations = Vec::with_capacity(manifest.binaries.len());
    for artifact in &manifest.binaries {
        let path = canonical_existing(&bundle_dir.join(&artifact.relative_path), "published target")?;
        let mut observation = measure_regular_file(&path, &artifact.relative_path, RebuildInputRole::PublishedTarget)?;
        if observation.identity.digest_blake3 != artifact.digest_blake3 {
            return Err(RunError::Internal(format!(
                "published target {} changed after release verification",
                artifact.relative_path
            )));
        }
        observation.identity.size_bytes = artifact.size_bytes;
        observations.push(observation);
    }
    if observations.is_empty() {
        return Err(RunError::Internal("genuine rebuild authority requires at least one published target".to_string()));
    }
    Ok(observations)
}

fn measure_declared_paths(
    bundle_dir: &Path,
    command_path: &Path,
    rebuild_args: &[OsString],
    source_path: &Path,
    provider_path: &Path,
    proof_root: &Path,
    ordinary_output_dir: &Path,
) -> Result<Vec<PathObservation>, RunError> {
    let command_is_recipe = file_starts_with_shebang(command_path)?;
    let recipe_arg = if command_is_recipe {
        None
    } else {
        first_existing_regular_argument(rebuild_args)?
    };
    let mut observations = Vec::new();
    observations.push(path_observation(
        measure_regular_file(command_path, "rebuild-executable", RebuildInputRole::Executable)?,
        command_path,
    ));
    if command_is_recipe {
        observations.push(path_observation(
            measure_regular_file(command_path, "rebuild-recipe", RebuildInputRole::Recipe)?,
            command_path,
        ));
    }
    observations.push(path_observation(
        measure_regular_file(source_path, "release-source-archive", RebuildInputRole::Source)?,
        source_path,
    ));
    observations.push(path_observation(
        measure_regular_file(provider_path, "provider-inventory", RebuildInputRole::Provider)?,
        provider_path,
    ));

    let mut tool_index = 0_u32;
    for arg in rebuild_args {
        let path = PathBuf::from(arg);
        if !path.is_absolute() || !path.exists() {
            continue;
        }
        let canonical = absolute_existing_no_follow(&path, "rebuild argument input")?;
        let role =
            classify_argument_role(&canonical, recipe_arg.as_deref(), bundle_dir, proof_root, ordinary_output_dir);
        let name = match role {
            RebuildInputRole::Recipe => "rebuild-recipe".to_string(),
            RebuildInputRole::Tool => {
                let name = format!("rebuild-tool-{tool_index:03}");
                tool_index = tool_index
                    .checked_add(1)
                    .ok_or_else(|| RunError::Internal("rebuild tool index overflowed u32".to_string()))?;
                name
            }
            _ => format!("forbidden-input-{tool_index:03}"),
        };
        let observation = measure_path(&canonical, &name, role)?;
        observations.push(path_observation(observation, &canonical));
    }
    Ok(observations)
}

fn classify_argument_role(
    path: &Path,
    recipe_arg: Option<&Path>,
    bundle_dir: &Path,
    proof_root: &Path,
    ordinary_output_dir: &Path,
) -> RebuildInputRole {
    if path == bundle_dir || bundle_dir.starts_with(path) {
        return RebuildInputRole::WholeBundle;
    }
    if paths_overlap(path, ordinary_output_dir) {
        return RebuildInputRole::OrdinaryOutput;
    }
    if paths_overlap(path, proof_root) {
        return RebuildInputRole::PriorOutput;
    }
    if recipe_arg == Some(path) {
        return RebuildInputRole::Recipe;
    }
    RebuildInputRole::Tool
}

fn file_starts_with_shebang(path: &Path) -> Result<bool, RunError> {
    const SHEBANG_BYTES: usize = 2;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|err| RunError::Internal(format!("metadata rebuild executable {}: {err}", path.display())))?;
    if !metadata.file_type().is_file() {
        return Ok(false);
    }
    let mut file = File::open(path)
        .map_err(|err| RunError::Internal(format!("open rebuild executable {}: {err}", path.display())))?;
    let mut prefix = [0_u8; SHEBANG_BYTES];
    let bytes_read = file
        .read(&mut prefix)
        .map_err(|err| RunError::Internal(format!("read rebuild executable {}: {err}", path.display())))?;
    Ok(bytes_read == SHEBANG_BYTES && prefix == *b"#!")
}

fn first_existing_regular_argument(args: &[OsString]) -> Result<Option<PathBuf>, RunError> {
    for arg in args {
        let path = PathBuf::from(arg);
        if !path.is_absolute() || !path.exists() {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|err| RunError::Internal(format!("metadata {}: {err}", path.display())))?;
        if metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Ok(Some(absolute_existing_no_follow(&path, "rebuild recipe argument")?));
        }
    }
    Ok(None)
}

fn append_policy_observations(observations: &mut Vec<PathObservation>) -> Result<(), RunError> {
    for (name, role, material) in [
        ("policy", RebuildInputRole::SandboxPolicy, SANDBOX_POLICY_IDENTITY),
        ("policy", RebuildInputRole::EffectPolicy, BUILD_EFFECT_POLICY_VERSION),
        ("policy", RebuildInputRole::NormalizationPolicy, NORMALIZATION_POLICY_IDENTITY),
    ] {
        let digest_blake3 = blake3::hash(material.as_bytes()).to_hex().to_string();
        observations.push(PathObservation {
            observation: RebuildInputObservation {
                identity: RebuildContentIdentity {
                    name: name.to_string(),
                    role,
                    kind: RebuildContentKind::SyntheticPolicy,
                    digest_blake3,
                    size_bytes: usize_to_u64(material.len(), "policy identity size")?,
                },
                normalized_path: format!("policy:{role:?}"),
                filesystem_object_identity: None,
            },
            original_path: None,
        });
    }
    Ok(())
}

fn append_recipe_authority_observations(
    observations: &mut Vec<PathObservation>,
    targets: &[RebuildInputObservation],
) -> Result<(), RunError> {
    let recipe = observations
        .iter()
        .find(|item| item.observation.identity.role == RebuildInputRole::Recipe)
        .and_then(|item| item.original_path.clone())
        .ok_or_else(|| RunError::Internal("genuine rebuild authority could not identify recipe bytes".to_string()))?;
    if recipe_contains(&recipe, FORBIDDEN_BUNDLE_ENV)? {
        observations.push(forbidden_recipe_observation(
            "recipe-reads-release-bundle",
            "recipe references MANTLE_REPRODUCE_BUNDLE_DIR",
        )?);
    }
    for target in targets {
        if recipe_contains(&recipe, target.observation_path_bytes())? {
            observations.push(forbidden_recipe_observation(
                &format!("recipe-target-path: {}", target.identity.name),
                "recipe embeds a published target path",
            )?);
        }
    }
    Ok(())
}

trait ObservationPathBytes {
    fn observation_path_bytes(&self) -> &[u8];
}

impl ObservationPathBytes for RebuildInputObservation {
    fn observation_path_bytes(&self) -> &[u8] {
        self.normalized_path.as_bytes()
    }
}

fn forbidden_recipe_observation(name: &str, detail: &str) -> Result<PathObservation, RunError> {
    let digest_blake3 = blake3::hash(detail.as_bytes()).to_hex().to_string();
    Ok(PathObservation {
        observation: RebuildInputObservation {
            identity: RebuildContentIdentity {
                name: name.to_string(),
                role: RebuildInputRole::PublishedTarget,
                kind: RebuildContentKind::SyntheticPolicy,
                digest_blake3,
                size_bytes: usize_to_u64(detail.len(), "recipe diagnostic size")?,
            },
            normalized_path: format!("recipe-observation:{name}"),
            filesystem_object_identity: None,
        },
        original_path: None,
    })
}

fn recipe_contains(path: &Path, needle: &[u8]) -> Result<bool, RunError> {
    let metadata = std::fs::metadata(path)
        .map_err(|err| RunError::Internal(format!("metadata rebuild recipe {}: {err}", path.display())))?;
    if metadata.len() > RECIPE_SCAN_BYTES_MAX {
        return Err(RunError::Internal(format!(
            "rebuild recipe {} exceeds bounded authority scan size {RECIPE_SCAN_BYTES_MAX}",
            path.display()
        )));
    }
    let bytes = std::fs::read(path)
        .map_err(|err| RunError::Internal(format!("reading rebuild recipe {}: {err}", path.display())))?;
    if needle.is_empty() || bytes.len() < needle.len() {
        return Ok(false);
    }
    Ok(bytes.windows(needle.len()).any(|window| window == needle))
}

fn build_descriptor(
    manifest: &ReleaseEvidenceManifest,
    rebuild_args: &[OsString],
    observations: &[PathObservation],
    run_roots: &[RebuildRunRootObservation],
) -> Result<ContentBoundRebuildDescriptor, RunError> {
    let identity_for = |role| {
        observations
            .iter()
            .find(|item| item.observation.identity.role == role)
            .map(|item| item.observation.identity.clone())
            .ok_or_else(|| RunError::Internal(format!("missing measured rebuild identity for {role:?}")))
    };
    let ordered_arguments = descriptor_arguments(rebuild_args, observations)?;
    let arguments_blake3 = rebuild_arguments_digest_blake3(ordered_arguments.clone()).map_err(core_error)?;
    let target_artifacts = manifest
        .binaries
        .iter()
        .map(|artifact| RebuildContentIdentity {
            name: artifact.relative_path.clone(),
            role: RebuildInputRole::PublishedTarget,
            kind: RebuildContentKind::RegularFile,
            digest_blake3: artifact.digest_blake3.clone(),
            size_bytes: artifact.size_bytes,
        })
        .collect();
    let tools = observations
        .iter()
        .filter(|item| item.observation.identity.role == RebuildInputRole::Tool)
        .map(|item| item.observation.identity.clone())
        .collect();
    let source_inputs = observations
        .iter()
        .filter(|item| item.observation.identity.role == RebuildInputRole::Source)
        .map(|item| item.observation.identity.clone())
        .collect();
    Ok(ContentBoundRebuildDescriptor {
        schema: CONTENT_BOUND_REBUILD_DESCRIPTOR_SCHEMA.to_string(),
        target_artifacts,
        recipe: identity_for(RebuildInputRole::Recipe)?,
        executable: identity_for(RebuildInputRole::Executable)?,
        tools,
        ordered_arguments,
        arguments_blake3,
        source_inputs,
        provider: identity_for(RebuildInputRole::Provider)?,
        policies: RebuildPolicyIdentities {
            sandbox_policy_blake3: identity_for(RebuildInputRole::SandboxPolicy)?.digest_blake3,
            effect_policy_blake3: identity_for(RebuildInputRole::EffectPolicy)?.digest_blake3,
            normalization_policy_blake3: identity_for(RebuildInputRole::NormalizationPolicy)?.digest_blake3,
        },
        run_roots: run_roots.iter().map(|root| root.identity.clone()).collect(),
    })
}

fn descriptor_arguments(args: &[OsString], observations: &[PathObservation]) -> Result<Vec<String>, RunError> {
    let mut by_path = BTreeMap::new();
    for item in observations {
        if let Some(path) = &item.original_path {
            by_path.insert(path.clone(), item.observation.identity.clone());
        }
    }
    args.iter()
        .map(|arg| {
            let text = arg
                .to_str()
                .ok_or_else(|| RunError::Internal("release rebuild arguments must be UTF-8".to_string()))?;
            let path = PathBuf::from(arg);
            if path.is_absolute() && path.exists() {
                let canonical = absolute_existing_no_follow(&path, "rebuild argument")?;
                if let Some(identity) = by_path.get(&canonical) {
                    return Ok(format!("input:{:?}:{}", identity.role, identity.digest_blake3));
                }
            }
            Ok(format!("literal:{text}"))
        })
        .collect()
}

fn run_root_observations(runs: &[RebuildRunPaths]) -> Result<Vec<RebuildRunRootObservation>, RunError> {
    let mut observations = Vec::with_capacity(runs.len());
    for run in runs {
        observations.push(RebuildRunRootObservation {
            identity: RebuildRunRootIdentity {
                run_id: run.run_id.clone(),
                output_root_identity: format!("{}:output", run.run_id),
                store_root_identity: format!("{}:store", run.run_id),
            },
            normalized_output_path: normalized_path(&run.output_dir)?,
            normalized_store_path: normalized_path(&run.store_dir)?,
        });
    }
    Ok(observations)
}

fn materialize_approved_inputs(
    proof_root: &Path,
    observations: &[PathObservation],
) -> Result<BTreeMap<PathBuf, PathBuf>, RunError> {
    let input_root = proof_root.join(INPUT_DIRECTORY_NAME);
    std::fs::create_dir(&input_root).map_err(|err| {
        RunError::Internal(format!("creating capability-scoped rebuild inputs {}: {err}", input_root.display()))
    })?;
    let originals = observations
        .iter()
        .filter(|item| is_declared_rebuild_input(item.observation.identity.role))
        .filter_map(|item| item.original_path.clone())
        .collect::<BTreeSet<_>>();
    let mut materialized = BTreeMap::new();
    for (index, original) in originals.iter().enumerate() {
        let index = u32::try_from(index)
            .map_err(|_| RunError::Internal("materialized rebuild input index overflowed u32".to_string()))?;
        let basename = original.file_name().and_then(|name| name.to_str()).unwrap_or("input");
        let destination_dir = input_root.join(format!("input-{index:03}"));
        std::fs::create_dir(&destination_dir).map_err(|err| {
            RunError::Internal(format!("creating rebuild input slot {}: {err}", destination_dir.display()))
        })?;
        let destination = destination_dir.join(basename);
        copy_regular_file(original, &destination)?;
        let original_digest = hash_file(original)?.1;
        let copied_digest = hash_file(&destination)?.1;
        if original_digest != copied_digest {
            return Err(RunError::Internal(format!(
                "materialized rebuild input digest drifted while copying {}",
                original.display()
            )));
        }
        materialized.insert(original.clone(), destination);
    }
    Ok(materialized)
}

fn remap_observations(
    observations: &[PathObservation],
    materialized: &BTreeMap<PathBuf, PathBuf>,
) -> Result<Vec<PathObservation>, RunError> {
    observations
        .iter()
        .map(|item| {
            let Some(original) = &item.original_path else {
                return Ok(item.clone());
            };
            let prepared = materialized.get(original).ok_or_else(|| {
                RunError::Internal(format!("approved rebuild input was not materialized: {}", original.display()))
            })?;
            let mut remapped = item.clone();
            remapped.observation.normalized_path = normalized_path(prepared)?;
            remapped.observation.filesystem_object_identity = filesystem_object_identity(prepared)?;
            remapped.original_path = Some(prepared.clone());
            Ok(remapped)
        })
        .collect()
}

fn remap_command_args(args: &[OsString], materialized: &BTreeMap<PathBuf, PathBuf>) -> Vec<OsString> {
    args.iter()
        .map(|arg| {
            let path = PathBuf::from(arg);
            if !path.is_absolute() || !path.exists() {
                return arg.clone();
            }
            match absolute_existing_no_follow(&path, "rebuild argument") {
                Ok(absolute) => materialized
                    .get(&absolute)
                    .map(|path| path.as_os_str().to_os_string())
                    .unwrap_or_else(|| arg.clone()),
                Err(_) => arg.clone(),
            }
        })
        .collect()
}

fn measure_path(path: &Path, name: &str, role: RebuildInputRole) -> Result<RebuildInputObservation, RunError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|err| RunError::Internal(format!("metadata rebuild input {}: {err}", path.display())))?;
    if metadata.file_type().is_symlink() {
        let target = std::fs::read_link(path)
            .map_err(|err| RunError::Internal(format!("read symlink rebuild input {}: {err}", path.display())))?;
        let bytes = target.as_os_str().as_encoded_bytes();
        return Ok(RebuildInputObservation {
            identity: RebuildContentIdentity {
                name: name.to_string(),
                role,
                kind: RebuildContentKind::Symlink,
                digest_blake3: blake3::hash(bytes).to_hex().to_string(),
                size_bytes: usize_to_u64(bytes.len(), "symlink target size")?,
            },
            normalized_path: normalized_path(path)?,
            filesystem_object_identity: filesystem_object_identity(path)?,
        });
    }
    if metadata.file_type().is_dir() {
        let digest_blake3 = blake3::hash(normalized_path(path)?.as_bytes()).to_hex().to_string();
        return Ok(RebuildInputObservation {
            identity: RebuildContentIdentity {
                name: name.to_string(),
                role,
                kind: RebuildContentKind::Other,
                digest_blake3,
                size_bytes: 0,
            },
            normalized_path: normalized_path(path)?,
            filesystem_object_identity: filesystem_object_identity(path)?,
        });
    }
    measure_regular_file(path, name, role)
}

fn measure_regular_file(path: &Path, name: &str, role: RebuildInputRole) -> Result<RebuildInputObservation, RunError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|err| RunError::Internal(format!("metadata rebuild input {}: {err}", path.display())))?;
    if !metadata.file_type().is_file() {
        return measure_path(path, name, role);
    }
    let (size_bytes, digest_blake3) = hash_file(path)?;
    Ok(RebuildInputObservation {
        identity: RebuildContentIdentity {
            name: name.to_string(),
            role,
            kind: RebuildContentKind::RegularFile,
            digest_blake3,
            size_bytes,
        },
        normalized_path: normalized_path(path)?,
        filesystem_object_identity: filesystem_object_identity(path)?,
    })
}

fn hash_file(path: &Path) -> Result<(u64, String), RunError> {
    let metadata =
        std::fs::metadata(path).map_err(|err| RunError::Internal(format!("metadata {}: {err}", path.display())))?;
    let mut file = File::open(path).map_err(|err| RunError::Internal(format!("open {}: {err}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| RunError::Internal(format!("read {}: {err}", path.display())))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok((metadata.len(), hasher.finalize().to_hex().to_string()))
}

fn copy_regular_file(source: &Path, destination: &Path) -> Result<(), RunError> {
    let metadata = std::fs::symlink_metadata(source)
        .map_err(|err| RunError::Internal(format!("metadata rebuild input {}: {err}", source.display())))?;
    if !metadata.file_type().is_file() {
        return Err(RunError::Internal(format!(
            "capability-scoped rebuild inputs must be regular files: {}",
            source.display()
        )));
    }
    std::fs::copy(source, destination).map_err(|err| {
        RunError::Internal(format!("copying rebuild input {} to {}: {err}", source.display(), destination.display()))
    })?;
    std::fs::set_permissions(destination, metadata.permissions())
        .map_err(|err| RunError::Internal(format!("setting permissions {}: {err}", destination.display())))
}

fn path_observation(observation: RebuildInputObservation, original_path: &Path) -> PathObservation {
    PathObservation {
        observation,
        original_path: Some(original_path.to_path_buf()),
    }
}

fn is_declared_rebuild_input(role: RebuildInputRole) -> bool {
    matches!(
        role,
        RebuildInputRole::Source
            | RebuildInputRole::Recipe
            | RebuildInputRole::Executable
            | RebuildInputRole::Tool
            | RebuildInputRole::Provider
            | RebuildInputRole::SandboxPolicy
            | RebuildInputRole::EffectPolicy
            | RebuildInputRole::NormalizationPolicy
    )
}

fn fail_if_authority_blocked(plan: &RebuildAuthorityPlan) -> Result<(), RunError> {
    if plan.eligible() {
        return Ok(());
    }
    let diagnostics = plan
        .blockers
        .iter()
        .map(|blocker| format!("{:?}:{}:{}", blocker.code, blocker.subject, blocker.detail))
        .collect::<Vec<_>>()
        .join("; ");
    Err(RunError::Internal(format!("deterministic genuine rebuild authority blocked: {diagnostics}")))
}

fn canonical_existing(path: &Path, label: &str) -> Result<PathBuf, RunError> {
    path.canonicalize()
        .map_err(|err| RunError::Internal(format!("resolving {label} {}: {err}", path.display())))
}

fn absolute_existing_no_follow(path: &Path, label: &str) -> Result<PathBuf, RunError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| RunError::Internal(format!("resolving current directory: {err}")))?
            .join(path)
    };
    std::fs::symlink_metadata(&absolute)
        .map_err(|err| RunError::Internal(format!("resolving {label} {}: {err}", absolute.display())))?;
    Ok(absolute)
}

fn normalized_path(path: &Path) -> Result<String, RunError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| RunError::Internal(format!("resolving current directory: {err}")))?
            .join(path)
    };
    absolute
        .to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| RunError::Internal(format!("rebuild authority path must be UTF-8: {}", absolute.display())))
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

#[cfg(unix)]
fn filesystem_object_identity(path: &Path) -> Result<Option<String>, RunError> {
    use std::os::unix::fs::MetadataExt;

    let metadata = std::fs::metadata(path)
        .map_err(|err| RunError::Internal(format!("metadata object identity {}: {err}", path.display())))?;
    Ok(Some(format!("device={};inode={}", metadata.dev(), metadata.ino())))
}

#[cfg(not(unix))]
fn filesystem_object_identity(_path: &Path) -> Result<Option<String>, RunError> {
    Ok(None)
}

fn usize_to_u64(value: usize, label: &str) -> Result<u64, RunError> {
    u64::try_from(value).map_err(|_| RunError::Internal(format!("{label} exceeds u64")))
}

fn core_error(err: crunch_release_core::ReleaseEvidenceError) -> RunError {
    RunError::Internal(err.to_string())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    const EXECUTABLE_MODE: u32 = 0o755;

    #[test]
    fn recipe_marker_is_classified_before_any_input_copy() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("recipe.sh");
        std::fs::write(&recipe, b"cp \"$MANTLE_REPRODUCE_BUNDLE_DIR/bin\" out\n").unwrap();
        let mut permissions = std::fs::metadata(&recipe).unwrap().permissions();
        permissions.set_mode(EXECUTABLE_MODE);
        std::fs::set_permissions(&recipe, permissions).unwrap();
        let mut observations = vec![path_observation(
            measure_regular_file(&recipe, "rebuild-recipe", RebuildInputRole::Recipe).unwrap(),
            &recipe,
        )];
        append_recipe_authority_observations(&mut observations, &[]).unwrap();

        assert_eq!(observations.len(), 2);
        assert_eq!(observations[1].observation.identity.role, RebuildInputRole::PublishedTarget);
    }

    #[test]
    fn materialization_copies_regular_files_without_hardlinking() {
        let dir = tempfile::tempdir().unwrap();
        let proof = dir.path().join("proof");
        std::fs::create_dir(&proof).unwrap();
        let source = dir.path().join("source");
        std::fs::write(&source, b"source-bytes").unwrap();
        let observations = vec![path_observation(
            measure_regular_file(&source, "source", RebuildInputRole::Source).unwrap(),
            &source,
        )];
        let materialized = materialize_approved_inputs(&proof, &observations).unwrap();
        let copied = materialized.get(&source).unwrap();

        assert_eq!(std::fs::read(copied).unwrap(), b"source-bytes");
        assert_ne!(filesystem_object_identity(&source).unwrap(), filesystem_object_identity(copied).unwrap());
    }
}
