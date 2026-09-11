//! Command-family taxonomy for the CLI composition root.
//!
//! Families group the public command roots by owning capability so the root
//! can dispatch to typed application operations instead of holding policy.

use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

/// Maximum admitted public command roots.
pub const MAX_COMMAND_ROOTS: u32 = 128;

/// Application command families with their owning capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandFamily {
    /// Realize, check, run, develop, shell, and file generation.
    Realization,
    /// Store administration, cache, logs, and receipts.
    StoreAdministration,
    /// Source admission, provenance, and attestations.
    SourceProvenance,
    /// Plan, graph, and explanation surfaces.
    Planning,
    /// Evaluation sessions and worker fixtures.
    Evaluation,
    /// Remote execution, secret workers, and staging.
    RemoteExecution,
    /// Release creation, export, and verification.
    Release,
    /// Project lifecycle and manifest management.
    Project,
    /// Bootstrap and self-build flows.
    Bootstrap,
    /// WebAssembly component flows.
    Component,
    /// Diagnostics and refactoring aids.
    Diagnostics,
}

impl CommandFamily {
    /// Every family in canonical order.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Realization,
            Self::StoreAdministration,
            Self::SourceProvenance,
            Self::Planning,
            Self::Evaluation,
            Self::RemoteExecution,
            Self::Release,
            Self::Project,
            Self::Bootstrap,
            Self::Component,
            Self::Diagnostics,
        ]
    }

    /// Family owning one public command root, or `None` when unclassified.
    pub fn of_root(root: &str) -> Option<Self> {
        let family = match root {
            "build" | "check" | "run" | "develop" | "shell" | "filegen" => Self::Realization,
            "store" | "rust-cache" | "log" | "receipt" => Self::StoreAdministration,
            "source" | "attest" | "foreign-import" | "import" | "artifact" => Self::SourceProvenance,
            "rust-plan" | "graph" | "why" | "dependents" | "nix-free-demo" | "operator-contract" => Self::Planning,
            "eval" | "evaluator-worker" | "evaluator-worker-fixture" | "transcript" => Self::Evaluation,
            "remote" | "remote-secret-worker" | "stage" => Self::RemoteExecution,
            "release" | "export" => Self::Release,
            "init" | "refresh" | "upgrade" | "show" | "list-stale" | "mantlepkgs" => Self::Project,
            "bootstrap" | "self-build" => Self::Bootstrap,
            "wasm-component" => Self::Component,
            "doctor" | "refactor" => Self::Diagnostics,
            _ => return None,
        };
        debug_assert!(Self::all().contains(&family));
        Some(family)
    }

    /// Command roots owned by this family, in canonical order.
    pub fn roots(self) -> Vec<&'static str> {
        let roots = match self {
            Self::Realization => vec!["build", "check", "develop", "filegen", "run", "shell"],
            Self::StoreAdministration => vec!["log", "receipt", "rust-cache", "store"],
            Self::SourceProvenance => vec!["artifact", "attest", "foreign-import", "import", "source"],
            Self::Planning => vec![
                "dependents",
                "graph",
                "nix-free-demo",
                "operator-contract",
                "rust-plan",
                "why",
            ],
            Self::Evaluation => vec!["eval", "evaluator-worker", "evaluator-worker-fixture", "transcript"],
            Self::RemoteExecution => vec!["remote", "remote-secret-worker", "stage"],
            Self::Release => vec!["export", "release"],
            Self::Project => vec!["init", "list-stale", "mantlepkgs", "refresh", "show", "upgrade"],
            Self::Bootstrap => vec!["bootstrap", "self-build"],
            Self::Component => vec!["wasm-component"],
            Self::Diagnostics => vec!["doctor", "refactor"],
        };
        debug_assert!(!roots.is_empty());
        roots
    }
}
