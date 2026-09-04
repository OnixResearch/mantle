#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourcePolicyCapability {
    UsageLedger,
    EvidencePublication,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourcePolicyPortError {
    pub capability: ResourcePolicyCapability,
    pub code: String,
    pub retryable: bool,
}

impl ResourcePolicyPortError {
    #[must_use]
    pub fn new(capability: ResourcePolicyCapability, code: impl Into<String>, retryable: bool) -> Self {
        Self {
            capability,
            code: code.into(),
            retryable,
        }
    }
}

impl core::fmt::Display for ResourcePolicyPortError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "resource-policy {:?} capability failed: {}", self.capability, self.code)
    }
}

pub trait UsageLedgerPort {
    fn load_usage_ledger(&mut self) -> Result<crunch_resource_policy_core::UsageLedgerState, ResourcePolicyPortError>;

    fn compare_and_commit_usage_ledger(
        &mut self,
        expected_state_blake3: &str,
        next_state: &crunch_resource_policy_core::UsageLedgerState,
    ) -> Result<(), ResourcePolicyPortError>;
}

pub trait ResourcePolicyEvidencePort {
    fn publish_valence_bundle(
        &mut self,
        bundle: &valence_core::build_service::BuildServiceBundle,
    ) -> Result<(), ResourcePolicyPortError>;
}

pub struct ResourcePolicyPortSet<'a> {
    pub ledger: &'a mut dyn UsageLedgerPort,
    pub evidence: &'a mut dyn ResourcePolicyEvidencePort,
}
