use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustPlanCapability {
    WorkspaceFacts,
    CargoOracle,
    CompilerInspection,
    UnitExecution,
    RustCache,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustPlanPortError {
    pub capability: RustPlanCapability,
    pub code: String,
    pub retryable: bool,
}

impl RustPlanPortError {
    #[must_use]
    pub fn new(capability: RustPlanCapability, code: impl Into<String>, retryable: bool) -> Self {
        let code = code.into();
        debug_assert!(!code.is_empty());
        debug_assert!(!retryable || capability != RustPlanCapability::WorkspaceFacts);
        Self {
            capability,
            code,
            retryable,
        }
    }
}

impl core::fmt::Display for RustPlanPortError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Rust-plan {:?} capability failed: {}", self.capability, self.code)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFactsRequest {
    pub workspace_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFactsObservation {
    pub facts: mantle_rust_plan_core::StructuralWorkspaceFacts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoOracleRequest {
    pub workspace_identity: String,
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoOracleObservation {
    pub facts: mantle_rust_plan_core::CargoOracleFacts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerInspectionRequest {
    pub compiler_hint: String,
    pub target_triple: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerInspectionObservation {
    pub fact: mantle_rust_plan_core::ToolchainFact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitExecutionRequest {
    pub effect: mantle_rust_plan_core::RustUnitEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitExecutionObservation {
    pub observation: mantle_rust_plan_core::RustUnitObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheLookupRequest {
    pub effect: mantle_rust_plan_core::RustUnitEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheLookupObservation {
    pub observation: mantle_rust_plan_core::CacheObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachePublishRequest {
    pub effect: mantle_rust_plan_core::RustUnitEffect,
    pub outcome: mantle_rust_plan_core::RustUnitOutcome,
}

pub trait WorkspaceFactsPort {
    fn load_workspace(
        &mut self,
        request: WorkspaceFactsRequest,
    ) -> Result<WorkspaceFactsObservation, RustPlanPortError>;
}

pub trait CargoOraclePort {
    fn capture_oracle(&mut self, request: CargoOracleRequest) -> Result<CargoOracleObservation, RustPlanPortError>;
}

pub trait CompilerInspectionPort {
    fn inspect_compiler(
        &mut self,
        request: CompilerInspectionRequest,
    ) -> Result<CompilerInspectionObservation, RustPlanPortError>;
}

pub trait UnitExecutionPort {
    fn execute_unit(&mut self, request: UnitExecutionRequest) -> Result<UnitExecutionObservation, RustPlanPortError>;
}

pub trait RustCachePort {
    fn lookup(&mut self, request: CacheLookupRequest) -> Result<CacheLookupObservation, RustPlanPortError>;

    fn publish(&mut self, request: CachePublishRequest) -> Result<(), RustPlanPortError>;
}

pub struct RustPlanPortSet<'a> {
    pub workspace: &'a mut dyn WorkspaceFactsPort,
    pub cargo_oracle: &'a mut dyn CargoOraclePort,
    pub compiler: &'a mut dyn CompilerInspectionPort,
    pub executor: &'a mut dyn UnitExecutionPort,
    pub cache: &'a mut dyn RustCachePort,
}
