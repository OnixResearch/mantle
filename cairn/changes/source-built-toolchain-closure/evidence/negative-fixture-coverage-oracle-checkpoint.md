# Oracle checkpoint: negative fixture coverage completeness

Task-ID: fixture-negative-oracle
Covers: rust_package_planning.source_built_toolchain_closure

## Question

Does the negative fixture suite satisfy the exact task text covering missing toolchain members, digest mismatches, placeholder seeds, host `rustc`/linker/pkg-config leakage, policy digest mismatch between stages, and attempts to promote current fixed-point evidence to source-built closure evidence?

## Inspected evidence

- `src/source_toolchain_closure.rs`
  - `validator_rejects_missing_required_role`
  - `enforcement_rejects_declared_path_with_digest_mismatch`
  - `validator_rejects_placeholder_seed_exception_reason`
  - `validator_rejects_unverified_seed_exception_name`
  - `enforcement_rejects_undeclared_host_rustc_path`
  - `enforcement_rejects_undeclared_host_linker_path`
  - `enforcement_rejects_undeclared_host_pkg_config_path`
- `src/cargo_free_self_build.rs`
  - `receipt_bound_enforcement_rejects_linker_digest_mismatch`
  - `receipt_bound_enforcement_rejects_pkg_config_digest_mismatch`
  - `receipt_bound_enforcement_rejects_c_compiler_digest_mismatch`
  - `receipt_bound_enforcement_rejects_sysroot_leakage`
  - `fixed_point_status_rejects_policy_digest_mismatch_before_success`
  - `fixed_point_status_rejects_policy_digest_presence_mismatch_before_success`
  - `fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches`
- `tests/cargo_free_self_build_cli.rs`
  - `cargo_free_self_build_rejects_invalid_toolchain_closure_manifest`
  - `cargo_free_self_build_rejects_undeclared_host_rustc_before_unit_execution`
  - `cargo_free_self_build_rejects_placeholder_seed_exception_manifest`
  - `cargo_free_fixed_point_rejects_invalid_toolchain_closure_manifest`
- `cairn/changes/source-built-toolchain-closure/evidence/fixture-test-validation.md`
  - pueue test transcripts for the focused source-toolchain, cargo-free unit, and CLI fixtures.

## Decision

Yes, after adding explicit pure enforcement tests for undeclared host linker and pkg-config paths, the negative fixture task can remain complete. The previously existing receipt-bound shell tests still matter, but the exact host linker/pkg-config leakage wording is now backed by concrete test names instead of a vague reference to existing coverage.

This decision does not promote current fixed-point evidence to a source-built toolchain closure proof. The promotion guard remains `fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches`, and the real end-to-end source-built closure fixed-point proof task remains unchecked.

## Owner

Mantle maintainer / current agent.

## Next action

Keep the real end-to-end source-built toolchain closure fixed-point proof unchecked until a positive proof bundle records closure digest, seed exceptions, stage receipts, stage binary digests, Cargo guard status, and remaining non-claims.
