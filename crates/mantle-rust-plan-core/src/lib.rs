#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Pure Rust-plan policy over adapter-decoded native package, feature, target,
//! topology, process-effect, cache-observation and receipt facts. Adapters own
//! host state and encoding; this core performs bounded deterministic admission
//! without touching files, processes, environments, clocks, caches or Cargo.
//!
//! Host authorities are intentionally absent from the core API. These
//! examples are compiled negative contracts, not source-text inspections.
//!
//! Filesystem access is an adapter observation, not a method on a target:
//! ```compile_fail
//! use mantle_rust_plan_core::NativeTargetCandidate;
//! fn read_source(target: &NativeTargetCandidate) {
//!     target.source_path.read_to_string();
//! }
//! ```
//!
//! Process invocation belongs to the shell:
//! ```compile_fail
//! use mantle_rust_plan_core::ResolvedUnitEffect;
//! fn execute(effect: &ResolvedUnitEffect) {
//!     effect.spawn();
//! }
//! ```
//!
//! Environment lookup is not a feature of admitted process facts:
//! ```compile_fail
//! use mantle_rust_plan_core::ResolvedUnitFacts;
//! fn read_environment(facts: &ResolvedUnitFacts) {
//!     facts.environment.get_var("PATH");
//! }
//! ```
//!
//! Cargo oracle capture cannot be called from core:
//! ```compile_fail
//! use mantle_rust_plan_core::cargo::metadata;
//! ```
//!
//! The admitted rustc invocation is data, not a compiler handle:
//! ```compile_fail
//! use mantle_rust_plan_core::ResolvedUnitEffect;
//! fn compile(effect: &ResolvedUnitEffect) {
//!     effect.invoke_rustc();
//! }
//! ```
//!
//! Store mutation belongs to an adapter:
//! ```compile_fail
//! use mantle_rust_plan_core::ResolvedUnitEffect;
//! fn store(effect: &ResolvedUnitEffect) {
//!     effect.materialize_store_artifact();
//! }
//! ```
//!
//! Cache restoration cannot be performed by a core observation:
//! ```compile_fail
//! use mantle_rust_plan_core::RestoredCompilerArtifactObservation;
//! fn restore(observation: &RestoredCompilerArtifactObservation) {
//!     observation.restore_cached_output();
//! }
//! ```
//!
//! Paths are opaque strings, not host path handles:
//! ```compile_fail
//! use mantle_rust_plan_core::NativeTargetCandidate;
//! fn traverse(target: &NativeTargetCandidate) {
//!     target.source_path.join("src");
//! }
//! ```
//!
//! Admitted effects are not async-runtime futures:
//! ```compile_fail
//! use mantle_rust_plan_core::ResolvedUnitEffect;
//! async fn drive(effect: ResolvedUnitEffect) {
//!     effect.await;
//! }
//! ```
//!
//! CLI dispatch is not exported by the core:
//! ```compile_fail
//! use mantle_rust_plan_core::cli::run;
//! ```
//!
//! Receipt rendering is likewise adapter-owned:
//! ```compile_fail
//! use mantle_rust_plan_core::rendering::render;
//! ```

extern crate alloc;

#[cfg(test)]
extern crate std;

mod cfg;
mod digest;
mod features;
mod model;
mod plan;
mod receipt;

pub use cfg::MAX_CFG_EVALUATION_STEPS;
pub use cfg::NativeDependencyEligibility;
pub use cfg::classify_native_dependency_from_selected_cfg;
pub use cfg::evaluate_supported_target_cfg;
pub use digest::Blake3Digest;
pub use digest::domain_digest;
pub use features::DEFAULT_FEATURE;
pub use features::MAX_NATIVE_FEATURE_STEPS;
pub use features::NativeDependencyFeatureEdge;
pub use features::NativeFeatureDefinition;
pub use features::NativeFeatureSelection;
pub use features::NativeFeatureSelectionRequest;
pub use features::classify_target_cfg_dependency;
pub use features::native_optional_dependency_selected;
pub use features::resolve_native_feature_selection;
pub use model::MAX_ARGS_PER_EFFECT;
pub use model::MAX_DEPENDENCIES_PER_PACKAGE;
pub use model::MAX_ENVIRONMENT_ENTRIES_PER_EFFECT;
pub use model::MAX_FEATURES_PER_PACKAGE;
pub use model::MAX_PACKAGES;
pub use model::MAX_TARGETS_PER_PACKAGE;
pub use model::NativeAdmittedTarget;
pub use model::NativeInheritedValue;
pub use model::NativeTargetCandidate;
pub use model::NativeTargetClassification;
pub use model::NormalizedPackageFacts;
pub use model::PlanBlocker;
pub use model::admit_native_targets;
pub use model::classify_native_target_triple;
pub use model::resolve_native_package_edition;
pub use model::resolve_native_package_version;
pub use model::select_native_execution_triple;
pub use model::select_normalized_package_closure;
pub use plan::BuildProfile;
pub use plan::CompilerRestoreKind;
pub use plan::EffectId;
pub use plan::ExistingTopologyUnit;
pub use plan::ExistingUnitEffect;
pub use plan::ExistingUnitFacts;
pub use plan::MAX_PROCESS_WORD_BYTES;
pub use plan::MAX_RESOLVED_EFFECT_BYTES;
pub use plan::MAX_UNITS;
pub use plan::PlanExecutionOutcome;
pub use plan::ProcessEffectRole;
pub use plan::ProcessWord;
pub use plan::ProducerArtifactObservation;
pub use plan::ResolvedProcessObservation;
pub use plan::ResolvedUnitEffect;
pub use plan::ResolvedUnitFacts;
pub use plan::RestoredCompilerArtifactObservation;
pub use plan::RustPlanCompatibilityDecision;
pub use plan::UnitId;
pub use plan::UnitObservation;
pub use plan::UnitObservationStatus;
pub use plan::admit_resolved_build_script_after_cache;
pub use plan::admit_resolved_unit_effect;
pub use plan::classify_cargo_free_planning_blockers;
pub use plan::classify_existing_unit_observations;
pub use plan::classify_resolved_process_observation;
pub use plan::classify_restored_compiler_artifact;
pub use plan::classify_rust_plan_compatibility;
pub use plan::order_existing_unit_topology;
pub use plan::plan_existing_unit_effects;
pub use plan::plan_native_runtime_arguments;
pub use receipt::hash_legacy_receipt_preimage;
