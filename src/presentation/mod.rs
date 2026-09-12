//! Presentation adapters: render typed results for humans or machines.
//!
//! Adapters here take owned or borrowed typed values and one output format, and
//! never read the environment, the clock, or the filesystem.

pub(crate) mod reports;
pub(crate) mod semantic_graph;
