#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Published consumer contract for Mantle WebAssembly component
//! materialization bundles.
//!
//! The facade re-exports the pure bundle DTOs and delegates every structural
//! and canonical-identity decision to `crunch-wasm-component-core`; it never
//! duplicates decision logic. The optional `std` feature enables a thin file
//! verifier that resolves declared members under one explicit capability
//! root, remeasures bytes under named bounds, and emits one bounded report.
//!
//! A passing report does not prove source trust, compiler correctness,
//! component behavior, runtime isolation, authority, reproducibility,
//! deployment safety, or release eligibility.

#[cfg(any(test, feature = "std"))]
extern crate std;

extern crate alloc;

pub mod contract;
pub mod facade;

#[cfg(feature = "std")]
pub mod shell;

pub use contract::ConsumerLayer;
pub use contract::ConsumerMemberObservation;
pub use contract::ConsumerMemberStatus;
pub use contract::ConsumerVerificationReport;
pub use contract::ConsumerVerificationStatus;
pub use facade::CONSUMER_REPORT_SCHEMA;
pub use facade::CONSUMER_VERIFIER_NON_CLAIMS;
pub use facade::verify_consumer_bundle;
#[cfg(feature = "std")]
pub use shell::ConsumerRoot;
#[cfg(feature = "std")]
pub use shell::ShellError;
