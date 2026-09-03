use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildActionObservationFacts {
    pub all_local: bool,
    pub any_remote: bool,
    pub any_build: bool,
    pub source_bundle_present: bool,
    pub doctor_preflight_ok: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BuildActionClass {
    Cached,
    Substitute,
    Build,
    PreflightError,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildActionDecision {
    pub action: BuildActionClass,
    pub reason_code: &'static str,
}

#[must_use]
pub fn select_build_action(facts: BuildActionObservationFacts) -> BuildActionDecision {
    if facts.all_local {
        debug_assert!(!facts.any_build);
        debug_assert!(!facts.any_remote);
        return BuildActionDecision {
            action: BuildActionClass::Cached,
            reason_code: "all-outputs-local",
        };
    }
    if facts.any_remote && !facts.any_build && !facts.source_bundle_present {
        return BuildActionDecision {
            action: BuildActionClass::Substitute,
            reason_code: "remote-only-output",
        };
    }
    if facts.doctor_preflight_ok {
        return BuildActionDecision {
            action: BuildActionClass::Build,
            reason_code: "local-preflight-ok",
        };
    }
    BuildActionDecision {
        action: BuildActionClass::PreflightError,
        reason_code: "local-preflight-failed",
    }
}
