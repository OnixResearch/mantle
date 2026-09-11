#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Pure Rust-planning core.
//!
//! Adapters decode manifests, lockfiles, Cargo metadata, and unit graphs into
//! the bounded structural facts below. This core admits those facts into
//! nominal package, target, feature, and unit values, plans deterministic
//! units and ordered effects, classifies typed observations, and builds
//! receipt preimages. It never touches files, processes, environment, clocks,
//! caches, or Cargo itself.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod digest;
mod features;
mod model;
mod plan;
mod receipt;

pub use digest::Blake3Digest;
pub use digest::domain_digest;
pub use features::DEFAULT_FEATURE;
pub use features::FeatureResolution;
pub use features::MAX_FEATURE_PASSES;
pub use features::PackageFeatureRequest;
pub use features::resolve_package_features;
pub use model::DependencyFacts;
pub use model::DependencyKind;
pub use model::FeatureFacts;
pub use model::FeatureReference;
pub use model::MAX_ARGS_PER_EFFECT;
pub use model::MAX_DEPENDENCIES_PER_PACKAGE;
pub use model::MAX_ENVIRONMENT_ENTRIES_PER_EFFECT;
pub use model::MAX_FEATURES_PER_PACKAGE;
pub use model::MAX_PACKAGES;
pub use model::MAX_TARGETS_PER_PACKAGE;
pub use model::PLAN_SCHEMA;
pub use model::PackageFacts;
pub use model::PackageSource;
pub use model::PackageSourceKind;
pub use model::PlanBlocker;
pub use model::TargetFacts;
pub use model::TargetKind;
pub use plan::BuildProfile;
pub use plan::EffectId;
pub use plan::MAX_EFFECTS;
pub use plan::MAX_UNITS;
pub use plan::PlanExecutionOutcome;
pub use plan::PlanOutcome;
pub use plan::PlanRequest;
pub use plan::PlannedUnit;
pub use plan::RustPlan;
pub use plan::UnitEffect;
pub use plan::UnitId;
pub use plan::UnitLimitFacts;
pub use plan::UnitObservation;
pub use plan::UnitObservationStatus;
pub use plan::classify_plan_observations;
pub use plan::plan_rust_units;
pub use receipt::PlanReceiptPreimage;
pub use receipt::RECEIPT_PREIMAGE_SCHEMA;
pub use receipt::build_receipt_preimage;
