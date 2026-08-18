# Verification: explainable store retention

## Result

The focused implementation checks pass. Implementation commit `071935a8` adds typed retention policy, versioned root provenance, project generations, shell leases, usage reports, explained GC plans, and plan-bound execution.

## Baseline

Before the retention changes, the existing focused store, build, and pipeline suites passed. The store used permanent source labels and one-step GC without owner, generation, lease, policy, or accepted-plan facts.

## Focused Rust rail

Pueue task `7644` ran this final chain with exit status 0:

```console
cargo fmt --all
cargo check --locked -p crunch-gc-core -p crunch-store -p crunch-build -p crunch-pipeline -p mantle --all-targets
cargo test --locked -p crunch-gc-core
cargo test --locked -p crunch-store roots::
cargo test --locked -p crunch-store gc::
cargo test --locked -p crunch-project retention
cargo test --locked -p crunch-pipeline --lib
cargo test --locked -p mantle --bin mantle shell_cmd::tests::
cargo test --locked -p mantle --bin mantle store_cmd::tests::
cargo test --locked -p mantle --test store_gc_cli
cargo clippy --locked -p crunch-gc-core --all-targets --no-deps -- -D warnings
cargo clippy --locked -p crunch-store --lib --no-deps -- -D warnings
cargo clippy --locked -p crunch-build --lib --no-deps -- -D warnings
cargo clippy --locked -p crunch-pipeline --lib --no-deps -- -D warnings
cargo check --locked -p crunch-gc-core --target wasm32-unknown-unknown
git diff --check
```

The final `crunch-store gc::` run passed 20 tests. The final CLI run passed 4 tests. Positive and negative cases cover owner scope, generations, lease rollback and expiry, legacy migration, stale plans, root changes, symlink substitution, shared closure, interrupted metadata rewrite, unknown bytes, bounded explanations, and continued independent cleanup.

Dependency-inclusive Clippy still stops in existing vendored or unrelated code. The focused first-party crates pass with warnings denied and no lint suppression.

## Exact plan and mutation evidence

The fixed execution-plan fixture binds candidate `/mantle/store/candidate` and observation `/mantle/store/observed-candidate` to this identity:

`b3:d018941192561471a85b01e1dc8365b1acde9a5664ec3f85c5c77146c8ede9c1`

Changing observed bytes or path kind changes the identity. The shared-closure fixture retains these exact logical paths:

- `/mantle/store/root-a`;
- `/mantle/store/root-b`;
- `/mantle/store/shared`.

The interrupted PathInfo rewrite fixture returns `execution_complete: false`, records `PathInfoRewrite`, performs no later operation, and leaves the candidate export present. Root-change and symlink-substitution fixtures reject the accepted plan as `stale-gc-plan` before deletion.

## Nickel and Nix policy checks

Pueue task `7641` passed:

```console
nix run .#check-store-retention-policy --no-write-lock-file
nix build .#checks.x86_64-linux.store-retention-policy -L --no-write-lock-file
git diff --check
```

The check typechecks and exports the positive policy. It rejects unknown class, missing owner scope, invalid generation, invalid lease, duplicate rule, and invalid limit fixtures for their expected diagnostics.

Full flake evaluation remains blocked by the existing unavailable repository:

`https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git`

## Machine-contract checks

The machine-contract self-test passed:

```console
cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
```

The repository-wide check still rejects 40 existing root JSON producers without inventory decisions. No finding names `src/store_cmd.rs` or a retention source.

## Octet

Pueue task `7554` completed with `Status: warning-only`, 883 warnings, and 0 errors. The findings are existing `crunch-build`, vendored Snix, and vendored compatibility findings. No error was suppressed.

## Cairn lifecycle

Repository validation returned `valid: true`.

Gate receipts:

- proposal: `0794344dbe37bc9b24c844607c47a48ebea6787d808bb3a2adf2006b3727be18`;
- design: `16ffa63a47ee0fccbfcedec607d8af447da0e1d9bbbde02e1fd27dbea8c1655e`;
- completed tasks: `abc7f3aaf010c99d6b6cf43b25aec904e7d6d50215c44635ad38bdb796d136ef`.

Executed sync receipt: `af9dd4bb6a4720988ebb1e141bc34df06e4ff6da4bb1e2659c67a9dead71221a`.

The sync added all six `store_lifecycle.*` requirements to `cairn/specs/store-lifecycle/spec.md`.

The default Tracey profile still reports 145 of 148 accepted release-provenance requirements referenced. Its three missing IDs are unrelated to store retention:

- `mantle.release_provenance.content_bound_evidence_manifest`;
- `mantle.release_provenance.content_bound_requirement_coverage`;
- `mantle.release_provenance.legacy_coverage_boundary`.

Tracey receipt: `068670c6966065d4ba6a0ba966a545bf655d02cacec4f90a630e809d3ea24c56`.

## Claim boundary

This evidence applies to the recorded source, policy, root facts, closure facts, supplied clock, observations, and tests. It does not prove content correctness, rebuildability, exclusive byte ownership, successful deletion outside the observed run, or release eligibility.
