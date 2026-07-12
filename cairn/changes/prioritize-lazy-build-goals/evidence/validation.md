# Validation evidence — deterministic lazy-goal priority

Date: 2026-07-12

## Claim boundary

This evidence supports deterministic ordering over the bounded facts known at each scheduling epoch. It does not claim global makespan optimality, future-graph knowledge, execution success, output trust, release reproducibility, or production throughput.

## Adversarial review and hardening

The mainline integration was reviewed for queue mutation ordering, pressure-cache invalidation, multi-slot accounting, fairness epochs, dynamic graph insertion, evidence redaction, machine-contract drift, and bounded failure behavior.

Hardening applied during review:

- `build.build-json-report` now contracts `scheduler_policy` and bounded `scheduler_priority_decisions`, including a non-empty Rust-produced positive fixture and malformed-digest/inverted-threshold negatives.
- The generated machine-contract rail gained an `integer-less-than` invariant so `aged_after_epochs < protected_after_epochs` is checked by both Rust instance validation and generated Nickel.
- Worker construction on production paths now returns invalid job-budget or policy errors instead of relying on a panic-only default constructor.
- Graph/root/topological collections use prevalidated bounded iterator collection, and root-identity lookup propagates a typed error instead of panicking.
- Scheduling arithmetic and BLAKE3 hex lengths use named saturating constants; decision evidence remains redacted to BLAKE3 goal identities and bounded normalized classes.
- The priority comparator and epoch advance were rechecked with Kani after hardening.

## Focused Cargo evidence

All commands used `CARGO_TARGET_DIR=/tmp/mantle-main-priority-target` inside `nix develop`.

- `cargo test -q -p crunch-build scheduling`
  - PASS: `30 passed; 0 failed`.
- `cargo test -q -p crunch-build worker::tests::`
  - PASS: `49 passed; 0 failed`.
- `cargo test -q -p mantle --test scheduling_policy`
  - PASS: `3 passed; 0 failed`.
- `cargo test -q -p mantle --bin mantle build_report::tests::`
  - PASS: `16 passed; 0 failed`.
- `cargo test -q -p mantle --bin mantle machine_contract_producer_tests::`
  - PASS: `5 passed; 0 failed`.
- `cargo test -q -p mantle --test machine_schema_contracts`
  - PASS: `4 passed; 0 failed`.
- `cargo check -q -p crunch-build -p crunch-pipeline -p mantle --bins --tests --examples`
  - PASS.

The combined rerun is pueue task `249`, ending with `priority final focused validation: PASS`.

## Formal comparator evidence

Successful command:

```text
nix develop -c env \
  CARGO_HOME=/tmp/mantle-kani-cargo-home \
  RUSTUP_HOME=/home/brittonr/.cache/onix-modules/kani/rustup \
  KANI_HOME=/home/brittonr/.cache/onix-modules/kani/kani-home \
  CARGO_TARGET_DIR=/tmp/mantle-main-kani-target \
  RUSTC_WRAPPER=/usr/bin/env \
  /home/brittonr/.cache/onix-modules/kani/install/0.67.0/bin/cargo-kani \
  -p crunch-build --lib \
  --harness comparator_is_antisymmetric \
  --harness comparator_is_transitive \
  --harness epoch_advance_never_wraps
```

Pueue task `247` reported:

```text
SUMMARY:
 ** 0 of 545 failed (5 unreachable)
VERIFICATION:- SUCCESSFUL
Manual Harness Summary:
Complete - 3 successfully verified harnesses, 0 failures, 3 total.
```

The isolated Cargo home prevents the ambient Kache rustc wrapper from wrapping Kani's compiler.

## Machine-contract evidence

Command sequence:

```text
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --generate
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
```

Pueue task `246` reported:

```text
machine schema contract generation: PASS (15 contracted, 44 classified)
machine schema contract self-test: PASS
machine schema contract check: PASS (15 contracted, 44 classified)
```

## Lint and Tiger Style evidence

- Focused first-party Clippy passed in pueue task `245` after allowing only nine pre-existing lint classes raised by unrelated `dynamic_plan.rs`, `network_policy.rs`, and pre-existing native-dynamic worker helpers:

```text
cargo clippy -p crunch-build --lib --no-deps -- -D warnings \
  -A clippy::needless-lifetimes \
  -A clippy::result-large-err \
  -A clippy::large-enum-variant \
  -A clippy::type-complexity
```

- The broad repo-pinned Tiger Style package command still fails on 89 pre-existing findings. Its post-hardening transcript contains no finding in `crates/crunch-build/src/scheduling.rs` and no finding in the touched Worker scheduling region; remaining Worker findings are in pre-existing native-dynamic helpers.
- Direct rustfmt was run over the touched Rust files, and `git diff --check` passed.

These broader pre-existing findings are not promoted to scheduler failures and are not claimed as resolved.

## Comparative benchmark evidence

Command:

```text
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-main-priority-target \
  cargo run -q -p mantle --example benchmark_scheduler_priority \
  > cairn/changes/prioritize-lazy-build-goals/scheduler-benchmark.json
```

Pueue task `248` passed. The checked-in report records six fixtures. Priority ordering selected the expected long-path, deep-diamond, shared-dependency, verified-locality, continuously-ready, and stress-priority candidates instead of each FIFO-first candidate. The bounded 1,024-ready-goal stress fixture recorded `244175743` total nanoseconds for 200 debug-build priority rankings. The report explicitly excludes graph-snapshot/pressure-recomputation overhead and makes no production-throughput or optimality claim.

## Cairn evidence

Using `cairn-policy/generated/cairn-policy.json`, the final pre-sync rerun passed:

- `cairn validate`: `valid: true`, 13 active changes, 39 specs.
- proposal gate: PASS.
- design gate: PASS.
- tasks gate: PASS.

Receipt hashes are intentionally retained in the external command transcripts rather than embedded here because active-package evidence participates in gate identity.
