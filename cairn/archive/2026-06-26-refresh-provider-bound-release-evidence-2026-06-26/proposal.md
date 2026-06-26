## Why

The provider-backed fixed-point proof now succeeds from current code after native static-PIE CRT normalization. Mantle should record a fresh provider-bound release evidence transcript that packages that current provider binary, verifies deterministic-release and provider fixed-point gates, and replaces the prior transcript's "current-code provider proof blocker" note with successful evidence.

## What Changes

- Create release evidence bundle `provider-bound-release-evidence-2026-06-26` under `target/release-evidence/`.
- Package provider fixed-point proof bundle `mantle-source-built-rust-provider-fixed-point-native-static-pie-crt-2026-06-26-rerun4` and bind its stage binary digest to `binaries/01-mantle`.
- Run release reproducibility with deterministic proof runs, required release verification, receipt checking, positive portable replay, and negative missing-proof replay.
- Record the durable transcript under this Cairn change.

## Scope

In scope: evidence transcript and validation for the refreshed provider-bound release bundle.

Out of scope: changing release evidence semantics, claiming full bootstrap reproducibility, compiler correctness, deploy success, or full Cargo compatibility.
