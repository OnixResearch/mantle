use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use clap::Subcommand;
use crunch_build::ExecutionProfile;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::RunError;
use crate::foreign_derivation_import::AtermDerivationInput;
use crate::foreign_derivation_import::AtermProducerConfig;
use crate::foreign_derivation_import::CacheHint;
use crate::foreign_derivation_import::ForeignDerivationGraph;
use crate::foreign_derivation_import::ImportDiagnostic;
use crate::foreign_derivation_import::ImportReceipt;
use crate::foreign_derivation_import::MAX_ATERM_BUNDLE_BYTES;
use crate::foreign_derivation_import::MAX_ATERM_BUNDLE_DERIVATIONS;
use crate::foreign_derivation_import::MAX_ATERM_DERIVATION_BYTES;
use crate::foreign_derivation_import::NixDerivationJsonExport;
use crate::foreign_derivation_import::NixProducerConfig;
use crate::foreign_derivation_import::PackageIndex;
use crate::foreign_derivation_import::TranslationPolicy;
use crate::foreign_derivation_import::admit_translated_graph;
use crate::foreign_derivation_import::foreign_import_non_claims;
use crate::foreign_derivation_import::lower_nix_derivation_json_closure;
use crate::foreign_derivation_import::lower_prefix_aware_aterm_closure;
use crate::foreign_derivation_import::normalize_nix_aterm_derivation_closure;
use crate::foreign_derivation_import::normalize_nix_derivation_json_export;
use crate::foreign_derivation_import::parse_prefix_aware_aterm_bundle;
use crate::foreign_derivation_import::select_nix_derivation_json_closure;
use crate::foreign_derivation_import::translate_foreign_graph;
use crate::foreign_executable_plan::ForeignExecutablePlan;
use crate::foreign_executable_plan::compile_foreign_executable_plan;
use crate::foreign_executable_plan::compile_foreign_executable_plan_with_profile;
use crate::foreign_provenance_audit::ForeignProvenanceAuditReceipt;
use crate::foreign_provenance_audit::ForeignProvenanceAuditRequest;
use crate::foreign_provenance_audit::audit_foreign_realization;
use crate::foreign_realization::ForeignCacheClosurePolicy;
use crate::foreign_realization::ForeignRealizationAdmission;
use crate::foreign_realization::ForeignSourceAdmission;
use crate::foreign_realization::is_cache_only_foreign_plan;
use crate::foreign_realization::validate_cache_only_source_bundle;
use crate::foreign_realization::validate_foreign_cache_closure_policy;
use crate::foreign_realization::validate_foreign_realization_admission;
use crate::foreign_realization::validate_foreign_source_admission;
use crate::foreign_realization_shell::ForeignRealizationRequest;
use crate::foreign_realization_shell::realize_foreign_plan;
use crate::nix_producer::NIX_PRODUCER_CONTRACT_SCHEMA;
use crate::nix_producer::NixProducerRequest as BackendContractRequest;
use crate::nix_producer::ProducerBackendKind;
use crate::nix_producer::ProducerBudget;
use crate::nix_producer::ProducerTarget;
use crate::nix_producer_shell::BackendRunConfig;
use crate::nix_producer_shell::run_backend;
use crate::source_bundle::ForeignSourcePathBinding;
use crate::source_bundle::SourceBundleManifest;
use crate::source_bundle::plan_bound_foreign_source_bundle;
use crate::source_bundle::plan_empty_source_bundle;
use crate::source_bundle::write_json_atomically;

const CLI_REPORT_SCHEMA: &str = "mantle-foreign-import-cli-v1";
const VALIDATE_COMMAND: &str = "validate";
const PLAN_COMMAND: &str = "plan";
const REALIZE_COMMAND: &str = "realize";
const AUDIT_COMMAND: &str = "audit";
const PREPARE_SOURCES_COMMAND: &str = "prepare-sources";
const PRODUCE_NIX_COMMAND: &str = "produce-nix";
const PRODUCE_ATERM_COMMAND: &str = "produce-aterm";
const PRODUCE_BACKEND_COMMAND: &str = "produce-backend";
const ACCEPTED_VERDICT: &str = "accepted";
const REJECTED_VERDICT: &str = "rejected";
const FAILURE_EXIT_CODE: u8 = 1;
const JSON_SERIALIZATION_CONTEXT: &str = "serializing foreign import CLI report";
const JSON_ARTIFACT_SERIALIZATION_CONTEXT: &str = "serializing foreign import producer artifact";
const DEFAULT_PACKAGE_NAME: &str = "hello";
const DEFAULT_SYSTEM: &str = "x86_64-linux";
const DEFAULT_PRODUCER_REVISION: &str = "unknown";
const DEFAULT_CACHE_TRUST_SCOPE: &str = "trusted-binary-cache";
const NIXPKGS_GRAPH_FILE: &str = "nixpkgs.graph.json";
const NIXPKGS_INDEX_FILE: &str = "nixpkgs.index.json";
const ATERM_GRAPH_FILE: &str = "foreign-aterm.graph.json";
const ATERM_INDEX_FILE: &str = "foreign-aterm.index.json";
const DRV_SPEC_SEPARATOR: char = '=';
const SOURCE_BINDING_SEPARATOR: char = '=';
const DRV_FILE_EXTENSION: &str = "drv";
const NIX_LOGICAL_STORE_PREFIX: &str = "/nix/store";
const NIX_LOGICAL_STORE_PREFIX_ROOT: &str = "/";
pub(crate) const DEFAULT_FOREIGN_JSON_ARTIFACT_BYTES_MAX: u64 = 268_435_456;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ForeignImportAction {
    /// Validate lowered foreign derivation graph, package-index, policy, and optional receipt JSON
    Validate {
        /// Path to `foreign-derivation-graph-v1` JSON
        #[arg(long)]
        graph: PathBuf,

        /// Path to `foreign-package-index-v1` JSON
        #[arg(long = "package-index")]
        package_index: Option<PathBuf>,

        /// Path to translation policy JSON
        #[arg(long)]
        policy: PathBuf,

        /// Optional existing `foreign-derivation-import-receipt-v1` JSON to admit against the
        /// inputs
        #[arg(long)]
        receipt: Option<PathBuf>,
    },

    /// Emit a receipt-bound Mantle executable plan from lowered foreign import artifacts
    Plan {
        /// Path to `foreign-derivation-graph-v1` JSON
        #[arg(long)]
        graph: PathBuf,

        /// Path to `foreign-package-index-v1` JSON
        #[arg(long = "package-index")]
        package_index: PathBuf,

        /// Path to translation policy JSON
        #[arg(long)]
        policy: PathBuf,

        /// Package name or alias to select from the package index
        #[arg(long, default_value = DEFAULT_PACKAGE_NAME)]
        package: String,

        /// System to select from the package index
        #[arg(long, default_value = DEFAULT_SYSTEM)]
        system: String,

        /// Optional exported `mantle-foreign-execution-profile-v1` JSON
        #[arg(long = "execution-profile")]
        execution_profile: Option<PathBuf>,

        /// Write the accepted executable plan atomically to this path
        #[arg(long = "plan-out")]
        plan_out: Option<PathBuf>,

        /// Write the accepted import receipt atomically to this path
        #[arg(long = "receipt-out")]
        receipt_out: Option<PathBuf>,
    },

    /// Bind local source payloads into an admitted foreign source bundle
    PrepareSources {
        /// Path to `mantle-foreign-executable-plan-v1` JSON
        #[arg(long)]
        plan: PathBuf,

        /// Source payload mapping: `payload-id=/path/to/payload`
        #[arg(long = "source")]
        sources: Vec<String>,

        /// Write the source-bundle manifest atomically to this path
        #[arg(long)]
        out: PathBuf,
    },

    /// Realize an admitted executable plan through Mantle's scheduler and store
    Realize {
        /// Path to `mantle-foreign-executable-plan-v1` JSON
        #[arg(long)]
        plan: PathBuf,

        /// Path to the exact admitted `foreign-derivation-import-receipt-v1` JSON
        #[arg(long = "import-receipt")]
        import_receipt: PathBuf,

        /// Path to an admitted source-bundle manifest with foreign source bindings
        #[arg(long = "source-bundle")]
        source_bundle: PathBuf,

        /// Expected source-bundle manifest BLAKE3
        #[arg(long = "source-bundle-blake3")]
        source_bundle_blake3: String,

        /// Exported execution-profile JSON. Repeat for each profile ID in the plan
        #[arg(long = "execution-profile", required = true)]
        execution_profiles: Vec<PathBuf>,

        /// Cache-closure policy JSON. Required only for cache-only preserved plans
        #[arg(long = "cache-closure-policy")]
        cache_closure_policy: Option<PathBuf>,

        /// Selected root node ID. Repeat to select multiple roots. Defaults to all plan roots
        #[arg(long = "root")]
        roots: Vec<String>,

        /// Write the complete realization receipt atomically to this path
        #[arg(long = "receipt-out")]
        receipt_out: PathBuf,

        /// Maximum concurrent jobs
        #[arg(short = 'j', long = "jobs")]
        jobs: Option<u32>,

        /// Permit only cache URLs already bound into the executable plan
        #[arg(long = "substitute", conflicts_with = "no_substitute")]
        substitute: bool,

        /// Explicitly disable cache substitution
        #[arg(long = "no-substitute")]
        no_substitute: bool,

        /// Reject network fetch and cache substitution
        #[arg(long)]
        offline: bool,

        /// Request remote execution. This version rejects the request before store mutation
        #[arg(long)]
        remote: bool,

        /// Optional Nix-format signing key path
        #[arg(long = "signing-key")]
        signing_key: Option<PathBuf>,
    },

    /// Audit one realized foreign closure from signed PathInfo and castore facts
    Audit {
        /// Path to `mantle-foreign-executable-plan-v1` JSON
        #[arg(long)]
        plan: PathBuf,

        /// Path to the complete `mantle-foreign-realization-receipt-v1` JSON
        #[arg(long = "realization-receipt")]
        realization_receipt: PathBuf,

        /// Path to `mantle-foreign-provenance-policy-v1` JSON
        #[arg(long)]
        policy: PathBuf,

        /// Selected root node ID. Repeat for each audited root
        #[arg(long = "root", required = true)]
        roots: Vec<String>,

        /// Write `mantle-foreign-provenance-audit-v1` atomically to this path
        #[arg(long)]
        out: PathBuf,

        /// Optional Nix-format signing key path used to verify local PathInfo
        #[arg(long = "signing-key")]
        signing_key: Option<PathBuf>,
    },

    /// Lower explicit prefix-aware ATerm derivations into foreign import artifacts
    ProduceAterm {
        /// Declared source store prefix: `/nix/store` or `/gnu/store`
        #[arg(long = "source-prefix")]
        source_prefix: String,

        /// Explicit logical `.drv` path to ATerm file mapping:
        /// `/gnu/store/...drv=/path/to/file.drv`
        #[arg(long = "drv")]
        drv_files: Vec<String>,

        /// Directory containing direct child `*.drv` ATerm files named by store basename
        #[arg(long = "drv-dir")]
        drv_dir: Option<PathBuf>,

        /// Concrete root `.drv` path inside the provided derivation closure
        #[arg(long = "root-derivation")]
        root_derivation: String,

        /// Producer kind recorded in `foreign-derivation-graph-v1`
        #[arg(long = "producer-kind")]
        producer_kind: String,

        /// Package name to write into `foreign-package-index-v1`
        #[arg(long, default_value = DEFAULT_PACKAGE_NAME)]
        package: String,

        /// Package system to write into `foreign-package-index-v1`
        #[arg(long, default_value = DEFAULT_SYSTEM)]
        system: String,

        /// Producer identity or selected package provenance
        #[arg(long = "producer-identity")]
        producer_identity: String,

        /// Producer revision or provenance revision when known
        #[arg(long = "producer-revision", default_value = DEFAULT_PRODUCER_REVISION)]
        producer_revision: String,

        /// Binary cache hint URL to record as policy data on the selected root
        #[arg(long = "cache-url")]
        cache_urls: Vec<String>,

        /// Trust scope label associated with every `--cache-url` hint
        #[arg(long = "cache-trust-scope", default_value = DEFAULT_CACHE_TRUST_SCOPE)]
        cache_trust_scope: String,

        /// Unsupported frontend metadata class to carry in the package index
        #[arg(long = "unsupported-metadata-class")]
        unsupported_metadata_classes: Vec<String>,

        /// Directory where graph and package-index JSON artifacts are written
        #[arg(long = "out-dir")]
        out_dir: PathBuf,
    },

    /// Run a registered Nix producer backend (fix or host-nix) and lower its `.drv` closure
    ProduceBackend {
        /// Backend kind: `fix` or `host-nix`
        #[arg(long)]
        backend: String,

        /// Absolute path to the backend binary (Mantle-built fix binary, or nix-instantiate)
        #[arg(long = "backend-binary")]
        backend_binary: PathBuf,

        /// Version fact recorded for the backend binary
        #[arg(long = "backend-version")]
        backend_version: String,

        /// Nix file to instantiate
        #[arg(long, conflicts_with = "expr")]
        file: Option<PathBuf>,

        /// Inline Nix expression text to instantiate
        #[arg(long)]
        expr: Option<String>,

        /// Attribute path to select from the evaluated value
        #[arg(long)]
        attribute: Option<String>,

        /// String evaluation argument as key=value; repeat for multiple arguments
        #[arg(long = "arg")]
        eval_args: Vec<String>,

        /// Package name to write into `foreign-package-index-v1`
        #[arg(long, default_value = DEFAULT_PACKAGE_NAME)]
        package: String,

        /// Package system to write into `foreign-package-index-v1`
        #[arg(long, default_value = DEFAULT_SYSTEM)]
        system: String,

        /// Owned scratch directory for the run; created when absent
        #[arg(long = "work-dir")]
        work_dir: PathBuf,

        /// Wall-time budget in milliseconds for the backend run
        #[arg(long = "wall-time-ms-max")]
        wall_time_ms_max: Option<u64>,

        /// Address-space limit in bytes for the backend process
        #[arg(long = "memory-bytes-max")]
        memory_bytes_max: Option<u64>,

        /// Total `.drv` closure byte budget
        #[arg(long = "output-bytes-max")]
        output_bytes_max: Option<u64>,

        /// `.drv` closure file count budget
        #[arg(long = "drv-file-count-max")]
        drv_file_count_max: Option<u32>,

        /// Directory where graph and package-index JSON artifacts are written
        #[arg(long = "out-dir")]
        out_dir: PathBuf,
    },

    /// Lower concrete Nix derivation facts into foreign import artifacts
    ProduceNix {
        /// Path to `nix derivation show --recursive --json`-style closure JSON
        #[arg(long = "derivation-json")]
        derivation_json: Option<PathBuf>,

        /// Explicit logical `.drv` path to ATerm file mapping:
        /// `/nix/store/...drv=/path/to/file.drv`
        #[arg(long = "drv")]
        drv_files: Vec<String>,

        /// Directory containing direct child `*.drv` ATerm files named by Nix store basename
        #[arg(long = "drv-dir")]
        drv_dir: Option<PathBuf>,

        /// Concrete root `.drv` path inside the provided derivation closure
        #[arg(long = "root-derivation")]
        root_derivation: String,

        /// Package name to write into `foreign-package-index-v1`
        #[arg(long, default_value = DEFAULT_PACKAGE_NAME)]
        package: String,

        /// Package system to write into `foreign-package-index-v1`
        #[arg(long, default_value = DEFAULT_SYSTEM)]
        system: String,

        /// Producer identity or selected nixpkgs attribute provenance
        #[arg(long = "producer-identity")]
        producer_identity: String,

        /// Producer revision, lock identity, or provenance revision when known
        #[arg(long = "producer-revision", default_value = DEFAULT_PRODUCER_REVISION)]
        producer_revision: String,

        /// Binary cache hint URL to record as policy data on the selected root
        #[arg(long = "cache-url")]
        cache_urls: Vec<String>,

        /// Trust scope label associated with every `--cache-url` hint
        #[arg(long = "cache-trust-scope", default_value = DEFAULT_CACHE_TRUST_SCOPE)]
        cache_trust_scope: String,

        /// Unsupported frontend metadata class to carry in the package index
        #[arg(long = "unsupported-metadata-class")]
        unsupported_metadata_classes: Vec<String>,

        /// Directory where graph and package-index JSON artifacts are written
        #[arg(long = "out-dir")]
        out_dir: PathBuf,
    },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ForeignImportCliReport {
    schema: String,
    command: String,
    verdict: String,
    accepted: bool,
    diagnostics: Vec<ImportDiagnostic>,
    receipt: Option<ImportReceipt>,
    plan: Option<ForeignExecutablePlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    producer_artifacts: Option<ProducerArtifactReport>,
    non_claims: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ProducerArtifactReport {
    graph_path: String,
    package_index_path: String,
}

struct ForeignPlanRequest<'a> {
    graph_path: &'a Path,
    index_path: &'a Path,
    policy_path: &'a Path,
    package: &'a str,
    system: &'a str,
    execution_profile_path: Option<&'a Path>,
    plan_out: Option<&'a Path>,
    receipt_out: Option<&'a Path>,
    json: bool,
}

struct AtermProducerRequest<'a> {
    source_prefix: &'a str,
    drv_file_specs: &'a [String],
    drv_dir: Option<&'a Path>,
    root_derivation: &'a str,
    producer_kind: &'a str,
    package: &'a str,
    system: &'a str,
    producer_identity: &'a str,
    producer_revision: &'a str,
    cache_urls: &'a [String],
    cache_trust_scope: &'a str,
    unsupported_metadata_classes: &'a [String],
    out_dir: &'a Path,
    json: bool,
}

struct NixProducerRequest<'a> {
    derivation_json_path: Option<&'a Path>,
    drv_file_specs: &'a [String],
    drv_dir: Option<&'a Path>,
    root_derivation: &'a str,
    package: &'a str,
    system: &'a str,
    producer_identity: &'a str,
    producer_revision: &'a str,
    cache_urls: &'a [String],
    cache_trust_scope: &'a str,
    unsupported_metadata_classes: &'a [String],
    out_dir: &'a Path,
    json: bool,
}

struct BackendProducerCliRequest<'a> {
    backend: &'a str,
    backend_binary: &'a Path,
    backend_version: &'a str,
    file: Option<&'a Path>,
    expr: Option<&'a str>,
    attribute: Option<&'a str>,
    eval_args: &'a [String],
    package: &'a str,
    system: &'a str,
    work_dir: &'a Path,
    wall_time_ms_max: Option<u64>,
    memory_bytes_max: Option<u64>,
    output_bytes_max: Option<u64>,
    drv_file_count_max: Option<u32>,
    out_dir: &'a Path,
    json: bool,
}

struct JsonReadRequest<'a> {
    path: &'a Path,
    artifact: &'a str,
    command: &'a str,
}

pub(crate) struct ForeignImportContext<'a> {
    pub(crate) output_dir: &'a Path,
    pub(crate) state_dir: &'a Path,
    pub(crate) base_state_dirs: &'a [PathBuf],
    pub(crate) source_bundle_bytes_max: u64,
    pub(crate) verbose: bool,
    pub(crate) json: bool,
}

struct ForeignAuditCommandRequest<'a> {
    plan_path: &'a Path,
    realization_receipt_path: &'a Path,
    policy_path: &'a Path,
    selected_roots: &'a [String],
    output_path: &'a Path,
    signing_key_path: Option<&'a Path>,
}

struct ForeignRealizeCommandRequest<'a> {
    plan_path: &'a Path,
    import_receipt_path: &'a Path,
    source_bundle_path: &'a Path,
    source_bundle_blake3: &'a str,
    execution_profile_paths: &'a [PathBuf],
    cache_closure_policy_path: Option<&'a Path>,
    selected_roots: &'a [String],
    receipt_out: &'a Path,
    jobs: Option<u32>,
    substitute: bool,
    no_substitute: bool,
    offline: bool,
    remote: bool,
    signing_key_path: Option<&'a Path>,
}

pub(crate) fn cmd_foreign_import(
    action: ForeignImportAction,
    context: ForeignImportContext<'_>,
) -> Result<(), RunError> {
    let json = context.json;
    match action {
        ForeignImportAction::Validate {
            graph,
            package_index,
            policy,
            receipt,
        } => run_validate(&graph, package_index.as_deref(), &policy, receipt.as_deref(), json),
        ForeignImportAction::Plan {
            graph,
            package_index,
            policy,
            package,
            system,
            execution_profile,
            plan_out,
            receipt_out,
        } => run_plan(ForeignPlanRequest {
            graph_path: &graph,
            index_path: &package_index,
            policy_path: &policy,
            package: &package,
            system: &system,
            execution_profile_path: execution_profile.as_deref(),
            plan_out: plan_out.as_deref(),
            receipt_out: receipt_out.as_deref(),
            json,
        }),
        ForeignImportAction::PrepareSources { plan, sources, out } => run_prepare_sources(&plan, &sources, &out, json),
        ForeignImportAction::Realize {
            plan,
            import_receipt,
            source_bundle,
            source_bundle_blake3,
            execution_profiles,
            cache_closure_policy,
            roots,
            receipt_out,
            jobs,
            substitute,
            no_substitute,
            offline,
            remote,
            signing_key,
        } => run_realize(
            ForeignRealizeCommandRequest {
                plan_path: &plan,
                import_receipt_path: &import_receipt,
                source_bundle_path: &source_bundle,
                source_bundle_blake3: &source_bundle_blake3,
                execution_profile_paths: &execution_profiles,
                cache_closure_policy_path: cache_closure_policy.as_deref(),
                selected_roots: &roots,
                receipt_out: &receipt_out,
                jobs,
                substitute,
                no_substitute,
                offline,
                remote,
                signing_key_path: signing_key.as_deref(),
            },
            &context,
        ),
        ForeignImportAction::Audit {
            plan,
            realization_receipt,
            policy,
            roots,
            out,
            signing_key,
        } => run_audit(
            ForeignAuditCommandRequest {
                plan_path: &plan,
                realization_receipt_path: &realization_receipt,
                policy_path: &policy,
                selected_roots: &roots,
                output_path: &out,
                signing_key_path: signing_key.as_deref(),
            },
            &context,
        ),
        ForeignImportAction::ProduceAterm {
            source_prefix,
            drv_files,
            drv_dir,
            root_derivation,
            producer_kind,
            package,
            system,
            producer_identity,
            producer_revision,
            cache_urls,
            cache_trust_scope,
            unsupported_metadata_classes,
            out_dir,
        } => run_produce_aterm(AtermProducerRequest {
            source_prefix: &source_prefix,
            drv_file_specs: &drv_files,
            drv_dir: drv_dir.as_deref(),
            root_derivation: &root_derivation,
            producer_kind: &producer_kind,
            package: &package,
            system: &system,
            producer_identity: &producer_identity,
            producer_revision: &producer_revision,
            cache_urls: &cache_urls,
            cache_trust_scope: &cache_trust_scope,
            unsupported_metadata_classes: &unsupported_metadata_classes,
            out_dir: &out_dir,
            json,
        }),
        ForeignImportAction::ProduceBackend {
            backend,
            backend_binary,
            backend_version,
            file,
            expr,
            attribute,
            eval_args,
            package,
            system,
            work_dir,
            wall_time_ms_max,
            memory_bytes_max,
            output_bytes_max,
            drv_file_count_max,
            out_dir,
        } => run_produce_backend(BackendProducerCliRequest {
            backend: &backend,
            backend_binary: &backend_binary,
            backend_version: &backend_version,
            file: file.as_deref(),
            expr: expr.as_deref(),
            attribute: attribute.as_deref(),
            eval_args: &eval_args,
            package: &package,
            system: &system,
            work_dir: &work_dir,
            wall_time_ms_max,
            memory_bytes_max,
            output_bytes_max,
            drv_file_count_max,
            out_dir: &out_dir,
            json,
        }),
        ForeignImportAction::ProduceNix {
            derivation_json,
            drv_files,
            drv_dir,
            root_derivation,
            package,
            system,
            producer_identity,
            producer_revision,
            cache_urls,
            cache_trust_scope,
            unsupported_metadata_classes,
            out_dir,
        } => run_produce_nix(NixProducerRequest {
            derivation_json_path: derivation_json.as_deref(),
            drv_file_specs: &drv_files,
            drv_dir: drv_dir.as_deref(),
            root_derivation: &root_derivation,
            package: &package,
            system: &system,
            producer_identity: &producer_identity,
            producer_revision: &producer_revision,
            cache_urls: &cache_urls,
            cache_trust_scope: &cache_trust_scope,
            unsupported_metadata_classes: &unsupported_metadata_classes,
            out_dir: &out_dir,
            json,
        }),
    }
}

fn run_validate(
    graph_path: &Path,
    index_path: Option<&Path>,
    policy_path: &Path,
    receipt_path: Option<&Path>,
    json: bool,
) -> Result<(), RunError> {
    assert!(!VALIDATE_COMMAND.is_empty(), "foreign validation command identity must not be empty");
    assert!(!CLI_REPORT_SCHEMA.is_empty(), "foreign import report schema must not be empty");
    let graph = match read_json::<ForeignDerivationGraph>(JsonReadRequest {
        path: graph_path,
        artifact: "graph",
        command: VALIDATE_COMMAND,
    })? {
        Ok(graph) => graph,
        Err(report) => return emit_report(report, json),
    };
    let index = match read_optional_index(index_path, VALIDATE_COMMAND)? {
        Ok(index) => index,
        Err(report) => return emit_report(report, json),
    };
    let policy = match read_json::<TranslationPolicy>(JsonReadRequest {
        path: policy_path,
        artifact: "policy",
        command: VALIDATE_COMMAND,
    })? {
        Ok(policy) => policy,
        Err(report) => return emit_report(report, json),
    };
    let receipt = match read_optional_receipt(receipt_path, VALIDATE_COMMAND)? {
        Ok(receipt) => receipt,
        Err(report) => return emit_report(report, json),
    };
    let outcome = validate_inputs(graph, index, policy, receipt);
    emit_report(outcome, json)
}

fn run_plan(request: ForeignPlanRequest<'_>) -> Result<(), RunError> {
    assert!(!PLAN_COMMAND.is_empty(), "foreign plan command identity must not be empty");
    assert!(!CLI_REPORT_SCHEMA.is_empty(), "foreign plan report schema must not be empty");
    let graph = match read_json::<ForeignDerivationGraph>(JsonReadRequest {
        path: request.graph_path,
        artifact: "graph",
        command: PLAN_COMMAND,
    })? {
        Ok(graph) => graph,
        Err(report) => return emit_report(report, request.json),
    };
    let index = match read_json::<PackageIndex>(JsonReadRequest {
        path: request.index_path,
        artifact: "package-index",
        command: PLAN_COMMAND,
    })? {
        Ok(index) => index,
        Err(report) => return emit_report(report, request.json),
    };
    let policy = match read_json::<TranslationPolicy>(JsonReadRequest {
        path: request.policy_path,
        artifact: "policy",
        command: PLAN_COMMAND,
    })? {
        Ok(policy) => policy,
        Err(report) => return emit_report(report, request.json),
    };
    let outcome = if let Some(path) = request.execution_profile_path {
        let execution_profile = match read_json::<ExecutionProfile>(JsonReadRequest {
            path,
            artifact: "execution-profile",
            command: PLAN_COMMAND,
        })? {
            Ok(profile) => profile,
            Err(report) => return emit_report(report, request.json),
        };
        plan_inputs_with_profile(graph, index, request.package, policy, request.system, &execution_profile)
    } else {
        plan_inputs(graph, index, request.package, policy, request.system)
    };
    if request.plan_out.is_some() && request.plan_out == request.receipt_out {
        return Err(RunError::Internal("foreign plan and import receipt output paths must differ".to_string()));
    }
    if outcome.accepted {
        if let Some(path) = request.plan_out {
            let plan = outcome
                .plan
                .as_ref()
                .ok_or_else(|| RunError::Internal("accepted foreign plan report has no plan".to_string()))?;
            write_json_atomically(path, plan, "foreign executable plan")?;
        }
        if let Some(path) = request.receipt_out {
            let receipt = outcome
                .receipt
                .as_ref()
                .ok_or_else(|| RunError::Internal("accepted foreign plan report has no receipt".to_string()))?;
            write_json_atomically(path, receipt, "foreign import receipt")?;
        }
    }
    emit_report(outcome, request.json)
}

fn run_prepare_sources(
    plan_path: &Path,
    source_specs: &[String],
    output_path: &Path,
    json: bool,
) -> Result<(), RunError> {
    let plan = match read_json::<ForeignExecutablePlan>(JsonReadRequest {
        path: plan_path,
        artifact: "plan",
        command: PREPARE_SOURCES_COMMAND,
    })? {
        Ok(plan) => plan,
        Err(report) => return emit_report(report, json),
    };
    crate::foreign_executable_plan::validate_foreign_executable_plan(&plan)
        .map_err(|diagnostic| RunError::Internal(format!("foreign source plan is invalid: {}", diagnostic.class)))?;
    let mut bindings = Vec::with_capacity(source_specs.len());
    for source_spec in source_specs {
        let (payload_id, path) = source_spec.split_once(SOURCE_BINDING_SEPARATOR).ok_or_else(|| {
            RunError::Internal(format!("foreign source binding must use payload-id=/path syntax: {source_spec}"))
        })?;
        if payload_id.is_empty() || path.is_empty() {
            return Err(RunError::Internal(format!(
                "foreign source binding has an empty payload ID or path: {source_spec}"
            )));
        }
        bindings.push(ForeignSourcePathBinding {
            payload_id: payload_id.to_string(),
            path: PathBuf::from(path),
        });
    }
    let manifest = if is_cache_only_foreign_plan(&plan) {
        if !bindings.is_empty() {
            return Err(RunError::Internal(
                "cache-only foreign source preparation rejects builder source bindings".to_string(),
            ));
        }
        plan_empty_source_bundle(&plan.target_store_prefix)?
    } else {
        plan_bound_foreign_source_bundle(&plan.source_requirements, &bindings, &plan.target_store_prefix)?
    };
    write_json_atomically(output_path, &manifest, "foreign source bundle")?;
    if json {
        println!(
            "{}",
            serde_json::to_string(&manifest)
                .map_err(|error| RunError::Internal(format!("serializing foreign source bundle: {error}")))?
        );
    } else {
        println!(
            "manifest_blake3={} records={} path={}",
            manifest.manifest_blake3,
            manifest.records.len(),
            output_path.display()
        );
    }
    Ok(())
}

fn run_audit(request: ForeignAuditCommandRequest<'_>, context: &ForeignImportContext<'_>) -> Result<(), RunError> {
    assert!(!AUDIT_COMMAND.is_empty(), "foreign audit command identity must not be empty");
    if request.output_path.exists() {
        return Err(RunError::Internal(format!(
            "foreign provenance audit output already exists: {}",
            request.output_path.display()
        )));
    }
    let plan =
        read_required_foreign_json::<ForeignExecutablePlan>(request.plan_path, "plan", AUDIT_COMMAND, context.json)?;
    let realization_receipt = read_required_foreign_json::<
        crate::foreign_realization_receipt::ForeignRealizationReceipt,
    >(
        request.realization_receipt_path, "realization-receipt", AUDIT_COMMAND, context.json
    )?;
    let policy = read_required_foreign_json::<crunch_store::ForeignProvenancePolicy>(
        request.policy_path,
        "provenance-policy",
        AUDIT_COMMAND,
        context.json,
    )?;
    let (keypair, _key_path) =
        crate::build_cmd::load_existing_signing_keypair(request.signing_key_path, context.state_dir)?;
    let cache_keys = trusted_cache_keys_from_plan(&plan)?;
    let trusted_keys = crunch_build::build_trusted_keys(&keypair, Some(&cache_keys));
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| RunError::Internal(format!("creating foreign provenance audit runtime: {error}")))?;
    let receipt: ForeignProvenanceAuditReceipt =
        runtime.block_on(audit_foreign_realization(ForeignProvenanceAuditRequest {
            plan: &plan,
            realization_receipt: &realization_receipt,
            policy: &policy,
            selected_root_node_ids: request.selected_roots,
            output_dir: context.output_dir,
            state_dir: context.state_dir,
            base_state_dirs: context.base_state_dirs,
            trusted_keys: &trusted_keys,
        }))?;
    write_json_atomically(request.output_path, &receipt, "foreign provenance audit receipt")?;
    if context.json {
        println!(
            "{}",
            serde_json::to_string(&receipt).map_err(|error| RunError::Internal(format!(
                "serializing foreign provenance audit receipt: {error}"
            )))?
        );
    } else {
        println!(
            "status={} strongest_state={} audit_blake3={} roots={} closure_paths={} findings={} path={}",
            receipt.status,
            receipt.strongest_state,
            receipt.audit_blake3,
            receipt.selected_root_node_ids.len(),
            receipt.closure_paths.len(),
            receipt.findings.len(),
            request.output_path.display()
        );
    }
    if receipt.status == "pass" {
        Ok(())
    } else {
        Err(RunError::Reported(FAILURE_EXIT_CODE))
    }
}

fn run_realize(request: ForeignRealizeCommandRequest<'_>, context: &ForeignImportContext<'_>) -> Result<(), RunError> {
    assert!(!REALIZE_COMMAND.is_empty(), "foreign realize command identity must not be empty");
    if request.receipt_out.exists() {
        return Err(RunError::Internal(format!(
            "foreign realization receipt output already exists: {}",
            request.receipt_out.display()
        )));
    }
    let plan = read_required_realization_json::<ForeignExecutablePlan>(request.plan_path, "plan", context.json)?;
    let import_receipt =
        read_required_realization_json::<ImportReceipt>(request.import_receipt_path, "import-receipt", context.json)?;
    let source_bundle = read_required_realization_json_with_limit::<SourceBundleManifest>(
        request.source_bundle_path,
        "source-bundle",
        context.json,
        context.source_bundle_bytes_max,
    )?;
    let cache_closure_policy = request
        .cache_closure_policy_path
        .map(|path| {
            read_required_realization_json::<ForeignCacheClosurePolicy>(path, "cache-closure-policy", context.json)
        })
        .transpose()?;
    let mut execution_profiles = BTreeMap::new();
    for profile_path in request.execution_profile_paths {
        let profile =
            read_required_realization_json::<ExecutionProfile>(profile_path, "execution-profile", context.json)?;
        if execution_profiles.insert(profile.profile_id.clone(), profile).is_some() {
            return Err(RunError::Internal("foreign realization repeats an execution profile ID".to_string()));
        }
    }
    let mut selected_roots = if request.selected_roots.is_empty() {
        plan.selected_roots.iter().map(|root| root.node_id.clone()).collect::<Vec<_>>()
    } else {
        request.selected_roots.to_vec()
    };
    let selected_root_count = selected_roots.len();
    selected_roots.sort();
    selected_roots.dedup();
    if selected_roots.len() != selected_root_count {
        return Err(RunError::Internal("foreign realization selected-root set contains a duplicate".to_string()));
    }
    if request.substitute && request.no_substitute {
        return Err(RunError::Internal(
            "foreign realization cannot enable and disable substitution together".to_string(),
        ));
    }
    if request.offline && request.substitute {
        return Err(RunError::Internal("offline foreign realization cannot enable cache substitution".to_string()));
    }
    let cache_only = is_cache_only_foreign_plan(&plan);
    if cache_only && (!request.substitute || request.offline) {
        return Err(RunError::Internal("cache-only foreign realization requires online substitution".to_string()));
    }
    if cache_only && cache_closure_policy.is_none() {
        return Err(RunError::Internal("cache-only foreign realization requires --cache-closure-policy".to_string()));
    }
    if !cache_only && cache_closure_policy.is_some() {
        return Err(RunError::Internal("cache closure policy is only valid for a cache-only foreign plan".to_string()));
    }
    validate_foreign_realization_admission(ForeignRealizationAdmission {
        plan: &plan,
        import_receipt: &import_receipt,
        selected_root_node_ids: &selected_roots,
        execution_profiles: &execution_profiles,
        remote_execution_requested: request.remote,
    })
    .map_err(|error| RunError::Internal(error.to_string()))?;
    if let Some(policy) = cache_closure_policy.as_ref() {
        validate_foreign_cache_closure_policy(policy).map_err(|error| RunError::Internal(error.to_string()))?;
        validate_cache_only_source_bundle(&source_bundle, request.source_bundle_blake3)
            .map_err(|error| RunError::Internal(error.to_string()))?;
    } else {
        validate_foreign_source_admission(ForeignSourceAdmission {
            source_requirements: &plan.source_requirements,
            source_bundle: &source_bundle,
            expected_manifest_blake3: request.source_bundle_blake3,
        })
        .map_err(|error| RunError::Internal(error.to_string()))?;
    }
    let keypair =
        crate::build_cmd::load_or_generate_signing_keypair(request.signing_key_path, context.state_dir, !context.json)?;
    let cache_keys = trusted_cache_keys_from_plan(&plan)?;
    let trusted_keys = crunch_build::build_trusted_keys(&keypair, Some(&cache_keys));
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| RunError::Internal(format!("creating foreign realization runtime: {error}")))?;
    let receipt = runtime.block_on(realize_foreign_plan(ForeignRealizationRequest {
        plan: &plan,
        import_receipt: &import_receipt,
        source_bundle: &source_bundle,
        expected_source_bundle_blake3: request.source_bundle_blake3,
        selected_root_node_ids: &selected_roots,
        execution_profiles: &execution_profiles,
        cache_closure_policy: cache_closure_policy.as_ref(),
        output_dir: context.output_dir,
        state_dir: context.state_dir,
        base_state_dirs: context.base_state_dirs,
        keypair: &keypair,
        trusted_keys: &trusted_keys,
        max_jobs: crunch_pipeline::resolve_max_jobs(request.jobs),
        substitution_enabled: request.substitute,
        offline: request.offline,
        remote_execution_requested: request.remote,
        verbose: context.verbose,
    }))?;
    write_json_atomically(request.receipt_out, &receipt, "foreign realization receipt")?;
    if context.json {
        println!(
            "{}",
            serde_json::to_string(&receipt)
                .map_err(|error| RunError::Internal(format!("serializing foreign realization receipt: {error}")))?
        );
    } else {
        println!(
            "status={} receipt_blake3={} roots={} units={} sources={} path={}",
            receipt.status,
            receipt.receipt_blake3,
            receipt.selected_root_node_ids.len(),
            receipt.units.len(),
            receipt.sources.len(),
            request.receipt_out.display()
        );
    }
    if receipt.failure.is_some() {
        Err(RunError::Reported(FAILURE_EXIT_CODE))
    } else {
        Ok(())
    }
}

fn trusted_cache_keys_from_plan(
    plan: &ForeignExecutablePlan,
) -> Result<Vec<nix_compat::narinfo::VerifyingKey>, RunError> {
    let cache_urls = plan
        .substitution_audit
        .iter()
        .map(|audit| audit.cache_url.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut keys_by_encoding = std::collections::BTreeMap::new();
    for cache_url in cache_urls {
        let keys = crunch_store::parse_remote_trusted_public_keys(cache_url).map_err(|error| {
            RunError::Internal(format!("foreign plan cache trust configuration is invalid: {error}"))
        })?;
        for key in keys {
            keys_by_encoding.entry(key.to_string()).or_insert(key);
        }
    }
    if is_cache_only_foreign_plan(plan) && keys_by_encoding.is_empty() {
        return Err(RunError::Internal(
            "cache-only foreign plan has no receipt-bound trusted cache public key".to_string(),
        ));
    }
    Ok(keys_by_encoding.into_values().collect())
}

fn read_required_realization_json<T: DeserializeOwned>(path: &Path, artifact: &str, json: bool) -> Result<T, RunError> {
    read_required_foreign_json(path, artifact, REALIZE_COMMAND, json)
}

fn read_required_realization_json_with_limit<T: DeserializeOwned>(
    path: &Path,
    artifact: &str,
    json: bool,
    max_bytes: u64,
) -> Result<T, RunError> {
    read_required_foreign_json_with_limit(path, artifact, REALIZE_COMMAND, json, max_bytes)
}

fn read_required_foreign_json<T: DeserializeOwned>(
    path: &Path,
    artifact: &str,
    command: &str,
    json: bool,
) -> Result<T, RunError> {
    read_required_foreign_json_with_limit(path, artifact, command, json, DEFAULT_FOREIGN_JSON_ARTIFACT_BYTES_MAX)
}

fn read_required_foreign_json_with_limit<T: DeserializeOwned>(
    path: &Path,
    artifact: &str,
    command: &str,
    json: bool,
    max_bytes: u64,
) -> Result<T, RunError> {
    match read_json_with_limit::<T>(
        JsonReadRequest {
            path,
            artifact,
            command,
        },
        max_bytes,
    )? {
        Ok(value) => Ok(value),
        Err(report) => match emit_report(report, json) {
            Err(error) => Err(error),
            Ok(()) => {
                Err(RunError::Internal(format!("rejected foreign {command} input unexpectedly returned success")))
            }
        },
    }
}

fn run_produce_nix(request: NixProducerRequest<'_>) -> Result<(), RunError> {
    assert!(!PRODUCE_NIX_COMMAND.is_empty(), "Nix producer command identity must not be empty");
    assert_ne!(NIXPKGS_GRAPH_FILE, NIXPKGS_INDEX_FILE, "Nix producer artifact names must differ");
    let closure = match read_nix_producer_closure(
        request.derivation_json_path,
        request.drv_file_specs,
        request.drv_dir,
        request.root_derivation,
    )? {
        Ok(closure) => closure,
        Err(report) => return emit_report(report, request.json),
    };
    let config = NixProducerConfig {
        package_name: request.package.to_string(),
        system: request.system.to_string(),
        root_derivation: request.root_derivation.to_string(),
        producer_identity: request.producer_identity.to_string(),
        producer_revision: request.producer_revision.to_string(),
        cache_hints: request
            .cache_urls
            .iter()
            .map(|cache_url| CacheHint {
                cache_url: cache_url.clone(),
                trust_scope: request.cache_trust_scope.to_string(),
            })
            .collect(),
        unsupported_metadata_classes: request.unsupported_metadata_classes.to_vec(),
    };
    let artifacts = match lower_nix_derivation_json_closure(&closure, &config) {
        Ok(artifacts) => artifacts,
        Err(diagnostic) => return emit_report(rejected_report(PRODUCE_NIX_COMMAND, diagnostic), request.json),
    };
    fs::create_dir_all(request.out_dir).map_err(|error| {
        RunError::Internal(format!("creating foreign import artifact directory {}: {error}", request.out_dir.display()))
    })?;
    let graph_path = request.out_dir.join(NIXPKGS_GRAPH_FILE);
    let index_path = request.out_dir.join(NIXPKGS_INDEX_FILE);
    write_json_file(&graph_path, &artifacts.graph)?;
    write_json_file(&index_path, &artifacts.package_index)?;
    emit_report(producer_report(PRODUCE_NIX_COMMAND, &graph_path, &index_path), request.json)
}

fn run_produce_aterm(request: AtermProducerRequest<'_>) -> Result<(), RunError> {
    assert!(!PRODUCE_ATERM_COMMAND.is_empty(), "ATerm producer command identity must not be empty");
    assert_ne!(ATERM_GRAPH_FILE, ATERM_INDEX_FILE, "ATerm producer artifact names must differ");
    let inputs = match read_aterm_producer_inputs(request.source_prefix, request.drv_file_specs, request.drv_dir)? {
        Ok(inputs) => inputs,
        Err(report) => return emit_report(report, request.json),
    };
    let closure = match parse_prefix_aware_aterm_bundle(request.source_prefix, &inputs) {
        Ok(closure) => closure,
        Err(diagnostic) => return emit_report(rejected_report(PRODUCE_ATERM_COMMAND, diagnostic), request.json),
    };
    let config = AtermProducerConfig {
        source_prefix: request.source_prefix.to_string(),
        producer_kind: request.producer_kind.to_string(),
        package_name: request.package.to_string(),
        system: request.system.to_string(),
        root_derivation: request.root_derivation.to_string(),
        producer_identity: request.producer_identity.to_string(),
        producer_revision: request.producer_revision.to_string(),
        cache_hints: producer_cache_hints(request.cache_urls, request.cache_trust_scope),
        unsupported_metadata_classes: request.unsupported_metadata_classes.to_vec(),
    };
    let artifacts = match lower_prefix_aware_aterm_closure(&closure, &config) {
        Ok(artifacts) => artifacts,
        Err(diagnostic) => return emit_report(rejected_report(PRODUCE_ATERM_COMMAND, diagnostic), request.json),
    };
    fs::create_dir_all(request.out_dir).map_err(|error| {
        RunError::Internal(format!("creating foreign import artifact directory {}: {error}", request.out_dir.display()))
    })?;
    let graph_path = request.out_dir.join(ATERM_GRAPH_FILE);
    let index_path = request.out_dir.join(ATERM_INDEX_FILE);
    write_json_file(&graph_path, &artifacts.graph)?;
    write_json_file(&index_path, &artifacts.package_index)?;
    emit_report(producer_report(PRODUCE_ATERM_COMMAND, &graph_path, &index_path), request.json)
}

fn backend_contract_request(
    request: &BackendProducerCliRequest<'_>,
) -> Result<BackendContractRequest, ForeignImportCliReport> {
    debug_assert!(!request.backend.is_empty(), "backend flag is required by clap");
    let kind = ProducerBackendKind::parse(request.backend).ok_or_else(|| {
        rejected_report(
            PRODUCE_BACKEND_COMMAND,
            diagnostic("unknown-backend", None, "backend must be one of: fix, host-nix"),
        )
    })?;
    let attribute = request.attribute.map(ToOwned::to_owned);
    let target = match (request.file, request.expr) {
        (Some(file), None) => ProducerTarget::File {
            path: file.display().to_string(),
            attribute,
        },
        (None, Some(expr)) => ProducerTarget::Expr {
            text: expr.to_string(),
            attribute,
        },
        _ => {
            return Err(rejected_report(
                PRODUCE_BACKEND_COMMAND,
                diagnostic("invalid-request", None, "provide exactly one of --file or --expr"),
            ));
        }
    };
    let mut eval_args: BTreeMap<String, String> = BTreeMap::new();
    for raw_arg in request.eval_args {
        let Some((key, value)) = raw_arg.split_once(DRV_SPEC_SEPARATOR) else {
            return Err(rejected_report(
                PRODUCE_BACKEND_COMMAND,
                diagnostic("invalid-request", None, "--arg entries must use key=value form"),
            ));
        };
        eval_args.insert(key.to_string(), value.to_string());
    }
    let defaults = ProducerBudget::default();
    Ok(BackendContractRequest {
        schema: NIX_PRODUCER_CONTRACT_SCHEMA.to_string(),
        backend: kind,
        target,
        eval_args,
        system: request.system.to_string(),
        budget: ProducerBudget {
            wall_time_ms_max: request.wall_time_ms_max.unwrap_or(defaults.wall_time_ms_max),
            memory_bytes_max: request.memory_bytes_max.unwrap_or(defaults.memory_bytes_max),
            output_bytes_max: request.output_bytes_max.unwrap_or(defaults.output_bytes_max),
            drv_file_count_max: request.drv_file_count_max.unwrap_or(defaults.drv_file_count_max),
        },
    })
}

fn lower_backend_closure(
    request: &BackendProducerCliRequest<'_>,
    kind: ProducerBackendKind,
    success: &crate::nix_producer::ProducerSuccess,
) -> Result<Result<(PathBuf, PathBuf), ForeignImportCliReport>, RunError> {
    let drv_dir = PathBuf::from(&success.drv_dir);
    let inputs = match read_aterm_drv_dir(NIX_LOGICAL_STORE_PREFIX, &drv_dir)? {
        Ok(inputs) => inputs,
        Err(report) => return Ok(Err(report)),
    };
    let closure = match parse_prefix_aware_aterm_bundle(NIX_LOGICAL_STORE_PREFIX, &inputs) {
        Ok(closure) => closure,
        Err(diagnostic) => return Ok(Err(rejected_report(PRODUCE_BACKEND_COMMAND, diagnostic))),
    };
    let config = AtermProducerConfig {
        source_prefix: NIX_LOGICAL_STORE_PREFIX.to_string(),
        producer_kind: kind.as_str().to_string(),
        package_name: request.package.to_string(),
        system: request.system.to_string(),
        root_derivation: success.root_drv_path.clone(),
        producer_identity: success.identity.binary_identity.clone(),
        producer_revision: success.identity.backend_version.clone(),
        cache_hints: Vec::new(),
        unsupported_metadata_classes: Vec::new(),
    };
    let artifacts = match lower_prefix_aware_aterm_closure(&closure, &config) {
        Ok(artifacts) => artifacts,
        Err(diagnostic) => return Ok(Err(rejected_report(PRODUCE_BACKEND_COMMAND, diagnostic))),
    };
    fs::create_dir_all(request.out_dir).map_err(|error| {
        RunError::Internal(format!("creating foreign import artifact directory {}: {error}", request.out_dir.display()))
    })?;
    let graph_path = request.out_dir.join(ATERM_GRAPH_FILE);
    let index_path = request.out_dir.join(ATERM_INDEX_FILE);
    write_json_file(&graph_path, &artifacts.graph)?;
    write_json_file(&index_path, &artifacts.package_index)?;
    Ok(Ok((graph_path, index_path)))
}

fn run_produce_backend(request: BackendProducerCliRequest<'_>) -> Result<(), RunError> {
    assert!(!PRODUCE_BACKEND_COMMAND.is_empty(), "backend producer command identity must not be empty");
    assert!(request.backend_binary.is_absolute(), "backend binary path must be absolute");
    let contract_request = match backend_contract_request(&request) {
        Ok(contract_request) => contract_request,
        Err(report) => return emit_report(report, request.json),
    };
    let run_config = BackendRunConfig {
        binary_path: request.backend_binary.to_path_buf(),
        version_label: request.backend_version.to_string(),
        store_read_root: PathBuf::from(NIX_LOGICAL_STORE_PREFIX_ROOT),
        work_dir: request.work_dir.to_path_buf(),
    };
    let success = match run_backend(&contract_request, &run_config) {
        Ok(success) => success,
        Err(error) => {
            return emit_report(
                rejected_report(PRODUCE_BACKEND_COMMAND, diagnostic(error.class.as_str(), None, &error.detail)),
                request.json,
            );
        }
    };
    let (graph_path, index_path) = match lower_backend_closure(&request, contract_request.backend, &success)? {
        Ok(paths) => paths,
        Err(report) => return emit_report(report, request.json),
    };
    emit_report(producer_report(PRODUCE_BACKEND_COMMAND, &graph_path, &index_path), request.json)
}

fn producer_cache_hints(cache_urls: &[String], trust_scope: &str) -> Vec<CacheHint> {
    let hints = cache_urls
        .iter()
        .map(|cache_url| CacheHint {
            cache_url: cache_url.clone(),
            trust_scope: trust_scope.to_string(),
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(hints.len(), cache_urls.len());
    debug_assert!(hints.iter().all(|hint| hint.trust_scope == trust_scope));
    hints
}

fn read_aterm_producer_inputs(
    source_prefix: &str,
    drv_file_specs: &[String],
    drv_dir: Option<&Path>,
) -> Result<Result<Vec<AtermDerivationInput>, ForeignImportCliReport>, RunError> {
    let mode_count = [!drv_file_specs.is_empty(), drv_dir.is_some()].into_iter().filter(|selected| *selected).count();
    if mode_count == 0 {
        return Ok(Err(rejected_report(
            PRODUCE_ATERM_COMMAND,
            diagnostic("missing-aterm-producer-input", None, "provide --drv logical=file inputs or --drv-dir"),
        )));
    }
    if mode_count != 1 {
        return Ok(Err(rejected_report(
            PRODUCE_ATERM_COMMAND,
            diagnostic("aterm-producer-input-mode-conflict", None, "provide exactly one of --drv inputs or --drv-dir"),
        )));
    }
    let result = if let Some(dir) = drv_dir {
        read_aterm_drv_dir(source_prefix, dir)
    } else {
        read_aterm_drv_specs(drv_file_specs)
    }?;
    debug_assert_eq!(mode_count, 1);
    debug_assert!(mode_count > 0);
    Ok(result)
}

fn read_aterm_drv_specs(
    drv_file_specs: &[String],
) -> Result<Result<Vec<AtermDerivationInput>, ForeignImportCliReport>, RunError> {
    if drv_file_specs.len() > MAX_ATERM_BUNDLE_DERIVATIONS {
        return Ok(Err(rejected_report(
            PRODUCE_ATERM_COMMAND,
            diagnostic(
                "foreign-aterm-derivation-count-out-of-range",
                None,
                "foreign ATerm explicit bundle exceeds the derivation limit",
            ),
        )));
    }
    let mut inputs = Vec::with_capacity(drv_file_specs.len());
    let mut total_bytes = 0usize;
    for spec in drv_file_specs {
        let (logical_path, file_path) = match parse_aterm_file_spec(spec) {
            Ok(pair) => pair,
            Err(diagnostic) => return Ok(Err(rejected_report(PRODUCE_ATERM_COMMAND, diagnostic))),
        };
        let bytes = match read_aterm_file(&logical_path, &file_path) {
            Ok(bytes) => bytes,
            Err(report) => return Ok(Err(report)),
        };
        if let Err(report) =
            push_bounded_aterm_input(&mut inputs, &mut total_bytes, AtermDerivationInput { logical_path, bytes })
        {
            return Ok(Err(report));
        }
    }
    debug_assert_eq!(inputs.len(), drv_file_specs.len());
    debug_assert!(total_bytes <= MAX_ATERM_BUNDLE_BYTES);
    debug_assert!(!inputs.is_empty());
    Ok(Ok(inputs))
}

fn read_aterm_drv_dir(
    source_prefix: &str,
    drv_dir: &Path,
) -> Result<Result<Vec<AtermDerivationInput>, ForeignImportCliReport>, RunError> {
    let entries = fs::read_dir(drv_dir).map_err(|error| {
        RunError::Internal(format!("reading foreign import drv dir {}: {error}", drv_dir.display()))
    })?;
    let mut file_paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            RunError::Internal(format!("reading foreign import drv dir {}: {error}", drv_dir.display()))
        })?;
        let file_path = entry.path();
        if file_path.extension().and_then(|extension| extension.to_str()) != Some(DRV_FILE_EXTENSION) {
            continue;
        }
        if file_paths.len() >= MAX_ATERM_BUNDLE_DERIVATIONS {
            return Ok(Err(rejected_report(
                PRODUCE_ATERM_COMMAND,
                diagnostic(
                    "foreign-aterm-derivation-count-out-of-range",
                    None,
                    "foreign ATerm directory bundle exceeds the derivation limit",
                ),
            )));
        }
        file_paths.push(file_path);
    }
    file_paths.sort();
    let file_count = file_paths.len();
    let mut inputs = Vec::with_capacity(file_count);
    let mut total_bytes = 0usize;
    for file_path in file_paths {
        let Some(file_name) = file_path.file_name().and_then(|name| name.to_str()) else {
            return Ok(Err(rejected_report(
                PRODUCE_ATERM_COMMAND,
                diagnostic("non-utf8-foreign-derivation-filename", None, "drv directory contains non-UTF-8 filename"),
            )));
        };
        let logical_path = format!("{source_prefix}/{file_name}");
        let bytes = match read_aterm_file(&logical_path, &file_path) {
            Ok(bytes) => bytes,
            Err(report) => return Ok(Err(report)),
        };
        if let Err(report) =
            push_bounded_aterm_input(&mut inputs, &mut total_bytes, AtermDerivationInput { logical_path, bytes })
        {
            return Ok(Err(report));
        }
    }
    debug_assert_eq!(inputs.len(), file_count);
    debug_assert!(inputs.len() <= MAX_ATERM_BUNDLE_DERIVATIONS);
    debug_assert!(total_bytes <= MAX_ATERM_BUNDLE_BYTES);
    Ok(Ok(inputs))
}

fn push_bounded_aterm_input(
    inputs: &mut Vec<AtermDerivationInput>,
    total_bytes: &mut usize,
    input: AtermDerivationInput,
) -> Result<(), ForeignImportCliReport> {
    let next_total = total_bytes.checked_add(input.bytes.len()).ok_or_else(|| {
        rejected_report(
            PRODUCE_ATERM_COMMAND,
            diagnostic(
                "foreign-aterm-bundle-bytes-out-of-range",
                None,
                "foreign ATerm bundle byte count overflowed while reading",
            ),
        )
    })?;
    if next_total > MAX_ATERM_BUNDLE_BYTES {
        return Err(rejected_report(
            PRODUCE_ATERM_COMMAND,
            diagnostic(
                "foreign-aterm-bundle-bytes-out-of-range",
                None,
                "foreign ATerm bundle exceeds the total byte limit while reading",
            ),
        ));
    }
    *total_bytes = next_total;
    inputs.push(input);
    debug_assert!(*total_bytes <= MAX_ATERM_BUNDLE_BYTES);
    debug_assert!(inputs.len() <= MAX_ATERM_BUNDLE_DERIVATIONS);
    Ok(())
}

fn read_aterm_file(logical_path: &str, file_path: &Path) -> Result<Vec<u8>, ForeignImportCliReport> {
    let read_limit = MAX_ATERM_DERIVATION_BYTES.checked_add(1).expect("ATerm read limit must fit usize");
    let read_limit_u64 = u64::try_from(read_limit).expect("ATerm read limit must fit u64");
    let file = fs::File::open(file_path).map_err(|error| unreadable_aterm_report(logical_path, file_path, error))?;
    let mut reader = file.take(read_limit_u64);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| unreadable_aterm_report(logical_path, file_path, error))?;
    debug_assert!(bytes.len() <= read_limit);
    debug_assert!(read_limit > MAX_ATERM_DERIVATION_BYTES);
    Ok(bytes)
}

fn unreadable_aterm_report(logical_path: &str, file_path: &Path, error: std::io::Error) -> ForeignImportCliReport {
    rejected_report(
        PRODUCE_ATERM_COMMAND,
        diagnostic(
            "unreadable-foreign-derivation-file",
            None,
            &format!("reading {logical_path} from {}: {error}", file_path.display()),
        ),
    )
}

fn parse_aterm_file_spec(spec: &str) -> Result<(String, PathBuf), ImportDiagnostic> {
    let Some((logical_path, file_path)) = spec.split_once(DRV_SPEC_SEPARATOR) else {
        return Err(diagnostic(
            "malformed-foreign-derivation-input",
            None,
            "--drv must be formatted as /store/prefix/name.drv=/path/to/file.drv",
        ));
    };
    if logical_path.is_empty() || file_path.is_empty() {
        return Err(diagnostic(
            "malformed-foreign-derivation-input",
            None,
            "--drv logical path and file path must both be non-empty",
        ));
    }
    Ok((logical_path.to_string(), PathBuf::from(file_path)))
}

fn read_nix_producer_closure(
    derivation_json_path: Option<&Path>,
    drv_file_specs: &[String],
    drv_dir: Option<&Path>,
    root_derivation: &str,
) -> Result<Result<crate::foreign_derivation_import::NixDerivationJsonClosure, ForeignImportCliReport>, RunError> {
    let mode_count = [
        derivation_json_path.is_some(),
        !drv_file_specs.is_empty(),
        drv_dir.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if mode_count == 0 {
        return Ok(Err(rejected_report(
            PRODUCE_NIX_COMMAND,
            diagnostic(
                "missing-nix-producer-input",
                None,
                "provide --derivation-json, --drv logical=file inputs, or --drv-dir",
            ),
        )));
    }
    if mode_count != 1 {
        return Ok(Err(rejected_report(
            PRODUCE_NIX_COMMAND,
            diagnostic(
                "nix-producer-input-mode-conflict",
                None,
                "provide exactly one of --derivation-json, --drv inputs, or --drv-dir",
            ),
        )));
    }
    assert_eq!(mode_count, 1, "admitted Nix producer input must select exactly one mode");
    assert!(mode_count > 0, "admitted Nix producer input mode count must be positive");
    if let Some(path) = derivation_json_path {
        return read_nix_derivation_json_closure(path);
    }
    if let Some(dir) = drv_dir {
        return read_nix_aterm_drv_dir_closure(dir, root_derivation);
    }
    read_nix_aterm_drv_closure(drv_file_specs)
}

fn read_nix_derivation_json_closure(
    derivation_json_path: &Path,
) -> Result<Result<crate::foreign_derivation_import::NixDerivationJsonClosure, ForeignImportCliReport>, RunError> {
    let json_input = match read_json::<NixDerivationJsonExport>(JsonReadRequest {
        path: derivation_json_path,
        artifact: "derivation-json",
        command: PRODUCE_NIX_COMMAND,
    })? {
        Ok(json_input) => json_input,
        Err(report) => return Ok(Err(report)),
    };
    Ok(match normalize_nix_derivation_json_export(json_input) {
        Ok(closure) => Ok(closure),
        Err(diagnostic) => Err(rejected_report(PRODUCE_NIX_COMMAND, diagnostic)),
    })
}

fn read_nix_aterm_drv_closure(
    drv_file_specs: &[String],
) -> Result<Result<crate::foreign_derivation_import::NixDerivationJsonClosure, ForeignImportCliReport>, RunError> {
    assert!(!drv_file_specs.is_empty(), "explicit ATerm closure requires at least one drv file");
    assert!(!NIX_LOGICAL_STORE_PREFIX.is_empty(), "Nix logical store prefix must not be empty");
    let mut derivations = BTreeMap::new();
    for spec in drv_file_specs {
        let (logical_path, file_path) = match parse_drv_file_spec(spec) {
            Ok(pair) => pair,
            Err(diagnostic) => return Ok(Err(rejected_report(PRODUCE_NIX_COMMAND, diagnostic))),
        };
        let derivation = match parse_nix_aterm_drv_file(&logical_path, &file_path) {
            Ok(derivation) => derivation,
            Err(report) => return Ok(Err(report)),
        };
        if derivations.insert(logical_path, derivation).is_some() {
            return Ok(Err(rejected_report(
                PRODUCE_NIX_COMMAND,
                diagnostic("duplicate-nix-derivation-path", None, "duplicate --drv logical derivation path"),
            )));
        }
    }
    normalize_nix_aterm_drv_map(derivations)
}

fn read_nix_aterm_drv_dir_closure(
    drv_dir: &Path,
    root_derivation: &str,
) -> Result<Result<crate::foreign_derivation_import::NixDerivationJsonClosure, ForeignImportCliReport>, RunError> {
    let derivations = match read_nix_aterm_drv_dir(drv_dir)? {
        Ok(derivations) => derivations,
        Err(report) => return Ok(Err(report)),
    };
    let closure = match normalize_nix_aterm_drv_map(derivations)? {
        Ok(closure) => closure,
        Err(report) => return Ok(Err(report)),
    };
    Ok(match select_nix_derivation_json_closure(&closure, root_derivation) {
        Ok(selected) => Ok(selected),
        Err(diagnostic) => Err(rejected_report(PRODUCE_NIX_COMMAND, diagnostic)),
    })
}

fn read_nix_aterm_drv_dir(
    drv_dir: &Path,
) -> Result<Result<BTreeMap<String, nix_compat::derivation::Derivation>, ForeignImportCliReport>, RunError> {
    assert!(!NIX_LOGICAL_STORE_PREFIX.is_empty(), "Nix logical store prefix must not be empty");
    assert!(!DRV_FILE_EXTENSION.is_empty(), "ATerm drv extension must not be empty");
    let mut entries = fs::read_dir(drv_dir)
        .map_err(|error| RunError::Internal(format!("reading foreign import drv dir {}: {error}", drv_dir.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            RunError::Internal(format!("reading foreign import drv dir {}: {error}", drv_dir.display()))
        })?;
    entries.sort_by_key(|entry| entry.path());
    let derivation_count_max = entries.len();
    let mut derivations = BTreeMap::new();
    for entry in entries {
        let file_path = entry.path();
        if file_path.extension().and_then(|extension| extension.to_str()) != Some(DRV_FILE_EXTENSION) {
            continue;
        }
        let Some(file_name) = file_path.file_name().and_then(|name| name.to_str()) else {
            return Ok(Err(rejected_report(
                PRODUCE_NIX_COMMAND,
                diagnostic("non-utf8-nix-derivation-filename", None, "drv directory contains non-UTF-8 filename"),
            )));
        };
        let logical_path = format!("{NIX_LOGICAL_STORE_PREFIX}/{file_name}");
        let derivation = match parse_nix_aterm_drv_file(&logical_path, &file_path) {
            Ok(derivation) => derivation,
            Err(report) => return Ok(Err(report)),
        };
        if derivations.len() >= derivation_count_max {
            return Err(RunError::Internal("ATerm derivation count exceeded directory entries".to_string()));
        }
        derivations.insert(logical_path, derivation);
    }
    Ok(Ok(derivations))
}

fn normalize_nix_aterm_drv_map(
    derivations: BTreeMap<String, nix_compat::derivation::Derivation>,
) -> Result<Result<crate::foreign_derivation_import::NixDerivationJsonClosure, ForeignImportCliReport>, RunError> {
    Ok(match normalize_nix_aterm_derivation_closure(derivations) {
        Ok(closure) => Ok(closure),
        Err(diagnostic) => Err(rejected_report(PRODUCE_NIX_COMMAND, diagnostic)),
    })
}

fn parse_nix_aterm_drv_file(
    logical_path: &str,
    file_path: &Path,
) -> Result<nix_compat::derivation::Derivation, ForeignImportCliReport> {
    let bytes = fs::read(file_path).map_err(|error| {
        rejected_report(
            PRODUCE_NIX_COMMAND,
            diagnostic("unreadable-nix-derivation-file", None, &format!("reading {}: {error}", file_path.display())),
        )
    })?;
    nix_compat::derivation::Derivation::from_aterm_bytes(&bytes).map_err(|error| {
        rejected_report(
            PRODUCE_NIX_COMMAND,
            diagnostic("malformed-nix-derivation", None, &format!("{logical_path}: {error:?}")),
        )
    })
}

fn parse_drv_file_spec(spec: &str) -> Result<(String, PathBuf), ImportDiagnostic> {
    let Some((logical_path, file_path)) = spec.split_once(DRV_SPEC_SEPARATOR) else {
        return Err(diagnostic(
            "malformed-nix-derivation-input",
            None,
            "--drv must be formatted as /nix/store/name.drv=/path/to/file.drv",
        ));
    };
    if logical_path.is_empty() || file_path.is_empty() {
        return Err(diagnostic(
            "malformed-nix-derivation-input",
            None,
            "--drv logical path and file path must both be non-empty",
        ));
    }
    Ok((logical_path.to_string(), PathBuf::from(file_path)))
}

fn validate_inputs(
    graph: ForeignDerivationGraph,
    index: Option<PackageIndex>,
    policy: TranslationPolicy,
    receipt: Option<ImportReceipt>,
) -> ForeignImportCliReport {
    let translated = match translate_foreign_graph(&graph, index.as_ref(), &policy) {
        Ok((_, receipt)) => receipt,
        Err(diagnostic) => return rejected_report(VALIDATE_COMMAND, diagnostic),
    };
    if let Some(receipt) = receipt.as_ref()
        && let Err(diagnostic) = admit_translated_graph(&graph, index.as_ref(), &policy, receipt)
    {
        return rejected_report(VALIDATE_COMMAND, diagnostic);
    }
    accepted_report(VALIDATE_COMMAND, Some(translated), None)
}

fn plan_inputs(
    graph: ForeignDerivationGraph,
    index: PackageIndex,
    package: &str,
    policy: TranslationPolicy,
    system: &str,
) -> ForeignImportCliReport {
    let (plan, receipt) = match compile_foreign_executable_plan(&graph, &index, &policy, package, system) {
        Ok(result) => result,
        Err(diagnostic) => return rejected_report(PLAN_COMMAND, diagnostic),
    };
    accepted_report(PLAN_COMMAND, Some(receipt), Some(plan))
}

fn plan_inputs_with_profile(
    graph: ForeignDerivationGraph,
    index: PackageIndex,
    package: &str,
    policy: TranslationPolicy,
    system: &str,
    execution_profile: &ExecutionProfile,
) -> ForeignImportCliReport {
    let (plan, receipt) =
        match compile_foreign_executable_plan_with_profile(&graph, &index, &policy, package, system, execution_profile)
        {
            Ok(result) => result,
            Err(diagnostic) => return rejected_report(PLAN_COMMAND, diagnostic),
        };
    accepted_report(PLAN_COMMAND, Some(receipt), Some(plan))
}

fn read_optional_index(
    path: Option<&Path>,
    command: &str,
) -> Result<Result<Option<PackageIndex>, ForeignImportCliReport>, RunError> {
    let Some(path) = path else {
        return Ok(Ok(None));
    };
    read_json::<PackageIndex>(JsonReadRequest {
        path,
        artifact: "package-index",
        command,
    })
    .map(|result| result.map(Some))
}

fn read_optional_receipt(
    path: Option<&Path>,
    command: &str,
) -> Result<Result<Option<ImportReceipt>, ForeignImportCliReport>, RunError> {
    let Some(path) = path else {
        return Ok(Ok(None));
    };
    read_json::<ImportReceipt>(JsonReadRequest {
        path,
        artifact: "receipt",
        command,
    })
    .map(|result| result.map(Some))
}

fn read_json<T: DeserializeOwned>(request: JsonReadRequest<'_>) -> Result<Result<T, ForeignImportCliReport>, RunError> {
    read_json_with_limit(request, DEFAULT_FOREIGN_JSON_ARTIFACT_BYTES_MAX)
}

fn read_json_with_limit<T: DeserializeOwned>(
    request: JsonReadRequest<'_>,
    max_bytes: u64,
) -> Result<Result<T, ForeignImportCliReport>, RunError> {
    assert!(max_bytes > 0, "foreign JSON byte limit must be positive");
    let read_limit = max_bytes
        .checked_add(1)
        .ok_or_else(|| RunError::Internal("foreign JSON byte limit overflowed".to_string()))?;
    let file = fs::File::open(request.path).map_err(|error| {
        RunError::Internal(format!("opening foreign import {} {}: {error}", request.artifact, request.path.display()))
    })?;
    let mut contents = Vec::new();
    file.take(read_limit).read_to_end(&mut contents).map_err(|error| {
        RunError::Internal(format!("reading foreign import {} {}: {error}", request.artifact, request.path.display()))
    })?;
    if u64::try_from(contents.len()).unwrap_or(u64::MAX) > max_bytes {
        return Ok(Err(rejected_report(
            request.command,
            diagnostic(
                "foreign-json-bytes-out-of-range",
                None,
                &format!("{} exceeds {max_bytes} bytes", request.artifact),
            ),
        )));
    }
    match serde_json::from_slice::<T>(&contents) {
        Ok(value) => Ok(Ok(value)),
        Err(error) => Ok(Err(rejected_report(
            request.command,
            diagnostic("malformed-json", None, &format!("{}: {error}", request.artifact)),
        ))),
    }
}

fn accepted_report(
    command: &str,
    receipt: Option<ImportReceipt>,
    plan: Option<ForeignExecutablePlan>,
) -> ForeignImportCliReport {
    ForeignImportCliReport {
        schema: CLI_REPORT_SCHEMA.to_string(),
        command: command.to_string(),
        verdict: ACCEPTED_VERDICT.to_string(),
        accepted: true,
        diagnostics: Vec::new(),
        receipt,
        plan,
        producer_artifacts: None,
        non_claims: foreign_import_non_claims(),
    }
}

fn producer_report(command: &str, graph_path: &Path, index_path: &Path) -> ForeignImportCliReport {
    ForeignImportCliReport {
        schema: CLI_REPORT_SCHEMA.to_string(),
        command: command.to_string(),
        verdict: ACCEPTED_VERDICT.to_string(),
        accepted: true,
        diagnostics: Vec::new(),
        receipt: None,
        plan: None,
        producer_artifacts: Some(ProducerArtifactReport {
            graph_path: graph_path.display().to_string(),
            package_index_path: index_path.display().to_string(),
        }),
        non_claims: foreign_import_non_claims(),
    }
}

fn rejected_report(command: &str, diagnostic: ImportDiagnostic) -> ForeignImportCliReport {
    ForeignImportCliReport {
        schema: CLI_REPORT_SCHEMA.to_string(),
        command: command.to_string(),
        verdict: REJECTED_VERDICT.to_string(),
        accepted: false,
        diagnostics: vec![diagnostic],
        receipt: None,
        plan: None,
        producer_artifacts: None,
        non_claims: foreign_import_non_claims(),
    }
}

fn emit_report(report: ForeignImportCliReport, json: bool) -> Result<(), RunError> {
    if json {
        println!("{}", serialize_report(&report)?);
    } else {
        println!("{}", human_report(&report));
    }
    if report.accepted {
        Ok(())
    } else {
        Err(RunError::Reported(FAILURE_EXIT_CODE))
    }
}

fn serialize_report(report: &ForeignImportCliReport) -> Result<String, RunError> {
    serde_json::to_string_pretty(report)
        .map_err(|error| RunError::Internal(format!("{JSON_SERIALIZATION_CONTEXT}: {error}")))
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), RunError> {
    let contents = serde_json::to_string_pretty(value)
        .map_err(|error| RunError::Internal(format!("{JSON_ARTIFACT_SERIALIZATION_CONTEXT}: {error}")))?;
    fs::write(path, contents)
        .map_err(|error| RunError::Internal(format!("writing foreign import artifact {}: {error}", path.display())))
}

fn human_report(report: &ForeignImportCliReport) -> String {
    assert_eq!(report.schema, CLI_REPORT_SCHEMA, "human output requires the CLI report schema");
    assert!(!report.command.is_empty(), "human output requires a command identity");
    let line_count_max = report.diagnostics.len().saturating_add(report.non_claims.len()).saturating_add(5);
    let mut lines = Vec::with_capacity(line_count_max);
    lines.push(format!("foreign import {}: {}", report.command, report.verdict));
    if report.diagnostics.is_empty() {
        lines.push("diagnostics: none".to_string());
    } else {
        lines.push("diagnostics:".to_string());
        for diagnostic in &report.diagnostics {
            lines.push(format!("- {}: {}", diagnostic.class, diagnostic.message));
        }
    }
    if let Some(artifacts) = report.producer_artifacts.as_ref() {
        lines.push("producer artifacts:".to_string());
        lines.push(format!("- graph: {}", artifacts.graph_path));
        lines.push(format!("- package-index: {}", artifacts.package_index_path));
    }
    lines.push("non-claims:".to_string());
    for non_claim in &report.non_claims {
        lines.push(format!("- {non_claim}"));
    }
    lines.join("\n")
}

fn diagnostic(class: &str, node_id: Option<&str>, message: &str) -> ImportDiagnostic {
    ImportDiagnostic {
        class: class.to_string(),
        node_id: node_id.map(ToOwned::to_owned),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn human_report_preserves_diagnostics_and_non_claims() {
        let report = rejected_report(VALIDATE_COMMAND, diagnostic("malformed-json", None, "graph: invalid"));

        let human = human_report(&report);

        assert!(human.contains("foreign import validate: rejected"));
        assert!(human.contains("malformed-json"));
        assert!(human.contains("non-claims"));
        assert!(human.contains("not-build-success"));
    }

    #[test]
    fn plan_inputs_returns_receipt_and_plan_without_process_invocations() {
        let (graph, index) = crate::foreign_derivation_import::guix_like_hello_fixture();
        let policy = fixture_policy();

        let report = plan_inputs(graph, index, DEFAULT_PACKAGE_NAME, policy, DEFAULT_SYSTEM);
        let plan = report
            .plan
            .as_ref()
            .unwrap_or_else(|| panic!("plan should be present; diagnostics={:?}", report.diagnostics));

        assert!(report.accepted);
        assert_eq!(report.verdict, ACCEPTED_VERDICT);
        assert!(report.receipt.is_some());
        assert!(plan.forbidden_process_invocations.is_empty());
        assert!(plan.non_claims.contains(&"not-build-success".to_string()));
    }

    fn fixture_policy() -> TranslationPolicy {
        let mut builtin_mappings = std::collections::BTreeMap::new();
        builtin_mappings.insert("fixed-output-fetch".to_string(), "mantle.fetch".to_string());
        TranslationPolicy {
            source_prefixes: vec!["/gnu/store".to_string(), "/nix/store".to_string()],
            target_prefix: "/mantle/store".to_string(),
            rewrite_builder: true,
            rewrite_args: true,
            rewrite_env: true,
            rewrite_sources: true,
            rewrite_declared_references: true,
            allow_embedded_source_payload_rewrite: false,
            builtin_mappings,
            output_path_recompute_mode: "recompute-blake3-v1".to_string(),
            trusted_cache_scopes: std::collections::BTreeSet::new(),
            allowed_sandbox_capabilities: std::collections::BTreeSet::new(),
        }
    }

    #[test]
    fn cache_only_plan_trust_keys_are_loaded_from_bound_cache_urls() {
        const CACHE_KEY: &str = "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=";
        let (graph, index) = crate::foreign_derivation_import::guix_like_hello_fixture();
        let report = plan_inputs(graph, index, DEFAULT_PACKAGE_NAME, fixture_policy(), DEFAULT_SYSTEM);
        let mut plan = report.plan.unwrap();
        plan.realization_route = Some(crate::foreign_executable_plan::CACHE_ONLY_PRESERVE_ROUTE.to_string());
        plan.substitution_audit = vec![crate::foreign_derivation_import::SubstitutionAuditEvent {
            node_id: plan.native_units[0].node_id.clone(),
            cache_url: format!("https://cache.nixos.org?trusted_public_keys[0]={CACHE_KEY}"),
            trust_scope: "trusted-binary-cache".to_string(),
            classification: "trusted-substitution-hint".to_string(),
            store_admission_required: true,
        }];
        let keys = trusted_cache_keys_from_plan(&plan).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].to_string(), CACHE_KEY);

        plan.substitution_audit[0].cache_url = "https://cache.nixos.org".to_string();
        assert!(trusted_cache_keys_from_plan(&plan).is_err());
    }

    #[test]
    fn bounded_json_reader_accepts_an_artifact_at_the_limit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("artifact.json");
        let contents = br#"{"value":7}"#;
        fs::write(&path, contents).unwrap();

        let value = read_json_with_limit::<serde_json::Value>(
            JsonReadRequest {
                path: &path,
                artifact: "fixture",
                command: VALIDATE_COMMAND,
            },
            u64::try_from(contents.len()).unwrap(),
        )
        .unwrap()
        .unwrap();

        assert_eq!(value["value"], 7);
    }

    #[test]
    fn bounded_json_reader_rejects_an_artifact_above_the_limit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("artifact.json");
        let contents = br#"{"value":7}"#;
        fs::write(&path, contents).unwrap();
        let max_bytes = u64::try_from(contents.len().saturating_sub(1)).unwrap();

        let report = read_json_with_limit::<serde_json::Value>(
            JsonReadRequest {
                path: &path,
                artifact: "fixture",
                command: VALIDATE_COMMAND,
            },
            max_bytes,
        )
        .unwrap()
        .unwrap_err();

        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].class, "foreign-json-bytes-out-of-range");
        assert!(report.diagnostics[0].message.contains(&max_bytes.to_string()));
    }

    #[test]
    fn malformed_report_is_rejected() {
        let report = rejected_report(VALIDATE_COMMAND, diagnostic("malformed-json", None, VALID_DIGEST));

        assert!(!report.accepted);
        assert_eq!(report.diagnostics[0].class, "malformed-json");
        assert!(report.receipt.is_none());
        assert!(report.plan.is_none());
    }
}
