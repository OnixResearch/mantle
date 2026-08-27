# V75 sysroot linker probes

## Result

V75 completed the full Rust-provider execution. Reconciliation still failed because four nonexistent sysroot linker probes were denied.

Reconciliation recorded 88,034 observed events, 88,030 matches, four denials, and 175 promotions. Its BLAKE3 is `0792eb9e4e6bf744e7e1e73fdb4ad1903803f2a81253b1afa30a8d95934b90fd`.

## Cause

Rustc probes these target-sysroot paths before it runs the explicit linker:

- `prefix-s/lib/rustlib/x86_64-unknown-linux-musl/bin/cc`;
- `prefix-s/lib/rustlib/x86_64-unknown-linux-musl/bin/self-contained/cc`.

Each missing path was tried twice. The later protected linker execution succeeded, but it did not cancel the denied probes.

## Repair

Before `run_rustc`, Mantle now publishes both exact sysroot paths as copies of the receipt-bound target linker wrapper. These are stage-produced executables with the same reviewed implementation and no ambient discovery.

The publication creates only the two expected paths, restores execute permission, verifies both files, and fails if the protected source wrapper is missing.

## Validation

The current validation transcript is `post-repair-validation.log`.

- Rust-provider tests: 75 passed, zero failed.
- `cargo check --bin mantle`: passed.
- Edition-2024 rustfmt check: passed.
- `git diff --check`: passed.
