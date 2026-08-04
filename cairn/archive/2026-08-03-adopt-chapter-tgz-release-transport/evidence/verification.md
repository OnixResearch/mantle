# Verification

Date: 2026-08-03

Toolchain: `rustc 1.99.0-nightly (0e29c21d9 2026-07-21)`

## Passed checks

### Formatting

```text
cargo fmt -p mantle -p crunch-release-core -- --check
```

Result: PASS.

### Pure core

```text
cargo test -p crunch-release-core --lib --tests
```

Result: PASS. 223 passed, 0 failed.

```text
cargo clippy -p crunch-release-core --all-targets -- -D warnings
```

Result: PASS.

```text
cargo check -p crunch-release-core --target wasm32-unknown-unknown
```

Result: PASS.

### Release transport shell

```text
cargo test -p mantle --bin mantle release_chapter_transport::tests -- --nocapture
```

Result: PASS. 12 passed, 0 failed, 1 representative benchmark ignored.

The suite covers deterministic output, standard readers, exact tree round trip, internal links, receipt and payload tamper, truncation with original and rebound receipts, marker limits, privileged modes, stale metadata, special tar types, unsafe indexes, legacy tgz, and destination races.

```text
cargo test -p mantle --bin mantle release_tree_copy::tests -- --nocapture
```

Result: PASS. 8 passed, 0 failed.

```text
cargo test -p mantle --bin mantle release_evidence::tests -- --nocapture
```

Result: PASS. 44 passed, 0 failed.

```text
cargo test -p mantle --bin mantle release_transport_commands_parse -- --nocapture
```

Result: PASS. 1 passed, 0 failed.

### Focused first-party Clippy

```text
cargo clippy -p mantle --bin mantle --tests --no-deps -- \
  -D warnings \
  -A dead_code \
  -A clippy::useless_format \
  -A clippy::chunks_exact_to_as_chunks \
  -A clippy::cmp_owned \
  -A clippy::field_reassign_with_default \
  -A clippy::assertions_on_constants
```

Result: PASS.

The allow list contains existing root-test findings from the current nightly toolchain. The production binary also passes with only the two existing `useless_format` and `chunks_exact_to_as_chunks` allowances.

### Locked metadata and diff hygiene

```text
cargo metadata --offline --locked --format-version 1 --no-deps
git diff --check
```

Result: PASS.

### Dependency policy

```text
cargo-deny check licenses bans sources
```

Result: `bans ok`, `licenses ok`, and `sources FAILED` on three existing `git.onix.computer` sources.

The new `chapter-tgz` dependency is a crates.io source and did not add a source-policy finding.

## Representative benchmark

```text
mantle-d1c7829ae6533b75 representative_release_transport_benchmark \
  --ignored --nocapture --test-threads=1
```

Result: PASS. See `benchmark.md` for the complete three-sample median report.

Parallel chapter reads improved by 8.3 percent. This is below the 20 percent production activation threshold.

## Lifecycle checks before sync

```text
cairn validate --root .
cairn gate proposal adopt-chapter-tgz-release-transport --root .
cairn gate design adopt-chapter-tgz-release-transport --root .
cairn gate tasks adopt-chapter-tgz-release-transport --root .
```

Result: PASS.

The accepted-spec Tracey profile reached 148 of 148 existing requirements after repairing its omitted content-bound bridge. It then reported the seven active chapter requirements as dangling. This was the expected pre-sync state.

Cairn sync completed with receipt `59dc5d99d10e6b236dba68b49e19265dbd123f04bd98253b918dfc3f5d5d7bd3`.

After sync, Tracey passed with 155 of 155 requirements referenced.

Cairn archive completed with receipt `f945dd4dfeb5b8526bccf4778698dadffc08d9fc50a241ba2080e9816f9649ca`.

Post-archive `cairn validate --root .` passed. Post-archive Tracey also passed with 155 of 155 requirements referenced.

## Bounded blockers

### Nix

```text
nix develop path:$PWD -c true
```

Result: BLOCKED before shell entry.

```text
error: Failed to fetch git repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'
```

This existing unavailable input also blocks the broad flake check.

### Tiger Style driver

The direct cached Tiger Style runner was invoked. The current nightly did not match the cached lint source API. The older available nightly lacked `rustc-dev` for Dylint.

The normal Nix-owned Tiger Style rail remains blocked by the same unavailable Nix input. Focused strict Clippy, no-std WASM, and the positive and negative test matrix pass.

### Shared-target doctest

A plain `cargo test -p crunch-release-core` reached a doctest failure after mixed nightly artifacts entered the shared Cargo target during Tiger Style probing. The repository notes identify this shared-target failure mode.

The required library and test targets pass with `--lib --tests`. No chapter transport doctests were added.

## Known limits

- The receipt needs an authenticated parent handoff for adversarial authority.
- Inspect validates the complete gzip stream and all metadata. It is not a range-only operation.
- Production extraction is sequential.
- The raw marker-prefix precheck is bound to the exact `chapter-tgz` 0.1.0 encoding.
- Archive and uncompressed byte limits are large policy ceilings, not memory promises.
- Atomic no-replace publication is a local Linux visibility guarantee, not power-loss durability.
