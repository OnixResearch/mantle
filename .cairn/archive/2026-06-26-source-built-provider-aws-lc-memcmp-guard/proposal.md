## Why

A fresh current-code provider-backed fixed-point proof now reaches `aws-lc-sys` and blocks in its GCC PR95189 `memcmp` compiler guard. The existing provider-bound release evidence remains valid for the recorded artifact set, but a current provider binary cannot yet be regenerated and packaged as its own verifier.

Mantle must handle this frontier honestly: if a source-built native compiler fails AWS-LC's guard, the proof must stay blocked with explicit compiler-risk evidence; if a safe source-built route exists, Mantle must select it through receipt-bound toolchain closure data rather than ambient host tools or environment spoofing.

## What Changes

- Add an explicit requirement for AWS-LC compiler-guard handling in provider-backed Cargo-free topology execution.
- Record the current blocker evidence from the fresh fixed-point attempt.
- Investigate and implement a source-built, receipt-bound route that either passes AWS-LC's guard or reports a deterministic unsupported-compiler blocker.
- Rerun the provider-backed fixed-point proof from current code and record whether the frontier moved.

## Impact

- **Files**: `src/rust_plan.rs`, `src/cargo_free_self_build.rs`, source toolchain closure helpers/tests, Cairn specs/evidence.
- **Testing**: focused Rust tests for compiler-guard policy, negative host/env spoofing tests, receipt checks for blocker classification, and a real provider-backed fixed-point rerun.
