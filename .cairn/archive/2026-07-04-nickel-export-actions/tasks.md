# Tasks

## Contract

- [x] [serial] Define the declared Nickel export action, export receipt fields, supported formats, import-path safety policy, and bounded non-claims. r[build_correctness.nickel_export_action]
- [x] [serial] Define Nickel evaluator/toolchain facts that bind binary identity, version, and evaluator options without importing Bazel toolchain semantics. r[build_correctness.nickel_toolchain_provider]
- [x] [serial] Define human and JSON diagnostics for export success, failure classes, and stdout/stderr boundaries. r[operator_diagnostics.nickel_export_diagnostics]

## Implementation

- [x] [serial] Implement pure export request normalization and validation for sources, deps, import paths, formats, output targets, and evaluator descriptors. r[build_correctness.nickel_export_action] r[build_correctness.nickel_toolchain_provider]
- [x] [serial] Implement the `mantle export` shell that evaluates Nickel, computes BLAKE3 source/output digests, writes explicit outputs, and emits stable diagnostics. r[build_correctness.nickel_export_action] r[operator_diagnostics.nickel_export_diagnostics]
- [x] [serial] Thread export receipts into build-correctness evidence where exported Nickel data feeds downstream action specs. r[build_correctness.nickel_export_action]

## Verification

- [x] [serial] Add positive tests for JSON export, safe import paths, declared dependency imports, stdout-only exports, output-file exports, and deterministic receipt fields. r[build_correctness.nickel_export_action]
- [x] [serial] Add negative tests for absolute import paths, `..` escapes, missing dependencies, unsupported formats, evaluator mismatch, and JSON stdout contamination. r[build_correctness.nickel_export_action] r[operator_diagnostics.nickel_export_diagnostics]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before implementation tasks are marked complete. r[build_correctness.nickel_export_action]

Evidence: `nix develop -c cargo test -p mantle --bin mantle nickel_export`, temp-directory `mantle --json export` smoke, and Cairn proposal/design/tasks gates passed for this change.
