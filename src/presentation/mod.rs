//! Presentation adapters: render typed results for humans or machines.
//!
//! Adapters here take owned or borrowed typed values and one output format, and
//! never read the environment, the clock, or the filesystem.

pub(crate) mod diagnostics;
pub(crate) mod refactor;
pub(crate) mod remote_client;
pub(crate) mod reports;
pub(crate) mod runtime_fingerprint;
pub(crate) mod semantic_graph;
pub(crate) mod source_root_provider;
