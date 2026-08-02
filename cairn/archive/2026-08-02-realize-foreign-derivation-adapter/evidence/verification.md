# Verification

Task-ID: V1, V2, V3
Covers: r[foreign_derivation_import.realization_adapter], r[foreign_derivation_import.realization_receipt]
Date: 2026-08-02

## V1 focused rail

All commands ran from the change worktree.

```text
nix develop -c check-nickel-configs
exit: 0
result: positive profiles passed; invalid unknown-field and setid fixtures failed as expected

nix develop -c cargo test -p crunch-build build_request::
exit: 0
test result: ok. 40 passed; 0 failed; 631 filtered out

nix develop -c cargo test -p crunch-build fetch_build_service::
exit: 0
test result: ok. 23 passed; 0 failed; 648 filtered out

nix develop -c cargo test -p mantle --test foreign_import_cli
exit: 0
test result: ok. 14 passed; 0 failed
```

Final hardening checks also passed:

```text
nix develop -c cargo check --workspace --all-targets
exit: 0

nix develop -c cargo test -p crunch-build worker::
exit: 0
test result: ok. 51 passed; 0 failed

nix develop -c cargo test -p crunch-store verified_source_ingest
exit: 0
test result: ok. 2 passed; 0 failed

nix develop -c cargo test -p crunch-pipeline
exit: 0
unit tests: 27 passed; 0 failed
integration tests: 19 passed; 0 failed; 4 ignored
```

## V2 local realization evidence

The first command runs the two-node realization and the Guix no-`/bin/sh` case.
Both cases use `--no-substitute` inside the checked CLI fixture.

```text
nix develop -c cargo test -p mantle --test foreign_import_cli foreign_import_cli_realizes_two_node_graph_and_reuses_exact_outputs -- --nocapture
exit: 0
two-node build_report_blake3=d43e02fbb71558fa67a938df41558be076a2ec53625795704a83fcc304e27a38
guix-no-bin-sh build_report_blake3=eb97a276c7768c7c8d85cb78fc2a3d233a63965233f529ff32308446fdcc0c9a
test result: ok. 1 passed; 0 failed

nix develop -c cargo test -p mantle --test foreign_import_cli foreign_import_cli_reports_fixed_output_mismatch_from_admitted_source_state -- --nocapture
exit: 0
fixed-output-mismatch build_report_blake3=f6c3a38233fc1be5df15293d7bec37258a4af578890b99afc000412fce20054d
test result: ok. 1 passed; 0 failed
```

The two-node case built the child before the parent and preserved the exact child output bytes.
A second run classified both units as `already-present`.
The Guix profile did not provide `/bin/sh` and produced bounded partial evidence.
The fixed-output mismatch selected admitted source state, rejected the wrong content, and did not admit PathInfo.

## V3 policy and lifecycle rail

```text
nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs
exit: 0
foreign import trust-model doc check passed

nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test
exit: 0
foreign import trust-model checker self-test passed

git diff --check
exit: 0

nix develop -c cargo clippy -p crunch-build -p crunch-pipeline -p crunch-store --lib --no-deps -- -D warnings
exit: 0

nix develop -c cargo clippy -p mantle --bin mantle --no-deps -- -D warnings
exit: 0
```

The broader all-target Clippy command reached pre-existing failures in vendored
`fuse-backend-rs` and an unrelated constant assertion in
`src/protected_exec_seccomp.rs`. The focused first-party library and Mantle
binary rails pass with warnings denied.

The proposal, design, and tasks gates passed with the external current Cairn
policy. Strict Cairn validation passed with no issues or findings.

The pre-sync Tracey comparison reported the known repository baseline:
259 references and 686 requirements. It also reported the new execution-profile
requirement as dangling because accepted-spec sync had not run yet. After sync,
Tracey reported 263 references and 690 requirements. None of the four new
requirement IDs remained missing or dangling.

The machine-schema checker reached pre-existing StageX and source-build
inventory debt. It did not report the new foreign realization sources.
