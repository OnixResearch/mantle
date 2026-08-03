// r[impl mantlepkgs.producer_boundary]
// r[impl mantlepkgs.catalog_generation]
// r[impl mantlepkgs.recomputed_rebuild]
// r[verify mantlepkgs.producer_boundary]
// r[verify mantlepkgs.recomputed_rebuild]
// r[verify mantlepkgs.validation]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use clap::Subcommand;
use crunch_build::ExecutionProfile;
use mantlepkgs_core::ArtifactBinding;
use mantlepkgs_core::ArtifactObservation;
use mantlepkgs_core::CATALOG_JSON_PATH;
use mantlepkgs_core::CATALOG_NICKEL_PATH;
use mantlepkgs_core::CatalogBlocker;
use mantlepkgs_core::CatalogPlan;
use mantlepkgs_core::CoreFailure;
use mantlepkgs_core::EXECUTION_PROFILE_PATH;
use mantlepkgs_core::MantlepkgsCatalog;
use mantlepkgs_core::MantlepkgsManifest;
use mantlepkgs_core::PACKAGE_INDEX_PATH;
use mantlepkgs_core::PRODUCER_COMMAND_CLASS;
use mantlepkgs_core::PRODUCER_RECEIPT_PATH;
use mantlepkgs_core::PackageDisposition;
use mantlepkgs_core::PackageGraphObservation;
use mantlepkgs_core::ProducerObservation;
use mantlepkgs_core::ProducerReceipt;
use mantlepkgs_core::ProducerSelectionReceipt;
use mantlepkgs_core::RECOMPUTE_CONVERSION_MODE;
use mantlepkgs_core::ROLE_EXECUTION_PROFILE;
use mantlepkgs_core::ROLE_PACKAGE_INDEX;
use mantlepkgs_core::ROLE_PRODUCER_RECEIPT;
use mantlepkgs_core::ROLE_SHARED_GRAPH;
use mantlepkgs_core::ROLE_SOURCE_INVENTORY;
use mantlepkgs_core::ROLE_TRANSLATION_POLICY;
use mantlepkgs_core::SHARED_GRAPH_PATH;
use mantlepkgs_core::SOURCE_INVENTORY_PATH;
use mantlepkgs_core::SourceRequirementInventory;
use mantlepkgs_core::TRANSLATION_POLICY_PATH;
use mantlepkgs_core::build_producer_receipt;
use mantlepkgs_core::finalize_catalog;
use mantlepkgs_core::lookup_catalog_package;
use mantlepkgs_core::manifest_digest_blake3;
use mantlepkgs_core::normalize_manifest;
use mantlepkgs_core::plan_catalog;
use mantlepkgs_core::producer_receipt_digest_blake3;
use mantlepkgs_core::render_catalog_nickel;
use mantlepkgs_core::source_requirement_inventory;
use mantlepkgs_core::validate_catalog_artifacts;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::RunError;
use crate::foreign_derivation_import::ForeignDerivationGraph;
use crate::foreign_derivation_import::NixDerivationJsonExport;
use crate::foreign_derivation_import::NixProducerConfig;
use crate::foreign_derivation_import::PackageIndex;
use crate::foreign_derivation_import::TranslationPolicy;
use crate::foreign_derivation_import::lower_nix_derivation_json_closure;
use crate::foreign_derivation_import::normalize_nix_derivation_json_export;
use crate::foreign_derivation_import::select_nix_derivation_json_closure;
use crate::foreign_executable_plan::ForeignExecutablePlan;
use crate::foreign_executable_plan::compile_foreign_executable_plan_with_profile;
use crate::foreign_graph_compiler::compile_foreign_graph_with_profile_and_output_mode;
use crate::foreign_import_cmd::ForeignImportAction;
use crate::foreign_import_cmd::ForeignImportContext;
use crate::foreign_import_cmd::cmd_foreign_import;
use crate::linux_rename::rename_path_no_replace;
use crate::mantlepkgs_adapter::MergedForeignArtifacts;
use crate::mantlepkgs_adapter::ProducedPackageFacts;
use crate::mantlepkgs_adapter::merge_buildable_packages;
use crate::mantlepkgs_adapter::observe_package;
use crate::source_bundle::ForeignSourcePathBinding;
use crate::source_bundle::plan_bound_foreign_source_bundle;
use crate::source_bundle::write_json_atomically;

const PRODUCER_BATCH_SCHEMA: &str = "mantlepkgs-producer-batch-v1";
const GENERATION_FAILURE_SCHEMA: &str = "mantlepkgs-generation-failure-v1";
const FAILURE_DIRECTORY: &str = "failures";
const STAGE_PREFIX: &str = ".stage-";
const CATALOG_MAX_BYTES: u64 = 16_777_216;
const NIX_EXECUTABLE_MAX_BYTES: u64 = 268_435_456;
const NIX_ROOT_STDOUT_MAX_BYTES: usize = 4_096;
const PRODUCER_STDERR_MAX_BYTES: usize = 16_384;
const PRODUCER_COMMAND_TIMEOUT_SECS: u64 = 900;
const PRODUCER_COMMAND_POLL_MILLIS: u64 = 100;
const CAPTURE_OVERFLOW_SENTINEL_BYTES: u64 = 1;
const NIX_EXPERIMENTAL_FEATURES: &str = "nix-command flakes";
const FAILURE_EXIT_CODE: u8 = 1;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum MantlepkgsAction {
    /// Evaluate and validate a typed Mantlepkgs Nickel manifest without running Nix
    Validate {
        #[arg(long)]
        manifest: PathBuf,
    },

    /// Run the explicit locked Nixpkgs producer and publish one complete catalog generation
    Generate {
        #[arg(long)]
        manifest: PathBuf,

        /// Exact Nix executable to record and run
        #[arg(long = "nix-program")]
        nix_program: PathBuf,

        /// Root under which the manifest generation directory is created
        #[arg(long = "output-root")]
        output_root: PathBuf,
    },

    /// Publish previously recorded producer facts without running Nix
    Publish {
        #[arg(long)]
        manifest: PathBuf,

        #[arg(long = "producer-batch")]
        producer_batch: PathBuf,

        #[arg(long = "output-root")]
        output_root: PathBuf,
    },

    /// Verify one complete catalog generation and every bound artifact
    Verify {
        #[arg(long)]
        generation: PathBuf,
    },

    /// Compile one verified catalog package into the ordinary foreign executable plan
    Plan {
        #[arg(long)]
        generation: PathBuf,

        #[arg(long)]
        package: String,

        #[arg(long)]
        system: String,

        #[arg(long = "plan-out")]
        plan_out: PathBuf,

        #[arg(long = "import-receipt-out")]
        import_receipt_out: PathBuf,
    },

    /// Bind recorded foreign source paths into one explicit source bundle
    PrepareSources {
        #[arg(long)]
        generation: PathBuf,

        #[arg(long)]
        package: String,

        #[arg(long)]
        system: String,

        #[arg(long)]
        out: PathBuf,
    },

    /// Rebuild one verified package with Nix-free catalog consumption and no substitution
    Build {
        #[arg(long)]
        generation: PathBuf,

        #[arg(long)]
        package: String,

        #[arg(long)]
        system: String,

        #[arg(long = "source-bundle")]
        source_bundle: PathBuf,

        #[arg(long = "source-bundle-blake3")]
        source_bundle_blake3: String,

        #[arg(long = "plan-out")]
        plan_out: PathBuf,

        #[arg(long = "import-receipt-out")]
        import_receipt_out: PathBuf,

        #[arg(long = "receipt-out")]
        receipt_out: PathBuf,

        #[arg(short = 'j', long = "jobs")]
        jobs: Option<u32>,

        /// Existing Nix-format signing key for local PathInfo receipts
        #[arg(long = "signing-key")]
        signing_key: Option<PathBuf>,

        /// Reject network source retrieval as well as cache substitution
        #[arg(long)]
        offline: bool,
    },
}

pub(crate) struct MantlepkgsContext<'a> {
    pub(crate) output_dir: &'a Path,
    pub(crate) state_dir: &'a Path,
    pub(crate) base_state_dirs: &'a [PathBuf],
    pub(crate) verbose: bool,
    pub(crate) json: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProducerBatch {
    schema: String,
    producer: ProducerObservation,
    packages: Vec<ProducedPackageRecord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProducedPackageRecord {
    selector_name: String,
    system: String,
    expected_graph_digest_blake3: String,
    expected_index_digest_blake3: String,
    graph: Option<ForeignDerivationGraph>,
    package_index: Option<PackageIndex>,
    blockers: Vec<CatalogBlocker>,
}

#[derive(Debug, Serialize)]
struct GenerationFailure<'a> {
    schema: &'static str,
    plan: &'a CatalogPlan,
}

struct VerifiedGeneration {
    catalog: MantlepkgsCatalog,
    graph: ForeignDerivationGraph,
    package_index: PackageIndex,
    translation_policy: TranslationPolicy,
    execution_profile: ExecutionProfile,
    execution_profile_path: PathBuf,
}

struct CompiledSelection {
    plan: ForeignExecutablePlan,
    import_receipt: crate::foreign_derivation_import::ImportReceipt,
    execution_profile_path: PathBuf,
}

pub(crate) fn cmd_mantlepkgs(action: MantlepkgsAction, context: MantlepkgsContext<'_>) -> Result<(), RunError> {
    match action {
        MantlepkgsAction::Validate { manifest } => run_validate(&manifest, context.json),
        MantlepkgsAction::Generate {
            manifest,
            nix_program,
            output_root,
        } => run_generate(&manifest, &nix_program, &output_root, context.json),
        MantlepkgsAction::Publish {
            manifest,
            producer_batch,
            output_root,
        } => run_publish(&manifest, &producer_batch, &output_root, context.json),
        MantlepkgsAction::Verify { generation } => run_verify(&generation, context.json),
        MantlepkgsAction::Plan {
            generation,
            package,
            system,
            plan_out,
            import_receipt_out,
        } => run_plan(&generation, &package, &system, &plan_out, &import_receipt_out, context.json),
        MantlepkgsAction::PrepareSources {
            generation,
            package,
            system,
            out,
        } => run_prepare_sources(&generation, &package, &system, &out, context.json),
        MantlepkgsAction::Build {
            generation,
            package,
            system,
            source_bundle,
            source_bundle_blake3,
            plan_out,
            import_receipt_out,
            receipt_out,
            jobs,
            signing_key,
            offline,
        } => run_build(
            BuildRequest {
                generation: &generation,
                package: &package,
                system: &system,
                source_bundle: &source_bundle,
                source_bundle_blake3: &source_bundle_blake3,
                plan_out: &plan_out,
                import_receipt_out: &import_receipt_out,
                receipt_out: &receipt_out,
                jobs,
                signing_key: signing_key.as_deref(),
                offline,
            },
            &context,
        ),
    }
}

struct BuildRequest<'a> {
    generation: &'a Path,
    package: &'a str,
    system: &'a str,
    source_bundle: &'a Path,
    source_bundle_blake3: &'a str,
    plan_out: &'a Path,
    import_receipt_out: &'a Path,
    receipt_out: &'a Path,
    jobs: Option<u32>,
    signing_key: Option<&'a Path>,
    offline: bool,
}

fn run_validate(manifest_path: &Path, json: bool) -> Result<(), RunError> {
    let manifest = evaluate_manifest(manifest_path)?;
    let normalized = normalize_manifest(&manifest).map_err(core_eval_error)?;
    let digest = manifest_digest_blake3(&normalized).map_err(core_eval_error)?;
    if json {
        println!(
            "{}",
            serde_json::to_string(&normalized)
                .map_err(|error| RunError::Internal(format!("serializing normalized Mantlepkgs manifest: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs manifest accepted: selectors={} systems={} manifest_blake3={digest}",
            normalized.selectors.len(),
            normalized.systems.len()
        );
    }
    Ok(())
}

fn run_generate(manifest_path: &Path, nix_program: &Path, output_root: &Path, json: bool) -> Result<(), RunError> {
    let manifest = normalize_manifest(&evaluate_manifest(manifest_path)?).map_err(core_eval_error)?;
    let _ = load_policy_artifacts(manifest_path, &manifest)?;
    let batch = produce_locked_batch(&manifest, nix_program)?;
    publish_batch(manifest_path, &manifest, &batch, output_root, json)
}

fn run_publish(
    manifest_path: &Path,
    producer_batch_path: &Path,
    output_root: &Path,
    json: bool,
) -> Result<(), RunError> {
    let manifest = normalize_manifest(&evaluate_manifest(manifest_path)?).map_err(core_eval_error)?;
    let batch = read_json_bounded::<ProducerBatch>(producer_batch_path, manifest.limits.max_graph_bytes)?;
    publish_batch(manifest_path, &manifest, &batch, output_root, json)
}

fn run_verify(generation: &Path, json: bool) -> Result<(), RunError> {
    let verified = verify_generation(generation)?;
    if json {
        println!(
            "{}",
            serde_json::to_string(&verified.catalog)
                .map_err(|error| RunError::Internal(format!("serializing verified Mantlepkgs catalog: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs catalog verified: identity={} packages={} artifacts={}",
            verified.catalog.catalog_identity_blake3,
            verified.catalog.packages.len(),
            verified.catalog.artifacts.len()
        );
    }
    Ok(())
}

fn run_plan(
    generation: &Path,
    package: &str,
    system: &str,
    plan_out: &Path,
    import_receipt_out: &Path,
    json: bool,
) -> Result<(), RunError> {
    let selection = compile_catalog_selection(generation, package, system)?;
    reject_equal_output_paths(plan_out, import_receipt_out)?;
    write_json_atomically(plan_out, &selection.plan, "Mantlepkgs foreign executable plan")?;
    write_json_atomically(import_receipt_out, &selection.import_receipt, "Mantlepkgs foreign import receipt")?;
    if json {
        println!(
            "{}",
            serde_json::to_string(&selection.plan)
                .map_err(|error| RunError::Internal(format!("serializing Mantlepkgs plan: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs package planned: package={package} system={system} plan_blake3={}",
            selection.plan.plan_identity.value
        );
    }
    Ok(())
}

fn run_prepare_sources(
    generation: &Path,
    package: &str,
    system: &str,
    output: &Path,
    json: bool,
) -> Result<(), RunError> {
    let selection = compile_catalog_selection(generation, package, system)?;
    let bindings = selection
        .plan
        .source_requirements
        .iter()
        .map(|requirement| ForeignSourcePathBinding {
            payload_id: requirement.payload_id.clone(),
            path: PathBuf::from(&requirement.foreign_path),
        })
        .collect::<Vec<_>>();
    let bundle = plan_bound_foreign_source_bundle(
        &selection.plan.source_requirements,
        &bindings,
        &selection.plan.target_store_prefix,
    )?;
    write_json_atomically(output, &bundle, "Mantlepkgs source bundle")?;
    if json {
        println!(
            "{}",
            serde_json::to_string(&bundle)
                .map_err(|error| RunError::Internal(format!("serializing Mantlepkgs source bundle: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs sources prepared: package={package} records={} manifest_blake3={} path={}",
            bundle.records.len(),
            bundle.manifest_blake3,
            output.display()
        );
    }
    Ok(())
}

fn run_build(request: BuildRequest<'_>, context: &MantlepkgsContext<'_>) -> Result<(), RunError> {
    let selection = compile_catalog_selection(request.generation, request.package, request.system)?;
    reject_distinct_build_outputs(&request)?;
    write_json_atomically(request.plan_out, &selection.plan, "Mantlepkgs foreign executable plan")?;
    write_json_atomically(request.import_receipt_out, &selection.import_receipt, "Mantlepkgs foreign import receipt")?;
    cmd_foreign_import(
        ForeignImportAction::Realize {
            plan: request.plan_out.to_path_buf(),
            import_receipt: request.import_receipt_out.to_path_buf(),
            source_bundle: request.source_bundle.to_path_buf(),
            source_bundle_blake3: request.source_bundle_blake3.into(),
            execution_profiles: vec![selection.execution_profile_path],
            cache_closure_policy: None,
            roots: Vec::new(),
            receipt_out: request.receipt_out.to_path_buf(),
            jobs: request.jobs,
            substitute: false,
            no_substitute: true,
            offline: request.offline,
            remote: false,
            signing_key: request.signing_key.map(Path::to_path_buf),
        },
        ForeignImportContext {
            output_dir: context.output_dir,
            state_dir: context.state_dir,
            base_state_dirs: context.base_state_dirs,
            verbose: context.verbose,
            json: context.json,
        },
    )
}

fn publish_batch(
    manifest_path: &Path,
    manifest: &MantlepkgsManifest,
    batch: &ProducerBatch,
    output_root: &Path,
    json: bool,
) -> Result<(), RunError> {
    validate_batch_header(batch)?;
    let (policy_bytes, profile_bytes, policy, profile) = load_policy_artifacts(manifest_path, manifest)?;
    let facts = batch.packages.iter().map(produced_facts).collect::<Vec<_>>();
    let observations = facts
        .iter()
        .map(|facts| {
            manifest
                .selectors
                .iter()
                .find(|selector| selector.system == facts.system && selector.name == facts.selector_name)
                .map(|selector| observe_buildability(selector, facts, &policy, &profile))
                .unwrap_or_else(|| unexpected_producer_observation(facts))
        })
        .collect::<Vec<_>>();
    let plan = plan_catalog(manifest, &batch.producer, &observations).map_err(core_eval_error)?;
    if !plan.batch_complete {
        return publish_failure_report(output_root, &plan, json);
    }
    let merged = merge_buildable_packages(&facts, &manifest.selectors).map_err(blocker_error)?;
    validate_compilation_contract(manifest, &merged, &policy, &profile)?;
    let publication = prepare_publication(&plan, &merged, &policy_bytes, &profile_bytes, batch)?;
    publish_generation(output_root, &plan, &publication, json)
}

struct PreparedPublication {
    catalog: MantlepkgsCatalog,
    files: BTreeMap<String, Vec<u8>>,
}

fn prepare_publication(
    plan: &CatalogPlan,
    merged: &MergedForeignArtifacts,
    policy_bytes: &[u8],
    profile_bytes: &[u8],
    batch: &ProducerBatch,
) -> Result<PreparedPublication, RunError> {
    let graph_bytes = pretty_json_bytes(&merged.graph, "shared foreign graph")?;
    let index_bytes = pretty_json_bytes(&merged.package_index, "package index")?;
    let source_inventory = source_requirement_inventory(plan);
    let inventory_bytes = pretty_json_bytes(&source_inventory, "source requirement inventory")?;
    let producer_artifacts = vec![
        binding(ROLE_SHARED_GRAPH, SHARED_GRAPH_PATH, &graph_bytes)?,
        binding(ROLE_PACKAGE_INDEX, PACKAGE_INDEX_PATH, &index_bytes)?,
        binding(ROLE_SOURCE_INVENTORY, SOURCE_INVENTORY_PATH, &inventory_bytes)?,
        binding(ROLE_TRANSLATION_POLICY, TRANSLATION_POLICY_PATH, policy_bytes)?,
        binding(ROLE_EXECUTION_PROFILE, EXECUTION_PROFILE_PATH, profile_bytes)?,
    ];
    let selections = producer_selection_receipts(plan, batch)?;
    let producer_receipt =
        build_producer_receipt(plan, &selections, &producer_artifacts).map_err(core_internal_error)?;
    let producer_receipt_digest = producer_receipt_digest_blake3(&producer_receipt).map_err(core_internal_error)?;
    let receipt_bytes = pretty_json_bytes(&producer_receipt, "producer receipt")?;
    let mut catalog_artifacts = producer_artifacts;
    catalog_artifacts.push(binding(ROLE_PRODUCER_RECEIPT, PRODUCER_RECEIPT_PATH, &receipt_bytes)?);
    let catalog = finalize_catalog(plan, &producer_receipt_digest, &catalog_artifacts).map_err(core_internal_error)?;
    let catalog_bytes = pretty_json_bytes(&catalog, "Mantlepkgs catalog")?;
    let catalog_nickel = render_catalog_nickel(&catalog).map_err(core_internal_error)?.into_bytes();
    let files = BTreeMap::from([
        (SHARED_GRAPH_PATH.into(), graph_bytes),
        (PACKAGE_INDEX_PATH.into(), index_bytes),
        (SOURCE_INVENTORY_PATH.into(), inventory_bytes),
        (TRANSLATION_POLICY_PATH.into(), policy_bytes.to_vec()),
        (EXECUTION_PROFILE_PATH.into(), profile_bytes.to_vec()),
        (PRODUCER_RECEIPT_PATH.into(), receipt_bytes),
        (CATALOG_JSON_PATH.into(), catalog_bytes),
        (CATALOG_NICKEL_PATH.into(), catalog_nickel),
    ]);
    Ok(PreparedPublication { catalog, files })
}

fn publish_generation(
    output_root: &Path,
    plan: &CatalogPlan,
    publication: &PreparedPublication,
    json: bool,
) -> Result<(), RunError> {
    let generations = output_root.join(&plan.manifest.output.generation_directory);
    fs::create_dir_all(&generations).map_err(|error| {
        RunError::Internal(format!("creating Mantlepkgs generation root {}: {error}", generations.display()))
    })?;
    let stage = generations.join(format!("{STAGE_PREFIX}{}", publication.catalog.catalog_identity_blake3));
    let final_path = generations.join(&publication.catalog.catalog_identity_blake3);
    fs::create_dir(&stage)
        .map_err(|error| RunError::Internal(format!("creating Mantlepkgs stage {}: {error}", stage.display())))?;
    let staged = stage_publication(&stage, publication).and_then(|()| verify_generation(&stage).map(|_| ()));
    if let Err(error) = staged {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    sync_directory(&stage)?;
    if let Err(error) = rename_path_no_replace(&stage, &final_path) {
        let _ = fs::remove_dir_all(&stage);
        return Err(RunError::Internal(format!(
            "publishing Mantlepkgs generation without replacement {}: {error}",
            final_path.display()
        )));
    }
    sync_directory(&generations)?;
    emit_publication(&publication.catalog, &final_path, json)
}

fn stage_publication(stage: &Path, publication: &PreparedPublication) -> Result<(), RunError> {
    for (relative, bytes) in &publication.files {
        if bytes.len() as u64 > publication.catalog.max_artifact_bytes {
            return Err(RunError::Internal(format!("Mantlepkgs staged artifact exceeds limit: {relative}")));
        }
        let target = stage.join(relative);
        let parent = target
            .parent()
            .ok_or_else(|| RunError::Internal(format!("Mantlepkgs artifact has no parent: {relative}")))?;
        fs::create_dir_all(parent).map_err(|error| {
            RunError::Internal(format!("creating Mantlepkgs artifact directory {}: {error}", parent.display()))
        })?;
        write_new_bytes(&target, bytes)?;
    }
    Ok(())
}

fn verify_generation(generation: &Path) -> Result<VerifiedGeneration, RunError> {
    reject_symlink_root(generation)?;
    let catalog_path = confined_file(generation, CATALOG_JSON_PATH)?;
    let catalog = read_json_bounded::<MantlepkgsCatalog>(&catalog_path, CATALOG_MAX_BYTES)?;
    let mut observations = Vec::with_capacity(catalog.artifacts.len());
    let mut artifact_bytes = BTreeMap::new();
    for artifact in &catalog.artifacts {
        if artifact.bytes > catalog.max_artifact_bytes {
            return Err(RunError::Internal(format!("catalog artifact exceeds declared byte limit: {}", artifact.path)));
        }
        let path = confined_file(generation, &artifact.path)?;
        let bytes = read_bounded(&path, catalog.max_artifact_bytes)?;
        observations.push(ArtifactObservation {
            path: artifact.path.clone(),
            digest_blake3: blake3_hex(&bytes),
            bytes: bytes.len() as u64,
        });
        artifact_bytes.insert(artifact.role.clone(), bytes);
    }
    validate_catalog_artifacts(&catalog, &observations).map_err(core_internal_error)?;
    verify_catalog_nickel(generation, &catalog)?;
    let receipt = deserialize_role::<ProducerReceipt>(&artifact_bytes, ROLE_PRODUCER_RECEIPT)?;
    verify_producer_receipt(&catalog, &receipt)?;
    let graph = deserialize_role::<ForeignDerivationGraph>(&artifact_bytes, ROLE_SHARED_GRAPH)?;
    let package_index = deserialize_role::<PackageIndex>(&artifact_bytes, ROLE_PACKAGE_INDEX)?;
    let translation_policy = deserialize_role::<TranslationPolicy>(&artifact_bytes, ROLE_TRANSLATION_POLICY)?;
    let execution_profile = deserialize_role::<ExecutionProfile>(&artifact_bytes, ROLE_EXECUTION_PROFILE)?;
    let source_inventory = deserialize_role::<SourceRequirementInventory>(&artifact_bytes, ROLE_SOURCE_INVENTORY)?;
    verify_policy(&catalog, &translation_policy)?;
    verify_generation_structure(
        &catalog,
        &graph,
        &package_index,
        &source_inventory,
        &translation_policy,
        &execution_profile,
    )?;
    let profile_artifact = catalog
        .artifacts
        .iter()
        .find(|artifact| artifact.role == ROLE_EXECUTION_PROFILE)
        .ok_or_else(|| RunError::Internal("catalog has no execution-profile artifact".into()))?;
    let execution_profile_path = confined_file(generation, &profile_artifact.path)?;
    Ok(VerifiedGeneration {
        catalog,
        graph,
        package_index,
        translation_policy,
        execution_profile,
        execution_profile_path,
    })
}

fn compile_catalog_selection(
    generation: &Path,
    requested_package: &str,
    system: &str,
) -> Result<CompiledSelection, RunError> {
    let verified = verify_generation(generation)?;
    let package = lookup_catalog_package(&verified.catalog, requested_package, system).map_err(core_eval_error)?;
    let entry = verified
        .package_index
        .entries
        .iter()
        .find(|entry| entry.name == package.name && entry.system == package.system)
        .cloned()
        .ok_or_else(|| {
            RunError::Internal(format!("verified catalog package has no exact index entry: {}", package.name))
        })?;
    let PackageDisposition::Buildable { root_node_id, .. } = &package.disposition else {
        return Err(RunError::Internal(format!("verified package is not buildable: {}", package.name)));
    };
    if entry.root_derivation_id != *root_node_id {
        return Err(RunError::Internal(format!("catalog and index roots differ for package {}", package.name)));
    }
    let mut scoped_graph = verified.graph.clone();
    scoped_graph.root_derivation_ids = vec![root_node_id.clone()];
    let scoped_index = PackageIndex {
        schema: verified.package_index.schema.clone(),
        entries: vec![entry],
    };
    let (plan, import_receipt) = compile_foreign_executable_plan_with_profile(
        &scoped_graph,
        &scoped_index,
        &verified.translation_policy,
        &package.name,
        system,
        &verified.execution_profile,
    )
    .map_err(import_diagnostic_error)?;
    if plan.target_store_prefix != verified.catalog.target_store_prefix {
        return Err(RunError::Internal("compiled Mantlepkgs plan changed the catalog target store prefix".into()));
    }
    Ok(CompiledSelection {
        plan,
        import_receipt,
        execution_profile_path: verified.execution_profile_path,
    })
}

fn produce_locked_batch(manifest: &MantlepkgsManifest, nix_program: &Path) -> Result<ProducerBatch, RunError> {
    if !nix_program.is_absolute() {
        return Err(RunError::Eval("--nix-program must be one exact absolute executable path".into()));
    }
    let executable_bytes = read_bounded(nix_program, NIX_EXECUTABLE_MAX_BYTES)?;
    let version_output = run_nix(nix_program, &["--version"], NIX_ROOT_STDOUT_MAX_BYTES)?;
    if !version_output.status.success() {
        return Err(RunError::Internal(format!(
            "Nix producer version command failed: {}",
            bounded_stderr(&version_output)
        )));
    }
    let producer = ProducerObservation {
        command_class: PRODUCER_COMMAND_CLASS.into(),
        implementation_id: fs::canonicalize(nix_program)
            .map_err(|error| RunError::Internal(format!("resolving Nix producer {}: {error}", nix_program.display())))?
            .display()
            .to_string(),
        version: String::from_utf8_lossy(&version_output.stdout).trim().into(),
        executable_digest_blake3: blake3_hex(&executable_bytes),
        source_lock: manifest.source.clone(),
    };
    let packages = manifest
        .selectors
        .iter()
        .map(|selector| produce_selector(manifest, selector, nix_program))
        .collect::<Vec<_>>();
    Ok(ProducerBatch {
        schema: PRODUCER_BATCH_SCHEMA.into(),
        producer,
        packages,
    })
}

fn produce_selector(
    manifest: &MantlepkgsManifest,
    selector: &mantlepkgs_core::PackageSelector,
    nix_program: &Path,
) -> ProducedPackageRecord {
    match produce_selector_result(manifest, selector, nix_program) {
        Ok(record) => record,
        Err(blocker) => ProducedPackageRecord {
            selector_name: selector.name.clone(),
            system: selector.system.clone(),
            expected_graph_digest_blake3: String::new(),
            expected_index_digest_blake3: String::new(),
            graph: None,
            package_index: None,
            blockers: vec![blocker],
        },
    }
}

fn produce_selector_result(
    manifest: &MantlepkgsManifest,
    selector: &mantlepkgs_core::PackageSelector,
    nix_program: &Path,
) -> Result<ProducedPackageRecord, CatalogBlocker> {
    let installable = format!("{}#{}", manifest.source.reference, selector.attribute);
    let root_output = run_nix(
        nix_program,
        &[
            "--extra-experimental-features",
            NIX_EXPERIMENTAL_FEATURES,
            "eval",
            "--raw",
            &format!("{installable}.drvPath"),
        ],
        NIX_ROOT_STDOUT_MAX_BYTES,
    )
    .map_err(|error| producer_blocker("nix-root-evaluation-failed", selector, &error.to_string()))?;
    require_success(&root_output, "nix-root-evaluation-failed", selector)?;
    let root_derivation = String::from_utf8_lossy(&root_output.stdout).trim().to_string();
    let graph_limit = usize::try_from(manifest.limits.max_graph_bytes).unwrap_or(usize::MAX);
    let graph_output = run_nix(
        nix_program,
        &[
            "--extra-experimental-features",
            NIX_EXPERIMENTAL_FEATURES,
            "derivation",
            "show",
            "--recursive",
            &root_derivation,
        ],
        graph_limit,
    )
    .map_err(|error| producer_blocker("nix-graph-evaluation-failed", selector, &error.to_string()))?;
    require_success(&graph_output, "nix-graph-evaluation-failed", selector)?;
    let export = serde_json::from_slice::<NixDerivationJsonExport>(&graph_output.stdout)
        .map_err(|error| producer_blocker("nix-graph-json-invalid", selector, &error.to_string()))?;
    let closure = normalize_nix_derivation_json_export(export)
        .map_err(|diagnostic| producer_blocker(&diagnostic.class, selector, &diagnostic.message))?;
    let selected = select_nix_derivation_json_closure(&closure, &root_derivation)
        .map_err(|diagnostic| producer_blocker(&diagnostic.class, selector, &diagnostic.message))?;
    let artifacts = lower_nix_derivation_json_closure(&selected, &NixProducerConfig {
        package_name: selector.name.clone(),
        system: selector.system.clone(),
        root_derivation,
        producer_identity: installable,
        producer_revision: manifest.source.revision.clone(),
        cache_hints: Vec::new(),
        unsupported_metadata_classes: Vec::new(),
    })
    .map_err(|diagnostic| producer_blocker(&diagnostic.class, selector, &diagnostic.message))?;
    Ok(ProducedPackageRecord {
        selector_name: selector.name.clone(),
        system: selector.system.clone(),
        expected_graph_digest_blake3: canonical_digest(&artifacts.graph).unwrap_or_default(),
        expected_index_digest_blake3: canonical_digest(&artifacts.package_index).unwrap_or_default(),
        graph: Some(artifacts.graph),
        package_index: Some(artifacts.package_index),
        blockers: Vec::new(),
    })
}

fn load_policy_artifacts(
    manifest_path: &Path,
    manifest: &MantlepkgsManifest,
) -> Result<(Vec<u8>, Vec<u8>, TranslationPolicy, ExecutionProfile), RunError> {
    let policy_bytes = read_manifest_artifact(manifest_path, &manifest.conversion_policy.translation_policy, manifest)?;
    let profile_bytes = read_manifest_artifact(manifest_path, &manifest.conversion_policy.execution_profile, manifest)?;
    let policy = deserialize_json::<TranslationPolicy>(&policy_bytes, "Mantlepkgs translation policy")?;
    let profile = deserialize_json::<ExecutionProfile>(&profile_bytes, "Mantlepkgs execution profile")?;
    if manifest.conversion_policy.mode != RECOMPUTE_CONVERSION_MODE
        || policy.output_path_recompute_mode != RECOMPUTE_CONVERSION_MODE
        || policy.target_prefix != manifest.conversion_policy.target_store_prefix
    {
        return Err(RunError::Eval(
            "Mantlepkgs translation policy does not match the recompute conversion contract".into(),
        ));
    }
    Ok((policy_bytes, profile_bytes, policy, profile))
}

fn observe_buildability(
    selector: &mantlepkgs_core::PackageSelector,
    facts: &ProducedPackageFacts,
    policy: &TranslationPolicy,
    profile: &ExecutionProfile,
) -> PackageGraphObservation {
    let mut observation = observe_package(selector, facts, &policy.target_prefix);
    if let (Some(graph), Some(index)) = (&facts.graph, &facts.package_index)
        && let Err(diagnostic) = compile_foreign_executable_plan_with_profile(
            graph,
            index,
            policy,
            &selector.name,
            &selector.system,
            profile,
        )
    {
        observation.blockers.push(CatalogBlocker::new(
            &diagnostic.class,
            diagnostic.node_id.as_deref().unwrap_or(&selector.name),
            &diagnostic.message,
        ));
    }
    observation.blockers.sort();
    observation.blockers.dedup();
    observation
}

fn validate_compilation_contract(
    manifest: &MantlepkgsManifest,
    merged: &MergedForeignArtifacts,
    policy: &TranslationPolicy,
    profile: &ExecutionProfile,
) -> Result<(), RunError> {
    if manifest.conversion_policy.mode != RECOMPUTE_CONVERSION_MODE
        || policy.output_path_recompute_mode != RECOMPUTE_CONVERSION_MODE
        || policy.target_prefix != manifest.conversion_policy.target_store_prefix
    {
        return Err(RunError::Eval(
            "Mantlepkgs translation policy does not match the recompute conversion contract".into(),
        ));
    }
    compile_foreign_graph_with_profile_and_output_mode(
        &merged.graph,
        &policy.target_prefix,
        profile,
        &policy.output_path_recompute_mode,
    )
    .map_err(import_diagnostic_error)?;
    Ok(())
}

fn producer_selection_receipts(
    plan: &CatalogPlan,
    batch: &ProducerBatch,
) -> Result<Vec<ProducerSelectionReceipt>, RunError> {
    let mut receipts = Vec::with_capacity(plan.packages.len());
    for package in &plan.packages {
        let produced = batch
            .packages
            .iter()
            .find(|item| item.selector_name == package.name && item.system == package.system)
            .ok_or_else(|| RunError::Internal(format!("completed plan has no producer package: {}", package.name)))?;
        let graph = produced
            .graph
            .as_ref()
            .ok_or_else(|| RunError::Internal(format!("completed package has no graph: {}", package.name)))?;
        let root_node = match &package.disposition {
            PackageDisposition::Buildable { root_node_id, .. } => root_node_id,
            PackageDisposition::Blocked { .. } => {
                return Err(RunError::Internal(format!("completed plan contains blocked package: {}", package.name)));
            }
        };
        let root_derivation = graph
            .nodes
            .iter()
            .find(|node| node.node_id == *root_node)
            .map(|node| node.original_derivation.clone())
            .ok_or_else(|| RunError::Internal(format!("producer graph has no selected root node: {}", package.name)))?;
        receipts.push(ProducerSelectionReceipt {
            name: package.name.clone(),
            attribute: package.attribute.clone(),
            system: package.system.clone(),
            root_derivation,
            graph_digest_blake3: produced.expected_graph_digest_blake3.clone(),
            index_digest_blake3: produced.expected_index_digest_blake3.clone(),
        });
    }
    Ok(receipts)
}

fn publish_failure_report(output_root: &Path, plan: &CatalogPlan, json: bool) -> Result<(), RunError> {
    let failures = output_root.join(&plan.manifest.output.generation_directory).join(FAILURE_DIRECTORY);
    fs::create_dir_all(&failures).map_err(|error| {
        RunError::Internal(format!("creating Mantlepkgs failure directory {}: {error}", failures.display()))
    })?;
    let target = failures.join(format!("{}.json", plan.plan_identity_blake3));
    let stage = failures.join(format!("{STAGE_PREFIX}{}.json", plan.plan_identity_blake3));
    let bytes = pretty_json_bytes(
        &GenerationFailure {
            schema: GENERATION_FAILURE_SCHEMA,
            plan,
        },
        "Mantlepkgs generation failure",
    )?;
    write_new_bytes(&stage, &bytes)?;
    rename_path_no_replace(&stage, &target).map_err(|error| {
        RunError::Internal(format!("publishing Mantlepkgs failure report {}: {error}", target.display()))
    })?;
    if json {
        println!("{}", String::from_utf8_lossy(&bytes).trim());
    } else {
        println!(
            "mantlepkgs generation blocked: plan_blake3={} report={}",
            plan.plan_identity_blake3,
            target.display()
        );
        for package in &plan.packages {
            if let PackageDisposition::Blocked { blockers } = &package.disposition {
                for blocker in blockers {
                    println!("blocked package={} code={} subject={}", package.name, blocker.code, blocker.subject);
                }
            }
        }
    }
    Err(RunError::Reported(FAILURE_EXIT_CODE))
}

fn evaluate_manifest(path: &Path) -> Result<MantlepkgsManifest, RunError> {
    let import_paths = path.parent().map(|parent| vec![OsString::from(parent)]).unwrap_or_default();
    crunch_eval::evaluate_and_deserialize(path, &import_paths)
        .map_err(|error| RunError::Eval(format!("evaluating typed Mantlepkgs manifest {}: {error}", path.display())))
}

fn read_manifest_artifact(
    manifest_path: &Path,
    artifact: &mantlepkgs_core::ManifestArtifact,
    manifest: &MantlepkgsManifest,
) -> Result<Vec<u8>, RunError> {
    let parent = manifest_path
        .parent()
        .ok_or_else(|| RunError::Eval(format!("Mantlepkgs manifest has no parent: {}", manifest_path.display())))?;
    let path = confined_file(parent, &artifact.path)?;
    let bytes = read_bounded(&path, manifest.limits.max_artifact_bytes)?;
    if blake3_hex(&bytes) != artifact.digest_blake3 {
        return Err(RunError::Eval(format!("Mantlepkgs manifest artifact digest mismatch: {}", artifact.path)));
    }
    Ok(bytes)
}

fn verify_catalog_nickel(generation: &Path, catalog: &MantlepkgsCatalog) -> Result<(), RunError> {
    let path = confined_file(generation, CATALOG_NICKEL_PATH)?;
    let rendered =
        crunch_eval::evaluate_and_deserialize::<MantlepkgsCatalog>(&path, &[generation.into()]).map_err(|error| {
            RunError::Eval(format!("evaluating generated Mantlepkgs catalog {}: {error}", path.display()))
        })?;
    if rendered != *catalog {
        return Err(RunError::Internal("generated Nickel catalog differs from catalog.json".into()));
    }
    Ok(())
}

fn verify_producer_receipt(catalog: &MantlepkgsCatalog, receipt: &ProducerReceipt) -> Result<(), RunError> {
    let digest = producer_receipt_digest_blake3(receipt).map_err(core_internal_error)?;
    if digest != catalog.producer_receipt_digest_blake3 || receipt.source_lock != catalog.source_lock {
        return Err(RunError::Internal(
            "Mantlepkgs producer receipt identity or source lock differs from the catalog".into(),
        ));
    }
    let catalog_roles = catalog
        .artifacts
        .iter()
        .filter(|artifact| artifact.role != ROLE_PRODUCER_RECEIPT)
        .cloned()
        .collect::<Vec<_>>();
    if receipt.artifacts != catalog_roles {
        return Err(RunError::Internal("Mantlepkgs producer receipt artifact set differs from the catalog".into()));
    }
    Ok(())
}

fn verify_policy(catalog: &MantlepkgsCatalog, policy: &TranslationPolicy) -> Result<(), RunError> {
    if policy.output_path_recompute_mode != RECOMPUTE_CONVERSION_MODE
        || policy.target_prefix != catalog.target_store_prefix
    {
        return Err(RunError::Internal(
            "Mantlepkgs catalog policy does not require recomputed Mantle identities".into(),
        ));
    }
    Ok(())
}

fn verify_generation_structure(
    catalog: &MantlepkgsCatalog,
    graph: &ForeignDerivationGraph,
    index: &PackageIndex,
    sources: &SourceRequirementInventory,
    policy: &TranslationPolicy,
    profile: &ExecutionProfile,
) -> Result<(), RunError> {
    let node_count = u32::try_from(graph.nodes.len())
        .map_err(|_| RunError::Internal("Mantlepkgs graph node count overflow".into()))?;
    let source_count = u32::try_from(sources.requirements.len())
        .map_err(|_| RunError::Internal("Mantlepkgs source requirement count overflow".into()))?;
    if node_count != catalog.shared_node_count || source_count != catalog.source_requirement_count {
        return Err(RunError::Internal("Mantlepkgs catalog count differs from a bound artifact".into()));
    }
    if graph.producer.revision != catalog.source_lock.revision {
        return Err(RunError::Internal("Mantlepkgs graph revision differs from the catalog source lock".into()));
    }
    for package in &catalog.packages {
        let PackageDisposition::Buildable { root_node_id, .. } = &package.disposition else {
            return Err(RunError::Internal("a published success catalog contains a blocked package".into()));
        };
        let exact_entry = index.entries.iter().any(|entry| {
            entry.name == package.name
                && entry.system == package.system
                && entry.aliases == package.aliases
                && entry.root_derivation_id == *root_node_id
        });
        if !exact_entry || !graph.root_derivation_ids.contains(root_node_id) {
            return Err(RunError::Internal(format!(
                "Mantlepkgs catalog, graph, and index differ for package {}",
                package.name
            )));
        }
    }
    let compiled = compile_foreign_graph_with_profile_and_output_mode(
        graph,
        &policy.target_prefix,
        profile,
        &policy.output_path_recompute_mode,
    )
    .map_err(import_diagnostic_error)?;
    if compiled.source_requirements.len() != sources.requirements.len() {
        return Err(RunError::Internal(
            "Mantlepkgs compiled source requirement count differs from the inventory".into(),
        ));
    }
    Ok(())
}

fn validate_batch_header(batch: &ProducerBatch) -> Result<(), RunError> {
    if batch.schema != PRODUCER_BATCH_SCHEMA {
        return Err(RunError::Eval(format!("unsupported Mantlepkgs producer batch schema: {}", batch.schema)));
    }
    Ok(())
}

fn unexpected_producer_observation(facts: &ProducedPackageFacts) -> PackageGraphObservation {
    PackageGraphObservation {
        selector_name: facts.selector_name.clone(),
        system: facts.system.clone(),
        root_node_id: String::new(),
        producer_graph_digest_blake3: facts.expected_graph_digest_blake3.clone(),
        observed_graph_digest_blake3: String::new(),
        producer_index_digest_blake3: facts.expected_index_digest_blake3.clone(),
        observed_index_digest_blake3: String::new(),
        nodes: Vec::new(),
        source_requirements: Vec::new(),
        blockers: facts.producer_blockers.clone(),
    }
}

fn produced_facts(record: &ProducedPackageRecord) -> ProducedPackageFacts {
    ProducedPackageFacts {
        selector_name: record.selector_name.clone(),
        system: record.system.clone(),
        expected_graph_digest_blake3: record.expected_graph_digest_blake3.clone(),
        expected_index_digest_blake3: record.expected_index_digest_blake3.clone(),
        graph: record.graph.clone(),
        package_index: record.package_index.clone(),
        producer_blockers: record.blockers.clone(),
    }
}

fn binding(role: &str, path: &str, bytes: &[u8]) -> Result<ArtifactBinding, RunError> {
    let byte_count = u64::try_from(bytes.len())
        .map_err(|_| RunError::Internal(format!("Mantlepkgs artifact byte count overflow: {path}")))?;
    Ok(ArtifactBinding {
        role: role.into(),
        path: path.into(),
        digest_blake3: blake3_hex(bytes),
        bytes: byte_count,
    })
}

fn pretty_json_bytes<T: Serialize>(value: &T, label: &str) -> Result<Vec<u8>, RunError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| RunError::Internal(format!("serializing {label}: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, RunError> {
    serde_json::to_vec(value)
        .map(|bytes| blake3_hex(&bytes))
        .map_err(|error| RunError::Internal(format!("serializing Mantlepkgs canonical value: {error}")))
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn read_json_bounded<T: DeserializeOwned>(path: &Path, max_bytes: u64) -> Result<T, RunError> {
    let bytes = read_bounded(path, max_bytes)?;
    deserialize_json(&bytes, &path.display().to_string())
}

fn deserialize_json<T: DeserializeOwned>(bytes: &[u8], label: &str) -> Result<T, RunError> {
    serde_json::from_slice(bytes).map_err(|error| RunError::Eval(format!("parsing {label}: {error}")))
}

fn deserialize_role<T: DeserializeOwned>(artifacts: &BTreeMap<String, Vec<u8>>, role: &str) -> Result<T, RunError> {
    let bytes = artifacts
        .get(role)
        .ok_or_else(|| RunError::Internal(format!("catalog has no artifact role: {role}")))?;
    deserialize_json(bytes, role)
}

fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, RunError> {
    let metadata = fs::metadata(path)
        .map_err(|error| RunError::Internal(format!("reading metadata {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err(RunError::Internal(format!("file is not regular or exceeds byte limit: {}", path.display())));
    }
    let bytes = fs::read(path).map_err(|error| RunError::Internal(format!("reading {}: {error}", path.display())))?;
    if bytes.len() as u64 != metadata.len() {
        return Err(RunError::Internal(format!("file changed while reading: {}", path.display())));
    }
    Ok(bytes)
}

fn confined_file(root: &Path, relative: &str) -> Result<PathBuf, RunError> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(RunError::Eval(format!("unsafe Mantlepkgs relative artifact path: {relative}")));
    }
    let root_canonical = fs::canonicalize(root)
        .map_err(|error| RunError::Internal(format!("resolving Mantlepkgs root {}: {error}", root.display())))?;
    let mut current = root.to_path_buf();
    for component in relative_path.components() {
        let Component::Normal(name) = component else {
            return Err(RunError::Eval(format!("unsafe Mantlepkgs artifact component: {relative}")));
        };
        current.push(name);
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            RunError::Internal(format!("reading Mantlepkgs artifact {}: {error}", current.display()))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(RunError::Eval(format!("Mantlepkgs artifact path contains a symlink: {}", current.display())));
        }
    }
    let canonical = fs::canonicalize(&current)
        .map_err(|error| RunError::Internal(format!("resolving Mantlepkgs artifact {}: {error}", current.display())))?;
    if !canonical.starts_with(&root_canonical) || !canonical.is_file() {
        return Err(RunError::Eval(format!("Mantlepkgs artifact escapes its generation: {relative}")));
    }
    Ok(canonical)
}

fn reject_symlink_root(root: &Path) -> Result<(), RunError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| RunError::Internal(format!("reading Mantlepkgs generation {}: {error}", root.display())))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(RunError::Eval(format!(
            "Mantlepkgs generation must be one non-symlink directory: {}",
            root.display()
        )));
    }
    Ok(())
}

fn write_new_bytes(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| RunError::Internal(format!("creating Mantlepkgs artifact {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| RunError::Internal(format!("writing Mantlepkgs artifact {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| RunError::Internal(format!("syncing Mantlepkgs artifact {}: {error}", path.display())))
}

fn sync_directory(path: &Path) -> Result<(), RunError> {
    fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| RunError::Internal(format!("syncing Mantlepkgs directory {}: {error}", path.display())))
}

fn run_nix(program: &Path, arguments: &[&str], max_stdout_bytes: usize) -> Result<Output, RunError> {
    let mut stdout = tempfile::tempfile()
        .map_err(|error| RunError::Internal(format!("creating Nix producer stdout capture: {error}")))?;
    let mut stderr = tempfile::tempfile()
        .map_err(|error| RunError::Internal(format!("creating Nix producer stderr capture: {error}")))?;
    let child_stdout = stdout
        .try_clone()
        .map_err(|error| RunError::Internal(format!("cloning Nix producer stdout capture: {error}")))?;
    let child_stderr = stderr
        .try_clone()
        .map_err(|error| RunError::Internal(format!("cloning Nix producer stderr capture: {error}")))?;
    let mut child = Command::new(program)
        .args(arguments)
        .stdout(Stdio::from(child_stdout))
        .stderr(Stdio::from(child_stderr))
        .spawn()
        .map_err(|error| RunError::Internal(format!("running explicit Nix producer {}: {error}", program.display())))?;
    let deadline = Instant::now() + Duration::from_secs(PRODUCER_COMMAND_TIMEOUT_SECS);
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| RunError::Internal(format!("waiting for explicit Nix producer: {error}")))?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RunError::Internal(format!(
                "explicit Nix producer exceeded {PRODUCER_COMMAND_TIMEOUT_SECS} seconds"
            )));
        }
        thread::sleep(Duration::from_millis(PRODUCER_COMMAND_POLL_MILLIS));
    };
    let stdout = read_capture(&mut stdout, max_stdout_bytes, "stdout")?;
    let stderr = read_capture(&mut stderr, PRODUCER_STDERR_MAX_BYTES, "stderr")?;
    Ok(Output { status, stdout, stderr })
}

fn read_capture(file: &mut fs::File, maximum: usize, label: &str) -> Result<Vec<u8>, RunError> {
    file.seek(SeekFrom::Start(0))
        .map_err(|error| RunError::Internal(format!("seeking Nix producer {label}: {error}")))?;
    let byte_limit = u64::try_from(maximum)
        .map_err(|_| RunError::Internal(format!("Nix producer {label} limit overflow")))?
        .checked_add(CAPTURE_OVERFLOW_SENTINEL_BYTES)
        .ok_or_else(|| RunError::Internal(format!("Nix producer {label} read limit overflow")))?;
    let mut bytes = Vec::new();
    file.take(byte_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading Nix producer {label}: {error}")))?;
    if bytes.len() > maximum {
        return Err(RunError::Internal(format!("explicit Nix producer {label} exceeded its byte limit")));
    }
    Ok(bytes)
}

fn require_success(
    output: &Output,
    code: &str,
    selector: &mantlepkgs_core::PackageSelector,
) -> Result<(), CatalogBlocker> {
    if output.status.success() {
        Ok(())
    } else {
        Err(producer_blocker(code, selector, &bounded_stderr(output)))
    }
}

fn bounded_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

fn producer_blocker(code: &str, selector: &mantlepkgs_core::PackageSelector, message: &str) -> CatalogBlocker {
    CatalogBlocker::new(code, &selector.name, message)
}

fn reject_equal_output_paths(first: &Path, second: &Path) -> Result<(), RunError> {
    if first == second {
        Err(RunError::Eval("Mantlepkgs plan and import receipt output paths must differ".into()))
    } else {
        Ok(())
    }
}

fn reject_distinct_build_outputs(request: &BuildRequest<'_>) -> Result<(), RunError> {
    let paths = [request.plan_out, request.import_receipt_out, request.receipt_out];
    if paths[0] == paths[1] || paths[0] == paths[2] || paths[1] == paths[2] {
        return Err(RunError::Eval("Mantlepkgs build output paths must all differ".into()));
    }
    Ok(())
}

fn emit_publication(catalog: &MantlepkgsCatalog, path: &Path, json: bool) -> Result<(), RunError> {
    if json {
        println!(
            "{}",
            serde_json::to_string(catalog)
                .map_err(|error| RunError::Internal(format!("serializing published Mantlepkgs catalog: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs generation published: identity={} packages={} path={}",
            catalog.catalog_identity_blake3,
            catalog.packages.len(),
            path.display()
        );
    }
    Ok(())
}

fn core_eval_error(failure: CoreFailure) -> RunError {
    RunError::Eval(format_core_failure(&failure))
}

fn core_internal_error(failure: CoreFailure) -> RunError {
    RunError::Internal(format_core_failure(&failure))
}

fn format_core_failure(failure: &CoreFailure) -> String {
    failure
        .diagnostics
        .iter()
        .map(|diagnostic| format!("{} {}: {}", diagnostic.code, diagnostic.path, diagnostic.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn blocker_error(blocker: CatalogBlocker) -> RunError {
    RunError::Internal(format!("{} {}: {}", blocker.code, blocker.subject, blocker.message))
}

fn import_diagnostic_error(diagnostic: crate::foreign_derivation_import::ImportDiagnostic) -> RunError {
    RunError::Eval(format!(
        "{} {}: {}",
        diagnostic.class,
        diagnostic.node_id.as_deref().unwrap_or("catalog"),
        diagnostic.message
    ))
}

#[cfg(test)]
mod tests {
    use crunch_build::foreign_profile_for_producer;
    use mantlepkgs_core::ConversionPolicy;
    use mantlepkgs_core::ManifestArtifact;
    use mantlepkgs_core::ManifestLimits;
    use mantlepkgs_core::NixpkgsSourceLock;
    use mantlepkgs_core::OutputLayout;
    use mantlepkgs_core::PackageSelector;
    use mantlepkgs_core::SourcePolicy;

    use super::*;

    const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SELECTOR_LIMIT: u32 = 8;
    const NODE_LIMIT: u32 = 1_024;
    const GRAPH_LIMIT: u64 = 16_777_216;
    const SOURCE_LIMIT: u32 = 1_024;
    const ARTIFACT_LIMIT: u64 = 33_554_432;
    const NO_NIX_GENERATION_ENV: &str = "MANTLEPKGS_TEST_GENERATION";
    const NO_NIX_CHILD_TEST: &str = "mantlepkgs_cmd::tests::no_nix_child_verifies_and_plans_catalog";
    const CAPTURE_CHILD_TEST: &str = "mantlepkgs_cmd::tests::producer_capture_child";
    const TINY_CAPTURE_LIMIT: usize = 1;

    #[test]
    fn fixture_batch_publishes_and_verifies_without_nix_in_path() {
        let temp = tempfile::tempdir().unwrap();
        let manifest_dir = temp.path().join("input");
        fs::create_dir(&manifest_dir).unwrap();
        let policy = fixture_policy();
        let profile = foreign_profile_for_producer("nixpkgs");
        let policy_bytes = serde_json::to_vec_pretty(&policy).unwrap();
        let profile_bytes = serde_json::to_vec_pretty(&profile).unwrap();
        fs::write(manifest_dir.join("translation.json"), &policy_bytes).unwrap();
        fs::write(manifest_dir.join("profile.json"), &profile_bytes).unwrap();
        let manifest = fixture_manifest(&policy_bytes, &profile_bytes);
        let manifest_path = manifest_dir.join("manifest.ncl");
        fs::write(&manifest_path, "{}\n").unwrap();
        let batch = fixture_batch(&manifest);

        publish_batch(&manifest_path, &manifest, &batch, temp.path(), false).unwrap();
        let plan = plan_catalog(&manifest, &batch.producer, &[observe_package(
            &manifest.selectors[0],
            &produced_facts(&batch.packages[0]),
            &manifest.conversion_policy.target_store_prefix,
        )])
        .unwrap();
        let facts = vec![produced_facts(&batch.packages[0])];
        let merged = merge_buildable_packages(&facts, &manifest.selectors).unwrap();
        let publication = prepare_publication(&plan, &merged, &policy_bytes, &profile_bytes, &batch).unwrap();
        let generation = temp.path().join("generations").join(publication.catalog.catalog_identity_blake3);
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", NO_NIX_CHILD_TEST, "--nocapture"])
            .env("PATH", temp.path().join("no-nix"))
            .env(NO_NIX_GENERATION_ENV, &generation)
            .output()
            .unwrap();
        assert!(child.status.success(), "child stderr: {}", String::from_utf8_lossy(&child.stderr));
        let catalog_before = fs::read(generation.join(CATALOG_JSON_PATH)).unwrap();
        let duplicate = publish_batch(&manifest_path, &manifest, &batch, temp.path(), false);
        assert!(matches!(duplicate, Err(RunError::Internal(_))));
        assert_eq!(fs::read(generation.join(CATALOG_JSON_PATH)).unwrap(), catalog_before);
        let stages = fs::read_dir(temp.path().join("generations"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(STAGE_PREFIX))
            .count();
        assert_eq!(stages, 0);
    }

    #[test]
    fn no_nix_child_verifies_and_plans_catalog() {
        let Some(generation) = std::env::var_os(NO_NIX_GENERATION_ENV) else {
            return;
        };
        assert!(Command::new("nix").status().is_err(), "Nix must be absent from the child PATH");
        let generation = PathBuf::from(generation);
        let verified = verify_generation(&generation).unwrap();
        let selected = compile_catalog_selection(&generation, "hi", "x86_64-linux").unwrap();
        assert_eq!(verified.catalog.packages.len(), 1);
        assert_eq!(selected.plan.selected_roots.len(), 1);
        assert!(selected.plan.forbidden_process_invocations.is_empty());
    }

    #[test]
    fn producer_capture_child() {
        println!("producer capture output");
    }

    #[test]
    fn producer_capture_rejects_output_above_the_limit() {
        let error = run_nix(
            &std::env::current_exe().unwrap(),
            &["--exact", CAPTURE_CHILD_TEST, "--nocapture"],
            TINY_CAPTURE_LIMIT,
        )
        .unwrap_err();
        assert!(error.message().contains("stdout exceeded its byte limit"));
    }

    #[test]
    fn partial_batch_writes_failure_report_and_no_success_generation() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("input");
        fs::create_dir(&input).unwrap();
        let policy_bytes = serde_json::to_vec_pretty(&fixture_policy()).unwrap();
        let profile_bytes = serde_json::to_vec_pretty(&foreign_profile_for_producer("nixpkgs")).unwrap();
        fs::write(input.join("translation.json"), &policy_bytes).unwrap();
        fs::write(input.join("profile.json"), &profile_bytes).unwrap();
        let manifest = fixture_manifest(&policy_bytes, &profile_bytes);
        let mut batch = fixture_batch(&manifest);
        batch.packages[0].graph = None;
        batch.packages[0]
            .blockers
            .push(CatalogBlocker::new("producer-failed", "hello", "fixture producer failure"));
        let result = publish_batch(&input.join("manifest.ncl"), &manifest, &batch, temp.path(), false);
        assert!(matches!(result, Err(RunError::Reported(FAILURE_EXIT_CODE))));
        let generation_root = temp.path().join("generations");
        assert!(generation_root.join(FAILURE_DIRECTORY).is_dir());
        let success_count = fs::read_dir(&generation_root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name() != FAILURE_DIRECTORY)
            .count();
        assert_eq!(success_count, 0);
    }

    fn fixture_manifest(policy_bytes: &[u8], profile_bytes: &[u8]) -> MantlepkgsManifest {
        MantlepkgsManifest {
            schema: mantlepkgs_core::MANIFEST_SCHEMA.into(),
            source: NixpkgsSourceLock {
                reference: format!("github:NixOS/nixpkgs/{REVISION}"),
                revision: REVISION.into(),
                lock_digest_blake3: DIGEST.into(),
            },
            systems: vec!["x86_64-linux".into()],
            selectors: vec![PackageSelector {
                name: "hello".into(),
                attribute: "hello".into(),
                system: "x86_64-linux".into(),
                aliases: vec!["hi".into()],
            }],
            conversion_policy: ConversionPolicy {
                mode: RECOMPUTE_CONVERSION_MODE.into(),
                target_store_prefix: "/mantle/store".into(),
                translation_policy: ManifestArtifact {
                    path: "translation.json".into(),
                    digest_blake3: blake3_hex(policy_bytes),
                },
                execution_profile: ManifestArtifact {
                    path: "profile.json".into(),
                    digest_blake3: blake3_hex(profile_bytes),
                },
            },
            source_policy: SourcePolicy {
                mode: mantlepkgs_core::SOURCE_BUNDLE_POLICY_MODE.into(),
                optional_transports: Vec::new(),
            },
            output: OutputLayout {
                generation_directory: "generations".into(),
            },
            limits: ManifestLimits {
                max_selectors: SELECTOR_LIMIT,
                max_graph_nodes: NODE_LIMIT,
                max_graph_bytes: GRAPH_LIMIT,
                max_source_requirements: SOURCE_LIMIT,
                max_artifact_bytes: ARTIFACT_LIMIT,
            },
        }
    }

    fn fixture_batch(manifest: &MantlepkgsManifest) -> ProducerBatch {
        let (mut graph, package_index) = crate::foreign_derivation_import::nix_like_hello_fixture();
        graph.producer.identity = manifest.source.reference.clone();
        graph.producer.revision = manifest.source.revision.clone();
        ProducerBatch {
            schema: PRODUCER_BATCH_SCHEMA.into(),
            producer: ProducerObservation {
                command_class: PRODUCER_COMMAND_CLASS.into(),
                implementation_id: "fixture-nix".into(),
                version: "nix fixture".into(),
                executable_digest_blake3: DIGEST.into(),
                source_lock: manifest.source.clone(),
            },
            packages: vec![ProducedPackageRecord {
                selector_name: "hello".into(),
                system: "x86_64-linux".into(),
                expected_graph_digest_blake3: canonical_digest(&graph).unwrap(),
                expected_index_digest_blake3: canonical_digest(&package_index).unwrap(),
                graph: Some(graph),
                package_index: Some(package_index),
                blockers: Vec::new(),
            }],
        }
    }

    fn fixture_policy() -> TranslationPolicy {
        TranslationPolicy {
            source_prefixes: vec!["/nix/store".into()],
            target_prefix: "/mantle/store".into(),
            rewrite_builder: true,
            rewrite_args: true,
            rewrite_env: true,
            rewrite_sources: true,
            rewrite_declared_references: true,
            allow_embedded_source_payload_rewrite: false,
            builtin_mappings: BTreeMap::from([
                ("fixed-output-fetch".into(), "fixed-output-fetch".into()),
                ("nix.derivation".into(), "nix.derivation".into()),
            ]),
            output_path_recompute_mode: RECOMPUTE_CONVERSION_MODE.into(),
            trusted_cache_scopes: std::collections::BTreeSet::new(),
            allowed_sandbox_capabilities: std::collections::BTreeSet::new(),
        }
    }
}
