use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

const CLASS_CARGO_ORACLE_EVIDENCE: &str = "cargo-oracle-evidence";
const CLASS_CARGO_FREE_BOUNDED_TOPOLOGY: &str = "cargo-free-bounded-topology";
const CLASS_BLOCKED_UNSUPPORTED_SURFACE: &str = "blocked-unsupported-surface";
const SURFACE_CARGO_ORACLE: &str = "cargo-oracle-reference";
const SURFACE_BLOCKED_UNSUPPORTED: &str = "blocked-unsupported-surface";
const SURFACE_BASIC_PATH_WORKSPACE: &str = "path-workspace-basic";
const SURFACE_LOCAL_PATH_DEPENDENCY: &str = "local-path-dependency";
const SURFACE_WORKSPACE_INHERITANCE: &str = "workspace-inheritance";
const SURFACE_SOURCE_CLOSURE_DIGEST: &str = "source-closure-digest";
const SURFACE_UNIT_GRAPH_FACTS: &str = "unit-graph-facts";
const COMMON_NON_CLAIMS: &[&str] = &[
    "not-full-cargo-compatibility",
    "not-compiler-correctness",
    "not-release-reproducibility",
    "not-bootstrap-correctness",
];

#[must_use]
pub fn summarize_compatibility(no_cargo_oracle: bool, blocker_classes: &[String]) -> crate::CompatibilitySummary {
    let compatibility_class = compatibility_class(no_cargo_oracle, blocker_classes).to_string();
    let status = compatibility_status(no_cargo_oracle, blocker_classes).to_string();
    let mut surface_ids = compatibility_surface_ids(no_cargo_oracle, blocker_classes);
    let mut blockers = blocker_classes.to_vec();
    let non_claims = compatibility_non_claims(no_cargo_oracle);
    surface_ids.sort();
    surface_ids.dedup();
    blockers.sort();
    blockers.dedup();
    debug_assert!(!compatibility_class.is_empty());
    debug_assert!(!status.is_empty());
    crate::CompatibilitySummary {
        compatibility_class,
        status,
        surface_ids,
        blocker_classes: blockers,
        non_claims,
    }
}

#[must_use]
pub fn compatibility_status(no_cargo_oracle: bool, blocker_classes: &[String]) -> &'static str {
    if !no_cargo_oracle {
        return "oracle";
    }
    if blocker_classes.is_empty() {
        return "supported";
    }
    "blocked"
}

#[must_use]
pub fn compatibility_class(no_cargo_oracle: bool, blocker_classes: &[String]) -> &'static str {
    if !no_cargo_oracle {
        return CLASS_CARGO_ORACLE_EVIDENCE;
    }
    if blocker_classes.is_empty() {
        return CLASS_CARGO_FREE_BOUNDED_TOPOLOGY;
    }
    CLASS_BLOCKED_UNSUPPORTED_SURFACE
}

#[must_use]
pub fn compatibility_surface_ids(no_cargo_oracle: bool, blocker_classes: &[String]) -> Vec<String> {
    if !no_cargo_oracle {
        return vec![SURFACE_CARGO_ORACLE.to_string()];
    }
    if !blocker_classes.is_empty() {
        return vec![SURFACE_BLOCKED_UNSUPPORTED.to_string()];
    }
    vec![
        SURFACE_BASIC_PATH_WORKSPACE.to_string(),
        SURFACE_LOCAL_PATH_DEPENDENCY.to_string(),
        SURFACE_WORKSPACE_INHERITANCE.to_string(),
        SURFACE_SOURCE_CLOSURE_DIGEST.to_string(),
        SURFACE_UNIT_GRAPH_FACTS.to_string(),
    ]
}

#[must_use]
pub fn compatibility_non_claims(no_cargo_oracle: bool) -> Vec<String> {
    let mut non_claims = if no_cargo_oracle {
        vec![
            "bounded-path-workspace-only".to_string(),
            "not-full-cargo-feature-resolution".to_string(),
            "declared-vendor-and-captured-git-only".to_string(),
            "not-network-or-ambient-cargo-source-resolution".to_string(),
        ]
    } else {
        vec!["cargo-used-for-oracle-metadata-and-unit-graph".to_string()]
    };
    non_claims.extend(COMMON_NON_CLAIMS.iter().map(|claim| (*claim).to_string()));
    non_claims.sort();
    non_claims.dedup();
    debug_assert!(!non_claims.is_empty());
    debug_assert!(non_claims.iter().all(|claim| !claim.is_empty()));
    non_claims
}
