// machine-artifact-public: mantlepkgs.catalog-domain-reports
// machine-artifact-public: mantlepkgs.package-impact-report
// r[impl mantlepkgs.producer_boundary]
// r[impl mantlepkgs.catalog_generation]
// r[impl mantlepkgs.recomputed_rebuild]
// r[verify mantlepkgs.producer_boundary]
// r[verify mantlepkgs.recomputed_rebuild]
// r[verify mantlepkgs.validation]
// r[impl mantlepkgs_domains.separate_validation_roots]
// r[impl mantlepkgs_domains.reference_corpus]
// r[verify mantlepkgs_domains.functional_core]
// r[impl mantlepkgs_impact.external_ci_boundary]
// r[verify mantlepkgs_impact.external_ci_boundary]
// machine-artifact-public: mantlepkgs.update-plan
// r[impl mantlepkgs_updates.source_observations]
// r[impl mantlepkgs_updates.preimage_bound_mutation]
// r[impl mantlepkgs_updates.advisory_evidence]
// r[impl mantlepkgs_updates.validation_evidence]
// r[verify mantlepkgs_updates.source_observations]
// r[verify mantlepkgs_updates.preimage_bound_mutation]
// r[verify mantlepkgs_updates.advisory_evidence]
// r[verify mantlepkgs_updates.validation_evidence]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
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
use mantlepkgs_core::ADVISORY_RESPONSE_OSV_SCHEMA;
use mantlepkgs_core::ADVISORY_RESPONSE_REPOLOGY_SCHEMA;
use mantlepkgs_core::AdvisoryFinding;
use mantlepkgs_core::AdvisoryObservation;
use mantlepkgs_core::AdvisoryService;
use mantlepkgs_core::AppliedOutput;
use mantlepkgs_core::ArtifactBinding;
use mantlepkgs_core::ArtifactObservation;
use mantlepkgs_core::CATALOG_JSON_PATH;
use mantlepkgs_core::CATALOG_NICKEL_PATH;
use mantlepkgs_core::CORPUS_ROLE_CATALOG;
use mantlepkgs_core::CatalogBlocker;
use mantlepkgs_core::CatalogPlan;
use mantlepkgs_core::CollectionFacts;
use mantlepkgs_core::CoreFailure;
use mantlepkgs_core::CorpusArtifactObservation;
use mantlepkgs_core::DOMAIN_CLASS_CORE;
use mantlepkgs_core::DomainCatalog;
use mantlepkgs_core::DomainCatalogManifest;
use mantlepkgs_core::DomainShardLimits;
use mantlepkgs_core::EXECUTION_PROFILE_PATH;
use mantlepkgs_core::ExternalCorpusEvidence;
use mantlepkgs_core::ImpactComparisonInput;
use mantlepkgs_core::ImpactComparisonPolicy;
use mantlepkgs_core::ImpactSnapshot;
use mantlepkgs_core::MantlepkgsCatalog;
use mantlepkgs_core::MantlepkgsManifest;
use mantlepkgs_core::MutationDocument;
use mantlepkgs_core::ObservationStatus;
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
use mantlepkgs_core::SourceCandidate;
use mantlepkgs_core::SourceObservation;
use mantlepkgs_core::SourceRequirementInventory;
use mantlepkgs_core::TRANSLATION_POLICY_PATH;
use mantlepkgs_core::UPDATE_ADVISORY_OBSERVATION_SCHEMA;
use mantlepkgs_core::UPDATE_MUTATION_DOCUMENT_SCHEMA;
use mantlepkgs_core::UPDATE_SOURCE_OBSERVATION_SCHEMA;
use mantlepkgs_core::UpdateExecutionDisposition;
use mantlepkgs_core::UpdatePlan;
use mantlepkgs_core::UpdatePlanInput;
use mantlepkgs_core::UpdatePolicy;
use mantlepkgs_core::UpdateValidationEvidence;
use mantlepkgs_core::V1DomainShardInput;
use mantlepkgs_core::VALIDATION_OBSERVATION_SCHEMA;
use mantlepkgs_core::ValidationObservation;
use mantlepkgs_core::ValidationRootPlan;
use mantlepkgs_core::adapt_v1_catalog_to_domain_shard;
use mantlepkgs_core::build_package_impact_report;
use mantlepkgs_core::build_producer_receipt;
use mantlepkgs_core::build_update_plan;
use mantlepkgs_core::canonical_mutation_document_bytes;
use mantlepkgs_core::compose_domain_catalog;
use mantlepkgs_core::finalize_catalog;
use mantlepkgs_core::lookup_catalog_package;
use mantlepkgs_core::manifest_digest_blake3;
use mantlepkgs_core::normalize_manifest;
use mantlepkgs_core::plan_catalog;
use mantlepkgs_core::plan_validation_root;
use mantlepkgs_core::producer_receipt_digest_blake3;
use mantlepkgs_core::record_update_execution;
use mantlepkgs_core::record_validation_observation;
use mantlepkgs_core::render_catalog_nickel;
use mantlepkgs_core::seal_advisory_observation;
use mantlepkgs_core::seal_domain_manifest;
use mantlepkgs_core::seal_external_corpus_evidence;
use mantlepkgs_core::seal_source_observation;
use mantlepkgs_core::seal_update_policy;
use mantlepkgs_core::source_requirement_inventory;
use mantlepkgs_core::update_plan_identity_blake3;
use mantlepkgs_core::validate_catalog_artifacts;
use mantlepkgs_core::validate_catalog_identity;
use mantlepkgs_core::validate_external_corpus_evidence;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::RunError;
use crate::foreign_derivation_import::FIXED_OUTPUT_SEED_KIND;
use crate::foreign_derivation_import::ForeignDerivationGraph;
use crate::foreign_derivation_import::NixDerivationJsonExport;
use crate::foreign_derivation_import::NixProducerConfig;
use crate::foreign_derivation_import::PackageIndex;
use crate::foreign_derivation_import::SourcePayload;
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
use crate::foreign_realization_receipt::ForeignRealizationReceipt;
use crate::linux_rename::rename_path_no_replace;
use crate::mantlepkgs_adapter::MergedForeignArtifacts;
use crate::mantlepkgs_adapter::ProducedPackageFacts;
use crate::mantlepkgs_adapter::merge_buildable_packages;
use crate::mantlepkgs_adapter::observe_package;
use crate::mantlepkgs_version_cmd::MantlepkgsVersionAction;
use crate::mantlepkgs_version_cmd::cmd_mantlepkgs_version;
use crate::source_bundle::ForeignSourcePathBinding;
use crate::source_bundle::digest_bound_foreign_source_path;
use crate::source_bundle::plan_bound_foreign_source_bundle;
use crate::source_bundle::write_json_atomically;

const PRODUCER_BATCH_SCHEMA: &str = "mantlepkgs-producer-batch-v1";
const GENERATION_FAILURE_SCHEMA: &str = "mantlepkgs-generation-failure-v1";
const FAILURE_DIRECTORY: &str = "failures";
const PRODUCER_SEED_ROOT_DIRECTORY: &str = ".mantlepkgs-producer-seed-roots";
const PRODUCER_SEED_RETENTION_SCHEMA: &str = "mantlepkgs-producer-seed-retention-v1";
const STAGE_PREFIX: &str = ".stage-";
const CATALOG_MAX_BYTES: u64 = 16_777_216;
const NIX_EXECUTABLE_MAX_BYTES: u64 = 268_435_456;
const NIX_ROOT_STDOUT_MAX_BYTES: usize = 4_096;
const SEED_PAYLOAD_HASH_HEX_CHARS: usize = 32;
const SEED_PATHS_STDOUT_MAX_BYTES: usize = 1_048_576;
const PRODUCER_STDERR_MAX_BYTES: usize = 16_384;
const PRODUCER_COMMAND_TIMEOUT_SECS: u64 = 900;
const PRODUCER_COMMAND_POLL_MILLIS: u64 = 100;
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const CAPTURE_OVERFLOW_SENTINEL_BYTES: u64 = 1;
const KIBIBYTE_BYTES: u64 = 1_024;
const GIBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const MANTLEPKGS_SOURCE_BUNDLE_GIBIBYTES_MAX: u64 = 4;
const MANTLEPKGS_SOURCE_BUNDLE_BYTES_MAX: u64 = MANTLEPKGS_SOURCE_BUNDLE_GIBIBYTES_MAX * GIBIBYTE_BYTES;
const DOMAIN_ARTIFACT_GIBIBYTES_MAX: u64 = 1;
const DOMAIN_ARTIFACT_BYTES_MAX: u64 = DOMAIN_ARTIFACT_GIBIBYTES_MAX * GIBIBYTE_BYTES;
const IMPACT_INPUT_MEBIBYTES_MAX: u64 = 64;
const MEBIBYTE_BYTES: u64 = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const IMPACT_INPUT_BYTES_MAX: u64 = IMPACT_INPUT_MEBIBYTES_MAX * MEBIBYTE_BYTES;
const IMPACT_ACTION_RESULT_REPORTS_MAX: u32 = 65_536;
const IMPACT_ACTION_RESULT_REPORT_BYTES_MAX: u64 = IMPACT_INPUT_BYTES_MAX;
const UPDATE_INPUT_BYTES_MAX: u64 = 64 * MEBIBYTE_BYTES;
const UPDATE_DOCUMENT_BYTES_MAX: u64 = 16 * MEBIBYTE_BYTES;
const UPDATE_EXECUTION_RECEIPT_FILE: &str = "update-execution-receipt.json";
const UPDATE_STAGE_PREFIX: &str = ".mantle-update-stage-";
const UPDATE_HTTP_USER_AGENT: &str = "mantlepkgs-update-observer/1";
const UPDATE_HTTP_SUCCESS_STATUS_MIN: u16 = 200;
const UPDATE_HTTP_SUCCESS_STATUS_MAX: u16 = 299;
const UPDATE_HTTP_REDIRECT_STATUS_MIN: u16 = 300;
const UPDATE_HTTP_REDIRECT_STATUS_MAX: u16 = 399;
const UPDATE_DUPLICATE_WINDOW_SIZE: usize = 2;
const UPDATE_DENIAL_GENERIC_REASON: &str = "update-execution-denied";
const DOMAIN_SHARD_PACKAGE_LIMIT: u32 = 65_536;
const DOMAIN_SHARD_ALIAS_LIMIT: u32 = 65_536;
const DOMAIN_SHARD_ARTIFACT_LIMIT: u32 = 65_536;
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

    /// Adapt one verified v1 generation into one explicit domain shard
    DomainAdapt {
        #[arg(long)]
        generation: PathBuf,

        #[arg(long)]
        name: String,

        #[arg(long, default_value = DOMAIN_CLASS_CORE)]
        class: String,

        #[arg(long = "owner-label")]
        owner_label: String,

        #[arg(long = "source-repository")]
        source_repository: String,

        #[arg(long)]
        out: PathBuf,
    },

    /// Seal typed domain identities and compose one deterministic public catalog
    DomainCompose {
        #[arg(long)]
        manifest: PathBuf,

        #[arg(long = "sealed-manifest-out")]
        sealed_manifest_out: PathBuf,

        #[arg(long)]
        out: PathBuf,
    },

    /// Realize one separate validation root through the ordinary foreign build boundary
    ValidationBuild {
        #[arg(long = "domain-catalog")]
        domain_catalog: PathBuf,

        #[arg(long = "validation-root")]
        validation_root: String,

        #[arg(long)]
        generation: PathBuf,

        #[arg(long = "source-bundle")]
        source_bundle: PathBuf,

        #[arg(long = "source-bundle-blake3")]
        source_bundle_blake3: String,

        #[arg(long = "validation-plan-out")]
        validation_plan_out: PathBuf,

        #[arg(long = "plan-out")]
        plan_out: PathBuf,

        #[arg(long = "import-receipt-out")]
        import_receipt_out: PathBuf,

        #[arg(long = "realization-receipt-out")]
        realization_receipt_out: PathBuf,

        #[arg(long = "validation-receipt-out")]
        validation_receipt_out: PathBuf,

        #[arg(short = 'j', long = "jobs")]
        jobs: Option<u32>,

        #[arg(long = "signing-key")]
        signing_key: Option<PathBuf>,

        #[arg(long)]
        offline: bool,
    },

    /// Seal and verify one pinned external-corpus evidence record
    CorpusVerify {
        #[arg(long)]
        evidence: PathBuf,

        #[arg(long = "artifact-root")]
        artifact_root: PathBuf,

        #[arg(long = "sealed-evidence-out")]
        sealed_evidence_out: PathBuf,
    },

    /// Compare two contracted snapshots and write one deterministic impact report
    Impact {
        #[arg(long)]
        policy: PathBuf,

        #[arg(long)]
        base: PathBuf,

        #[arg(long)]
        head: PathBuf,

        /// Admitted current-policy runtime report for each action-result observation
        #[arg(long = "action-result-report")]
        action_result_reports: Vec<PathBuf>,

        #[arg(long)]
        out: PathBuf,
    },

    /// Resolve historical package versions before ordinary Mantlepkgs production
    Version {
        #[command(subcommand)]
        action: MantlepkgsVersionAction,
    },

    /// Seal one typed update policy with its canonical BLAKE3 identity
    UpdatePolicySeal {
        #[arg(long)]
        policy: PathBuf,

        #[arg(long)]
        out: PathBuf,
    },

    /// Record one bounded source response or explicit source failure
    UpdateSourceObserve {
        #[arg(long)]
        policy: PathBuf,

        #[arg(long, conflicts_with = "url")]
        response: Option<PathBuf>,

        #[arg(long, conflicts_with = "response")]
        url: Option<String>,

        #[arg(long, default_value = "success")]
        status: String,

        #[arg(long = "reason")]
        reasons: Vec<String>,

        #[arg(long)]
        out: PathBuf,
    },

    /// Record one bounded OSV or Repology response or explicit failure
    UpdateAdvisoryObserve {
        #[arg(long)]
        policy: PathBuf,

        #[arg(long)]
        service: String,

        #[arg(long)]
        version: String,

        #[arg(long, conflicts_with = "url")]
        response: Option<PathBuf>,

        #[arg(long, conflicts_with = "response")]
        url: Option<String>,

        #[arg(long, default_value = "success")]
        status: String,

        #[arg(long = "reason")]
        reasons: Vec<String>,

        #[arg(long)]
        out: PathBuf,
    },

    /// Replay saved observations and write one deterministic dry-run update plan
    UpdatePlan {
        #[arg(long)]
        policy: PathBuf,

        #[arg(long = "source-observation")]
        source_observation: PathBuf,

        #[arg(long = "advisory-observation")]
        advisory_observations: Vec<PathBuf>,

        #[arg(long = "validation-evidence")]
        validation_evidence: PathBuf,

        #[arg(long = "source-root")]
        source_root: PathBuf,

        #[arg(long)]
        out: PathBuf,
    },

    /// Publish one immutable updated output tree after complete preimage validation
    UpdateExecute {
        #[arg(long)]
        plan: PathBuf,

        #[arg(long = "source-root")]
        source_root: PathBuf,

        #[arg(long = "output-root")]
        output_root: PathBuf,

        #[arg(long = "denial-receipt-out")]
        denial_receipt_out: PathBuf,
    },
}

pub(crate) struct MantlepkgsContext<'a> {
    pub(crate) output_dir: &'a Path,
    pub(crate) state_dir: &'a Path,
    pub(crate) backend: crunch_store::StoreBackend,
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct FixedOutputSeedCandidate {
    derivation_path: String,
    output_name: String,
    output_path: String,
    payload_id: String,
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
        MantlepkgsAction::DomainAdapt {
            generation,
            name,
            class,
            owner_label,
            source_repository,
            out,
        } => run_domain_adapt(&generation, &name, &class, &owner_label, &source_repository, &out, context.json),
        MantlepkgsAction::DomainCompose {
            manifest,
            sealed_manifest_out,
            out,
        } => run_domain_compose(&manifest, &sealed_manifest_out, &out, context.json),
        MantlepkgsAction::ValidationBuild {
            domain_catalog,
            validation_root,
            generation,
            source_bundle,
            source_bundle_blake3,
            validation_plan_out,
            plan_out,
            import_receipt_out,
            realization_receipt_out,
            validation_receipt_out,
            jobs,
            signing_key,
            offline,
        } => run_validation_build(
            ValidationBuildRequest {
                domain_catalog: &domain_catalog,
                validation_root: &validation_root,
                generation: &generation,
                source_bundle: &source_bundle,
                source_bundle_blake3: &source_bundle_blake3,
                validation_plan_out: &validation_plan_out,
                plan_out: &plan_out,
                import_receipt_out: &import_receipt_out,
                realization_receipt_out: &realization_receipt_out,
                validation_receipt_out: &validation_receipt_out,
                jobs,
                signing_key: signing_key.as_deref(),
                offline,
            },
            &context,
        ),
        MantlepkgsAction::CorpusVerify {
            evidence,
            artifact_root,
            sealed_evidence_out,
        } => run_corpus_verify(&evidence, &artifact_root, &sealed_evidence_out, context.json),
        MantlepkgsAction::Impact {
            policy,
            base,
            head,
            action_result_reports,
            out,
        } => run_impact(ImpactRequest {
            policy_path: &policy,
            base_path: &base,
            head_path: &head,
            action_result_report_paths: &action_result_reports,
            output: &out,
            is_json: context.json,
        }),
        MantlepkgsAction::Version { action } => cmd_mantlepkgs_version(action, context.json),
        MantlepkgsAction::UpdatePolicySeal { policy, out } => run_update_policy_seal(&policy, &out, context.json),
        MantlepkgsAction::UpdateSourceObserve {
            policy,
            response,
            url,
            status,
            reasons,
            out,
        } => run_update_source_observe(UpdateObserveRequest {
            policy_path: &policy,
            response_path: response.as_deref(),
            url: url.as_deref(),
            requested_status: &status,
            reasons: &reasons,
            output: &out,
            is_json: context.json,
        }),
        MantlepkgsAction::UpdateAdvisoryObserve {
            policy,
            service,
            version,
            response,
            url,
            status,
            reasons,
            out,
        } => run_update_advisory_observe(UpdateAdvisoryObserveRequest {
            policy_path: &policy,
            service: &service,
            version: &version,
            response_path: response.as_deref(),
            url: url.as_deref(),
            requested_status: &status,
            reasons: &reasons,
            output: &out,
            is_json: context.json,
        }),
        MantlepkgsAction::UpdatePlan {
            policy,
            source_observation,
            advisory_observations,
            validation_evidence,
            source_root,
            out,
        } => run_update_plan(UpdatePlanRequest {
            policy_path: &policy,
            source_observation_path: &source_observation,
            advisory_observation_paths: &advisory_observations,
            validation_evidence_path: &validation_evidence,
            source_root: &source_root,
            output: &out,
            is_json: context.json,
        }),
        MantlepkgsAction::UpdateExecute {
            plan,
            source_root,
            output_root,
            denial_receipt_out,
        } => run_update_execute(UpdateExecuteRequest {
            plan_path: &plan,
            source_root: &source_root,
            output_root: &output_root,
            denial_receipt_output: &denial_receipt_out,
            is_json: context.json,
        }),
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

struct ImpactRequest<'a> {
    policy_path: &'a Path,
    base_path: &'a Path,
    head_path: &'a Path,
    action_result_report_paths: &'a [PathBuf],
    output: &'a Path,
    is_json: bool,
}

struct UpdateObserveRequest<'a> {
    policy_path: &'a Path,
    response_path: Option<&'a Path>,
    url: Option<&'a str>,
    requested_status: &'a str,
    reasons: &'a [String],
    output: &'a Path,
    is_json: bool,
}

struct UpdateAdvisoryObserveRequest<'a> {
    policy_path: &'a Path,
    service: &'a str,
    version: &'a str,
    response_path: Option<&'a Path>,
    url: Option<&'a str>,
    requested_status: &'a str,
    reasons: &'a [String],
    output: &'a Path,
    is_json: bool,
}

struct UpdatePlanRequest<'a> {
    policy_path: &'a Path,
    source_observation_path: &'a Path,
    advisory_observation_paths: &'a [PathBuf],
    validation_evidence_path: &'a Path,
    source_root: &'a Path,
    output: &'a Path,
    is_json: bool,
}

struct UpdateExecuteRequest<'a> {
    plan_path: &'a Path,
    source_root: &'a Path,
    output_root: &'a Path,
    denial_receipt_output: &'a Path,
    is_json: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceResponsePayload {
    schema: String,
    candidates: Vec<SourceCandidate>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdvisoryResponsePayload {
    schema: String,
    package_coordinate: String,
    version: String,
    findings: Vec<AdvisoryFinding>,
}

struct ResponseAcquisition {
    status: ObservationStatus,
    bytes: Option<Vec<u8>>,
    response_identity_blake3: Option<String>,
    reason_codes: Vec<String>,
    collection: CollectionFacts,
}

struct ValidationBuildRequest<'a> {
    domain_catalog: &'a Path,
    validation_root: &'a str,
    generation: &'a Path,
    source_bundle: &'a Path,
    source_bundle_blake3: &'a str,
    validation_plan_out: &'a Path,
    plan_out: &'a Path,
    import_receipt_out: &'a Path,
    realization_receipt_out: &'a Path,
    validation_receipt_out: &'a Path,
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

fn run_domain_adapt(
    generation: &Path,
    name: &str,
    class: &str,
    owner_label: &str,
    source_repository: &str,
    output: &Path,
    json: bool,
) -> Result<(), RunError> {
    if output.starts_with(generation) {
        return Err(RunError::Eval("domain shard output must remain outside the immutable source generation".into()));
    }
    let verified = verify_generation(generation)?;
    let shard = adapt_v1_catalog_to_domain_shard(V1DomainShardInput {
        catalog: &verified.catalog,
        name,
        class,
        owner_label,
        source_repository,
        limits: DomainShardLimits {
            max_packages: DOMAIN_SHARD_PACKAGE_LIMIT,
            max_aliases: DOMAIN_SHARD_ALIAS_LIMIT,
            max_artifacts: DOMAIN_SHARD_ARTIFACT_LIMIT,
        },
    })
    .map_err(core_eval_error)?;
    write_json_atomically(output, &shard, "Mantlepkgs domain shard")?;
    if json {
        println!(
            "{}",
            serde_json::to_string(&shard)
                .map_err(|error| RunError::Internal(format!("serializing Mantlepkgs domain shard: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs domain shard adapted: identity={} packages={} path={}",
            shard.shard_identity_blake3,
            shard.packages.len(),
            output.display()
        );
    }
    Ok(())
}

fn run_domain_compose(
    manifest_path: &Path,
    sealed_manifest_out: &Path,
    output: &Path,
    json: bool,
) -> Result<(), RunError> {
    reject_output_path_collisions(&[sealed_manifest_out, output])?;
    let manifest = evaluate_domain_manifest(manifest_path)?;
    let sealed = seal_domain_manifest(&manifest).map_err(core_eval_error)?;
    let catalog = compose_domain_catalog(&sealed).map_err(core_eval_error)?;
    write_json_atomically(sealed_manifest_out, &sealed, "sealed Mantlepkgs domain manifest")?;
    write_json_atomically(output, &catalog, "Mantlepkgs domain catalog")?;
    if json {
        println!(
            "{}",
            serde_json::to_string(&catalog)
                .map_err(|error| RunError::Internal(format!("serializing Mantlepkgs domain catalog: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs domain catalog composed: identity={} packages={} variants={} validations={} path={}",
            catalog.catalog_identity_blake3,
            catalog.packages.len(),
            catalog.variants.len(),
            catalog.validation_roots.len(),
            output.display()
        );
    }
    Ok(())
}

fn run_validation_build(request: ValidationBuildRequest<'_>, context: &MantlepkgsContext<'_>) -> Result<(), RunError> {
    reject_output_path_collisions(&[
        request.validation_plan_out,
        request.plan_out,
        request.import_receipt_out,
        request.realization_receipt_out,
        request.validation_receipt_out,
    ])?;
    let domain_catalog = read_json_bounded::<DomainCatalog>(request.domain_catalog, DOMAIN_ARTIFACT_BYTES_MAX)?;
    let validation_plan = plan_validation_root(&domain_catalog, request.validation_root).map_err(core_eval_error)?;
    let verified_generation = verify_generation(request.generation)?;
    require_validation_generation_binding(&domain_catalog, &validation_plan, &verified_generation.catalog)?;
    write_json_atomically(request.validation_plan_out, &validation_plan, "Mantlepkgs validation-root plan")?;
    let build_started = Instant::now();
    let build_result = run_build(
        BuildRequest {
            generation: request.generation,
            package: &validation_plan.validation_selector,
            system: validation_system(&domain_catalog, request.validation_root)?,
            source_bundle: request.source_bundle,
            source_bundle_blake3: request.source_bundle_blake3,
            plan_out: request.plan_out,
            import_receipt_out: request.import_receipt_out,
            receipt_out: request.realization_receipt_out,
            jobs: request.jobs,
            signing_key: request.signing_key,
            offline: request.offline,
        },
        context,
    );
    if !request.realization_receipt_out.is_file() {
        return build_result;
    }
    let realization =
        read_json_bounded::<ForeignRealizationReceipt>(request.realization_receipt_out, DOMAIN_ARTIFACT_BYTES_MAX)?;
    let elapsed_milliseconds = u64::try_from(build_started.elapsed().as_millis())
        .map_err(|_| RunError::Internal("validation elapsed milliseconds overflow".into()))?;
    let observation = validation_observation(&validation_plan, &realization, elapsed_milliseconds)?;
    let receipt = record_validation_observation(&validation_plan, &observation).map_err(core_eval_error)?;
    write_json_atomically(request.validation_receipt_out, &receipt, "Mantlepkgs validation-root receipt")?;
    if context.json {
        println!(
            "{}",
            serde_json::to_string(&receipt)
                .map_err(|error| RunError::Internal(format!("serializing validation-root receipt: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs validation recorded: root={} outcome={} accepted={} receipt={}",
            receipt.validation_root_identity_blake3, receipt.outcome, receipt.accepted, receipt.receipt_identity_blake3
        );
    }
    match build_result {
        Ok(()) => {}
        Err(error) => return Err(error),
    }
    if !receipt.accepted {
        return Err(RunError::Reported(FAILURE_EXIT_CODE));
    }
    Ok(())
}

fn run_corpus_verify(
    evidence_path: &Path,
    artifact_root: &Path,
    sealed_evidence_out: &Path,
    json: bool,
) -> Result<(), RunError> {
    let evidence = read_json_bounded::<ExternalCorpusEvidence>(evidence_path, DOMAIN_ARTIFACT_BYTES_MAX)?;
    let sealed = seal_external_corpus_evidence(&evidence).map_err(core_eval_error)?;
    let mut observations = Vec::with_capacity(sealed.artifacts.len());
    for artifact in &sealed.artifacts {
        let path = confined_file(artifact_root, &artifact.path)?;
        let bytes = read_bounded(&path, DOMAIN_ARTIFACT_BYTES_MAX)?;
        let digest_blake3 = blake3_hex(&bytes);
        let byte_count =
            u64::try_from(bytes.len()).map_err(|_| RunError::Internal("corpus artifact byte count overflow".into()))?;
        let catalog_packages = if digest_blake3 == artifact.digest_blake3 && byte_count == artifact.bytes {
            observed_corpus_catalog_packages(&artifact.role, &bytes)?
        } else {
            Vec::new()
        };
        observations.push(CorpusArtifactObservation {
            path: artifact.path.clone(),
            digest_blake3,
            bytes: byte_count,
            catalog_packages,
        });
    }
    validate_external_corpus_evidence(&sealed, &observations).map_err(core_eval_error)?;
    write_json_atomically(sealed_evidence_out, &sealed, "sealed Mantlepkgs external-corpus evidence")?;
    if json {
        println!(
            "{}",
            serde_json::to_string(&sealed)
                .map_err(|error| RunError::Internal(format!("serializing corpus evidence: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs corpus evidence verified: identity={} packages={} artifacts={}",
            sealed.evidence_identity_blake3,
            sealed.selected_packages.len(),
            sealed.artifacts.len()
        );
    }
    Ok(())
}

fn run_impact(request: ImpactRequest<'_>) -> Result<(), RunError> {
    reject_impact_output_collision(&request)?;
    let policy = read_json_bounded::<ImpactComparisonPolicy>(request.policy_path, IMPACT_INPUT_BYTES_MAX)?;
    let base = read_json_bounded::<ImpactSnapshot>(request.base_path, IMPACT_INPUT_BYTES_MAX)?;
    let head = read_json_bounded::<ImpactSnapshot>(request.head_path, IMPACT_INPUT_BYTES_MAX)?;
    let action_results = load_impact_action_result_reports(request.action_result_report_paths)?;
    validate_impact_action_result_bindings(&base, &head, &action_results)?;
    let report = build_package_impact_report(ImpactComparisonInput {
        policy: &policy,
        base: &base,
        head: &head,
    })
    .map_err(core_eval_error)?;
    write_json_atomically(request.output, &report, "Mantlepkgs package-impact report")?;
    if request.is_json {
        println!(
            "{}",
            serde_json::to_string(&report)
                .map_err(|error| RunError::Internal(format!("serializing Mantlepkgs impact report: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs impact report written: identity={} packages={} path={}",
            report.report_identity_blake3,
            report.package_impacts.len(),
            request.output.display()
        );
    }
    Ok(())
}

fn reject_impact_output_collision(request: &ImpactRequest<'_>) -> Result<(), RunError> {
    let is_primary_collision = request.output == request.policy_path
        || request.output == request.base_path
        || request.output == request.head_path;
    let is_action_result_collision = request.action_result_report_paths.iter().any(|path| path == request.output);
    if is_primary_collision || is_action_result_collision {
        return Err(RunError::Eval("impact output must differ from every input artifact".into()));
    }
    Ok(())
}

fn load_impact_action_result_reports(
    paths: &[PathBuf],
) -> Result<BTreeMap<String, crunch_build::ActionResultRuntimeReport>, RunError> {
    let report_count = u32::try_from(paths.len())
        .map_err(|_| RunError::Eval("impact action-result report count exceeds u32".into()))?;
    if report_count > IMPACT_ACTION_RESULT_REPORTS_MAX {
        return Err(RunError::Eval("impact action-result report count exceeds policy".into()));
    }
    let mut total_bytes = 0u64;
    let mut reports = BTreeMap::new();
    for path in paths {
        let bytes = read_bounded(path, IMPACT_ACTION_RESULT_REPORT_BYTES_MAX)?;
        let byte_count = u64::try_from(bytes.len())
            .map_err(|_| RunError::Eval("impact action-result byte count exceeds u64".into()))?;
        total_bytes = total_bytes
            .checked_add(byte_count)
            .ok_or_else(|| RunError::Eval("impact action-result byte count overflow".into()))?;
        if total_bytes > IMPACT_ACTION_RESULT_REPORT_BYTES_MAX {
            return Err(RunError::Eval("impact action-result reports exceed aggregate byte limit".into()));
        }
        let report = deserialize_json::<crunch_build::ActionResultRuntimeReport>(&bytes, &path.display().to_string())?;
        let identity = canonical_digest(&report)?;
        if reports.len() >= paths.len() {
            return Err(RunError::Internal("impact action-result map exceeded input count".into()));
        }
        if reports.insert(identity.clone(), report).is_some() {
            return Err(RunError::Eval(format!("duplicate impact action-result report: {identity}")));
        }
    }
    Ok(reports)
}

fn validate_impact_action_result_bindings(
    base: &ImpactSnapshot,
    head: &ImpactSnapshot,
    reports: &BTreeMap<String, crunch_build::ActionResultRuntimeReport>,
) -> Result<(), RunError> {
    for observation in base.observations.iter().chain(&head.observations) {
        let mantlepkgs_core::BuildObservationAuthority::AdmittedActionResult {
            runtime_report_identity_blake3,
            action_ref,
            selected_result_ref,
            ..
        } = &observation.authority
        else {
            continue;
        };
        let report = reports.get(runtime_report_identity_blake3).ok_or_else(|| {
            RunError::Eval(format!("impact-action-result-report-missing {}", observation.key.public_selector))
        })?;
        validate_admitted_runtime_report(report, action_ref, selected_result_ref)?;
    }
    Ok(())
}

fn validate_admitted_runtime_report(
    report: &crunch_build::ActionResultRuntimeReport,
    action_ref: &str,
    selected_result_ref: impl AsRef<str>,
) -> Result<(), RunError> {
    let selected_result_ref = selected_result_ref.as_ref();
    let is_header_valid = report.schema == crunch_build::action_result::ACTION_RESULT_RUNTIME_REPORT_SCHEMA
        && report.phase == crunch_build::action_result::ACTION_RESULT_PHASE_DISCOVERY
        && report.disposition == crunch_build::action_result::ACTION_RESULT_DISPOSITION_REUSED;
    if !is_header_valid {
        return Err(action_result_admission_error(action_ref, selected_result_ref));
    }
    let is_selection_valid = report.action_ref == action_ref
        && report.selected_result_ref.as_deref() == Some(selected_result_ref)
        && report.conflict_class.is_none();
    if !is_selection_valid {
        return Err(action_result_admission_error(action_ref, selected_result_ref));
    }
    let is_candidate_admitted = report
        .candidate_decisions
        .iter()
        .any(|candidate| candidate.result_ref == selected_result_ref && candidate.admitted);
    if !is_candidate_admitted {
        return Err(action_result_admission_error(action_ref, selected_result_ref));
    }
    if report.trust_basis.is_empty() {
        return Err(action_result_admission_error(action_ref, selected_result_ref));
    }
    Ok(())
}

fn action_result_admission_error(action_ref: &str, selected_result_ref: impl AsRef<str>) -> RunError {
    RunError::Eval(format!(
        "impact-action-result-not-admitted action={} result={}",
        action_ref,
        selected_result_ref.as_ref()
    ))
}

fn run_update_policy_seal(policy_path: &Path, output: &Path, is_json: bool) -> Result<(), RunError> {
    if policy_path == output {
        return Err(RunError::Eval("update policy output must differ from its input".into()));
    }
    let policy = read_json_bounded::<UpdatePolicy>(policy_path, UPDATE_INPUT_BYTES_MAX)?;
    let sealed = seal_update_policy(&policy).map_err(core_eval_error)?;
    write_json_atomically(output, &sealed, "sealed Mantlepkgs update policy")?;
    if is_json {
        println!(
            "{}",
            serde_json::to_string(&sealed)
                .map_err(|error| RunError::Internal(format!("serializing sealed update policy: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs update policy sealed: identity={} path={}",
            sealed.policy_identity_blake3,
            output.display()
        );
    }
    Ok(())
}

fn run_update_source_observe(request: UpdateObserveRequest<'_>) -> Result<(), RunError> {
    reject_update_observation_output_collision(request.response_path, request.output)?;
    let policy = read_sealed_update_policy(request.policy_path)?;
    let expected_url = configured_source_url(&policy);
    let mut acquisition = acquire_update_response(
        &policy,
        request.response_path,
        request.url,
        request.requested_status,
        request.reasons,
        &expected_url,
    )?;
    let candidates = parse_source_response(&policy, &mut acquisition);
    let observation = seal_source_observation(&policy, &SourceObservation {
        schema: UPDATE_SOURCE_OBSERVATION_SCHEMA.into(),
        observation_identity_blake3: String::new(),
        policy_identity_blake3: policy.policy_identity_blake3.clone(),
        adapter_identity: policy.adapter_identity.clone(),
        source_kind: policy.source_kind,
        query: policy.source_query.clone(),
        source_authority: policy.source_authority.clone(),
        response_schema: policy.response_schema.clone(),
        response_identity_blake3: acquisition.response_identity_blake3,
        status: acquisition.status,
        candidates,
        reason_codes: acquisition.reason_codes,
        collection: acquisition.collection,
    })
    .map_err(core_eval_error)?;
    write_json_atomically(request.output, &observation, "Mantlepkgs source observation")?;
    print_update_observation_result(
        "source",
        &observation.observation_identity_blake3,
        observation.status,
        request.output,
        request.is_json.then_some(&observation),
    )
}

fn run_update_advisory_observe(request: UpdateAdvisoryObserveRequest<'_>) -> Result<(), RunError> {
    reject_update_observation_output_collision(request.response_path, request.output)?;
    let policy = read_sealed_update_policy(request.policy_path)?;
    let service = parse_advisory_service(request.service)?;
    let requirement = match service {
        AdvisoryService::Osv => &policy.advisory_policy.osv,
        AdvisoryService::Repology => &policy.advisory_policy.repology,
    };
    let expected_url = requirement.query.clone();
    let mut acquisition = acquire_update_response(
        &policy,
        request.response_path,
        request.url,
        request.requested_status,
        request.reasons,
        &expected_url,
    )?;
    let findings = parse_advisory_response(service, &requirement.package_coordinate, request.version, &mut acquisition);
    let observation = seal_advisory_observation(&policy, &AdvisoryObservation {
        schema: UPDATE_ADVISORY_OBSERVATION_SCHEMA.into(),
        observation_identity_blake3: String::new(),
        policy_identity_blake3: policy.policy_identity_blake3.clone(),
        service,
        service_identity: requirement.service_identity.clone(),
        query: requirement.query.clone(),
        package_coordinate: requirement.package_coordinate.clone(),
        version: request.version.into(),
        response_schema: match service {
            AdvisoryService::Osv => ADVISORY_RESPONSE_OSV_SCHEMA.into(),
            AdvisoryService::Repology => ADVISORY_RESPONSE_REPOLOGY_SCHEMA.into(),
        },
        response_identity_blake3: acquisition.response_identity_blake3,
        status: acquisition.status,
        findings,
        reason_codes: acquisition.reason_codes,
        collection: acquisition.collection,
    })
    .map_err(core_eval_error)?;
    write_json_atomically(request.output, &observation, "Mantlepkgs advisory observation")?;
    print_update_observation_result(
        "advisory",
        &observation.observation_identity_blake3,
        observation.status,
        request.output,
        request.is_json.then_some(&observation),
    )
}

fn run_update_plan(request: UpdatePlanRequest<'_>) -> Result<(), RunError> {
    reject_update_plan_output_collisions(&request)?;
    let policy = read_sealed_update_policy(request.policy_path)?;
    let source_observation =
        read_json_bounded::<SourceObservation>(request.source_observation_path, UPDATE_INPUT_BYTES_MAX)?;
    let advisory_observations = request
        .advisory_observation_paths
        .iter()
        .map(|path| read_json_bounded::<AdvisoryObservation>(path, UPDATE_INPUT_BYTES_MAX))
        .collect::<Result<Vec<_>, _>>()?;
    let validation_evidence =
        read_json_bounded::<UpdateValidationEvidence>(request.validation_evidence_path, UPDATE_INPUT_BYTES_MAX)?;
    let mutation_document = observe_update_mutation_document(&policy, request.source_root)?;
    let plan = build_update_plan(UpdatePlanInput {
        policy: &policy,
        source_observation: &source_observation,
        advisory_observations: &advisory_observations,
        validation_evidence: &validation_evidence,
        mutation_documents: &[mutation_document],
    })
    .map_err(core_eval_error)?;
    write_json_atomically(request.output, &plan, "Mantlepkgs update plan")?;
    if request.is_json {
        println!(
            "{}",
            serde_json::to_string(&plan)
                .map_err(|error| RunError::Internal(format!("serializing Mantlepkgs update plan: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs update plan written: identity={} status={:?} effects={} path={}",
            plan.plan_identity_blake3,
            plan.proposal_status,
            plan.effects.len(),
            request.output.display()
        );
    }
    Ok(())
}

fn run_update_execute(request: UpdateExecuteRequest<'_>) -> Result<(), RunError> {
    reject_update_execution_path_collisions(&request)?;
    let plan = read_json_bounded::<UpdatePlan>(request.plan_path, UPDATE_INPUT_BYTES_MAX)?;
    let expected_identity = update_plan_identity_blake3(&plan).map_err(core_eval_error)?;
    if plan.plan_identity_blake3 != expected_identity {
        return Err(RunError::Eval("update-identity-mismatch plan.plan_identity_blake3".into()));
    }
    let result = execute_update_plan(&plan, &request);
    match result {
        Ok(receipt) => {
            if request.is_json {
                println!(
                    "{}",
                    serde_json::to_string(&receipt).map_err(|error| {
                        RunError::Internal(format!("serializing Mantlepkgs update execution receipt: {error}"))
                    })?
                );
            } else {
                println!(
                    "mantlepkgs update tree published: receipt={} path={}",
                    receipt.receipt_identity_blake3,
                    request.output_root.display()
                );
            }
            Ok(())
        }
        Err(error) => {
            let denial = record_update_execution(
                &plan,
                UpdateExecutionDisposition::Denied,
                None,
                Vec::new(),
                update_denial_reason_codes(&error),
            )
            .map_err(core_eval_error)?;
            write_json_atomically(request.denial_receipt_output, &denial, "Mantlepkgs update denial receipt")?;
            Err(error)
        }
    }
}

fn update_denial_reason_codes(error: &RunError) -> Vec<String> {
    let mut reasons = vec![UPDATE_DENIAL_GENERIC_REASON.into()];
    let message = error.message();
    let code = message.split_ascii_whitespace().next().unwrap_or_default();
    if is_stable_update_denial_code(code) {
        reasons.push(code.into());
    }
    reasons.sort();
    reasons
}

fn is_stable_update_denial_code(code: &str) -> bool {
    if !code.starts_with("update-") {
        return false;
    }
    if code == UPDATE_DENIAL_GENERIC_REASON {
        return false;
    }
    code.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn execute_update_plan(
    plan: &UpdatePlan,
    request: &UpdateExecuteRequest<'_>,
) -> Result<mantlepkgs_core::UpdateExecutionReceipt, RunError> {
    if plan.selection.selected.is_none() || plan.effects.is_empty() {
        return Err(RunError::Eval("update plan has no selected mutation effects".into()));
    }
    reject_symlink_root(request.source_root)?;
    let output_parent = request
        .output_root
        .parent()
        .ok_or_else(|| RunError::Eval("update output root has no parent".into()))?;
    reject_symlink_root(output_parent)?;
    if request.output_root.exists() {
        return Err(RunError::Eval("update output root already exists".into()));
    }
    let stage = update_stage_path(request.output_root)?;
    if stage.exists() {
        return Err(RunError::Eval(format!("update stage already exists: {}", stage.display())));
    }
    create_private_update_stage(&stage)?;
    let execution = write_update_effects(plan, request.source_root, &stage).and_then(|outputs| {
        let receipt = record_update_execution(
            plan,
            UpdateExecutionDisposition::Applied,
            Some(request.output_root.display().to_string()),
            outputs,
            Vec::new(),
        )
        .map_err(core_eval_error)?;
        write_update_json_create_new(&stage.join(UPDATE_EXECUTION_RECEIPT_FILE), &receipt)?;
        sync_directory(&stage)?;
        rename_path_no_replace(&stage, request.output_root)
            .map_err(|error| RunError::Internal(format!("publishing update output tree: {error}")))?;
        sync_directory(output_parent)?;
        Ok(receipt)
    });
    if execution.is_err() && stage.exists() {
        fs::remove_dir_all(&stage).map_err(|error| {
            RunError::Internal(format!("removing failed update stage {}: {error}", stage.display()))
        })?;
    }
    execution
}

fn write_update_effects(plan: &UpdatePlan, source_root: &Path, stage: &Path) -> Result<Vec<AppliedOutput>, RunError> {
    let mut outputs = Vec::with_capacity(plan.effects.len());
    for effect in &plan.effects {
        let source = secure_update_source_path(source_root, &effect.relative_path)?;
        let input = read_bounded(&source, UPDATE_DOCUMENT_BYTES_MAX)?;
        let input_digest = blake3::hash(&input).to_hex().to_string();
        if input_digest != effect.input_digest_blake3 {
            return Err(RunError::Eval(format!("update-stale-preimage {}", effect.relative_path)));
        }
        let input_document = deserialize_json::<serde_json::Value>(&input, &source.display().to_string())?;
        validate_update_effect_old_values(effect, &input_document)?;
        let output = canonical_mutation_document_bytes(&effect.output_document).map_err(core_eval_error)?;
        let output_digest = blake3::hash(&output).to_hex().to_string();
        if output_digest != effect.output_digest_blake3 {
            return Err(RunError::Eval(format!("update-output-digest-mismatch {}", effect.relative_path)));
        }
        let target = stage.join(&effect.relative_path);
        let parent = target.parent().ok_or_else(|| RunError::Eval("update target has no parent".into()))?;
        fs::create_dir_all(parent).map_err(|error| {
            RunError::Internal(format!("creating update stage directory {}: {error}", parent.display()))
        })?;
        write_bytes_create_new(&target, &output)?;
        outputs.push(AppliedOutput {
            relative_path: effect.relative_path.clone(),
            output_digest_blake3: output_digest,
        });
    }
    outputs.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(outputs)
}

fn validate_update_effect_old_values(
    effect: &mantlepkgs_core::DocumentMutationEffect,
    document: &serde_json::Value,
) -> Result<(), RunError> {
    let mut pointers = BTreeSet::new();
    for edit in &effect.edits {
        if !pointers.insert(edit.field_pointer.as_str()) {
            return Err(RunError::Eval(format!("update-duplicate-field {}", edit.field_pointer)));
        }
        let observed = document
            .pointer(&edit.field_pointer)
            .ok_or_else(|| RunError::Eval(format!("update-field-missing {}", edit.field_pointer)))?;
        if observed != &edit.old_value {
            return Err(RunError::Eval(format!("update-old-value-mismatch {}", edit.field_pointer)));
        }
    }
    Ok(())
}

fn observe_update_mutation_document(policy: &UpdatePolicy, source_root: &Path) -> Result<MutationDocument, RunError> {
    reject_symlink_root(source_root)?;
    let source = secure_update_source_path(source_root, &policy.mutation.relative_path)?;
    let bytes = read_bounded(&source, policy.limits.max_document_bytes)?;
    let input_bytes = u64::try_from(bytes.len())
        .map_err(|_| RunError::Eval("update mutation document byte count exceeds u64".into()))?;
    let document = deserialize_json::<serde_json::Value>(&bytes, &source.display().to_string())?;
    Ok(MutationDocument {
        schema: UPDATE_MUTATION_DOCUMENT_SCHEMA.into(),
        relative_path: policy.mutation.relative_path.clone(),
        input_bytes,
        input_digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        document,
    })
}

fn secure_update_source_path(root: &Path, relative: &str) -> Result<PathBuf, RunError> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(RunError::Eval(format!("update-unsafe-path {relative}")));
    }
    let mut current = root.to_path_buf();
    for component in relative_path.components() {
        let Component::Normal(name) = component else {
            return Err(RunError::Eval(format!("update-unsafe-path {relative}")));
        };
        current.push(name);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| RunError::Eval(format!("reading update source {}: {error}", current.display())))?;
        if metadata.file_type().is_symlink() {
            return Err(RunError::Eval(format!("update-symlink-forbidden {}", current.display())));
        }
    }
    let metadata = fs::metadata(&current)
        .map_err(|error| RunError::Eval(format!("reading update source {}: {error}", current.display())))?;
    if !metadata.is_file() {
        return Err(RunError::Eval(format!("update-source-not-regular {}", current.display())));
    }
    Ok(current)
}

fn read_sealed_update_policy(path: &Path) -> Result<UpdatePolicy, RunError> {
    let policy = read_json_bounded::<UpdatePolicy>(path, UPDATE_INPUT_BYTES_MAX)?;
    let sealed = seal_update_policy(&policy).map_err(core_eval_error)?;
    if policy != sealed {
        return Err(RunError::Eval("update-identity-mismatch policy.policy_identity_blake3".into()));
    }
    Ok(policy)
}

fn acquire_update_response(
    policy: &UpdatePolicy,
    response_path: Option<&Path>,
    url: Option<&str>,
    requested_status: &str,
    reasons: &[String],
    expected_url: &str,
) -> Result<ResponseAcquisition, RunError> {
    let status = parse_observation_status(requested_status)?;
    if status != ObservationStatus::Success {
        if response_path.is_some() || url.is_some() {
            return Err(RunError::Eval(
                "explicit unavailable or failed observation cannot include a response input".into(),
            ));
        }
        return Ok(ResponseAcquisition {
            status,
            bytes: None,
            response_identity_blake3: None,
            reason_codes: reasons.to_vec(),
            collection: CollectionFacts {
                response_bytes: 0,
                redirect_count: 0,
                retry_count: 0,
                elapsed_millis: 0,
            },
        });
    }
    if !reasons.is_empty() {
        return Err(RunError::Eval("successful observation cannot include failure reasons".into()));
    }
    match (response_path, url) {
        (Some(path), None) => acquire_saved_update_response(path, policy.limits.max_response_bytes),
        (None, Some(url)) => acquire_live_update_response(policy, url, expected_url),
        _ => Err(RunError::Eval("successful observation requires exactly one --response or --url input".into())),
    }
}

fn acquire_saved_update_response(path: &Path, limit: u64) -> Result<ResponseAcquisition, RunError> {
    const SAVED_REPLAY_ELAPSED_MILLIS: u64 = 0;
    match read_bounded(path, limit) {
        Ok(bytes) => Ok(successful_acquisition(bytes, SAVED_REPLAY_ELAPSED_MILLIS)),
        Err(_) => Ok(failed_acquisition(
            ObservationStatus::Failed,
            "saved-response-read-failed",
            SAVED_REPLAY_ELAPSED_MILLIS,
        )),
    }
}

fn acquire_live_update_response(
    policy: &UpdatePolicy,
    url: &str,
    expected_url: &str,
) -> Result<ResponseAcquisition, RunError> {
    if url != expected_url || !url.starts_with("https://") {
        return Err(RunError::Eval("update URL differs from configured HTTPS query authority".into()));
    }
    let started = Instant::now();
    let agent = ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_millis(policy.limits.max_elapsed_millis)))
        .timeout_connect(Some(Duration::from_millis(policy.limits.max_elapsed_millis)))
        .build()
        .new_agent();
    let response = agent.get(url).header("user-agent", UPDATE_HTTP_USER_AGENT).call();
    let elapsed_millis = elapsed_millis_bounded(started, policy.limits.max_elapsed_millis);
    let Ok(response) = response else {
        return Ok(failed_acquisition(ObservationStatus::Unavailable, "transport-unavailable", elapsed_millis));
    };
    let status = response.status().as_u16();
    let body_limit = policy
        .limits
        .max_response_bytes
        .checked_add(1)
        .ok_or_else(|| RunError::Eval("update response byte limit overflow".into()))?;
    let mut bytes = Vec::new();
    let read = response.into_body().with_config().limit(body_limit).reader().read_to_end(&mut bytes);
    if read.is_err() || u64::try_from(bytes.len()).unwrap_or(u64::MAX) > policy.limits.max_response_bytes {
        return Ok(failed_acquisition(ObservationStatus::Failed, "response-byte-limit-exceeded", elapsed_millis));
    }
    Ok(classify_update_http_response(status, bytes, elapsed_millis))
}

fn classify_update_http_response(status: u16, bytes: Vec<u8>, elapsed_millis: u64) -> ResponseAcquisition {
    if (UPDATE_HTTP_REDIRECT_STATUS_MIN..=UPDATE_HTTP_REDIRECT_STATUS_MAX).contains(&status) {
        return failed_acquisition(ObservationStatus::Failed, "redirect-forbidden", elapsed_millis);
    }
    if !(UPDATE_HTTP_SUCCESS_STATUS_MIN..=UPDATE_HTTP_SUCCESS_STATUS_MAX).contains(&status) {
        let mut acquisition = successful_acquisition(bytes, elapsed_millis);
        acquisition.status = ObservationStatus::Failed;
        acquisition.reason_codes = vec!["http-status-invalid".into()];
        return acquisition;
    }
    successful_acquisition(bytes, elapsed_millis)
}

fn successful_acquisition(bytes: Vec<u8>, elapsed_millis: u64) -> ResponseAcquisition {
    let response_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    let identity = blake3::hash(&bytes).to_hex().to_string();
    ResponseAcquisition {
        status: ObservationStatus::Success,
        bytes: Some(bytes),
        response_identity_blake3: Some(identity),
        reason_codes: Vec::new(),
        collection: CollectionFacts {
            response_bytes,
            redirect_count: 0,
            retry_count: 0,
            elapsed_millis,
        },
    }
}

fn failed_acquisition(
    status: ObservationStatus,
    reason: impl Into<String>,
    elapsed_millis: u64,
) -> ResponseAcquisition {
    ResponseAcquisition {
        status,
        bytes: None,
        response_identity_blake3: None,
        reason_codes: vec![reason.into()],
        collection: CollectionFacts {
            response_bytes: 0,
            redirect_count: 0,
            retry_count: 0,
            elapsed_millis,
        },
    }
}

fn parse_source_response(policy: &UpdatePolicy, acquisition: &mut ResponseAcquisition) -> Vec<SourceCandidate> {
    if acquisition.status != ObservationStatus::Success {
        return Vec::new();
    }
    let parsed = acquisition.bytes.as_deref().ok_or_else(|| "response-body-missing".to_string()).and_then(|bytes| {
        serde_json::from_slice::<SourceResponsePayload>(bytes).map_err(|_| "response-schema-invalid".into())
    });
    match parsed {
        Ok(payload) if payload.schema == policy.response_schema => payload.candidates,
        Ok(_) => {
            mark_acquisition_failed(acquisition, "response-schema-mismatch");
            Vec::new()
        }
        Err(reason) => {
            mark_acquisition_failed(acquisition, reason);
            Vec::new()
        }
    }
}

fn parse_advisory_response(
    service: AdvisoryService,
    expected_package_coordinate: &str,
    expected_version: &str,
    acquisition: &mut ResponseAcquisition,
) -> Vec<AdvisoryFinding> {
    if acquisition.status != ObservationStatus::Success {
        return Vec::new();
    }
    let expected_schema = match service {
        AdvisoryService::Osv => ADVISORY_RESPONSE_OSV_SCHEMA,
        AdvisoryService::Repology => ADVISORY_RESPONSE_REPOLOGY_SCHEMA,
    };
    let parsed = acquisition.bytes.as_deref().ok_or_else(|| "response-body-missing".to_string()).and_then(|bytes| {
        serde_json::from_slice::<AdvisoryResponsePayload>(bytes).map_err(|_| "response-schema-invalid".into())
    });
    match parsed {
        Ok(payload) if payload.schema != expected_schema => {
            mark_acquisition_failed(acquisition, "response-schema-mismatch");
            Vec::new()
        }
        Ok(payload) if payload.package_coordinate != expected_package_coordinate => {
            mark_acquisition_failed(acquisition, "advisory-package-coordinate-mismatch");
            Vec::new()
        }
        Ok(payload) if payload.version != expected_version => {
            mark_acquisition_failed(acquisition, "advisory-version-mismatch");
            Vec::new()
        }
        Ok(mut payload) => {
            payload.findings.sort();
            if payload.findings.windows(UPDATE_DUPLICATE_WINDOW_SIZE).any(|window| window.first() == window.last()) {
                mark_acquisition_failed(acquisition, "advisory-finding-duplicate");
                Vec::new()
            } else {
                payload.findings
            }
        }
        Err(reason) => {
            mark_acquisition_failed(acquisition, reason);
            Vec::new()
        }
    }
}

fn mark_acquisition_failed(acquisition: &mut ResponseAcquisition, reason: impl Into<String>) {
    acquisition.status = ObservationStatus::Failed;
    acquisition.reason_codes = vec![reason.into()];
    acquisition.bytes = None;
}

fn parse_observation_status(value: &str) -> Result<ObservationStatus, RunError> {
    match value {
        "success" => Ok(ObservationStatus::Success),
        "unavailable" => Ok(ObservationStatus::Unavailable),
        "failed" => Ok(ObservationStatus::Failed),
        _ => Err(RunError::Eval(format!("unsupported update observation status: {value}"))),
    }
}

fn parse_advisory_service(value: &str) -> Result<AdvisoryService, RunError> {
    match value {
        "osv" => Ok(AdvisoryService::Osv),
        "repology" => Ok(AdvisoryService::Repology),
        _ => Err(RunError::Eval(format!("unsupported advisory service: {value}"))),
    }
}

fn configured_source_url(policy: &UpdatePolicy) -> String {
    format!("{}/{}", policy.source_authority, policy.source_query.trim_start_matches('/'))
}

fn reject_update_observation_output_collision(response: Option<&Path>, output: &Path) -> Result<(), RunError> {
    if response == Some(output) {
        return Err(RunError::Eval("update observation output must differ from its response input".into()));
    }
    Ok(())
}

fn reject_update_plan_output_collisions(request: &UpdatePlanRequest<'_>) -> Result<(), RunError> {
    let primary_collision = request.output == request.policy_path
        || request.output == request.source_observation_path
        || request.output == request.validation_evidence_path;
    let advisory_collision = request.advisory_observation_paths.iter().any(|path| path == request.output);
    if primary_collision || advisory_collision {
        return Err(RunError::Eval("update plan output must differ from every input artifact".into()));
    }
    Ok(())
}

fn reject_update_execution_path_collisions(request: &UpdateExecuteRequest<'_>) -> Result<(), RunError> {
    let output_overlaps_source =
        request.output_root.starts_with(request.source_root) || request.source_root.starts_with(request.output_root);
    if output_overlaps_source
        || request.denial_receipt_output == request.plan_path
        || request.denial_receipt_output.starts_with(request.output_root)
        || request.denial_receipt_output.starts_with(request.source_root)
    {
        return Err(RunError::Eval("update execution paths conflict".into()));
    }
    Ok(())
}

fn update_stage_path(output: &Path) -> Result<PathBuf, RunError> {
    let parent = output.parent().ok_or_else(|| RunError::Eval("update output root has no parent".into()))?;
    let name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| RunError::Eval("update output root has no UTF-8 name".into()))?;
    Ok(parent.join(format!("{UPDATE_STAGE_PREFIX}{name}-{}", std::process::id())))
}

fn create_private_update_stage(stage: &Path) -> Result<(), RunError> {
    fs::create_dir(stage)
        .map_err(|error| RunError::Internal(format!("creating private update stage {}: {error}", stage.display())))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
        fs::set_permissions(stage, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE)).map_err(|error| {
            RunError::Internal(format!("setting private update stage permissions {}: {error}", stage.display()))
        })?;
    }
    Ok(())
}

fn write_bytes_create_new(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| RunError::Internal(format!("creating update output {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| RunError::Internal(format!("writing update output {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| RunError::Internal(format!("syncing update output {}: {error}", path.display())))?;
    Ok(())
}

fn write_update_json_create_new(path: &Path, value: &impl Serialize) -> Result<(), RunError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| RunError::Internal(format!("serializing update receipt: {error}")))?;
    bytes.push(b'\n');
    write_bytes_create_new(path, &bytes)
}

fn elapsed_millis_bounded(started: Instant, maximum: u64) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX).min(maximum)
}

fn print_update_observation_result<T: Serialize>(
    kind: &str,
    identity: &str,
    status: ObservationStatus,
    output: &Path,
    json_value: Option<&T>,
) -> Result<(), RunError> {
    if let Some(value) = json_value {
        println!(
            "{}",
            serde_json::to_string(value)
                .map_err(|error| RunError::Internal(format!("serializing Mantlepkgs {kind} observation: {error}")))?
        );
    } else {
        println!(
            "mantlepkgs {kind} observation written: identity={identity} status={status:?} path={}",
            output.display()
        );
    }
    Ok(())
}

fn observed_corpus_catalog_packages(role: &str, bytes: &[u8]) -> Result<Vec<String>, RunError> {
    if role != CORPUS_ROLE_CATALOG {
        return Ok(Vec::new());
    }
    let catalog = serde_json::from_slice::<MantlepkgsCatalog>(bytes)
        .map_err(|error| RunError::Eval(format!("parsing bound corpus catalog: {error}")))?;
    validate_catalog_identity(&catalog).map_err(core_eval_error)?;
    let mut packages = catalog.packages.iter().map(|package| package.name.clone()).collect::<Vec<_>>();
    packages.sort();
    packages.dedup();
    Ok(packages)
}

fn validation_observation(
    plan: &ValidationRootPlan,
    realization: &ForeignRealizationReceipt,
    elapsed_milliseconds: u64,
) -> Result<ValidationObservation, RunError> {
    let timeout_milliseconds = plan
        .timeout_seconds
        .checked_mul(MILLISECONDS_PER_SECOND)
        .ok_or_else(|| RunError::Internal("validation timeout milliseconds overflow".into()))?;
    let outcome = if elapsed_milliseconds > timeout_milliseconds {
        mantlepkgs_core::OBSERVED_VALIDATION_TIMEOUT
    } else if realization.failure.is_some() {
        mantlepkgs_core::OBSERVED_VALIDATION_FAIL
    } else {
        mantlepkgs_core::OBSERVED_VALIDATION_PASS
    };
    let observed_output_bytes = realization.units.iter().try_fold(0u64, |total, unit| {
        unit.outputs.iter().try_fold(total, |unit_total, output| {
            unit_total
                .checked_add(output.nar_size)
                .ok_or_else(|| RunError::Internal("validation output byte count overflow".into()))
        })
    })?;
    if observed_output_bytes > plan.max_output_bytes {
        return Err(RunError::Eval(format!(
            "validation output bytes {observed_output_bytes} exceed plan limit {}",
            plan.max_output_bytes
        )));
    }
    let diagnostic_digest_blake3 = realization
        .failure
        .as_ref()
        .map(|failure| {
            serde_json::to_vec(failure)
                .map(|bytes| blake3_hex(&bytes))
                .map_err(|error| RunError::Internal(format!("serializing validation failure: {error}")))
        })
        .transpose()?;
    Ok(ValidationObservation {
        schema: VALIDATION_OBSERVATION_SCHEMA.into(),
        validation_root_identity_blake3: plan.validation_root_identity_blake3.clone(),
        outcome: outcome.into(),
        realization_receipt_blake3: realization.receipt_blake3.clone(),
        observed_output_bytes,
        elapsed_milliseconds,
        diagnostic_digest_blake3,
    })
}

fn validation_system<'a>(catalog: &'a DomainCatalog, identity: &str) -> Result<&'a str, RunError> {
    catalog
        .validation_roots
        .iter()
        .find(|root| root.validation_root_identity_blake3 == identity)
        .map(|root| root.system.as_str())
        .ok_or_else(|| RunError::Eval("validation root disappeared after planning".into()))
}

fn require_validation_generation_binding(
    domain_catalog: &DomainCatalog,
    plan: &ValidationRootPlan,
    generation: &MantlepkgsCatalog,
) -> Result<(), RunError> {
    let system = validation_system(domain_catalog, &plan.validation_root_identity_blake3)?;
    let validation_package = domain_catalog
        .packages
        .iter()
        .find(|package| package.system == system && package.public_selector == plan.validation_selector)
        .ok_or_else(|| RunError::Eval("validation package is absent from the domain catalog".into()))?;
    let shard = domain_catalog
        .shards
        .iter()
        .find(|shard| shard.shard_identity_blake3 == validation_package.shard_identity_blake3)
        .ok_or_else(|| RunError::Eval("validation package has no source shard".into()))?;
    if shard.source_catalog_identity_blake3 != generation.catalog_identity_blake3 {
        return Err(RunError::Eval("validation package source catalog differs from the selected generation".into()));
    }
    Ok(())
}

fn reject_output_path_collisions(paths: &[&Path]) -> Result<(), RunError> {
    let unique = paths.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != paths.len() {
        return Err(RunError::Eval("Mantlepkgs output paths must be distinct".into()));
    }
    Ok(())
}

fn evaluate_domain_manifest(path: &Path) -> Result<DomainCatalogManifest, RunError> {
    let import_paths = path.parent().map(|parent| vec![OsString::from(parent)]).unwrap_or_default();
    crunch_eval::evaluate_and_deserialize(path, &import_paths).map_err(|error| {
        RunError::Eval(format!("evaluating typed Mantlepkgs domain manifest {}: {error}", path.display()))
    })
}

fn run_generate(manifest_path: &Path, nix_program: &Path, output_root: &Path, json: bool) -> Result<(), RunError> {
    let manifest = normalize_manifest(&evaluate_manifest(manifest_path)?).map_err(core_eval_error)?;
    let _ = load_policy_artifacts(manifest_path, &manifest)?;
    let seed_retention_root = prepare_seed_retention_root(output_root)?;
    let batch = produce_locked_batch(&manifest, nix_program, &seed_retention_root)?;
    publish_batch(manifest_path, &manifest, &batch, output_root, json)
}

fn prepare_seed_retention_root(output_root: &Path) -> Result<PathBuf, RunError> {
    let root = output_root.join(PRODUCER_SEED_ROOT_DIRECTORY);
    fs::create_dir_all(&root).map_err(|error| {
        RunError::Internal(format!("creating producer seed retention root {}: {error}", root.display()))
    })?;
    let canonical = fs::canonicalize(&root).map_err(|error| {
        RunError::Internal(format!("resolving producer seed retention root {}: {error}", root.display()))
    })?;
    debug_assert!(canonical.is_absolute());
    debug_assert!(canonical.is_dir());
    Ok(canonical)
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
            backend: context.backend,
            base_state_dirs: context.base_state_dirs,
            source_bundle_bytes_max: MANTLEPKGS_SOURCE_BUNDLE_BYTES_MAX,
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

fn produce_locked_batch(
    manifest: &MantlepkgsManifest,
    nix_program: &Path,
    seed_retention_root: &Path,
) -> Result<ProducerBatch, RunError> {
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
        .map(|selector| produce_selector(manifest, selector, nix_program, &producer, seed_retention_root))
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
    producer: &ProducerObservation,
    seed_retention_root: &Path,
) -> ProducedPackageRecord {
    match produce_selector_result(manifest, selector, nix_program, producer, seed_retention_root) {
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
    producer: &ProducerObservation,
    seed_retention_root: &Path,
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
    let mut artifacts = lower_nix_derivation_json_closure(&selected, &NixProducerConfig {
        package_name: selector.name.clone(),
        system: selector.system.clone(),
        root_derivation,
        producer_identity: installable,
        producer_revision: manifest.source.revision.clone(),
        cache_hints: Vec::new(),
        unsupported_metadata_classes: Vec::new(),
    })
    .map_err(|diagnostic| producer_blocker(&diagnostic.class, selector, &diagnostic.message))?;
    let seed_retention_link = fixed_output_seed_retention_link(seed_retention_root, producer, selector)
        .map_err(|error| producer_blocker("fixed-output-seed-retention-invalid", selector, &error.to_string()))?;
    bind_fixed_output_seeds(&mut artifacts.graph, nix_program, selector, &seed_retention_link)?;
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

fn fixed_output_seed_candidates(graph: &ForeignDerivationGraph) -> Result<Vec<FixedOutputSeedCandidate>, String> {
    let mut seen_outputs = BTreeSet::new();
    let mut candidates = Vec::new();
    for node in &graph.nodes {
        if node.fixed_output.is_none() {
            continue;
        }
        if node.outputs.len() != 1 {
            return Err(format!("fixed-output derivation {} must have one output", node.node_id));
        }
        let (output_name, output) = node
            .outputs
            .first_key_value()
            .ok_or_else(|| format!("fixed-output derivation {} has no output", node.node_id))?;
        if !seen_outputs.insert(output.path.clone()) {
            continue;
        }
        let output_digest = blake3_hex(output.path.as_bytes());
        let payload_hash = &output_digest[..SEED_PAYLOAD_HASH_HEX_CHARS];
        candidates.push(FixedOutputSeedCandidate {
            derivation_path: node.original_derivation.clone(),
            output_name: output_name.clone(),
            output_path: output.path.clone(),
            payload_id: format!("nix-fixed-output-seed:{payload_hash}"),
        });
    }
    candidates.sort();
    debug_assert!(candidates.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert_eq!(seen_outputs.len(), candidates.len());
    Ok(candidates)
}

fn fixed_output_seed_retention_link(
    seed_retention_root: &Path,
    producer: &ProducerObservation,
    selector: &mantlepkgs_core::PackageSelector,
) -> Result<PathBuf, RunError> {
    let identity = canonical_digest(&(PRODUCER_SEED_RETENTION_SCHEMA, producer, selector))?;
    let link = seed_retention_root.join(identity);
    debug_assert!(seed_retention_root.is_absolute());
    debug_assert!(link.starts_with(seed_retention_root));
    Ok(link)
}

fn fixed_output_seed_realization_args(
    candidates: &[FixedOutputSeedCandidate],
    seed_retention_link: &Path,
) -> Result<Vec<String>, String> {
    let retention_link = seed_retention_link
        .to_str()
        .ok_or_else(|| "producer seed retention link is not valid UTF-8".to_string())?;
    let mut args = vec![
        "--extra-experimental-features".to_string(),
        NIX_EXPERIMENTAL_FEATURES.to_string(),
        "build".to_string(),
        "--out-link".to_string(),
        retention_link.to_string(),
        "--print-out-paths".to_string(),
    ];
    args.extend(
        candidates
            .iter()
            .map(|candidate| format!("{}^{}", candidate.derivation_path, candidate.output_name)),
    );
    debug_assert!(args.iter().any(|arg| arg == "--out-link"));
    debug_assert!(!args.iter().any(|arg| arg == "--no-link"));
    Ok(args)
}

fn bind_fixed_output_seeds(
    graph: &mut ForeignDerivationGraph,
    nix_program: &Path,
    selector: &mantlepkgs_core::PackageSelector,
    seed_retention_link: &Path,
) -> Result<(), CatalogBlocker> {
    let candidates = fixed_output_seed_candidates(graph)
        .map_err(|message| producer_blocker("fixed-output-seed-invalid", selector, &message))?;
    if candidates.is_empty() {
        return Ok(());
    }
    let args = fixed_output_seed_realization_args(&candidates, seed_retention_link)
        .map_err(|message| producer_blocker("fixed-output-seed-retention-invalid", selector, &message))?;
    let borrowed_args = args.iter().map(String::as_str).collect::<Vec<_>>();
    let output = run_nix(nix_program, &borrowed_args, SEED_PATHS_STDOUT_MAX_BYTES)
        .map_err(|error| producer_blocker("fixed-output-seed-realization-failed", selector, &error.to_string()))?;
    require_success(&output, "fixed-output-seed-realization-failed", selector)?;
    let observed_paths = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let expected_paths = candidates.iter().map(|candidate| candidate.output_path.clone()).collect::<BTreeSet<_>>();
    if observed_paths != expected_paths {
        return Err(producer_blocker(
            "fixed-output-seed-path-mismatch",
            selector,
            "the Nix producer realized a different fixed-output path set",
        ));
    }
    for candidate in candidates {
        let expected_content_blake3 =
            digest_bound_foreign_source_path(&candidate.payload_id, Path::new(&candidate.output_path))
                .map_err(|error| producer_blocker("fixed-output-seed-binding-failed", selector, &error.to_string()))?;
        graph.source_payloads.push(SourcePayload {
            payload_id: candidate.payload_id,
            kind: FIXED_OUTPUT_SEED_KIND.into(),
            content_ref: candidate.output_path,
            embedded_text: None,
            mirrors: Vec::new(),
            expected_content_blake3: Some(expected_content_blake3),
        });
    }
    graph.source_payloads.sort_by(|left, right| left.payload_id.cmp(&right.payload_id));
    debug_assert!(graph.source_payloads.windows(2).all(|pair| pair[0].payload_id < pair[1].payload_id));
    Ok(())
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

pub(crate) fn run_nix(program: &Path, arguments: &[&str], max_stdout_bytes: usize) -> Result<Output, RunError> {
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

pub(crate) fn read_capture(file: &mut fs::File, maximum: usize, label: &str) -> Result<Vec<u8>, RunError> {
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

pub(crate) fn core_eval_error(failure: CoreFailure) -> RunError {
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
    #[cfg(unix)]
    use std::os::unix::ffi::OsStringExt as _;

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
    const COMPOSED_PUBLIC_PACKAGE_COUNT: usize = 6;
    const IMPACT_FIXTURE_ITEM_LIMIT: u32 = 8;
    const IMPACT_FIXTURE_DIAGNOSTIC_LIMIT: u32 = 16;
    const IMPACT_FIXTURE_REPORT_BYTES: u64 = 1_048_576;
    const UPDATE_FIXTURE_LIMIT: u32 = 64;
    const UPDATE_FIXTURE_BYTES: u64 = 1_048_576;
    const UPDATE_FIXTURE_MILLIS: u64 = 30_000;
    const UPDATE_FIXTURE_MAX_COMPONENT: u64 = 9_999;
    const UPDATE_FIXTURE_COMPONENT_COUNT: u8 = 3;
    const UPDATE_TINY_RESPONSE_BYTES: u64 = 32;
    const UPDATE_OVERSIZED_RESPONSE_BYTES: usize = 33;
    const UPDATE_FIXTURE_HTTP_SUCCESS_STATUS: u16 = 200;
    const UPDATE_FIXTURE_HTTP_REDIRECT_STATUS: u16 = 302;
    const UPDATE_FIXTURE_HTTP_ERROR_STATUS: u16 = 500;
    const OTHER_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const IMPACT_ACTION_REF: &str = "mantle-action://blake3/impact-fixture";
    const IMPACT_RESULT_REF: &str = "mantle-action-result://blake3/impact-fixture";
    const NO_NIX_GENERATION_ENV: &str = "MANTLEPKGS_TEST_GENERATION";
    const NO_NIX_CHILD_TEST: &str = "mantlepkgs_cmd::tests::no_nix_child_verifies_and_plans_catalog";
    const CAPTURE_CHILD_TEST: &str = "mantlepkgs_cmd::tests::producer_capture_child";
    const TINY_CAPTURE_LIMIT: usize = 1;
    #[cfg(unix)]
    const NON_UTF8_PATH_BYTE: u8 = 0xff;

    fn update_policy_fixture() -> UpdatePolicy {
        seal_update_policy(&UpdatePolicy {
            schema: mantlepkgs_core::UPDATE_POLICY_SCHEMA.into(),
            policy_identity_blake3: String::new(),
            migration: mantlepkgs_core::UpdatePolicyMigration {
                source_schema: None,
                source_policy_identity_blake3: None,
                reviewed: false,
            },
            package: mantlepkgs_core::ImpactPackageKey {
                public_selector: "hello".into(),
            },
            system: "x86_64-linux".into(),
            source_kind: mantlepkgs_core::UpdateSourceKind::GitTags,
            adapter_identity: "mantle-git-tags-v1".into(),
            source_authority: "https://api.example.invalid".into(),
            source_query: "/repos/example/hello/tags".into(),
            response_schema: mantlepkgs_core::SOURCE_RESPONSE_GIT_TAGS_SCHEMA.into(),
            current_version: "v1.2.3".into(),
            current_source_ref: "refs/tags/v1.2.3".into(),
            current_source_identity_blake3: DIGEST.into(),
            version_rules: mantlepkgs_core::VersionRules {
                required_prefix: "v".into(),
                component_count: UPDATE_FIXTURE_COMPONENT_COUNT,
                max_component: UPDATE_FIXTURE_MAX_COMPONENT,
                minimum_version: Some("v1.0.0".into()),
                maximum_version: Some("v2.0.0".into()),
                ignored_versions: Vec::new(),
                allow_prerelease: false,
                odd_minor_is_development: false,
                high_patch_development_from: None,
            },
            patch_policy: mantlepkgs_core::PatchPolicy {
                allow_major: false,
                allow_minor: true,
                allow_patch: true,
            },
            advisory_policy: mantlepkgs_core::AdvisoryPolicy {
                osv: mantlepkgs_core::AdvisoryRequirement {
                    mode: mantlepkgs_core::AdvisoryRequirementMode::Required,
                    service_identity: "osv-v1".into(),
                    query: "https://api.osv.dev/v1/query".into(),
                    package_coordinate: "pkg:generic/hello".into(),
                },
                repology: mantlepkgs_core::AdvisoryRequirement {
                    mode: mantlepkgs_core::AdvisoryRequirementMode::Required,
                    service_identity: "repology-v1".into(),
                    query: "https://repology.org/api/v1/project/hello".into(),
                    package_coordinate: "hello".into(),
                },
                block_on_findings: true,
            },
            validation_policy: mantlepkgs_core::UpdateValidationPolicy {
                require_catalog: true,
                require_build_observations: true,
                require_validation_roots: true,
                require_impact_report: true,
            },
            mutation: mantlepkgs_core::UpdateMutationTarget {
                relative_path: "locks/hello.json".into(),
                version_pointer: "/version".into(),
                source_ref_pointer: "/source/ref".into(),
                source_identity_pointer: "/source/identity_blake3".into(),
            },
            allow_ambient_credentials: false,
            allow_ambient_proxy: false,
            limits: mantlepkgs_core::UpdateLimits {
                max_response_bytes: UPDATE_FIXTURE_BYTES,
                max_document_bytes: UPDATE_FIXTURE_BYTES,
                max_plan_bytes: UPDATE_FIXTURE_BYTES,
                max_candidates: UPDATE_FIXTURE_LIMIT,
                max_findings: UPDATE_FIXTURE_LIMIT,
                max_effects: UPDATE_FIXTURE_LIMIT,
                max_diagnostics: UPDATE_FIXTURE_LIMIT,
                max_artifacts: UPDATE_FIXTURE_LIMIT,
                max_redirects: 0,
                max_retries: 0,
                max_elapsed_millis: UPDATE_FIXTURE_MILLIS,
            },
        })
        .unwrap()
    }

    fn update_validation_fixture() -> UpdateValidationEvidence {
        let success = || mantlepkgs_core::LinkedEvidence {
            status: mantlepkgs_core::EvidenceStatus::Success,
            artifact_identity_blake3: vec![DIGEST.into()],
            reason_codes: Vec::new(),
        };
        UpdateValidationEvidence {
            schema: mantlepkgs_core::UPDATE_VALIDATION_EVIDENCE_SCHEMA.into(),
            candidate_version: "v1.3.0".into(),
            candidate_source_identity_blake3: OTHER_DIGEST.into(),
            catalog: success(),
            build_observations: success(),
            validation_roots: success(),
            impact_report: success(),
        }
    }

    fn write_update_fixture_inputs(root: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
        let policy_path = root.join("policy.json");
        let source_response = root.join("source-response.json");
        let osv_response = root.join("osv-response.json");
        let repology_response = root.join("repology-response.json");
        let validation_path = root.join("validation.json");
        fs::write(&policy_path, serde_json::to_vec_pretty(&update_policy_fixture()).unwrap()).unwrap();
        fs::write(
            &source_response,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": mantlepkgs_core::SOURCE_RESPONSE_GIT_TAGS_SCHEMA,
                "candidates": [{
                    "version": "v1.3.0",
                    "source_ref": "refs/tags/v1.3.0",
                    "source_identity_blake3": OTHER_DIGEST,
                }],
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            &osv_response,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": ADVISORY_RESPONSE_OSV_SCHEMA,
                "package_coordinate": "pkg:generic/hello",
                "version": "v1.3.0",
                "findings": [],
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            &repology_response,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": ADVISORY_RESPONSE_REPOLOGY_SCHEMA,
                "package_coordinate": "hello",
                "version": "v1.3.0",
                "findings": [],
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(&validation_path, serde_json::to_vec_pretty(&update_validation_fixture()).unwrap()).unwrap();
        (policy_path, source_response, osv_response, repology_response, validation_path)
    }

    fn write_update_source_tree(root: &Path) -> PathBuf {
        let source_root = root.join("source");
        fs::create_dir_all(source_root.join("locks")).unwrap();
        fs::write(
            source_root.join("locks/hello.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "version": "v1.2.3",
                "source": {
                    "ref": "refs/tags/v1.2.3",
                    "identity_blake3": DIGEST,
                },
            }))
            .unwrap(),
        )
        .unwrap();
        source_root
    }

    fn build_update_cli_plan(root: &Path) -> (PathBuf, PathBuf) {
        let (policy, source_response, osv_response, repology_response, validation) = write_update_fixture_inputs(root);
        let source_observation = root.join("source-observation.json");
        let osv_observation = root.join("osv-observation.json");
        let repology_observation = root.join("repology-observation.json");
        run_update_source_observe(UpdateObserveRequest {
            policy_path: &policy,
            response_path: Some(&source_response),
            url: None,
            requested_status: "success",
            reasons: &[],
            output: &source_observation,
            is_json: false,
        })
        .unwrap();
        run_update_advisory_observe(UpdateAdvisoryObserveRequest {
            policy_path: &policy,
            service: "osv",
            version: "v1.3.0",
            response_path: Some(&osv_response),
            url: None,
            requested_status: "success",
            reasons: &[],
            output: &osv_observation,
            is_json: false,
        })
        .unwrap();
        run_update_advisory_observe(UpdateAdvisoryObserveRequest {
            policy_path: &policy,
            service: "repology",
            version: "v1.3.0",
            response_path: Some(&repology_response),
            url: None,
            requested_status: "success",
            reasons: &[],
            output: &repology_observation,
            is_json: false,
        })
        .unwrap();
        let source_root = write_update_source_tree(root);
        let plan = root.join("update-plan.json");
        run_update_plan(UpdatePlanRequest {
            policy_path: &policy,
            source_observation_path: &source_observation,
            advisory_observation_paths: &[osv_observation, repology_observation],
            validation_evidence_path: &validation,
            source_root: &source_root,
            output: &plan,
            is_json: false,
        })
        .unwrap();
        (source_root, plan)
    }

    fn impact_policy_fixture() -> ImpactComparisonPolicy {
        mantlepkgs_core::seal_impact_policy(&ImpactComparisonPolicy {
            schema: mantlepkgs_core::IMPACT_POLICY_SCHEMA.into(),
            policy_identity_blake3: String::new(),
            max_packages: IMPACT_FIXTURE_ITEM_LIMIT,
            max_variants: IMPACT_FIXTURE_ITEM_LIMIT,
            max_closure_members: IMPACT_FIXTURE_ITEM_LIMIT,
            max_dependency_edges: IMPACT_FIXTURE_ITEM_LIMIT,
            max_observations: IMPACT_FIXTURE_ITEM_LIMIT,
            max_diagnostics: IMPACT_FIXTURE_DIAGNOSTIC_LIMIT,
            max_report_bytes: IMPACT_FIXTURE_REPORT_BYTES,
        })
        .unwrap()
    }

    fn empty_impact_snapshot(policy: &ImpactComparisonPolicy) -> ImpactSnapshot {
        ImpactSnapshot {
            schema: mantlepkgs_core::IMPACT_SNAPSHOT_SCHEMA.into(),
            snapshot_identity_blake3: String::new(),
            comparison_policy_identity_blake3: policy.policy_identity_blake3.clone(),
            catalog_schema: mantlepkgs_core::DOMAIN_CATALOG_SCHEMA.into(),
            catalog_identity_blake3: DIGEST.into(),
            system: "x86_64-linux".into(),
            store_prefix: "/mantle/store".into(),
            conversion_policy_identity_blake3: DIGEST.into(),
            package_record_schema: mantlepkgs_core::IMPACT_PACKAGE_RECORD_SCHEMA.into(),
            observation_schema: mantlepkgs_core::IMPACT_OBSERVATION_SCHEMA.into(),
            identity_domain: mantlepkgs_core::IMPACT_IDENTITY_DOMAIN.into(),
            packages: Vec::new(),
            observations: Vec::new(),
            closures: Vec::new(),
        }
    }

    fn admitted_impact_runtime_report() -> crunch_build::ActionResultRuntimeReport {
        crunch_build::ActionResultRuntimeReport {
            schema: crunch_build::action_result::ACTION_RESULT_RUNTIME_REPORT_SCHEMA.into(),
            phase: crunch_build::action_result::ACTION_RESULT_PHASE_DISCOVERY.into(),
            action_ref: IMPACT_ACTION_REF.into(),
            disposition: crunch_build::action_result::ACTION_RESULT_DISPOSITION_REUSED.into(),
            selected_result_ref: Some(IMPACT_RESULT_REF.into()),
            selected_source_id: Some("local-action-results".into()),
            selected_source_class: Some("local".into()),
            trust_basis: vec!["record-signature-verified:fixture".into()],
            conflict_class: None,
            candidate_decisions: vec![crunch_action_result_core::CandidateDecision {
                result_ref: IMPACT_RESULT_REF.into(),
                source_id: "local-action-results".into(),
                source_class: "local".into(),
                admitted: true,
                diagnostics: Vec::new(),
                trust_basis: vec!["record-signature-verified:fixture".into()],
                output_set_digest_blake3: Some(DIGEST.into()),
            }],
            publication_result_refs: Vec::new(),
            transfer: None,
            diagnostics: Vec::new(),
            non_claims: vec!["index-presence-is-not-output-trust".into()],
        }
    }

    #[test]
    fn impact_action_result_adapter_accepts_only_current_admission() {
        let report = admitted_impact_runtime_report();
        validate_admitted_runtime_report(&report, IMPACT_ACTION_REF, IMPACT_RESULT_REF).unwrap();
        let mut rejected = report;
        rejected.candidate_decisions[0].admitted = false;

        let error = validate_admitted_runtime_report(&rejected, IMPACT_ACTION_REF, IMPACT_RESULT_REF)
            .expect_err("a rejected action result must fail");

        assert!(error.message().contains("impact-action-result-not-admitted"));
    }

    #[test]
    fn impact_command_writes_a_local_atomic_report() {
        let temp = tempfile::tempdir().unwrap();
        let policy = impact_policy_fixture();
        let snapshot = empty_impact_snapshot(&policy);
        let policy_path = temp.path().join("policy.json");
        let base_path = temp.path().join("base.json");
        let head_path = temp.path().join("head.json");
        let output = temp.path().join("impact.json");
        fs::write(&policy_path, serde_json::to_vec(&policy).unwrap()).unwrap();
        fs::write(&base_path, serde_json::to_vec(&snapshot).unwrap()).unwrap();
        fs::write(&head_path, serde_json::to_vec(&snapshot).unwrap()).unwrap();

        run_impact(ImpactRequest {
            policy_path: &policy_path,
            base_path: &base_path,
            head_path: &head_path,
            action_result_report_paths: &[],
            output: &output,
            is_json: false,
        })
        .unwrap();

        let report =
            read_json_bounded::<mantlepkgs_core::MantlePackageImpactReport>(&output, IMPACT_INPUT_BYTES_MAX).unwrap();
        assert_eq!(report.schema, mantlepkgs_core::IMPACT_REPORT_SCHEMA);
        assert!(report.package_impacts.is_empty());
        assert!(Command::new("nix").env("PATH", temp.path().join("no-nix")).status().is_err());
    }

    #[test]
    fn impact_command_rejects_a_stale_snapshot_before_publication() {
        let temp = tempfile::tempdir().unwrap();
        let policy = impact_policy_fixture();
        let snapshot = empty_impact_snapshot(&policy);
        let mut stale = mantlepkgs_core::seal_impact_snapshot(&policy, &snapshot).unwrap();
        stale.catalog_identity_blake3 = OTHER_DIGEST.into();
        let policy_path = temp.path().join("policy.json");
        let base_path = temp.path().join("base.json");
        let head_path = temp.path().join("head.json");
        let output = temp.path().join("impact.json");
        fs::write(&policy_path, serde_json::to_vec(&policy).unwrap()).unwrap();
        fs::write(&base_path, serde_json::to_vec(&stale).unwrap()).unwrap();
        fs::write(&head_path, serde_json::to_vec(&snapshot).unwrap()).unwrap();

        let error = run_impact(ImpactRequest {
            policy_path: &policy_path,
            base_path: &base_path,
            head_path: &head_path,
            action_result_report_paths: &[],
            output: &output,
            is_json: false,
        })
        .expect_err("a stale snapshot identity must fail");

        assert!(error.message().contains("impact-identity-mismatch"));
        assert!(!output.exists());
    }

    #[test]
    fn update_commands_replay_saved_evidence_and_publish_one_immutable_tree() {
        let temp = tempfile::tempdir().unwrap();
        let (source_root, plan_path) = build_update_cli_plan(temp.path());
        let output_root = temp.path().join("published-update");
        let denial = temp.path().join("denial.json");

        run_update_execute(UpdateExecuteRequest {
            plan_path: &plan_path,
            source_root: &source_root,
            output_root: &output_root,
            denial_receipt_output: &denial,
            is_json: false,
        })
        .unwrap();

        let updated =
            read_json_bounded::<serde_json::Value>(&output_root.join("locks/hello.json"), UPDATE_DOCUMENT_BYTES_MAX)
                .unwrap();
        let original =
            read_json_bounded::<serde_json::Value>(&source_root.join("locks/hello.json"), UPDATE_DOCUMENT_BYTES_MAX)
                .unwrap();
        let receipt = read_json_bounded::<mantlepkgs_core::UpdateExecutionReceipt>(
            &output_root.join(UPDATE_EXECUTION_RECEIPT_FILE),
            UPDATE_INPUT_BYTES_MAX,
        )
        .unwrap();
        assert_eq!(updated["version"], "v1.3.0");
        assert_eq!(updated["source"]["identity_blake3"], OTHER_DIGEST);
        assert_eq!(original["version"], "v1.2.3");
        assert_eq!(receipt.disposition, UpdateExecutionDisposition::Applied);
        assert!(!denial.exists());
        assert!(Command::new("nix").env("PATH", temp.path().join("no-nix")).status().is_err());
    }

    #[test]
    fn http_response_classification_preserves_failure_status() {
        let body = b"bounded-response".to_vec();
        let success = classify_update_http_response(UPDATE_FIXTURE_HTTP_SUCCESS_STATUS, body.clone(), 0);
        let redirect = classify_update_http_response(UPDATE_FIXTURE_HTTP_REDIRECT_STATUS, body.clone(), 0);
        let error = classify_update_http_response(UPDATE_FIXTURE_HTTP_ERROR_STATUS, body, 0);
        let transport = failed_acquisition(ObservationStatus::Unavailable, "transport-unavailable", 0);

        assert_eq!(success.status, ObservationStatus::Success);
        assert_eq!(redirect.status, ObservationStatus::Failed);
        assert_eq!(redirect.reason_codes, vec!["redirect-forbidden"]);
        assert_eq!(error.status, ObservationStatus::Failed);
        assert_eq!(error.reason_codes, vec!["http-status-invalid"]);
        assert!(error.response_identity_blake3.is_some());
        assert_eq!(transport.status, ObservationStatus::Unavailable);
        assert_eq!(transport.reason_codes, vec!["transport-unavailable"]);
    }

    #[test]
    fn update_policy_seal_writes_one_canonical_policy() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("policy-unsealed.json");
        let output = temp.path().join("policy-sealed.json");
        let mut policy = update_policy_fixture();
        policy.policy_identity_blake3.clear();
        fs::write(&input, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();

        run_update_policy_seal(&input, &output, false).unwrap();

        let sealed = read_json_bounded::<UpdatePolicy>(&output, UPDATE_INPUT_BYTES_MAX).unwrap();
        assert_eq!(sealed, seal_update_policy(&policy).unwrap());
        let error =
            run_update_policy_seal(&input, &input, false).expect_err("policy sealing must not overwrite its input");
        assert!(error.message().contains("must differ"));
    }

    #[test]
    fn release_index_saved_response_replays_without_network() {
        let temp = tempfile::tempdir().unwrap();
        let mut policy = update_policy_fixture();
        policy.policy_identity_blake3.clear();
        policy.source_kind = mantlepkgs_core::UpdateSourceKind::ReleaseIndex;
        policy.adapter_identity = "mantle-release-index-v1".into();
        policy.source_query = "/releases/hello.json".into();
        policy.response_schema = mantlepkgs_core::SOURCE_RESPONSE_RELEASE_INDEX_SCHEMA.into();
        let policy = seal_update_policy(&policy).unwrap();
        let policy_path = temp.path().join("policy.json");
        let response_path = temp.path().join("release-index.json");
        let output = temp.path().join("source-observation.json");
        fs::write(&policy_path, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
        fs::write(
            &response_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": mantlepkgs_core::SOURCE_RESPONSE_RELEASE_INDEX_SCHEMA,
                "candidates": [{
                    "version": "v1.3.0",
                    "source_ref": "release/hello/v1.3.0",
                    "source_identity_blake3": OTHER_DIGEST,
                }],
            }))
            .unwrap(),
        )
        .unwrap();

        run_update_source_observe(UpdateObserveRequest {
            policy_path: &policy_path,
            response_path: Some(&response_path),
            url: None,
            requested_status: "success",
            reasons: &[],
            output: &output,
            is_json: false,
        })
        .unwrap();

        let observation = read_json_bounded::<SourceObservation>(&output, UPDATE_INPUT_BYTES_MAX).unwrap();
        assert_eq!(observation.status, ObservationStatus::Success);
        assert_eq!(observation.candidates.len(), 1);
        assert_eq!(observation.candidates[0].source_ref, "release/hello/v1.3.0");
    }

    #[test]
    fn malformed_saved_source_response_becomes_explicit_failed_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let policy_path = temp.path().join("policy.json");
        let response_path = temp.path().join("malformed.json");
        let output = temp.path().join("source-observation.json");
        fs::write(&policy_path, serde_json::to_vec_pretty(&update_policy_fixture()).unwrap()).unwrap();
        fs::write(&response_path, b"{\"schema\":true}").unwrap();

        run_update_source_observe(UpdateObserveRequest {
            policy_path: &policy_path,
            response_path: Some(&response_path),
            url: None,
            requested_status: "success",
            reasons: &[],
            output: &output,
            is_json: false,
        })
        .unwrap();

        let observation = read_json_bounded::<SourceObservation>(&output, UPDATE_INPUT_BYTES_MAX).unwrap();
        assert_eq!(observation.status, ObservationStatus::Failed);
        assert_eq!(observation.reason_codes, vec!["response-schema-invalid"]);
        assert!(observation.candidates.is_empty());
        assert!(observation.response_identity_blake3.is_some());
    }

    #[test]
    fn oversized_saved_source_response_becomes_explicit_failed_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let mut policy = update_policy_fixture();
        policy.policy_identity_blake3.clear();
        policy.limits.max_response_bytes = UPDATE_TINY_RESPONSE_BYTES;
        let policy = seal_update_policy(&policy).unwrap();
        let policy_path = temp.path().join("policy.json");
        let response_path = temp.path().join("oversized.json");
        let output = temp.path().join("source-observation.json");
        fs::write(&policy_path, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
        fs::write(&response_path, vec![b'x'; UPDATE_OVERSIZED_RESPONSE_BYTES]).unwrap();

        run_update_source_observe(UpdateObserveRequest {
            policy_path: &policy_path,
            response_path: Some(&response_path),
            url: None,
            requested_status: "success",
            reasons: &[],
            output: &output,
            is_json: false,
        })
        .unwrap();

        let observation = read_json_bounded::<SourceObservation>(&output, UPDATE_INPUT_BYTES_MAX).unwrap();
        assert_eq!(observation.status, ObservationStatus::Failed);
        assert_eq!(observation.reason_codes, vec!["saved-response-read-failed"]);
        assert!(observation.candidates.is_empty());
        assert!(observation.response_identity_blake3.is_none());
    }

    #[test]
    fn advisory_coordinate_mismatch_becomes_explicit_failed_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let (policy_path, _, osv_response, _, _) = write_update_fixture_inputs(temp.path());
        let output = temp.path().join("osv-observation.json");
        fs::write(
            &osv_response,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": ADVISORY_RESPONSE_OSV_SCHEMA,
                "package_coordinate": "pkg:generic/not-hello",
                "version": "v1.3.0",
                "findings": [],
            }))
            .unwrap(),
        )
        .unwrap();

        run_update_advisory_observe(UpdateAdvisoryObserveRequest {
            policy_path: &policy_path,
            service: "osv",
            version: "v1.3.0",
            response_path: Some(&osv_response),
            url: None,
            requested_status: "success",
            reasons: &[],
            output: &output,
            is_json: false,
        })
        .unwrap();

        let observation = read_json_bounded::<AdvisoryObservation>(&output, UPDATE_INPUT_BYTES_MAX).unwrap();
        assert_eq!(observation.status, ObservationStatus::Failed);
        assert_eq!(observation.reason_codes, vec!["advisory-package-coordinate-mismatch"]);
        assert!(observation.findings.is_empty());
        assert!(observation.response_identity_blake3.is_some());
    }

    #[test]
    fn duplicate_advisory_findings_become_explicit_failed_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let (policy_path, _, osv_response, _, _) = write_update_fixture_inputs(temp.path());
        let output = temp.path().join("osv-observation.json");
        let finding = serde_json::json!({
            "finding_id": "OSV-DUPLICATE",
            "finding_identity_blake3": DIGEST,
        });
        fs::write(
            &osv_response,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": ADVISORY_RESPONSE_OSV_SCHEMA,
                "package_coordinate": "pkg:generic/hello",
                "version": "v1.3.0",
                "findings": [finding.clone(), finding],
            }))
            .unwrap(),
        )
        .unwrap();

        run_update_advisory_observe(UpdateAdvisoryObserveRequest {
            policy_path: &policy_path,
            service: "osv",
            version: "v1.3.0",
            response_path: Some(&osv_response),
            url: None,
            requested_status: "success",
            reasons: &[],
            output: &output,
            is_json: false,
        })
        .unwrap();

        let observation = read_json_bounded::<AdvisoryObservation>(&output, UPDATE_INPUT_BYTES_MAX).unwrap();
        assert_eq!(observation.status, ObservationStatus::Failed);
        assert_eq!(observation.reason_codes, vec!["advisory-finding-duplicate"]);
        assert!(observation.findings.is_empty());
    }

    #[test]
    fn explicit_unavailable_source_observation_stays_unavailable() {
        let temp = tempfile::tempdir().unwrap();
        let policy_path = temp.path().join("policy.json");
        let output = temp.path().join("source-unavailable.json");
        fs::write(&policy_path, serde_json::to_vec_pretty(&update_policy_fixture()).unwrap()).unwrap();

        run_update_source_observe(UpdateObserveRequest {
            policy_path: &policy_path,
            response_path: None,
            url: None,
            requested_status: "unavailable",
            reasons: &["source-timeout".into()],
            output: &output,
            is_json: false,
        })
        .unwrap();

        let observation = read_json_bounded::<SourceObservation>(&output, UPDATE_INPUT_BYTES_MAX).unwrap();
        assert_eq!(observation.status, ObservationStatus::Unavailable);
        assert!(observation.candidates.is_empty());
        assert_eq!(observation.reason_codes, vec!["source-timeout"]);
    }

    #[test]
    fn stale_update_preimage_leaves_source_unchanged_and_writes_denial() {
        let temp = tempfile::tempdir().unwrap();
        let (source_root, plan_path) = build_update_cli_plan(temp.path());
        let source_file = source_root.join("locks/hello.json");
        let changed = serde_json::json!({
            "version": "v1.2.99",
            "source": {
                "ref": "refs/tags/v1.2.99",
                "identity_blake3": DIGEST,
            },
        });
        fs::write(&source_file, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
        let before = fs::read(&source_file).unwrap();
        let output_root = temp.path().join("rejected-update");
        let denial = temp.path().join("stale-denial.json");

        let error = run_update_execute(UpdateExecuteRequest {
            plan_path: &plan_path,
            source_root: &source_root,
            output_root: &output_root,
            denial_receipt_output: &denial,
            is_json: false,
        })
        .expect_err("a stale preimage must fail");

        let receipt =
            read_json_bounded::<mantlepkgs_core::UpdateExecutionReceipt>(&denial, UPDATE_INPUT_BYTES_MAX).unwrap();
        assert!(error.message().contains("update-stale-preimage"));
        assert_eq!(fs::read(&source_file).unwrap(), before);
        assert!(!output_root.exists());
        assert!(!update_stage_path(&output_root).unwrap().exists());
        assert_eq!(receipt.disposition, UpdateExecutionDisposition::Denied);
        assert!(receipt.applied_outputs.is_empty());
        assert_eq!(receipt.reason_codes, vec!["update-execution-denied", "update-stale-preimage"]);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_update_source_is_denied_without_publication() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let (source_root, plan_path) = build_update_cli_plan(temp.path());
        let source_file = source_root.join("locks/hello.json");
        let replacement = temp.path().join("replacement.json");
        fs::rename(&source_file, &replacement).unwrap();
        symlink(&replacement, &source_file).unwrap();
        let output_root = temp.path().join("symlink-update");
        let denial = temp.path().join("symlink-denial.json");

        let error = run_update_execute(UpdateExecuteRequest {
            plan_path: &plan_path,
            source_root: &source_root,
            output_root: &output_root,
            denial_receipt_output: &denial,
            is_json: false,
        })
        .expect_err("a symlinked source must fail");

        assert!(error.message().contains("update-symlink-forbidden"));
        assert!(!output_root.exists());
        assert!(!update_stage_path(&output_root).unwrap().exists());
        assert!(denial.is_file());
        assert_eq!(fs::read_link(&source_file).unwrap(), replacement);
    }

    #[test]
    fn output_digest_tamper_is_denied_before_publication() {
        let temp = tempfile::tempdir().unwrap();
        let (source_root, plan_path) = build_update_cli_plan(temp.path());
        let mut plan = read_json_bounded::<UpdatePlan>(&plan_path, UPDATE_INPUT_BYTES_MAX).unwrap();
        plan.effects[0].output_digest_blake3 = DIGEST.into();
        plan.plan_identity_blake3.clear();
        plan.plan_identity_blake3 = update_plan_identity_blake3(&plan).unwrap();
        fs::write(&plan_path, serde_json::to_vec_pretty(&plan).unwrap()).unwrap();
        let output_root = temp.path().join("digest-update");
        let denial = temp.path().join("digest-denial.json");

        let error = run_update_execute(UpdateExecuteRequest {
            plan_path: &plan_path,
            source_root: &source_root,
            output_root: &output_root,
            denial_receipt_output: &denial,
            is_json: false,
        })
        .expect_err("an output digest mismatch must fail");

        assert!(error.message().contains("update-output-digest-mismatch"));
        assert!(!output_root.exists());
        assert!(!update_stage_path(&output_root).unwrap().exists());
        assert!(denial.is_file());
    }

    #[test]
    fn execution_outputs_and_denials_cannot_overlap_the_source_tree() {
        let source_root = Path::new("source");
        let plan = Path::new("plan.json");
        let output_inside_source = Path::new("source/published");
        let denial_outside_source = Path::new("denial.json");
        let output_outside_source = Path::new("published");
        let denial_inside_source = Path::new("source/denial.json");

        let output_error = reject_update_execution_path_collisions(&UpdateExecuteRequest {
            plan_path: plan,
            source_root,
            output_root: output_inside_source,
            denial_receipt_output: denial_outside_source,
            is_json: false,
        })
        .expect_err("an output inside the source tree must fail");
        let denial_error = reject_update_execution_path_collisions(&UpdateExecuteRequest {
            plan_path: plan,
            source_root,
            output_root: output_outside_source,
            denial_receipt_output: denial_inside_source,
            is_json: false,
        })
        .expect_err("a denial receipt inside the source tree must fail");

        assert!(output_error.message().contains("paths conflict"));
        assert!(denial_error.message().contains("paths conflict"));
    }

    #[test]
    fn publication_conflict_preserves_existing_output_and_records_denial() {
        let temp = tempfile::tempdir().unwrap();
        let (source_root, plan_path) = build_update_cli_plan(temp.path());
        let output_root = temp.path().join("existing-update");
        fs::create_dir(&output_root).unwrap();
        fs::write(output_root.join("sentinel"), b"existing").unwrap();
        let denial = temp.path().join("conflict-denial.json");

        run_update_execute(UpdateExecuteRequest {
            plan_path: &plan_path,
            source_root: &source_root,
            output_root: &output_root,
            denial_receipt_output: &denial,
            is_json: false,
        })
        .expect_err("an existing output root must fail");

        assert_eq!(fs::read(output_root.join("sentinel")).unwrap(), b"existing");
        assert!(!update_stage_path(&output_root).unwrap().exists());
        assert!(denial.is_file());
        assert!(!output_root.join(UPDATE_EXECUTION_RECEIPT_FILE).exists());
    }

    #[test]
    fn domain_manifest_composes_without_nix() {
        let temp = tempfile::tempdir().unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture = root.join("mantlepkgs/domains/fixtures/valid-domain.ncl");
        let sealed = temp.path().join("sealed.json");
        let catalog = temp.path().join("catalog.json");

        run_domain_compose(&fixture, &sealed, &catalog, false).unwrap();

        let composed = read_json_bounded::<DomainCatalog>(&catalog, DOMAIN_ARTIFACT_BYTES_MAX).unwrap();
        assert_eq!(composed.packages.len(), COMPOSED_PUBLIC_PACKAGE_COUNT);
        assert_eq!(composed.variants.len(), 1);
        assert_eq!(composed.validation_roots.len(), 1);
        assert!(Command::new("nix").env("PATH", temp.path().join("no-nix")).status().is_err());
    }

    #[test]
    fn domain_contract_rejects_unknown_class() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture = root.join("mantlepkgs/domains/fixtures/invalid-domain-class.ncl");

        let error = evaluate_domain_manifest(&fixture).expect_err("unknown domain class must fail");

        assert!(error.message().contains("contract"));
    }

    #[test]
    fn domain_contract_rejects_missing_fields_and_nonpositive_limits() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixtures = ["invalid-missing-owner.ncl", "invalid-limit.ncl"];

        for fixture_name in fixtures {
            let fixture = root.join("mantlepkgs/domains/fixtures").join(fixture_name);
            let error = evaluate_domain_manifest(&fixture).expect_err("invalid typed domain fixture must fail");
            assert!(error.message().contains("contract"), "fixture {fixture_name}: {}", error.message());
        }
    }

    #[test]
    fn domain_composition_rejects_duplicate_shard_identity() {
        let temp = tempfile::tempdir().unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture = root.join("mantlepkgs/domains/fixtures/invalid-duplicate-shard-identity.ncl");
        let sealed = temp.path().join("sealed.json");
        let catalog = temp.path().join("catalog.json");

        let error =
            run_domain_compose(&fixture, &sealed, &catalog, false).expect_err("duplicate shard identity must fail");

        assert!(error.message().contains("duplicate-shard-identity"));
        assert!(!sealed.exists());
        assert!(!catalog.exists());
    }

    #[test]
    fn corpus_verification_rejects_artifact_tampering() {
        let temp = tempfile::tempdir().unwrap();
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("mantlepkgs/corepkgs-corpus");
        let evidence_path = fixture_root.join("evidence.json");
        let evidence = read_json_bounded::<ExternalCorpusEvidence>(&evidence_path, DOMAIN_ARTIFACT_BYTES_MAX).unwrap();
        let artifact_root = temp.path().join("artifacts");
        fs::create_dir(&artifact_root).unwrap();
        for artifact in &evidence.artifacts {
            fs::copy(fixture_root.join("evidence").join(&artifact.path), artifact_root.join(&artifact.path)).unwrap();
        }
        let sealed_path = temp.path().join("sealed-evidence.json");
        run_corpus_verify(&evidence_path, &artifact_root, &sealed_path, false).unwrap();
        fs::write(artifact_root.join("catalog.json"), b"tampered").unwrap();

        let error = run_corpus_verify(&evidence_path, &artifact_root, &sealed_path, false)
            .expect_err("artifact tampering must fail");

        assert!(error.message().contains("corpus-artifact-mismatch"));
    }

    #[test]
    fn validation_output_paths_must_be_distinct() {
        let path = Path::new("same.json");
        let error = reject_output_path_collisions(&[path, path]).expect_err("duplicate outputs must fail");
        assert!(error.message().contains("must be distinct"));
    }

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
        fs::write(generation.join(SHARED_GRAPH_PATH), b"tampered graph").unwrap();
        let tamper = match verify_generation(&generation) {
            Ok(_) => panic!("artifact tampering must fail verification"),
            Err(error) => error,
        };
        assert!(tamper.message().contains("catalog-artifact-mismatch"));
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
    fn fixed_output_without_url_becomes_a_bound_seed_candidate() {
        let (graph, _) = crate::foreign_derivation_import::nix_like_hello_fixture();
        let candidates = fixed_output_seed_candidates(&graph).unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].output_path.starts_with("/nix/store/"));
        assert!(candidates[0].payload_id.starts_with("nix-fixed-output-seed:"));
    }

    #[test]
    fn fixed_output_with_a_url_becomes_a_bound_seed_candidate() {
        let (mut graph, _) = crate::foreign_derivation_import::nix_like_hello_fixture();
        let fixed = graph.nodes.iter_mut().find(|node| node.fixed_output.is_some()).unwrap();
        fixed.env.insert("url".into(), "https://example.invalid/source".into());

        let candidates = fixed_output_seed_candidates(&graph).unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].output_path.starts_with("/nix/store/"));
    }

    #[test]
    fn fixed_output_seed_realization_creates_a_durable_out_link() {
        let (graph, _) = crate::foreign_derivation_import::nix_like_hello_fixture();
        let candidates = fixed_output_seed_candidates(&graph).unwrap();
        let seed_retention_link = Path::new("/tmp/mantlepkgs-seed-root");

        let args = fixed_output_seed_realization_args(&candidates, seed_retention_link).unwrap();

        assert!(args.windows(2).any(|pair| pair == ["--out-link", "/tmp/mantlepkgs-seed-root"]));
        assert!(!args.iter().any(|arg| arg == "--no-link"));
        assert!(args.iter().any(|arg| arg.ends_with("^out")));
    }

    #[cfg(unix)]
    #[test]
    fn fixed_output_seed_realization_rejects_non_utf8_out_link() {
        let (graph, _) = crate::foreign_derivation_import::nix_like_hello_fixture();
        let candidates = fixed_output_seed_candidates(&graph).unwrap();
        let invalid_link = PathBuf::from(OsString::from_vec(vec![NON_UTF8_PATH_BYTE]));

        let error = fixed_output_seed_realization_args(&candidates, &invalid_link).unwrap_err();

        assert!(error.contains("retention link is not valid UTF-8"));
    }

    #[test]
    fn malformed_fixed_output_seed_candidate_is_rejected() {
        let (mut graph, _) = crate::foreign_derivation_import::nix_like_hello_fixture();
        let fixed = graph.nodes.iter_mut().find(|node| node.fixed_output.is_some()).unwrap();
        let extra = fixed.outputs.first_key_value().unwrap().1.clone();
        fixed.outputs.insert("debug".into(), extra);
        let error = fixed_output_seed_candidates(&graph).unwrap_err();
        assert!(error.contains("must have one output"));
        assert!(error.contains("fixed-output derivation"));
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
