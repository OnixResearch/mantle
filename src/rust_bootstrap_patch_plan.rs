//! Pure Rust bootstrap patch-plan core.
//!
//! This module decides *which* compiler-bootstrap repair operations are needed
//! from explicit route/source/compiler facts. It does not read files, inspect
//! environment, spawn processes, observe clocks, or mutate state. The provider
//! materializer is the imperative shell that applies returned operations.

use serde::Deserialize;
use serde::Serialize;

pub(crate) const RUST_BOOTSTRAP_PATCH_PLAN_SCHEMA: &str = "mantle-rust-bootstrap-patch-plan-v1";
pub(crate) const MUSL_COMPILER_HOST_TRIPLE: &str = "x86_64-unknown-linux-musl";
#[cfg(test)]
pub(crate) const GNU_COMPILER_HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu";
pub(crate) const SUPPORTED_MRUSTC_VERSION: &str = "0.12.0";
pub(crate) const SUPPORTED_FIRST_STAGE_RUST_VERSION: &str = "1.90.0";
pub(crate) const SUPPORTED_RUST_BOOTSTRAP_VERSIONS: [&str; 4] = ["1.91.1", "1.92.0", "1.93.1", "1.94.0"];

const DIGEST_HEX_CHAR_COUNT: usize = 64;
const PATCH_PLAN_MAX_SOURCE_IDENTITIES: usize = 16;
const PATCH_PLAN_MAX_OPERATIONS: usize = 64;
const PATCH_PLAN_BASE_RECEIPT_ARGUMENTS: usize = 10;
const FIRST_STAGE_MUSL_REPAIR_OPERATION_COUNT: usize = 6;
const RUST_BOOTSTRAP_REPAIR_OPERATION_COUNT: usize = 5;
const OPERATION_LABEL_SEPARATOR: &str = ":";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustBootstrapPatchPlanInput {
    pub(crate) stage: RustBootstrapPatchStage,
    pub(crate) route_id: String,
    pub(crate) route_plan_digest_blake3: String,
    pub(crate) route_policy_digest_blake3: String,
    pub(crate) stage_id: String,
    pub(crate) rust_version: String,
    pub(crate) mrustc_version: Option<String>,
    pub(crate) host_triple: String,
    pub(crate) target_triple: String,
    pub(crate) source_identities: Vec<RustBootstrapPatchSourceIdentity>,
    pub(crate) capabilities: RustBootstrapPatchCapabilities,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustBootstrapPatchSourceIdentity {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) digest_kind: String,
    pub(crate) digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustBootstrapPatchCapabilities {
    pub(crate) source_built_provider: bool,
    pub(crate) proc_macro_runtime: bool,
    pub(crate) static_executable_linking: bool,
    pub(crate) dynamic_tool_runtime: bool,
    pub(crate) rustdoc_tool: bool,
    pub(crate) provider_contract_version: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RustBootstrapPatchStage {
    FirstStage,
    RustBootstrap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RustBootstrapPatchPhase {
    BeforeFirstStageMainMake,
    AfterFirstStageMinicargo,
    AfterFirstStageRustSourceExtract,
    AfterFirstStageTranslatedCargo,
    FirstStageRunRustcHost,
    FirstStageRunRustcTarget,
    RustBootstrapBeforeXpy,
    ProviderEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RustBootstrapPatchOperationKind {
    MinicargoBuildOutDir,
    MinicargoRustcThreads,
    MinicargoLlvmStaticArchiveTargets,
    SourceRootMuslLlvmRuntime,
    RustExplicitSysroot,
    RustcDriverRlib,
    RunRustcHostRuntime,
    RunRustcTargetRustlib,
    RustBootstrapTargetToolConfig,
    RustBootstrapWorkspaceIsolation,
    RustBootstrapRustcDriverRlib,
    RustBootstrapSysrootFallback,
    RustBootstrapRustcPrivateToolRlibLookup,
    ProviderContractAssertion,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustBootstrapPatchOperation {
    pub(crate) id: String,
    pub(crate) kind: RustBootstrapPatchOperationKind,
    pub(crate) phase: RustBootstrapPatchPhase,
    pub(crate) summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) expected_anchor: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustBootstrapPatchPlan {
    pub(crate) schema: String,
    pub(crate) input_digest_blake3: String,
    pub(crate) output_digest_blake3: String,
    pub(crate) operation_count: u32,
    pub(crate) input: RustBootstrapPatchPlanInput,
    pub(crate) operations: Vec<RustBootstrapPatchOperation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RustBootstrapPatchPlanError {
    message: String,
}

impl RustBootstrapPatchPlanError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for RustBootstrapPatchPlanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RustBootstrapPatchPlanError {}

impl RustBootstrapPatchPlan {
    pub(crate) fn receipt_arguments(&self) -> Vec<String> {
        let argument_capacity_entries = PATCH_PLAN_BASE_RECEIPT_ARGUMENTS.saturating_add(self.operations.len());
        let mut arguments = Vec::with_capacity(argument_capacity_entries);
        arguments.push(format!("schema={}", self.schema));
        arguments.push(format!("stage={:?}", self.input.stage));
        arguments.push(format!("stage-id={}", self.input.stage_id));
        arguments.push(format!("route={}", self.input.route_id));
        arguments.push(format!("host={}", self.input.host_triple));
        arguments.push(format!("target={}", self.input.target_triple));
        arguments.push(format!("rust-version={}", self.input.rust_version));
        if let Some(mrustc_version) = &self.input.mrustc_version {
            arguments.push(format!("mrustc-version={mrustc_version}"));
        }
        arguments.push(format!("input-digest={}", self.input_digest_blake3));
        arguments.push(format!("output-digest={}", self.output_digest_blake3));
        arguments.push(format!("operation-count={}", self.operation_count));
        for operation in &self.operations {
            arguments.push(operation.receipt_argument());
        }
        arguments
    }

    #[cfg(test)]
    pub(crate) fn contains_operation(&self, kind: RustBootstrapPatchOperationKind) -> bool {
        self.operations.iter().any(|operation| operation.kind == kind)
    }
}

impl RustBootstrapPatchOperation {
    pub(crate) fn receipt_argument(&self) -> String {
        let mut value = format!(
            "operation={}{}{:?}{}{}",
            self.id, OPERATION_LABEL_SEPARATOR, self.kind, OPERATION_LABEL_SEPARATOR, self.summary
        );
        if let Some(source_id) = &self.source_id {
            value.push_str(OPERATION_LABEL_SEPARATOR);
            value.push_str("source=");
            value.push_str(source_id);
        }
        value
    }
}

pub(crate) fn derive_rust_bootstrap_patch_plan(
    input: RustBootstrapPatchPlanInput,
) -> Result<RustBootstrapPatchPlan, RustBootstrapPatchPlanError> {
    debug_assert!(PATCH_PLAN_MAX_OPERATIONS > 0);
    debug_assert!(PATCH_PLAN_MAX_SOURCE_IDENTITIES > 0);
    validate_plan_input(&input)?;
    let operations = derive_operations(&input)?;
    validate_operation_count(&operations)?;
    let input_digest_blake3 = digest_json(&input, "patch-plan input")?;
    let output_digest_blake3 = digest_json(
        &RustBootstrapPatchPlanOutputDigest {
            schema: RUST_BOOTSTRAP_PATCH_PLAN_SCHEMA,
            input_digest_blake3: &input_digest_blake3,
            operations: &operations,
        },
        "patch-plan output",
    )?;
    let plan = RustBootstrapPatchPlan {
        schema: RUST_BOOTSTRAP_PATCH_PLAN_SCHEMA.to_string(),
        operation_count: operations
            .len()
            .try_into()
            .map_err(|_| RustBootstrapPatchPlanError::new("patch-plan operation count overflow"))?,
        input_digest_blake3,
        output_digest_blake3,
        input,
        operations,
    };
    debug_assert_eq!(usize::try_from(plan.operation_count).ok(), Some(plan.operations.len()));
    debug_assert!(plan.operations.len() <= PATCH_PLAN_MAX_OPERATIONS);
    Ok(plan)
}

#[derive(Serialize)]
struct RustBootstrapPatchPlanOutputDigest<'a> {
    schema: &'a str,
    input_digest_blake3: &'a str,
    operations: &'a [RustBootstrapPatchOperation],
}

fn validate_plan_input(input: &RustBootstrapPatchPlanInput) -> Result<(), RustBootstrapPatchPlanError> {
    require_non_empty("route_id", &input.route_id)?;
    require_non_empty("stage_id", &input.stage_id)?;
    require_non_empty("rust_version", &input.rust_version)?;
    require_non_empty("host_triple", &input.host_triple)?;
    require_non_empty("target_triple", &input.target_triple)?;
    require_digest("route_plan_digest_blake3", &input.route_plan_digest_blake3)?;
    require_digest("route_policy_digest_blake3", &input.route_policy_digest_blake3)?;
    require_non_empty("provider_contract_version", &input.capabilities.provider_contract_version)?;
    if !input.capabilities.source_built_provider {
        return Err(RustBootstrapPatchPlanError::new("patch-plan input must describe a source-built provider"));
    }
    validate_sources(&input.source_identities)?;
    validate_versions(input)?;
    Ok(())
}

fn validate_sources(sources: &[RustBootstrapPatchSourceIdentity]) -> Result<(), RustBootstrapPatchPlanError> {
    if sources.is_empty() {
        return Err(RustBootstrapPatchPlanError::new("patch-plan input has no source identities"));
    }
    if sources.len() > PATCH_PLAN_MAX_SOURCE_IDENTITIES {
        return Err(RustBootstrapPatchPlanError::new(format!(
            "patch-plan input has more than {PATCH_PLAN_MAX_SOURCE_IDENTITIES} source identities"
        )));
    }
    for source in sources {
        require_non_empty("source id", &source.id)?;
        require_non_empty("source name", &source.name)?;
        require_non_empty("source digest kind", &source.digest_kind)?;
        require_digest("source digest", &source.digest)?;
    }
    Ok(())
}

fn validate_versions(input: &RustBootstrapPatchPlanInput) -> Result<(), RustBootstrapPatchPlanError> {
    match input.stage {
        RustBootstrapPatchStage::FirstStage => validate_first_stage_versions(input),
        RustBootstrapPatchStage::RustBootstrap => validate_rust_bootstrap_versions(input),
    }
}

fn validate_first_stage_versions(input: &RustBootstrapPatchPlanInput) -> Result<(), RustBootstrapPatchPlanError> {
    require_equal("first-stage Rust version", &input.rust_version, SUPPORTED_FIRST_STAGE_RUST_VERSION)?;
    let Some(mrustc_version) = &input.mrustc_version else {
        return Err(RustBootstrapPatchPlanError::new("first-stage patch plan requires an mrustc version"));
    };
    require_equal("first-stage mrustc version", mrustc_version, SUPPORTED_MRUSTC_VERSION)?;
    require_source_id(input, &format!("rust-{}", SUPPORTED_FIRST_STAGE_RUST_VERSION))?;
    require_source_id(input, &format!("mrustc-{SUPPORTED_MRUSTC_VERSION}"))
}

fn validate_rust_bootstrap_versions(input: &RustBootstrapPatchPlanInput) -> Result<(), RustBootstrapPatchPlanError> {
    if !SUPPORTED_RUST_BOOTSTRAP_VERSIONS.contains(&input.rust_version.as_str()) {
        return Err(RustBootstrapPatchPlanError::new(format!(
            "unsupported Rust bootstrap version {}",
            input.rust_version
        )));
    }
    require_source_id(input, &format!("rust-{}", input.rust_version))
}

fn derive_operations(
    input: &RustBootstrapPatchPlanInput,
) -> Result<Vec<RustBootstrapPatchOperation>, RustBootstrapPatchPlanError> {
    let mut operations = Vec::with_capacity(PATCH_PLAN_MAX_OPERATIONS);
    match input.stage {
        RustBootstrapPatchStage::FirstStage => derive_first_stage_operations(input, &mut operations)?,
        RustBootstrapPatchStage::RustBootstrap => derive_rust_bootstrap_operations(input, &mut operations)?,
    }
    operations.push(provider_contract_assertion_operation(input));
    Ok(operations)
}

fn derive_first_stage_operations(
    input: &RustBootstrapPatchPlanInput,
    operations: &mut Vec<RustBootstrapPatchOperation>,
) -> Result<(), RustBootstrapPatchPlanError> {
    if is_musl_source_route(input) {
        require_musl_capabilities(input)?;
        derive_first_stage_musl_repair_operations(operations);
    }
    derive_first_stage_run_operations(operations);
    Ok(())
}

struct PatchOperationInput<'a> {
    id: &'a str,
    kind: RustBootstrapPatchOperationKind,
    phase: RustBootstrapPatchPhase,
    summary: &'a str,
    source_id: Option<&'a str>,
    expected_anchor: Option<&'a str>,
}

macro_rules! patch_operation {
    ($id:expr, $kind:expr, $phase:expr, $summary:expr, $source_id:expr, $expected_anchor:expr $(,)?) => {
        operation(PatchOperationInput {
            id: $id,
            kind: $kind,
            phase: $phase,
            summary: $summary,
            source_id: $source_id,
            expected_anchor: $expected_anchor,
        })
    };
}

fn derive_first_stage_musl_repair_operations(operations: &mut Vec<RustBootstrapPatchOperation>) {
    let initial_operation_count = operations.len();
    debug_assert!(initial_operation_count <= PATCH_PLAN_MAX_OPERATIONS);
    operations.push(patch_operation!(
        "first-stage-minicargo-out-dir",
        RustBootstrapPatchOperationKind::MinicargoBuildOutDir,
        RustBootstrapPatchPhase::BeforeFirstStageMainMake,
        "share generated build-script outputs with target crate compiles",
        Some(&format!("mrustc-{SUPPORTED_MRUSTC_VERSION}")),
        Some("tools/minicargo/build.cpp OUT_DIR assignment"),
    ));
    operations.push(patch_operation!(
        "first-stage-minicargo-rustc-threads",
        RustBootstrapPatchOperationKind::MinicargoRustcThreads,
        RustBootstrapPatchPhase::BeforeFirstStageMainMake,
        "limit temporary static compiler worker threads",
        Some(&format!("mrustc-{SUPPORTED_MRUSTC_VERSION}")),
        Some("tools/minicargo/build.cpp rustc argument construction"),
    ));
    operations.push(patch_operation!(
        "first-stage-llvm-static-archives",
        RustBootstrapPatchOperationKind::MinicargoLlvmStaticArchiveTargets,
        RustBootstrapPatchPhase::BeforeFirstStageMainMake,
        "disable LLVM shared probes and request generated headers plus static archives",
        Some(&format!("mrustc-{SUPPORTED_MRUSTC_VERSION}")),
        Some("minicargo.mk LLVM CMake configuration"),
    ));
    operations.push(patch_operation!(
        "first-stage-source-root-musl-runtime",
        RustBootstrapPatchOperationKind::SourceRootMuslLlvmRuntime,
        RustBootstrapPatchPhase::AfterFirstStageMinicargo,
        "prepare source-root musl compiler runtime for LLVM and helper links",
        Some(&format!("mrustc-{SUPPORTED_MRUSTC_VERSION}")),
        Some("source-root musl target toolchain"),
    ));
    operations.push(patch_operation!(
        "first-stage-rust-explicit-sysroot",
        RustBootstrapPatchOperationKind::RustExplicitSysroot,
        RustBootstrapPatchPhase::AfterFirstStageRustSourceExtract,
        "keep explicit sysroot from probing dynamic compiler location",
        Some(&format!("rust-{SUPPORTED_FIRST_STAGE_RUST_VERSION}")),
        Some("compiler/rustc_session/src/config.rs Sysroot::new"),
    ));
    operations.push(patch_operation!(
        "first-stage-rustc-driver-rlib",
        RustBootstrapPatchOperationKind::RustcDriverRlib,
        RustBootstrapPatchPhase::AfterFirstStageTranslatedCargo,
        "build rustc_driver as rlib for temporary static compiler linking",
        Some(&format!("rust-{SUPPORTED_FIRST_STAGE_RUST_VERSION}")),
        Some("compiler/rustc_driver/Cargo.toml crate-type"),
    ));
    debug_assert_eq!(operations.len(), initial_operation_count.saturating_add(FIRST_STAGE_MUSL_REPAIR_OPERATION_COUNT));
}

fn derive_first_stage_run_operations(operations: &mut Vec<RustBootstrapPatchOperation>) {
    operations.push(patch_operation!(
        "first-stage-run-rustc-host-runtime",
        RustBootstrapPatchOperationKind::RunRustcHostRuntime,
        RustBootstrapPatchPhase::FirstStageRunRustcHost,
        "build host compiler prefix with route runtime policy",
        Some(&format!("mrustc-{SUPPORTED_MRUSTC_VERSION}")),
        Some("run_rustc/Makefile host compiler rules"),
    ));
    operations.push(patch_operation!(
        "first-stage-run-rustc-target",
        RustBootstrapPatchOperationKind::RunRustcTargetRustlib,
        RustBootstrapPatchPhase::FirstStageRunRustcTarget,
        "build provider target rustlib after host compiler prefix",
        Some(&format!("mrustc-{SUPPORTED_MRUSTC_VERSION}")),
        Some("run_rustc target rustlib rule"),
    ));
}

fn derive_rust_bootstrap_operations(
    input: &RustBootstrapPatchPlanInput,
    operations: &mut Vec<RustBootstrapPatchOperation>,
) -> Result<(), RustBootstrapPatchPlanError> {
    if !is_musl_source_route(input) {
        return Ok(());
    }
    require_musl_capabilities(input)?;
    let initial_operation_count = operations.len();
    debug_assert!(initial_operation_count <= PATCH_PLAN_MAX_OPERATIONS);
    operations.push(patch_operation!(
        "rust-bootstrap-target-tool-config",
        RustBootstrapPatchOperationKind::RustBootstrapTargetToolConfig,
        RustBootstrapPatchPhase::RustBootstrapBeforeXpy,
        "bind source-root target tools and runtime paths in Rust bootstrap config",
        Some(&format!("rust-{}", input.rust_version)),
        Some("Rust bootstrap target tool configuration"),
    ));
    operations.push(patch_operation!(
        "rust-bootstrap-workspace-isolation",
        RustBootstrapPatchOperationKind::RustBootstrapWorkspaceIsolation,
        RustBootstrapPatchPhase::RustBootstrapBeforeXpy,
        "keep optional compiler workspaces outside the bootstrap member set",
        Some(&format!("rust-{}", input.rust_version)),
        Some("src/bootstrap/Cargo.toml workspace members"),
    ));
    operations.push(patch_operation!(
        "rust-bootstrap-rustc-driver-rlib",
        RustBootstrapPatchOperationKind::RustBootstrapRustcDriverRlib,
        RustBootstrapPatchPhase::RustBootstrapBeforeXpy,
        "build rustc_driver as rlib for static musl compiler host",
        Some(&format!("rust-{}", input.rust_version)),
        Some("compiler/rustc_driver/Cargo.toml crate-type"),
    ));
    operations.push(patch_operation!(
        "rust-bootstrap-sysroot-fallback",
        RustBootstrapPatchOperationKind::RustBootstrapSysrootFallback,
        RustBootstrapPatchPhase::RustBootstrapBeforeXpy,
        "prefer explicit sysroot environment before dynamic compiler fallback",
        Some(&format!("rust-{}", input.rust_version)),
        Some("compiler/rustc_session/src/filesearch.rs default sysroot"),
    ));
    operations.push(patch_operation!(
        "rust-bootstrap-rustc-private-tool-rlibs",
        RustBootstrapPatchOperationKind::RustBootstrapRustcPrivateToolRlibLookup,
        RustBootstrapPatchPhase::RustBootstrapBeforeXpy,
        "teach rustc-private tools where stage2 compiler rlibs live",
        Some(&format!("rust-{}", input.rust_version)),
        Some("src/bootstrap/src/core/build_steps/tool.rs ToolBuild cargo"),
    ));
    debug_assert_eq!(operations.len(), initial_operation_count.saturating_add(RUST_BOOTSTRAP_REPAIR_OPERATION_COUNT));
    Ok(())
}

fn require_musl_capabilities(input: &RustBootstrapPatchPlanInput) -> Result<(), RustBootstrapPatchPlanError> {
    if !input.capabilities.static_executable_linking {
        return Err(RustBootstrapPatchPlanError::new("musl-host route requires static executable linking capability"));
    }
    if !input.capabilities.dynamic_tool_runtime {
        return Err(RustBootstrapPatchPlanError::new("musl-host route requires dynamic tool runtime capability"));
    }
    if !input.capabilities.proc_macro_runtime {
        return Err(RustBootstrapPatchPlanError::new("musl-host route requires proc-macro runtime capability"));
    }
    Ok(())
}

fn provider_contract_assertion_operation(input: &RustBootstrapPatchPlanInput) -> RustBootstrapPatchOperation {
    patch_operation!(
        "provider-contract-assertion",
        RustBootstrapPatchOperationKind::ProviderContractAssertion,
        RustBootstrapPatchPhase::ProviderEvidence,
        &format!(
            "assert provider contract {} from explicit route facts",
            input.capabilities.provider_contract_version
        ),
        None,
        Some("provider metadata contract"),
    )
}

fn operation(fields: PatchOperationInput<'_>) -> RustBootstrapPatchOperation {
    debug_assert!(!fields.id.trim().is_empty());
    debug_assert!(!fields.summary.trim().is_empty());
    RustBootstrapPatchOperation {
        id: fields.id.to_string(),
        kind: fields.kind,
        phase: fields.phase,
        summary: fields.summary.to_string(),
        source_id: fields.source_id.map(ToOwned::to_owned),
        expected_anchor: fields.expected_anchor.map(ToOwned::to_owned),
    }
}

fn validate_operation_count(operations: &[RustBootstrapPatchOperation]) -> Result<(), RustBootstrapPatchPlanError> {
    if operations.is_empty() {
        return Err(RustBootstrapPatchPlanError::new("patch plan has no operations"));
    }
    if operations.len() > PATCH_PLAN_MAX_OPERATIONS {
        return Err(RustBootstrapPatchPlanError::new(format!(
            "patch plan has more than {PATCH_PLAN_MAX_OPERATIONS} operations"
        )));
    }
    Ok(())
}

fn is_musl_source_route(input: &RustBootstrapPatchPlanInput) -> bool {
    input.host_triple == MUSL_COMPILER_HOST_TRIPLE || input.target_triple == MUSL_COMPILER_HOST_TRIPLE
}

fn require_source_id(
    input: &RustBootstrapPatchPlanInput,
    expected_id: &str,
) -> Result<(), RustBootstrapPatchPlanError> {
    if input.source_identities.iter().any(|source| source.id == expected_id) {
        return Ok(());
    }
    Err(RustBootstrapPatchPlanError::new(format!("patch-plan input missing source identity {expected_id}")))
}

fn require_equal(label: &str, actual: impl AsRef<str>, expected: &str) -> Result<(), RustBootstrapPatchPlanError> {
    let actual = actual.as_ref();
    if actual == expected {
        return Ok(());
    }
    Err(RustBootstrapPatchPlanError::new(format!("{label} expected {expected}, got {actual}")))
}

fn require_non_empty(label: &str, value: impl AsRef<str>) -> Result<(), RustBootstrapPatchPlanError> {
    if !value.as_ref().trim().is_empty() {
        return Ok(());
    }
    Err(RustBootstrapPatchPlanError::new(format!("{label} is empty")))
}

fn require_digest(label: &str, value: impl AsRef<str>) -> Result<(), RustBootstrapPatchPlanError> {
    let value = value.as_ref();
    require_non_empty(label, value)?;
    if value.len() != DIGEST_HEX_CHAR_COUNT {
        return Err(RustBootstrapPatchPlanError::new(format!(
            "{label} must be {DIGEST_HEX_CHAR_COUNT} hex characters"
        )));
    }
    if value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(());
    }
    Err(RustBootstrapPatchPlanError::new(format!("{label} contains non-hex characters")))
}

fn digest_json<T: Serialize>(value: &T, label: &str) -> Result<String, RustBootstrapPatchPlanError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|err| RustBootstrapPatchPlanError::new(format!("serialize {label}: {err}")))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const EXPECTED_FIRST_STAGE_MUSL_OPERATION_COUNT: u32 = 9;
    const EXPECTED_GNU_OPERATION_COUNT: u32 = 3;

    #[test]
    fn musl_host_first_stage_plan_derives_ordered_operations_and_stable_digest() {
        let input = musl_first_stage_input();

        let plan = derive_rust_bootstrap_patch_plan(input.clone()).unwrap();
        let plan_again = derive_rust_bootstrap_patch_plan(input).unwrap();

        assert_eq!(plan.operation_count, EXPECTED_FIRST_STAGE_MUSL_OPERATION_COUNT);
        assert_eq!(plan.input_digest_blake3, plan_again.input_digest_blake3);
        assert_eq!(plan.output_digest_blake3, plan_again.output_digest_blake3);
        assert_eq!(plan.operations[0].kind, RustBootstrapPatchOperationKind::MinicargoBuildOutDir);
        assert_eq!(plan.operations[0].phase, RustBootstrapPatchPhase::BeforeFirstStageMainMake);
        assert!(plan.contains_operation(RustBootstrapPatchOperationKind::RunRustcHostRuntime));
        assert!(plan.contains_operation(RustBootstrapPatchOperationKind::ProviderContractAssertion));
        assert!(!plan.contains_operation(RustBootstrapPatchOperationKind::RustBootstrapRustcPrivateToolRlibLookup));
    }

    #[test]
    fn non_musl_first_stage_plan_omits_musl_route_repairs() {
        let mut input = musl_first_stage_input();
        input.host_triple = GNU_COMPILER_HOST_TRIPLE.to_string();
        input.target_triple = GNU_COMPILER_HOST_TRIPLE.to_string();

        let plan = derive_rust_bootstrap_patch_plan(input).unwrap();

        assert_eq!(plan.operation_count, EXPECTED_GNU_OPERATION_COUNT);
        assert!(plan.contains_operation(RustBootstrapPatchOperationKind::ProviderContractAssertion));
        assert!(!plan.contains_operation(RustBootstrapPatchOperationKind::MinicargoBuildOutDir));
        assert!(plan.contains_operation(RustBootstrapPatchOperationKind::RunRustcHostRuntime));
    }

    #[test]
    fn rust_bootstrap_plan_includes_final_rustdoc_rlib_lookup_repair() {
        let input = rust_bootstrap_input("1.94.0", true);

        let plan = derive_rust_bootstrap_patch_plan(input).unwrap();

        assert!(plan.contains_operation(RustBootstrapPatchOperationKind::RustBootstrapTargetToolConfig));
        assert!(plan.contains_operation(RustBootstrapPatchOperationKind::RustBootstrapRustcPrivateToolRlibLookup));
        assert!(plan.contains_operation(RustBootstrapPatchOperationKind::ProviderContractAssertion));
        assert!(plan.receipt_arguments().iter().any(|argument| argument.contains("output-digest=")));
    }

    #[test]
    fn patch_plan_rejects_mismatched_first_stage_versions() {
        let mut input = musl_first_stage_input();
        input.rust_version = "1.89.0".to_string();

        let err = derive_rust_bootstrap_patch_plan(input).unwrap_err();

        assert!(err.message().contains("first-stage Rust version expected 1.90.0"));
        assert!(!err.message().contains("filesystem"));
    }

    #[test]
    fn patch_plan_rejects_missing_source_identity_before_shell_mutation() {
        let mut input = musl_first_stage_input();
        input.source_identities.retain(|source| source.id != "mrustc-0.12.0");

        let err = derive_rust_bootstrap_patch_plan(input).unwrap_err();

        assert!(err.message().contains("missing source identity mrustc-0.12.0"));
        assert!(!err.message().contains("panic"));
    }

    #[test]
    fn patch_plan_rejects_missing_musl_runtime_capability() {
        let mut input = rust_bootstrap_input("1.94.0", true);
        input.capabilities.proc_macro_runtime = false;

        let err = derive_rust_bootstrap_patch_plan(input).unwrap_err();

        assert!(err.message().contains("proc-macro runtime capability"));
        assert!(!err.message().contains("unsupported Rust bootstrap version"));
    }

    #[test]
    fn patch_plan_rejects_unsupported_bootstrap_version() {
        let input = rust_bootstrap_input("1.95.0", true);

        let err = derive_rust_bootstrap_patch_plan(input).unwrap_err();

        assert!(err.message().contains("unsupported Rust bootstrap version 1.95.0"));
        assert!(!err.message().contains("source identity rust-1.94.0"));
    }

    fn musl_first_stage_input() -> RustBootstrapPatchPlanInput {
        RustBootstrapPatchPlanInput {
            stage: RustBootstrapPatchStage::FirstStage,
            route_id: "musl-host-source-route".to_string(),
            route_plan_digest_blake3: DIGEST_A.to_string(),
            route_policy_digest_blake3: DIGEST_B.to_string(),
            stage_id: "mrustc-to-rust-1.90.0".to_string(),
            rust_version: SUPPORTED_FIRST_STAGE_RUST_VERSION.to_string(),
            mrustc_version: Some(SUPPORTED_MRUSTC_VERSION.to_string()),
            host_triple: MUSL_COMPILER_HOST_TRIPLE.to_string(),
            target_triple: MUSL_COMPILER_HOST_TRIPLE.to_string(),
            source_identities: vec![
                source("mrustc-0.12.0", "mrustc", DIGEST_C),
                source("rust-1.90.0", "rust", DIGEST_D),
            ],
            capabilities: musl_capabilities(false),
        }
    }

    fn rust_bootstrap_input(version: &str, rustdoc_tool: bool) -> RustBootstrapPatchPlanInput {
        RustBootstrapPatchPlanInput {
            stage: RustBootstrapPatchStage::RustBootstrap,
            route_id: "musl-host-source-route".to_string(),
            route_plan_digest_blake3: DIGEST_A.to_string(),
            route_policy_digest_blake3: DIGEST_B.to_string(),
            stage_id: format!("rust-{version}-stage"),
            rust_version: version.to_string(),
            mrustc_version: None,
            host_triple: MUSL_COMPILER_HOST_TRIPLE.to_string(),
            target_triple: MUSL_COMPILER_HOST_TRIPLE.to_string(),
            source_identities: vec![source(&format!("rust-{version}"), "rust", DIGEST_C)],
            capabilities: musl_capabilities(rustdoc_tool),
        }
    }

    fn source(id: &str, name: &str, digest: &str) -> RustBootstrapPatchSourceIdentity {
        RustBootstrapPatchSourceIdentity {
            id: id.to_string(),
            name: name.to_string(),
            digest_kind: "sha256".to_string(),
            digest: digest.to_string(),
        }
    }

    fn musl_capabilities(rustdoc_tool: bool) -> RustBootstrapPatchCapabilities {
        RustBootstrapPatchCapabilities {
            source_built_provider: true,
            proc_macro_runtime: true,
            static_executable_linking: true,
            dynamic_tool_runtime: true,
            rustdoc_tool,
            provider_contract_version: "mantle-rust-source-provider-v1".to_string(),
        }
    }
}
