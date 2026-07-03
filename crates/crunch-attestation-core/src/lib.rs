#![no_std]

//! `crunch-attestation-core`: compiler-enforced no-std home for extracted
//! attestation functional-core logic.
//!
//! This first wave owns foundational attestation data types plus digest and
//! schema/version validation. Std-facing filesystem discovery and higher-level
//! adapter ergonomics stay in `crunch-attestation`.

extern crate alloc;

mod canonical;
mod digest;
mod error;
mod policy;
mod release;
mod schema;
mod version;

pub use canonical::artifact_attestation_canonical_bytes;
pub use canonical::artifact_attestation_canonical_digest;
pub use canonical::canonical_artifact_attestation;
pub use canonical::canonical_closure_attestation;
pub use canonical::canonical_project_attestation;
pub use canonical::closure_attestation_canonical_bytes;
pub use canonical::closure_attestation_canonical_digest;
pub use canonical::project_attestation_canonical_bytes;
pub use canonical::project_attestation_canonical_digest;
pub use digest::AttestationDigest;
pub use error::Error;
pub use policy::PolicyEvaluation;
pub use policy::PolicyEvaluationInput;
pub use policy::PolicyFailureReason;
pub use policy::RELEASE_POLICY_SCHEMA;
pub use policy::RELEASE_REVOCATIONS_SCHEMA;
pub use policy::ReleasePolicy;
pub use policy::ReleaseRevocations;
pub use policy::ValidatedWitness;
pub use policy::evaluate_policy;
pub use release::AgreementWitnessClassification;
pub use release::BinaryDigest;
pub use release::BinaryDigestMatchInput;
pub use release::DetachedSignature;
pub use release::FinalClass;
pub use release::INDEPENDENT_AGREEMENT_REPORT_SCHEMA;
pub use release::IndependentAgreementReport;
pub use release::IndependentAgreementReportInit;
pub use release::IndependentAgreementStatus;
pub use release::PolicyStatus;
pub use release::RELEASE_ATTESTATION_SCHEMA;
pub use release::RebuildEnvironmentSummary;
pub use release::ReleaseAttestation;
pub use release::ReleaseAttestationInit;
pub use release::SignatureSuite;
pub use release::TechnicalClass;
pub use release::TrustTier;
pub use release::WITNESS_ATTESTATION_SCHEMA;
pub use release::WITNESS_SOURCE_ACQUISITION_MODE_NOT_RECORDED;
pub use release::WITNESS_SOURCE_ACQUISITION_MODE_UNSPECIFIED;
pub use release::WitnessAttestation;
pub use release::WitnessClassificationReason;
pub use release::Workflow;
pub use release::binary_digests_match;
pub use release::canonical_independent_agreement_report;
pub use release::canonical_release_attestation;
pub use release::canonical_witness_attestation;
pub use release::encode_detached_signature;
pub use release::independent_agreement_report_canonical_bytes;
pub use release::independent_agreement_report_canonical_digest;
pub use release::parse_detached_signature;
pub use release::release_attestation_canonical_bytes;
pub use release::release_attestation_canonical_digest;
pub use release::witness_attestation_canonical_bytes;
pub use release::witness_attestation_canonical_digest;
pub use schema::ArtifactAttestation;
pub use schema::ArtifactFacts;
pub use schema::ArtifactReference;
pub use schema::Claims;
pub use schema::ClosureAttestation;
pub use schema::ClosureFacts;
pub use schema::ClosureSemantics;
pub use schema::Edge;
pub use schema::EdgeKind;
pub use schema::Node;
pub use schema::NodeKind;
pub use schema::ProjectAttestation;
pub use schema::ProjectFacts;
pub use version::SchemaVersion;
