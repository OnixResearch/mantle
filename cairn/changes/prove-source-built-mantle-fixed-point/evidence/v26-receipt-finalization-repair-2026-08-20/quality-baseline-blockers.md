# Unchanged quality-gate blockers

These findings are outside the receipt-finalization repair.

## Package formatting

Pueue task `773` ran `nix develop -c cargo fmt --check -p mantle -v`.
It reported import-order drift in unchanged files:

- `examples/distributed_eval_assess.rs`
- `src/source_built_fixed_point_shell.rs`

Changed-file `rustfmt --check` passes.

## Ordinary focused Clippy

Pueue task `775` ran strict root-package Clippy. It reported three unchanged
findings:

- two unused `as_str` methods generated in `src/remote_nominal.rs`;
- one `large_enum_variant` finding in `src/nix_free_demo_cmd.rs`.

The isolated receipt-focused Clippy command permits only those two known lint
classes at command scope. It passes without source suppressions. See
`local-validation.log`.

The first shared-target retry hit a rustc incremental-cache ICE while decoding
`AttrId` for `crunch-delta`. See `clippy-shared-cache-ice.log`. An isolated
`CARGO_TARGET_DIR` removed the cache fault.

## Tiger Style

Pueue task `781` ran `./scripts/check-first-party-tigerstyle.sh -p mantle`.
It reported existing findings in `bootstrap.rs`, `operator_contract.rs`,
`protected_exec.rs`, `protected_exec_seccomp.rs`, and `errors.rs`. It reported
no finding in `src/preserved_evidence_tree.rs` or the changed receipt code.

This file records blockers only. It does not claim the package-wide quality
rails pass.
