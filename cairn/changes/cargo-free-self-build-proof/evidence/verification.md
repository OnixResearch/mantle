# Verification Evidence

## Commands

- `cargo test -p mantle --bin mantle rust_plan::tests::native_feature_resolver_ -- --nocapture`
  - Result: 5 passed, 0 failed.
  - Covers positive feature propagation and negative malformed/unknown feature entries.
- `cargo test -p mantle --bin mantle rust_plan::tests::no_cargo_capture_ -- --nocapture`
  - Result: 3 passed, 0 failed.
  - Covers declared vendored registry source binding, captured git source binding, and missing-vendor failure.
- `cargo test -p mantle --bin mantle rust_plan::tests::native_git_ -- --nocapture`
  - Result: 5 passed, 0 failed.
  - Covers captured git source binding, missing captured manifest, mismatched revision, dev-dependency mismatch, and ambiguous same-URL sources.
- `cargo build -p mantle --bin mantle`
  - Result: success with existing unused-code warnings.
- `scripts/prove-cargo-free-rust-plan.sh --full --bundle-dir /tmp/mantle-cargo-free-full-v2 --mantle-bin /home/brittonr/.cargo-target/debug/mantle`
  - Result: success; generated path-workspace smoke output `42`; failed unit count 0.
- `scripts/prove-cargo-free-rust-plan.sh --self-build --bundle-dir target/no-cargo-self-build-probe/script-rerun-20260530T024946Z --mantle-bin /home/brittonr/.cargo-target/debug/mantle --root .`
  - Evidence: pueue task 32.
  - Result summary: schema `mantle-cargo-free-rust-plan-proof-v2`; status `success`; `no_cargo=true`; `mode_blockers=[]`; `executions=599`; `execution_status=success`; `failed_units=0`; Cargo marker absent.
  - Smoke: produced `mantle --help` output starts with `Mantle build system on the Nix store protocol`.
- Fake-mantle blocked-proof probe: `scripts/prove-cargo-free-rust-plan.sh --self-build --bundle-dir target/no-cargo-self-build-probe/blocked-empty-receipt-20260530T025033Z --mantle-bin <fake-mantle-exits-77> --root .`
  - Result: exit 77; wrote `meta.json` with `status=blocked`; wrote `blocker-summary.json` as `[]`; tolerated empty `receipt.json`.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: `valid=true`; changes 1; specs validated 2; no issues.

## Bundle

Self-build bundle: `target/no-cargo-self-build-probe/script-rerun-20260530T024946Z/`

Key files:

- `preflight.json` — tool identity and proof root.
- `receipt.json` — native Rust planning and execution receipt.
- `output-digests.json` — output artifact digests from all unit executions.
- `blocker-summary.json` — empty failed-unit summary (`[]`).
- `meta.json` — success summary, produced binary path, and Cargo guard status.
- `smoke-stdout.txt` / `smoke-stderr.txt` — produced CLI smoke command streams.
- `non-claims.txt` — explicit non-claims.

## Non-claims

This proof is a Mantle native Rust topology self-build proof with Cargo forbidden. It is not Crunch fixed-point self-hosting, source-built bootstrap closure provenance, or release reproducibility evidence.
