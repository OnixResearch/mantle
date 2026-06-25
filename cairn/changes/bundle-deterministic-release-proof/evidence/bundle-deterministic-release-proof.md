# Bundle deterministic release proof evidence

Date: 2026-06-25
Change: `bundle-deterministic-release-proof`
Requirement: `r[rust_package_planning.bundle_deterministic_release_proof]`

## Implementation summary

Mantle release evidence manifests now carry optional bounded bundle artifacts for:

- `deterministic_build_proof` at `deterministic-release/deterministic-build-proof.json` with role `deterministic-build-proof`
- `deterministic_sandbox_isolation_evidence` at `deterministic-release/deterministic-sandbox-isolation-evidence.json` with role `deterministic-sandbox-isolation-evidence`

`mantle release reproduce --deterministic-proof-runs ...` copies the generated proof receipt and sandbox evidence into the bundle, writes their BLAKE3 digests/sizes/roles to `manifest.json`, and keeps the manifest consistent with the copied files.

`mantle release verify --require-deterministic-release` now resolves deterministic proof evidence from explicit sidecars when supplied, otherwise from the bundle-local manifest records. JSON/human output reports the source as `external` or `bundled`.

## Validation transcript

Environment for Rust commands:

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox
```

### Core manifest tests

Command:

```sh
cargo test -q -p crunch-release-core manifest::tests -- --nocapture
```

Evidence: pueue task 76.

```text
running 15 tests
...............
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.00s
```

### Deterministic release verification subset

Command:

```sh
cargo test -q -p mantle --test release_cli deterministic_release -- --nocapture
```

Evidence: pueue task 77 initially, rerun after final diagnostic cleanup as pueue task 126.

```text
running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.29s
```

### Bundled deterministic proof blockers

Command:

```sh
cargo test -q -p mantle --test release_cli bundled_deterministic -- --nocapture
```

Evidence: pueue task 84 initially, rerun after final diagnostic cleanup as pueue task 127.

```text
running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.04s
```

### External deterministic proof override

Command:

```sh
cargo test -q -p mantle --test release_cli external_deterministic_override -- --nocapture
```

Evidence: pueue task 87.

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.05s
```

### Release reproduce writes bundled proof artifacts

Command:

```sh
cargo test -q -p mantle --test release_cli release_reproduce_writes_deterministic_proof_from_repeated_clean_runs -- --nocapture
```

Evidence: pueue task 89.

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.07s
```

### Release reproduce proof verifies from bundle-local artifacts

Command:

```sh
cargo test -q -p mantle --test release_cli release_reproduce_generated_two_clean_store_proof_verifies_deterministic_release -- --nocapture
```

Evidence: pueue task 88.

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.08s
```

### Formatting

Command:

```sh
rustfmt --check crates/crunch-release-core/src/lib.rs crates/crunch-release-core/src/manifest.rs src/release_attestation.rs src/release_cmd.rs src/release_evidence.rs src/release_reproducibility.rs tests/release_cli.rs
```

Evidence: pueue task 91 completed successfully.

### Mantle binary build

Command:

```sh
cargo build -p mantle --bin mantle
```

Evidence: pueue task 100.

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 39.11s
```

### Whitespace diff check

Command:

```sh
git diff --check
```

Evidence: pueue task 101 completed successfully before evidence finalization; pueue task 131 completed successfully after evidence updates.

### Cairn validation and gates

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal bundle-deterministic-release-proof --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design bundle-deterministic-release-proof --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks bundle-deterministic-release-proof --root .
```

Evidence: pueue task 102 completed successfully before tasks were checked; pueue task 111 completed successfully after tasks were checked. The final post-task gate receipt ended with:

```json
{
  "input_hash": "c324c3b37ecd94ad58c300cd55b5928b4fbc14ba7c2b2beeac69a00d7cb77093",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ce4ad6e1a05576d7071c26d0d6df8ca5953619f4609f8081232f0e1c8b8ec74a",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

### Cairn sync and post-sync gates

Commands:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- sync bundle-deterministic-release-proof --root .
nix run path:/home/brittonr/git/cairn#cairn -- sync bundle-deterministic-release-proof --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal bundle-deterministic-release-proof --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design bundle-deterministic-release-proof --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks bundle-deterministic-release-proof --root .
```

Evidence: sync dry-run pueue task 114, sync execute pueue task 115, post-sync validation/gates pueue task 117. The synced accepted spec contains `r[rust_package_planning.bundle_deterministic_release_proof]` in `cairn/specs/rust-package-planning/spec.md`.

Post-sync gate receipt ended with:

```json
{
  "input_hash": "c324c3b37ecd94ad58c300cd55b5928b4fbc14ba7c2b2beeac69a00d7cb77093",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ce4ad6e1a05576d7071c26d0d6df8ca5953619f4609f8081232f0e1c8b8ec74a",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
