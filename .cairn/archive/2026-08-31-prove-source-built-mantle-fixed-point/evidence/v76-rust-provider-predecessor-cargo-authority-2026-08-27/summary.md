# V76 predecessor Cargo authority

## Result

V76 completed the first Rust stage with clean reconciliation and advanced to Rust 1.91.1. That stage failed when Python launched the predecessor provider's Cargo binary.

The Rust 1.91.1 reconciliation recorded 63 observed events, 62 matches, one denial, and two promotions. Its BLAKE3 is `1d739f850dfa2f1b001f591831df2facb6d3d07c07de8ef43e1ed0b6e75e0023`.

## Cause

The predecessor candidate directory was a declared output root for the first stage. Its Cargo binary was never executed during that stage, so first-use promotion did not occur. The next stage then treated its first execution as undeclared.

The file mode was `0755`; the observed `EACCES` came from fail-closed seccomp authority, not filesystem permissions.

## Repair

Every provider candidate smoke now runs `cargo --version` before its producing stage closes. This promotes the exact candidate Cargo bytes under the producer's output-root authority. Later stages can then execute that predecessor without ambient or path-only authority.

The Cargo smoke is bounded, uses an empty environment, requires successful nonempty output, and fails closed. It applies to first-stage, chained stage, and final provider candidates.

## Validation

The current validation transcript is `post-repair-validation.log`.

- Rust-provider tests: 77 passed, zero failed.
- `cargo check --bin mantle`: passed.
- Edition-2024 rustfmt check: passed.
- `git diff --check`: passed.
