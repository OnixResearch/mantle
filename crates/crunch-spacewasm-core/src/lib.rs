#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Pure deterministic admission, comparison, decision, bundle, and report
//! logic for Mantle's pinned NASA SpaceWasm reference cohort. Filesystem,
//! process, network, environment, clock, and rendering effects belong to the
//! `crunch-spacewasm` shell or the Nix materialization lane.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod admission;
mod bundle;
mod diagnostic;
mod digest;
mod model;
mod profile;
mod report;
mod results;
mod stable_report;
mod support;

pub use admission::SourceAdmission;
pub use admission::SourceFacts;
pub use admission::admit_source;
pub use bundle::BUNDLE_MANIFEST_SCHEMA;
pub use bundle::BundleManifest;
pub use bundle::BundleManifestInput;
pub use bundle::BundleVerification;
pub use bundle::build_bundle_manifest;
pub use bundle::plan_bundle_parent_edges;
pub use bundle::verify_bundle_manifest;
pub use diagnostic::Diagnostic;
pub use diagnostic::DiagnosticSeverity;
pub use digest::BLAKE3_HEX_LENGTH;
pub use digest::Blake3Digest;
pub use digest::DigestError;
pub use model::*;
pub use profile::PROFILE_SCHEMA;
pub use profile::ProfileValidation;
pub use profile::REQUIRED_NON_CLAIMS;
pub use profile::cohort_identity;
pub use profile::validate_profile;
pub use report::MATERIALIZATION_REPORT_SCHEMA;
pub use report::MaterializationReport;
pub use report::REFERENCE_CLAIM_CLASS;
pub use report::ReportBuildInput;
pub use report::ReportDisposition;
pub use report::ReportValidation;
pub use report::build_materialization_report;
pub use report::validate_materialization_report;
pub use results::CheckDecision;
pub use results::CheckEvaluation;
pub use results::evaluate_checks;
pub use stable_report::HarnessLine;
pub use stable_report::STABLE_REPORT_ENCODING_VERSION;
pub use stable_report::STABLE_REPORT_SCHEMA;
pub use stable_report::StableReport;
pub use stable_report::StableReportRequest;
pub use stable_report::StableReportResult;
pub use stable_report::StableTestRecord;
pub use stable_report::StableTestStatus;
pub use stable_report::admit_stable_report;
pub use stable_report::parse_libtest_events;
pub use support::SupportComparison;
pub use support::compare_support_matrix;

pub const ADJACENT_WINDOW_LENGTH: usize = 2;

#[cfg(test)]
mod tests;
