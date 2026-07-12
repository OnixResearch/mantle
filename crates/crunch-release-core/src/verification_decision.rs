use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Serialize;

/// Stable schema for the terminal release-verification decision.
pub const RELEASE_VERIFICATION_DECISION_SCHEMA: &str = "mantle-release-verification-decision-v1";
/// Bounds diagnostics copied from any one specialized verifier.
pub const MAX_RELEASE_VERIFICATION_DIAGNOSTICS_PER_CONTRIBUTOR: u32 = 32;
/// Bounds external evidence roles selected by one CLI invocation.
pub const MAX_RELEASE_VERIFICATION_REQUIRED_EXTERNAL_ROLES: u32 = 64;

// Keep the contributor list as the single source of truth. Adding an entry
// extends the ordered list and both normalized input records, so every record
// literal must provide the new fact and requirement before the crate compiles.
macro_rules! define_release_verification_contributors {
    ($(($variant:ident, $field:ident, $label:literal)),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
        pub enum ReleaseVerificationContributor {
            $(
                #[serde(rename = $label)]
                $variant,
            )+
        }

        impl ReleaseVerificationContributor {
            pub const ALL: &'static [Self] = &[
                $(Self::$variant,)+
            ];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)+
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct ReleaseVerificationFacts {
            $(pub $field: ReleaseVerificationFact,)+
        }

        impl ReleaseVerificationFacts {
            fn for_contributor(&self, contributor: ReleaseVerificationContributor) -> &ReleaseVerificationFact {
                match contributor {
                    $(ReleaseVerificationContributor::$variant => &self.$field,)+
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct ReleaseVerificationRequirements {
            $(pub $field: ReleaseVerificationRequirement,)+
        }

        impl ReleaseVerificationRequirements {
            fn for_contributor(
                &self,
                contributor: ReleaseVerificationContributor,
            ) -> ReleaseVerificationRequirement {
                match contributor {
                    $(ReleaseVerificationContributor::$variant => self.$field,)+
                }
            }
        }
    };
}

// r[impl mantle.release_provenance.verification_decision.completeness]
define_release_verification_contributors!(
    (ManifestIntegrity, manifest_integrity, "manifest-integrity"),
    (Reproducibility, reproducibility, "reproducibility"),
    (DeterministicRelease, deterministic_release, "deterministic-release"),
    (ProviderFixedPointProof, provider_fixed_point_proof, "provider-fixed-point-proof"),
    (StackProvenance, stack_provenance, "stack-provenance"),
    (ExternalEvidenceRoles, external_evidence_roles, "external-evidence-roles"),
    (StagexNoQuorum, stagex_no_quorum, "stagex-no-quorum"),
    (FunctionAddress, function_address, "function-address"),
    (CairnHandoff, cairn_handoff, "cairn-handoff"),
);

pub const RELEASE_VERIFICATION_CONTRIBUTOR_COUNT: usize = ReleaseVerificationContributor::ALL.len();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseVerificationRequirement {
    Mandatory,
    Required,
    Advisory,
    NotSelected,
}

impl ReleaseVerificationRequirement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mandatory => "mandatory",
            Self::Required => "required",
            Self::Advisory => "advisory",
            Self::NotSelected => "not-selected",
        }
    }

    const fn is_enforced(self) -> bool {
        matches!(self, Self::Mandatory | Self::Required)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseVerificationFactDisposition {
    Satisfied,
    Absent,
    Rejected,
    NotEvaluated,
}

impl ReleaseVerificationFactDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Absent => "absent",
            Self::Rejected => "rejected",
            Self::NotEvaluated => "not-evaluated",
        }
    }

    const fn is_satisfied(self) -> bool {
        matches!(self, Self::Satisfied)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseVerificationFact {
    disposition: ReleaseVerificationFactDisposition,
    diagnostics: Vec<String>,
}

impl ReleaseVerificationFact {
    pub fn satisfied() -> Self {
        Self {
            disposition: ReleaseVerificationFactDisposition::Satisfied,
            diagnostics: Vec::new(),
        }
    }

    pub fn absent(diagnostics: Vec<String>) -> Self {
        Self {
            disposition: ReleaseVerificationFactDisposition::Absent,
            diagnostics,
        }
    }

    pub fn rejected(diagnostics: Vec<String>) -> Self {
        Self {
            disposition: ReleaseVerificationFactDisposition::Rejected,
            diagnostics,
        }
    }

    pub fn not_evaluated() -> Self {
        Self {
            disposition: ReleaseVerificationFactDisposition::NotEvaluated,
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseVerificationCheck {
    pub contributor: ReleaseVerificationContributor,
    pub requirement: ReleaseVerificationRequirement,
    pub disposition: ReleaseVerificationFactDisposition,
    pub blocking: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseVerificationDecisionDisposition {
    Accepted,
    PolicyRejected,
}

impl ReleaseVerificationDecisionDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::PolicyRejected => "policy-rejected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseVerificationDecision {
    pub schema: &'static str,
    pub valid: bool,
    pub disposition: ReleaseVerificationDecisionDisposition,
    pub checks: Vec<ReleaseVerificationCheck>,
    pub diagnostics: Vec<String>,
}

// r[impl mantle.release_provenance.verification_decision.complete]
// r[impl mantle.release_provenance.verification_decision.boundary]
pub fn aggregate_release_verification(
    facts: ReleaseVerificationFacts,
    requirements: ReleaseVerificationRequirements,
) -> ReleaseVerificationDecision {
    assert!(!ReleaseVerificationContributor::ALL.is_empty(), "release verification needs contributors");
    assert!(
        MAX_RELEASE_VERIFICATION_DIAGNOSTICS_PER_CONTRIBUTOR > 0,
        "release verification diagnostic limit must be positive"
    );

    let mut checks = Vec::with_capacity(RELEASE_VERIFICATION_CONTRIBUTOR_COUNT);
    let mut diagnostics = Vec::new();
    for contributor in ReleaseVerificationContributor::ALL {
        let fact = facts.for_contributor(*contributor);
        let requirement = requirements.for_contributor(*contributor);
        let check = release_verification_check(*contributor, requirement, fact);
        if check.blocking {
            diagnostics.extend(check.diagnostics.iter().cloned());
        }
        checks.push(check);
    }

    let valid = checks.iter().all(|check| !check.blocking);
    let disposition = if valid {
        ReleaseVerificationDecisionDisposition::Accepted
    } else {
        ReleaseVerificationDecisionDisposition::PolicyRejected
    };
    assert_eq!(checks.len(), RELEASE_VERIFICATION_CONTRIBUTOR_COUNT);
    assert_eq!(valid, diagnostics.is_empty());

    ReleaseVerificationDecision {
        schema: RELEASE_VERIFICATION_DECISION_SCHEMA,
        valid,
        disposition,
        checks,
        diagnostics,
    }
}

fn release_verification_check(
    contributor: ReleaseVerificationContributor,
    requirement: ReleaseVerificationRequirement,
    fact: &ReleaseVerificationFact,
) -> ReleaseVerificationCheck {
    assert!(!contributor.as_str().is_empty(), "release verification contributor label must not be empty");
    assert!(!requirement.as_str().is_empty(), "release verification requirement label must not be empty");

    let (mut diagnostics, diagnostics_overflowed) = bounded_diagnostics(contributor, &fact.diagnostics);
    let policy_blocking = requirement.is_enforced() && !fact.disposition.is_satisfied();
    let blocking = diagnostics_overflowed || policy_blocking;
    if blocking && diagnostics.is_empty() {
        diagnostics.push(format!(
            "release verification contributor {} is {} but requirement is {}",
            contributor.as_str(),
            fact.disposition.as_str(),
            requirement.as_str()
        ));
    }

    assert!(!blocking || !diagnostics.is_empty(), "blocking verification checks need diagnostics");
    assert!(diagnostics.len() <= diagnostic_limit(), "verification diagnostics must remain bounded");
    ReleaseVerificationCheck {
        contributor,
        requirement,
        disposition: fact.disposition,
        blocking,
        diagnostics,
    }
}

fn bounded_diagnostics(contributor: ReleaseVerificationContributor, diagnostics: &[String]) -> (Vec<String>, bool) {
    let limit = diagnostic_limit();
    assert!(limit > 0, "release verification diagnostic limit must be positive");
    assert!(!contributor.as_str().is_empty(), "release verification contributor label must not be empty");
    if diagnostics.len() <= limit {
        return (diagnostics.to_vec(), false);
    }

    let retained_diagnostic_count = limit.saturating_sub(1);
    let mut bounded = diagnostics.iter().take(retained_diagnostic_count).cloned().collect::<Vec<_>>();
    bounded.push(format!(
        "release verification contributor {} diagnostic count exceeds limit {}: observed {}",
        contributor.as_str(),
        MAX_RELEASE_VERIFICATION_DIAGNOSTICS_PER_CONTRIBUTOR,
        diagnostics.len()
    ));
    assert_eq!(bounded.len(), limit);
    assert!(diagnostics.len() > bounded.len());
    (bounded, true)
}

const fn diagnostic_limit() -> usize {
    MAX_RELEASE_VERIFICATION_DIAGNOSTICS_PER_CONTRIBUTOR as usize
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::vec;

    use super::*;

    const FIRST_BLOCKER: &str = "first required blocker";
    const SECOND_BLOCKER: &str = "second required blocker";

    fn satisfied_facts() -> ReleaseVerificationFacts {
        ReleaseVerificationFacts {
            manifest_integrity: ReleaseVerificationFact::satisfied(),
            reproducibility: ReleaseVerificationFact::satisfied(),
            deterministic_release: ReleaseVerificationFact::satisfied(),
            provider_fixed_point_proof: ReleaseVerificationFact::satisfied(),
            stack_provenance: ReleaseVerificationFact::satisfied(),
            external_evidence_roles: ReleaseVerificationFact::satisfied(),
            stagex_no_quorum: ReleaseVerificationFact::satisfied(),
            function_address: ReleaseVerificationFact::satisfied(),
            cairn_handoff: ReleaseVerificationFact::satisfied(),
        }
    }

    fn required_requirements() -> ReleaseVerificationRequirements {
        ReleaseVerificationRequirements {
            manifest_integrity: ReleaseVerificationRequirement::Mandatory,
            reproducibility: ReleaseVerificationRequirement::Required,
            deterministic_release: ReleaseVerificationRequirement::Required,
            provider_fixed_point_proof: ReleaseVerificationRequirement::Required,
            stack_provenance: ReleaseVerificationRequirement::Required,
            external_evidence_roles: ReleaseVerificationRequirement::Required,
            stagex_no_quorum: ReleaseVerificationRequirement::Required,
            function_address: ReleaseVerificationRequirement::Required,
            cairn_handoff: ReleaseVerificationRequirement::Required,
        }
    }

    // r[verify mantle.release_provenance.verification_decision.boundary.test]
    #[test]
    fn optional_absence_does_not_block_acceptance() {
        let mut facts = satisfied_facts();
        facts.reproducibility = ReleaseVerificationFact::absent(Vec::new());
        facts.function_address = ReleaseVerificationFact::not_evaluated();
        let mut requirements = required_requirements();
        requirements.reproducibility = ReleaseVerificationRequirement::Advisory;
        requirements.function_address = ReleaseVerificationRequirement::NotSelected;

        let decision = aggregate_release_verification(facts, requirements);

        assert!(decision.valid);
        assert_eq!(decision.disposition, ReleaseVerificationDecisionDisposition::Accepted);
        assert!(decision.diagnostics.is_empty());
        assert!(!decision.checks[1].blocking);
    }

    // r[verify mantle.release_provenance.verification_decision.fixtures.positive]
    #[test]
    fn all_required_contributors_accept_only_when_satisfied() {
        let decision = aggregate_release_verification(satisfied_facts(), required_requirements());

        assert!(decision.valid);
        assert_eq!(decision.schema, RELEASE_VERIFICATION_DECISION_SCHEMA);
        assert_eq!(decision.checks.len(), RELEASE_VERIFICATION_CONTRIBUTOR_COUNT);
        assert!(decision.checks.iter().all(|check| !check.blocking));
    }

    // r[verify mantle.release_provenance.verification_decision.fixtures.negative]
    #[test]
    fn one_required_blocker_rejects_manifest_success() {
        let mut facts = satisfied_facts();
        facts.reproducibility = ReleaseVerificationFact::absent(vec![FIRST_BLOCKER.into()]);

        let decision = aggregate_release_verification(facts, required_requirements());

        assert!(!decision.valid);
        assert_eq!(decision.disposition, ReleaseVerificationDecisionDisposition::PolicyRejected);
        assert_eq!(decision.diagnostics, vec![String::from(FIRST_BLOCKER)]);
        assert!(!decision.checks[0].blocking);
        assert!(decision.checks[1].blocking);
    }

    #[test]
    fn multiple_required_blockers_keep_contributor_and_diagnostic_order() {
        let mut facts = satisfied_facts();
        facts.reproducibility = ReleaseVerificationFact::rejected(vec![FIRST_BLOCKER.into()]);
        facts.stagex_no_quorum = ReleaseVerificationFact::rejected(vec![SECOND_BLOCKER.into()]);

        let decision = aggregate_release_verification(facts, required_requirements());

        assert!(!decision.valid);
        assert_eq!(decision.diagnostics, vec![String::from(FIRST_BLOCKER), String::from(SECOND_BLOCKER)]);
        let reproducibility = decision
            .checks
            .iter()
            .find(|check| check.contributor == ReleaseVerificationContributor::Reproducibility)
            .unwrap();
        let stagex = decision
            .checks
            .iter()
            .find(|check| check.contributor == ReleaseVerificationContributor::StagexNoQuorum)
            .unwrap();
        assert!(reproducibility.blocking);
        assert!(stagex.blocking);
    }

    // r[verify mantle.release_provenance.verification_decision.completeness.test]
    #[test]
    fn completeness_guard_maps_every_required_contributor() {
        let facts = ReleaseVerificationFacts {
            manifest_integrity: rejected_fact(ReleaseVerificationContributor::ManifestIntegrity),
            reproducibility: rejected_fact(ReleaseVerificationContributor::Reproducibility),
            deterministic_release: rejected_fact(ReleaseVerificationContributor::DeterministicRelease),
            provider_fixed_point_proof: rejected_fact(ReleaseVerificationContributor::ProviderFixedPointProof),
            stack_provenance: rejected_fact(ReleaseVerificationContributor::StackProvenance),
            external_evidence_roles: rejected_fact(ReleaseVerificationContributor::ExternalEvidenceRoles),
            stagex_no_quorum: rejected_fact(ReleaseVerificationContributor::StagexNoQuorum),
            function_address: rejected_fact(ReleaseVerificationContributor::FunctionAddress),
            cairn_handoff: rejected_fact(ReleaseVerificationContributor::CairnHandoff),
        };

        let decision = aggregate_release_verification(facts, required_requirements());
        let mapped = decision.checks.iter().map(|check| check.contributor).collect::<Vec<_>>();

        assert_eq!(mapped.as_slice(), ReleaseVerificationContributor::ALL);
        assert_eq!(decision.checks.len(), RELEASE_VERIFICATION_CONTRIBUTOR_COUNT);
        assert!(decision.checks.iter().all(|check| check.blocking));
        assert_eq!(decision.diagnostics.len(), RELEASE_VERIFICATION_CONTRIBUTOR_COUNT);
    }

    #[test]
    fn diagnostic_overflow_is_bounded_and_fails_closed() {
        let overflow_count = diagnostic_limit().saturating_add(1);
        let diagnostics = (0..overflow_count).map(|index| format!("diagnostic-{index}")).collect::<Vec<_>>();
        let mut facts = satisfied_facts();
        facts.function_address = ReleaseVerificationFact::rejected(diagnostics);
        let mut requirements = required_requirements();
        requirements.function_address = ReleaseVerificationRequirement::Advisory;

        let decision = aggregate_release_verification(facts, requirements);
        let check = decision
            .checks
            .iter()
            .find(|check| check.contributor == ReleaseVerificationContributor::FunctionAddress)
            .unwrap();

        assert!(!decision.valid);
        assert!(check.blocking);
        assert_eq!(check.diagnostics.len(), diagnostic_limit());
        assert!(check.diagnostics.last().unwrap().contains("diagnostic count exceeds limit"));
    }

    fn rejected_fact(contributor: ReleaseVerificationContributor) -> ReleaseVerificationFact {
        assert!(!contributor.as_str().is_empty());
        assert!(ReleaseVerificationContributor::ALL.contains(&contributor));
        ReleaseVerificationFact::rejected(vec![format!("{} rejected", contributor.as_str())])
    }
}
