# Focused Rust-plan validation

Task-ID: mantle.rust_package_planning.hexagonal_core
Covers: native Rust-plan core, application-owned ports, admitted adapter effects, compatibility, and bounded Cargo-free execution.

## Environment and first coherent binary

Validation used the checkout at `/home/brittonr/git/OnixResearch/mantle`, `TMPDIR=/home/brittonr/scratch`, the checked-in lockfile, and `nix develop --offline --no-write-lock-file`. The first post-cutover root `cargo check -p mantle --bin mantle --locked --offline` and `cargo build -p mantle --bin mantle --locked --offline` exited 0. The actual built executable was:

```text
/home/brittonr/scratch/rust-plan-integrated-target/current-tree-stable/debug/mantle
SHA256 9bff7a785ac1e74bff7966cf95e8f1501daed8ce39095e16c407b14949409eb0
size 1551831776 bytes; mtime 2026-10-01 00:45:02 -0400
```

The build preceded the live concurrent-socket proof, so this fingerprint identifies the binary used for the following runtime observations. A later shared-source edit is **not** covered by this fingerprint.

## Pure-core and application tests

Command inside the offline Nix shell:

```text
cargo test -p mantle-rust-plan-core -p mantle-rust-plan-app --locked --offline -- --nocapture
```

Exit 0. Exact test result lines from the completed run (`artifact://8343`):

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

The 10 application and 24 core fixtures include admitted port observations, rejected malformed and wrong observations, bounded planning, and package/feature/topology cases. The two application and eleven core rustdoc tests include compiled negative authority examples. `cargo clippy -p mantle-rust-plan-core -p mantle-rust-plan-app --all-targets --locked --offline -- -D warnings` exited 0; `cargo check -p mantle-rust-plan-core --target wasm32-unknown-unknown --locked --offline` exited 0. A wasm target check is a compilation boundary, not a claim that WebAssembly was run.

The eleven compiled `compile_fail` examples in `crates/mantle-rust-plan-core/src/lib.rs:13-90` reject filesystem access, process invocation, ambient environment lookup, Cargo oracle capture, rustc invocation, store mutation, cache restoration, host path operations, async-runtime driving, CLI dispatch, and receipt rendering from core values. Two compiled application-port examples in `crates/mantle-rust-plan-app/src/ports.rs` reject replacing `AdapterError` with a CLI `RunError` on workspace and cache ports. These examples execute through rustdoc's compiler; a source-text scanner is not counted as a core authority boundary.

The checked-in `crates/mantle-rust-plan-core/tests/plan_fixtures.rs` supplies durable compatibility goldens: `accepted_compatibility_claims_distinguish_oracle_and_bounded_surface` fixes the Cargo-oracle, bounded Cargo-free, and blocked classes and ordered surface IDs; `typed_receipt_preimage_hashes_exact_field_order_without_new_domain` compares the receipt hash against the literal `{"unit_id":"unit","receipt_hash":""}` preimage. These assertions ran in the 24/24 core result above; they complement, rather than replace, the detached byte-for-byte CLI golden comparisons below.

The actual bounded Cargo oracle adapter was exercised with `cargo test -p mantle --bin mantle rust_plan::tests::cargo_oracle_ --locked --offline -- --test-threads=1 --nocapture`: exit 0, 2 passed. Real `/bin/sh` child fixtures covered bounded stdout and stderr, child kill/reap after exceeding the pipe bounds, and a successful bounded JSON read. The production adapter caps Cargo stdout at 64 MiB and stderr at 1 MiB while reading the pipes, not after unbounded capture.

## Detached accepted/rejected JSON compatibility

The first binary was invoked directly with the Nix development shell toolchain `PATH`; each invocation exited 0 with **zero stderr bytes**. Full stdout was compared byte-for-byte against detached pre-cutover `.before.json` receipts, not merely parsed JSON fields:

| Fixture | First-binary stdout bytes | SHA-256 of exact stdout | Historical receipt byte equality |
| --- | ---: | --- | --- |
| Cargo-oracle accepted local-path workspace | 19827 | `3c9685761bcecdbb5cff28f15edc96e3309c979a518ca4bf7a3f5c74f1c173da` | yes |
| Cargo-forbidden accepted local-path workspace | 18812 | `20c3a5632753981ac6c0b425eb3f5717bf0237729344c8fcd42a434878389900` | yes |
| Cargo-forbidden unsupported bench surface | 8662 | `62052224715b7a8f59306eaf908bc1aa16afb767a8e37ad9862b8214c1f6b139` | yes |
| Cargo-forbidden unknown feature rejection | 19610 | `5b6a16e8cbf20b273cdbf05d5b26724f48461c5b698ad5f073503402c44447f3` | yes |
| Cargo-forbidden cold two-unit rustc execution | 23634 | `beeaac4c50e3ce54ebab0d70aff100d177bec5d4ce00092297cd9d2f123884f4` | yes |

The cold execution used the same explicit `/home/brittonr/scratch/rust-plan-fixtures/exec-parity` output root as the historical golden; its previous scratch outputs were moved aside and restored, and new compiler outputs were archived separately under `exec-parity.first-binary-cold-2026-10-01`. Both real units recorded `rebuilt-explicit-unit`, completed successfully, and emitted the exact historical receipt bytes. The prior cached-output state was preserved, not deleted. The deliberately failing Cargo shim marker remained absent.

## Cargo-forbidden bounded compiler proof

Executed against that fingerprinted binary:

```text
scripts/prove-cargo-free-rust-plan.sh --full --bundle-dir /home/brittonr/scratch/rust-plan-first-proof-2026-10-01 --mantle-bin /home/brittonr/scratch/rust-plan-integrated-target/current-tree-stable/debug/mantle
cargo-free proof OK: /home/brittonr/scratch/rust-plan-first-proof-2026-10-01
```

Exit 0. Generated `meta.json` records `"status": "success"`, `"smoke_stdout": "42"`, and `"cargo_forbidden_marker_absent": true`; `status.txt` contains `0`, `smoke-stdout.txt` contains `42`, and declared library and executable digests are in `output-digests.json`. This proves the bounded fixture's real rustc execution and resulting program output, not full Cargo compatibility, bootstrap correctness, or compiler correctness.

A separate first-binary real no-Cargo build-script fixture with no package `description` also completed both custom-build and lib units; the script supplied `mantle_build_script`, `BUILD_VALUE=env-ok`, and `generated.txt`, with no forbidden Cargo shim invocation. Its explicitly requested local cache **rejected** two action candidates because the Nix absolute C linker and the nested `src/lib.rs` manifest-root path were not both classified; both compilers executed. This is successful fail-closed compiler fallback, **not** a cache hit. The proposal explicitly prohibits changing linker or cache decisions; the post-first regression checks the same fail-closed behavior and a second real build-script metadata run, but has not yet been executed against a coherent second-stage source snapshot.

## Pre-final focused-root diagnostic

The first isolated-target `cargo test -p mantle --bin mantle rust_plan::tests:: --locked --offline -- --test-threads=1 --nocapture` completed with the exact result:

```text
test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 2434 filtered out; finished in 4.51s
```

The two failing legacy fixtures manually cleared a planned effect's declared outputs or called an isolated target while it had a selected host producer. New selected-effect preflight rejected them earlier with typed `incomplete-unit-effect` and `isolated-unit-has-selected-producer`, respectively. These fixtures had asserted obsolete later adapter errors (`missing-declared-output`, `missing-host-artifact`) and were removed rather than re-pinned to the new error words. No passing-after focused-root result is claimed here yet; the final V4 and V5 results must be recorded against a coherent post-integration source snapshot.

## Provisional second-source type check

The one user-authorized shared-root command ran against a separate warm target, **not** the pinned first executable:

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/rust-plan-private-root-target NIX_CONFIG='eval-cache = false' nix develop --offline --no-write-lock-file -c cargo check -p mantle --bin mantle --locked --offline
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 23s
```

Exit 0 (`artifact://9689`); the Mantle binary target emitted 13 dead-code warnings from other owner files and **no new type errors**. Retention's shared `crates/crunch-store/src/roots.rs` unique per-interest selection and test changed during this check, so this run is **provisional, not proof of a final coherent source snapshot**. It did not link or replace the SHA-256-pinned first executable. The changed-source focused gate and final V4/V5 tests must follow the post-backend source leases.

## First coherent pre-watch-CLI root type check

After the Watch Worker and pipeline authors released their source, the private-target worktree check ran with `TMPDIR=/home/brittonr/scratch`, `CARGO_TARGET_DIR=/home/brittonr/scratch/rust-plan-private-root-target`, and `NIX_CONFIG='eval-cache = false'`:

```text
nix develop --offline --no-write-lock-file -c cargo check -p mantle --bin mantle --locked --offline
warning: `mantle` (bin "mantle") generated 13 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 15s
```

Exit 0; no Rust source type errors. Three earlier attempts stopped in vendored `bzip2-sys`, `zstd-sys`, and `nickel-lang-parser` build scripts because already-generated files in this **private scratch target** were read-only and their `fs::copy` calls could not overwrite them. Restoring write permission to this private target's build artifacts resolved those failures without changing repository or vendor sources. This check preceded the guarded Watch CLI source changes and does not certify the final committed binary or complete V4/V5.
