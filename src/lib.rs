#![feature(register_tool)]
#![register_tool(tigerstyle)]
// The attestation async dispatcher nests store futures beyond rustc's default query depth.
#![recursion_limit = "256"]
// crunch library crate -- CLI, domain modules, and re-exports for integration tests.

// Evidence enums preserve the stable machine-contract shape; boxing or narrowing errors would alter
// that boundary.
#[allow(clippy::large_enum_variant, clippy::result_large_err)]
mod ambient_env;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::function_length,
    tigerstyle::numeric_units
)]
mod artifact_cmd;
mod ast_grep_evidence;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::catch_all_on_enum,
    tigerstyle::function_length,
    tigerstyle::no_unwrap,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters
)]
mod attest_cmd;
pub mod bootstrap;
mod bootstrap_effect_port;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::numeric_units)]
mod bootstrap_operation;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::compound_condition,
    tigerstyle::numeric_units
)]
mod bootstrap_parity;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod bootstrap_pin_cmd;
mod bootstrap_source_root;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::no_panic,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters
)]
mod bootstrap_validate;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::ambient_clock, tigerstyle::assertion_density, tigerstyle::bool_naming)]
mod build_application;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters
)]
mod build_cmd;
// Compatibility correctness primitives currently exercised by `build_correctness::tests`.
#[allow(dead_code)]
pub mod build_correctness;
mod build_failure;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::no_panic, tigerstyle::numeric_units)]
mod build_interchange;
mod build_log;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::bool_naming)]
mod build_observation;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod build_plan;
// Build-report variants intentionally carry complete stable JSON payloads rather than indirect
// boxed fragments.
#[allow(clippy::large_enum_variant)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::bool_naming, tigerstyle::function_length)]
mod build_report;
// Compatibility-only cache substitution surface retained for `.cairn` specs.
#[allow(dead_code)]
mod cache_substitution;
mod cairn_release_handoff;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow
)]
mod cargo_acquisition;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_loop
)]
mod cargo_build_bundle;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::function_length,
    tigerstyle::numeric_units
)]
mod cargo_build_bundle_cmd;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::bool_naming)]
mod cargo_free_self_build;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::no_unwrap,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod cargo_import;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow
)]
mod cargo_import_cli;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::explicit_defaults,
    tigerstyle::numeric_units
)]
mod cargo_profile;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::no_recursion,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod cargo_profile_manifest;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::unbounded_loop)]
mod causal_trace_diagnostic;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density)]
mod child_exec;
mod cli;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::bool_naming)]
mod command_input;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::explicit_defaults,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow
)]
mod content_bound_requirement_evidence;
// Core tests for the distributed-evaluation feasibility assessment; the probe
// shell (`distributed_eval_assess`) is the only other consumer.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::unbounded_collection_growth
)]
mod distributed_eval_assessment;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::compound_condition,
    tigerstyle::too_many_parameters
)]
mod early_native_row_receipt;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::bool_naming,
    tigerstyle::unbounded_collection_growth
)]
mod early_native_row_receipt_shell;
pub mod elf_dynamic_fixup_core;
pub mod elf_dynamic_fixup_shell;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop,
    tigerstyle::unjustified_allow
)]
mod elf_local_symbol_core;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::compound_condition, tigerstyle::ignored_result)]
mod elf_local_symbol_shell;
pub mod errors;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::too_many_parameters
)]
mod evaluation_cmd;
mod evaluation_stream_output;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::no_panic,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::sentinel_fallback,
    tigerstyle::unbounded_loop
)]
mod evaluator_budget;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::numeric_units, tigerstyle::unbounded_loop)]
mod expression_scratch;
// Retains compatibility-only batch dispatch surfaces pending full caller wiring.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::compound_condition,
    tigerstyle::numeric_units
)]
mod external_batch_dispatch;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::bool_naming)]
mod filegen_cmd;
mod final_native_row_receipt_shell;
mod fix;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod foreign_derivation_import;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::nested_conditionals,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod foreign_executable_plan;
// Retains compatibility-only graph compiler helpers pending caller consolidation.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop,
    tigerstyle::unjustified_allow
)]
mod foreign_graph_compiler;
// Foreign-import adapters mirror external receipt fields and preserve their typed error payloads at
// the CLI boundary.
#[allow(clippy::result_large_err, clippy::too_many_arguments)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod foreign_import_cmd;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::numeric_units,
    tigerstyle::unbounded_collection_growth
)]
mod foreign_provenance_audit;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::unbounded_collection_growth
)]
mod foreign_realization;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::nested_conditionals
)]
mod foreign_realization_receipt;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod foreign_realization_shell;
pub mod fresh_clone_fixed_point;
mod frontend_artifact_export;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density)]
mod frontend_artifact_spec;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::no_unwrap,
    tigerstyle::raw_arithmetic_overflow
)]
mod frontend_artifact_store;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod full_source_provider;
// Retains compatibility-only Rust binding helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::acronym_style,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::explicit_defaults,
    tigerstyle::numeric_units,
    tigerstyle::too_many_parameters
)]
mod full_source_rust_binding;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_loop,
    tigerstyle::unchecked_narrowing
)]
mod full_source_rust_binding_shell;
mod function_address_binding_cmd;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::bool_naming)]
mod generated_file_check;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::sentinel_fallback)]
mod generated_file_write;
mod global_reproducibility_cmd;
mod global_reproducibility_release;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::numeric_units
)]
mod host_environment;
mod linux_rename;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::no_unwrap,
    tigerstyle::raw_arithmetic_overflow
)]
mod log_cmd;
#[cfg(test)]
mod machine_contract_producer_tests;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unjustified_allow
)]
mod mantlepkgs_adapter;
// Mantlepkgs command enum preserves unboxed CLI payloads and existing dispatch adapter arguments.
#[allow(clippy::large_enum_variant, clippy::too_many_arguments)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod mantlepkgs_cmd;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::compound_assertion,
    tigerstyle::ignored_result,
    tigerstyle::no_recursion,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod mantlepkgs_version_cmd;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density)]
mod native_toolchain_closure;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow
)]
mod nickel_export;
mod nickel_export_core_adapter;
// Demo command variants retain complete validation inputs so clap and JSON compatibility stay
// stable.
#[allow(clippy::large_enum_variant)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::no_panic,
    tigerstyle::numeric_units,
    tigerstyle::unbounded_collection_growth
)]
mod nix_derivation_adapter;
// Nix evidence keeps compatibility-only typed proofs available even when a selected command does
// not consume them.
#[allow(dead_code, clippy::type_complexity)]
mod nix_evidence_core;
// Retains compatibility-only demo bundle fields exercised by unit tests.
#[allow(dead_code)]
mod nix_free_demo_bundle;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::explicit_defaults, tigerstyle::numeric_units)]
mod nix_free_demo_cmd;
#[cfg(target_os = "linux")]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::catch_all_on_enum,
    tigerstyle::compound_condition,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod nix_gateway;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::explicit_defaults
)]
mod nix_producer;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod nix_producer_shell;
pub mod oci_projection;
mod oci_projection_shell;
pub mod oci_registry;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::compound_assertion,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::unbounded_collection_growth
)]
mod oci_registry_shell;
// Retains compatibility-only offline Cargo helpers exercised by unit tests.
#[allow(dead_code)]
mod offline_cargo;
pub mod operator_contract;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow
)]
mod operator_diagnostics;
// The comparison report shell lands in task I5 of
// `evaluate-picolibc-stagex-runtime`; until then only core tests consume it.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::unbounded_collection_growth
)]
mod picolibc_comparison;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters
)]
mod pin_import;
// Retains compatibility-only portable receipt helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::too_many_parameters
)]
mod portable_receipt;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::unbounded_collection_growth
)]
mod presentation;
// Preserves carrier types intentionally encode the full external evidence graph and optional
// compatibility surfaces.
#[allow(dead_code, clippy::type_complexity)]
mod preserves_release_carrier;
// Test-only process environment helper module with shared fixture utilities.
#[allow(dead_code)]
#[cfg(test)]
mod process_env;
mod profile_cli;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::ambiguous_params, tigerstyle::assertion_density)]
mod project_build;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::function_length,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units
)]
mod project_cmd;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::bool_naming)]
mod project_resolve;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::sentinel_fallback)]
mod project_retention_fact;
mod proof_clock_seccomp;
pub mod protected_exec;
// Seccomp policy helpers are Linux-only; non-Linux targets retain type definitions for parity.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub mod protected_exec_seccomp;
mod provider_output_publication;
// Retains compatibility-only routing helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::bool_naming)]
mod realization_routing;
mod rebuild_authority;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density)]
mod release_attestation;
// Retains compatibility-only release capability helpers exercised by unit tests.
#[allow(dead_code)]
mod release_capability;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::unbounded_collection_growth
)]
mod release_chapter_transport;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::function_length,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters
)]
mod release_cmd;
// Compatibility-only current pointer model retained for `.cairn` specs.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::ignored_result, tigerstyle::no_unwrap, tigerstyle::numeric_units)]
mod release_current_pointer;
// Retains compatibility-only release evidence helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::compound_condition,
    tigerstyle::no_unwrap,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow
)]
mod release_evidence;
mod release_nix_witness;
mod release_publication;
// Retains compatibility-only release reproducibility helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::no_unwrap,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_loop
)]
mod release_reproducibility;
mod release_source;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::numeric_units)]
mod release_tree_copy;
// Retains compatibility-only remote attempt log store helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::numeric_units)]
mod remote_attempt_log_store;
// Remote build messages retain complete protocol payloads; boxing would change established internal
// handoff shapes.
#[allow(dead_code, clippy::large_enum_variant)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::catch_all_on_enum,
    tigerstyle::compound_condition,
    tigerstyle::contradictory_time,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::nested_conditionals,
    tigerstyle::no_panic,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop,
    tigerstyle::unchecked_narrowing
)]
mod remote_build;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::no_recursion,
    tigerstyle::numeric_units,
    tigerstyle::unbounded_loop
)]
mod remote_credential_state;
pub mod remote_credentials;
mod remote_failure_debug;
mod remote_failure_replay_identity;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::bool_naming)]
mod remote_farm_config;
pub mod remote_nominal;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_loop
)]
mod remote_service_secrets;
mod remote_telemetry_export;
mod remote_test_interruption;
mod remote_trace_context;
// Transfer variants preserve complete resumable protocol records, including compatibility-only
// states.
#[allow(dead_code, clippy::large_enum_variant)]
mod remote_transfer;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density)]
mod run_binary_selection;
mod rust_bootstrap_patch_plan;
// Rust-plan compatibility receipts expose wide Cargo-shaped adapters and preserve detailed typed
// failures.
#[allow(dead_code, clippy::result_large_err, clippy::too_many_arguments)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::nested_conditionals,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_channel,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod rust_plan;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::numeric_units
)]
mod rust_plan_execution;
// Source-provider receipts retain full stage payloads and detailed fail-closed errors across the
// shell boundary.
#[allow(dead_code, clippy::large_enum_variant, clippy::result_large_err)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::no_panic,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters
)]
mod rust_source_provider;
mod rust_source_provider_scratch;
// rustc-dev-guide evidence mirrors upstream compiler graph relationships that are intentionally
// structurally complex.
#[allow(dead_code, clippy::type_complexity)]
mod rustc_dev_guide;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::usize_in_public_api
)]
mod self_build;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::compound_condition,
    tigerstyle::explicit_defaults,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow
)]
mod semantic_graph;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::numeric_units)]
mod semantic_graph_cmd;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::raw_arithmetic_overflow)]
mod shared_rust_signing_key;
mod shell_cmd;
pub mod signing_key;
mod source_built_fixed_point;
mod source_built_fixed_point_action_trust;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::raw_arithmetic_overflow
)]
mod source_built_fixed_point_dev_cache;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod source_built_fixed_point_mantle_actions;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::assertion_density,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod source_built_fixed_point_native_actions;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::platform_dependent_cast)]
mod source_built_fixed_point_planning;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::too_many_parameters
)]
mod source_built_fixed_point_receipt;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod source_built_fixed_point_resume;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::ambiguous_params, tigerstyle::too_many_parameters)]
mod source_built_fixed_point_resume_bundle;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::function_length,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_loop
)]
mod source_built_fixed_point_rust_actions;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::no_recursion,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod source_built_fixed_point_shell;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod source_built_fixed_point_stagex_actions;
pub mod source_built_fixed_point_stagex_adoption;
pub mod source_built_fixed_point_trust_report;
pub mod source_built_fixed_point_verifier_self_test;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::negated_predicate,
    tigerstyle::nested_conditionals,
    tigerstyle::no_panic,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::usize_in_public_api
)]
mod source_built_mantle_programs;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::catch_all_on_enum,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::nested_conditionals,
    tigerstyle::no_panic,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod source_built_native_programs;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::compound_assertion, tigerstyle::compound_condition)]
mod source_built_program_authoring;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::nested_conditionals,
    tigerstyle::no_recursion,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop,
    tigerstyle::unjustified_allow
)]
mod source_built_recipe;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::nested_conditionals,
    tigerstyle::no_recursion,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop,
    tigerstyle::unchecked_narrowing
)]
mod source_built_recipe_exec;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::catch_all_on_enum,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::nested_conditionals,
    tigerstyle::no_recursion,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop,
    tigerstyle::unjustified_allow
)]
mod source_built_rust_programs;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::catch_all_on_enum,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::nested_conditionals,
    tigerstyle::no_recursion,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod source_built_stagex_programs;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_assertion,
    tigerstyle::compound_condition,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop,
    tigerstyle::unchecked_division,
    tigerstyle::unjustified_allow
)]
mod source_bundle;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length
)]
mod source_observation;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::bool_naming,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow
)]
mod source_root_capability;
mod source_root_manifest;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::function_length,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow
)]
mod source_root_provider;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::bool_naming, tigerstyle::numeric_units)]
mod source_root_provider_flow;
mod source_toolchain_closure;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow
)]
mod stagex_archive_core;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod stagex_bash;
// Retains compatibility-only StageX full bash helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::bool_naming,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback
)]
mod stagex_bash_full;
// Retains compatibility-only StageX binutils helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod stagex_binutils;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters
)]
mod stagex_bison;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters
)]
mod stagex_bzip2;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
mod stagex_coreutils;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters
)]
mod stagex_diffutils;
// Retains compatibility-only StageX flex helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters
)]
mod stagex_flex;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters
)]
mod stagex_gawk;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow
)]
mod stagex_gnu_patch;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::raw_arithmetic_overflow
)]
mod stagex_grep;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::bool_naming,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow
)]
mod stagex_gzip;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters
)]
mod stagex_m4;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::bool_naming,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow
)]
mod stagex_make;
// Retains `NYACC_SOURCE_DIR` constant for StageX MES compatibility.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::compound_assertion,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod stagex_mes;
// Retains compatibility-only `stagex_command_capture` helpers loaded via `#[path]`.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop,
    tigerstyle::unjustified_allow
)]
mod stagex_mes_lib;
mod stagex_mes_sources;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod stagex_musl;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::bool_naming,
    tigerstyle::compound_assertion,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unjustified_allow
)]
mod stagex_musl_native;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod stagex_oyacc;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::bool_naming,
    tigerstyle::compound_assertion,
    tigerstyle::sentinel_fallback,
    tigerstyle::unbounded_collection_growth
)]
mod stagex_patch;
// Retains compatibility-only StageX provider helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback
)]
mod stagex_provider;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback
)]
mod stagex_sed;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::raw_arithmetic_overflow)]
mod stagex_sources;
// Retains `derive_stage0_mini_inventory` helper for StageX stage0 compatibility.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::compound_assertion,
    tigerstyle::compound_condition,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::unbounded_loop
)]
mod stagex_stage0;
// Retains compatibility-only StageX full stage0 helpers exercised by unit tests.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::assertion_density,
    tigerstyle::compound_assertion,
    tigerstyle::compound_condition,
    tigerstyle::ignored_result,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::unbounded_loop
)]
mod stagex_stage0_full;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::too_many_parameters
)]
mod stagex_tar;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod stagex_tcc_musl;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod stagex_tcc_musl_prep;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::unbounded_collection_growth
)]
mod stagex_tcc_musl_v2;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::numeric_units,
    tigerstyle::platform_dependent_cast,
    tigerstyle::raw_arithmetic_overflow
)]
mod stagex_tcc_selfhost;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::compound_assertion,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback
)]
mod stagex_tinycc;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow
)]
mod stagex_tinycc27;
// Retains `EXPECTED_PROMOTION_COUNT` constant until M4-C.2 hardening.
#[allow(dead_code)]
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambient_clock,
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::compound_assertion,
    tigerstyle::compound_condition,
    tigerstyle::function_length,
    tigerstyle::ignored_result,
    tigerstyle::no_panic,
    tigerstyle::no_unwrap,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth,
    tigerstyle::unbounded_loop
)]
mod stagex_transition;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::catch_all_on_enum,
    tigerstyle::explicit_defaults,
    tigerstyle::function_length,
    tigerstyle::no_panic,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters
)]
mod store_cmd;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::catch_all_on_enum,
    tigerstyle::function_length,
    tigerstyle::numeric_units
)]
mod structured_refactor;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::numeric_units)]
mod test_cli;
mod test_sentinel;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::function_length,
    tigerstyle::numeric_units
)]
mod transcript_cmd;
mod trusted_keys;
pub mod user_profile_cmd;
// Vendor manifests preserve heterogeneous source identities and compatibility-only validation
// branches.
#[allow(dead_code, clippy::type_complexity)]
mod vendor_source_manifest;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::assertion_density, tigerstyle::too_many_parameters)]
mod verification_gauntlet_cmd;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::numeric_units,
    tigerstyle::raw_arithmetic_overflow,
    tigerstyle::sentinel_fallback,
    tigerstyle::too_many_parameters
)]
mod wasm_component_cmd;
mod witness_handoff;
// HARDENING-BACKLOG 2026-11-25: pre-existing tigerstyle findings from binary-root
// consolidation; scheduled for standalone hardening pass.
#[allow(tigerstyle::ambient_clock)]
mod witness_rebuild;

#[cfg(test)]
pub(crate) use cli::Args;
pub(crate) use cli::ArtifactAction;
pub(crate) use cli::AttestAction;
pub(crate) use cli::AttestPolicyProfileArg;
pub(crate) use cli::AttestVerifyAction;
#[cfg(test)]
pub(crate) use cli::BUILD_JSON_REPORT_SCHEMA;
pub(crate) use cli::BuildOutputMode;
pub(crate) use cli::BuildProjectExprRequest;
#[cfg(test)]
pub(crate) use cli::Command;
pub(crate) use cli::EXPECTED_RUN_OUTCOME_COUNT;
pub(crate) use cli::FilegenCommandAction;
pub(crate) use cli::GauntletReportKind;
pub(crate) use cli::PreparedRemoteDispatch;
pub(crate) use cli::ReceiptAction;
pub(crate) use cli::ReceiptBundleAction;
pub(crate) use cli::ReleaseAction;
pub(crate) use cli::ReleaseGauntletAction;
pub(crate) use cli::ReleaseTransportAction;
pub(crate) use cli::RemoteAction;
pub(crate) use cli::RemoteBuildSelection;
pub(crate) use cli::RemoteBuildSelectionInput;
pub(crate) use cli::RemoteDispatchExecutionInput;
pub(crate) use cli::RemoteDispatchPreparationInput;
pub(crate) use cli::RemoteFailureDebugAction;
pub(crate) use cli::RemotePoolAction;
pub(crate) use cli::RemoteServeBinding;
pub(crate) use cli::RemoteServeExecutor;
pub(crate) use cli::RemoteTicketAction;
pub(crate) use cli::RunContext;
pub(crate) use cli::RustPlanExecutionMode;
pub(crate) use cli::RustPlanExecutionRequest;
pub(crate) use cli::SourceAction;
pub(crate) use cli::SourceBundleAction;
pub(crate) use cli::StoreAction;
pub(crate) use cli::StoreArchiveAction;
pub(crate) use cli::StoreArchiveFormat;
pub(crate) use cli::StoreCompositionAction;
pub(crate) use cli::WasmComponentAction;
pub(crate) use cli::build_project_expr;
pub(crate) use cli::configure_remote_dispatch_diagnostics;
pub(crate) use cli::current_dir_or_error;
pub(crate) use cli::empty_remote_telemetry_buffer;
pub(crate) use cli::execute_remote_dispatch;
pub use cli::main as cli_main;
pub(crate) use cli::name_to_build_target;
pub(crate) use cli::open_remote_dispatch_store;
pub(crate) use cli::prepare_remote_dispatch;
pub(crate) use cli::remote_build_selection;
pub(crate) use cli::remote_client_build_json_report;
pub(crate) use cli::run_remote_live_command;
pub(crate) use cli::source_root_provider_store_name;
pub(crate) use cli::standard_remote_transfer_policy;
pub(crate) use cli::unix_time_now_s;
pub use errors::RunError;
