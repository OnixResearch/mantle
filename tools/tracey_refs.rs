// Mantle Tracey coverage bridge.
//
// Cairn's built-in tracey coverage rail currently scans `crates/` and `tools/`.
// Mantle's CLI/root-package implementation lives under top-level `src/`, so
// synced requirements implemented there need a small bridge until the coverage
// rail scans the package root directly.

// r[impl rust_package_planning.source_built_toolchain_closure]
// Implemented by `src/source_toolchain_closure.rs`, `src/cargo_free_self_build.rs`,
// `src/rust_plan.rs`, and `src/main.rs`.

// r[verify rust_package_planning.source_built_toolchain_closure]
// Verified by archived evidence in
// `cairn/archive/2026-05-31-source-built-toolchain-closure/evidence/` and focused
// unit tests under the root package modules named above.

// Build-tool boundary first-batch bridge.
//
// r[impl build_tool_boundary.mantle_not_module_layer]
// Mantle-side implementation is the frontend-neutral build boundary in the root
// CLI/build path plus removal of the old `mantle system` surface. Durable design
// evidence lives in `adr/0010-keep-mantle-build-tool-boundary.md` and archived
// Cairn evidence under `cairn/archive/2026-05-31-onix-module-eval-boundary/`.
//
// r[verify build_tool_boundary.mantle_not_module_layer]
// Verified by `tests/removed_system_cli.rs`: build-shaped frontend inputs reach
// planning, raw module inventory is rejected, and removed module-layer surfaces
// stay absent from public and implementation scans.
//
// r[impl build_tool_boundary.onix_owns_module_lowering]
// Mantle implements only its side of this boundary: Onix module lowering remains
// outside Mantle core, and Mantle accepts build-shaped inputs after an external
// frontend has done module evaluation. This is traceability for the Mantle-side
// non-ownership rule; it does not claim Onix module lowering is implemented here.
//
// r[verify build_tool_boundary.onix_owns_module_lowering]
// Verified by `tests/removed_system_cli.rs` and the archived
// `onix-module-eval-boundary` evidence, which guard against raw Onix role/tag /
// provider semantics becoming Mantle build inputs or diagnostics.
//
// r[impl build_tool_boundary.synthetic_system_eval_not_integration]
// Mantle-side implementation quarantines the old synthetic system scaffold by
// removing it from supported CLI/docs/stdlib surfaces and by keeping production
// integration dependent on an external frontend lowering into build inputs.
//
// r[verify build_tool_boundary.synthetic_system_eval_not_integration]
// Verified by `tests/removed_system_cli.rs::system_eval_is_not_a_supported_subcommand`
// and the public/implementation surface scans in that test module.

// Compiled-eval legacy OpenSpec bridge.
//
// These references close Tracey linkage to the archived, legacy OpenSpec
// compiled-eval prototype work only. They do not claim a shipped compiled
// evaluator, full Nickel semantic support, or renewed priority for codegen work.
// Evidence mapping is recorded in the active Tracey change evidence file
// `compiled-eval-backfill-map-2026-06-01.md`.
//
// r[impl compiled-eval.backend-boundary]
// r[verify compiled-eval.backend-boundary]
// Implemented by the private `crunch-eval` backend seam in
// `crates/crunch-eval/src/backend.rs` and existing public helpers in
// `crates/crunch-eval/src/lib.rs`; verified by archived V1/V3 evidence.
//
// r[impl compiled-eval.backend-boundary.default]
// r[verify compiled-eval.backend-boundary.default]
// Current default helpers call `default_backend()` / `NickelBackend`, while the
// Cranelift path is gated behind `cranelift-proto`; archived tests cover default
// interpreter behavior.
//
// r[related compiled-eval.backend-boundary.swap]
// Future backend-swap compatibility is represented as a private-seam design
// constraint only; no future backend support is claimed by this bridge.
//
// r[related compiled-eval.profiling-gate]
// r[related compiled-eval.profiling-gate.no-evidence]
// r[related compiled-eval.profiling-gate.evidence]
// Archived V2/design evidence records profiling/benchmark gates and the current
// decision to keep compiled-eval tabled unless future data reopens it.
//
// r[related compiled-eval.benchmark-guardrail]
// r[related compiled-eval.benchmark-guardrail.recorded]
// r[related compiled-eval.benchmark-guardrail.regression]
// Archived benchmark evidence records the baseline bundle and a non-eval
// guardrail regression, so the prototype stayed non-shipping.
//
// r[related compiled-eval.cranelift-first]
// r[related compiled-eval.cranelift-first.initial]
// r[related compiled-eval.cranelift-first.llvm-later]
// Archived design records Cranelift as the first experiment and keeps LLVM as
// optional future work requiring a later measured reason.
//
// r[impl compiled-eval.private-backend-seam]
// r[verify compiled-eval.private-backend-seam]
// r[impl compiled-eval.private-backend-seam.callers]
// r[verify compiled-eval.private-backend-seam.callers]
// r[impl compiled-eval.private-backend-seam.no-leak]
// r[verify compiled-eval.private-backend-seam.no-leak]
// `EvalBackend`, `EvalRequest`, and Cranelift request handling stay private to
// `crunch-eval`; existing public helper signatures remain interpreter-shaped.
//
// r[impl compiled-eval.cranelift-prototype-subset]
// r[verify compiled-eval.cranelift-prototype-subset]
// r[impl compiled-eval.cranelift-prototype-subset.default]
// r[verify compiled-eval.cranelift-prototype-subset.default]
// r[impl compiled-eval.cranelift-prototype-subset.unsupported]
// r[verify compiled-eval.cranelift-prototype-subset.unsupported]
// The prototype is feature-gated, default-off, limited to flat derivation
// fields, and rejects unsupported fields/import paths; archived V1/V4 evidence
// records the focused tests and source audit.
//
// r[related compiled-eval.future-semantics]
// r[related compiled-eval.future-semantics.contracts]
// r[related compiled-eval.future-semantics.shape]
// Future semantic-equivalence requirements remain non-claims: the current guard
// is that unsupported broader semantics stay outside the prototype and the
// interpreter remains the reference path.
