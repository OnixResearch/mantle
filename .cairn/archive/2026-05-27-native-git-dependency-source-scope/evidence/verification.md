# Verification Evidence: Native Git Dependency Source Scope

Task-ID: native-git-dependency-source-scope.V1
Covers: r[rust_package_planning.native_git_dependency_source_scope]
Date: 2026-05-27
Decision owner: coding agent

## Baseline blocker chain

The prior archived feature-edge evidence recorded `target/mantle-self-rust-plan-probe-after-review-fix/receipt.json` with native package-target planning blocked by `snix-castore -> wu-manber`:

```text
native_package_target_planning=false
native_unit_graph_planning=false
native_host_unit_graph_planning=false

package target blockers:
- native-missing-cargo-package: Cargo oracle workspace package is absent from native planning fragment
- unsupported-non-path-dependency: dependency `wu-manber` is outside the bounded path-or-declared-registry-dependency fragment
```

## Implementation checkpoint

Decision: add `native_git_source_planning` as a separate receipt fragment rather than overloading `native_registry_source_planning`.

Inspected code seams:

- `source_closure` already records git package identity, URL, resolved revision, and manifest path.
- Native package dependency resolution only accepted path dependencies or ready registry source facts.
- The new bounded git scope binds selected git dependencies to captured source-closure material only; it does not invoke git, fetch the network, solve versions, or scan arbitrary Cargo caches.
- Git URL/revision are identity evidence only. Source bytes stay provider-agnostic and BLAKE3-addressed so the default materialization path can be snix-store-backed rather than hard-coupled to a git pull.
- Dependency resolution now requires package name, optional manifest version, git URL, and requested rev/tag/branch identity to match a ready captured fact; mismatches and ambiguous matches produce deterministic blockers.

## Commands and observed outputs

### Focused git source tests

Pueue task: 24

Command:

```sh
PATH=/run/current-system/sw/bin:/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/etc/profiles/per-user/brittonr/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
RUSTC_WRAPPER= CARGO_TARGET_DIR=target/native-git-source-check \
  cargo test -p mantle --bin mantle native_git_source -- --nocapture
```

Observed output excerpt:

```text
running 2 tests
test rust_plan::tests::native_git_source_planning_blocks_missing_captured_manifest ... ok
test rust_plan::tests::native_git_source_planning_binds_captured_source_and_resolves_dependency_edge ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 452 filtered out; finished in 0.00s
```

### Review-fix git identity and dev-dependency negative tests

Pueue tasks: 17 and 25

Command:

```sh
PATH=/run/current-system/sw/bin:/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/etc/profiles/per-user/brittonr/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
RUSTC_WRAPPER= CARGO_TARGET_DIR=target/native-git-source-check \
  cargo test -p mantle --bin mantle native_git -- --nocapture
```

Observed output excerpt:

```text
running 4 tests
test rust_plan::tests::native_git_source_planning_blocks_missing_captured_manifest ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_source_planning_binds_captured_source_and_resolves_dependency_edge ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_ambiguous_same_url_sources ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 452 filtered out; finished in 0.00s

review-fix dev-dependency rerun:

running 5 tests
test rust_plan::tests::native_git_source_planning_blocks_missing_captured_manifest ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_dev_dependency_resolution_blocks_mismatched_revision ... ok
test rust_plan::tests::native_git_source_planning_binds_captured_source_and_resolves_dependency_edge ... ok
test rust_plan::tests::native_git_dependency_resolution_blocks_ambiguous_same_url_sources ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 452 filtered out; finished in 0.00s
```

### Focused existing source-regression tests

Pueue tasks: 28, 29, and review-fix registry rerun 20

Observed output excerpts:

```text
running 2 tests
test rust_plan::tests::native_registry_source_planning_blocks_missing_vendor_material ... ok
test rust_plan::tests::native_registry_source_planning_binds_declared_vendor_source ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 452 filtered out; finished in 0.00s

review-fix rerun:

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 454 filtered out; finished in 0.00s
```

```text
running 3 tests
test rust_plan::tests::source_closure_blocks_registry_without_checksum ... ok
test rust_plan::tests::source_closure_records_registry_git_and_path_identities ... ok
test rust_plan::tests::rust_unit_execution_blocks_source_closure_blocker_before_rustc ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 451 filtered out; finished in 0.00s
```

### Formatting

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/usr/bin:/bin \
  cargo fmt -p mantle --check -- src/rust_plan.rs tests/rust_plan_cli.rs
```

Observed output: command exited successfully with no stdout/stderr.

Review-fix rerun:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/usr/bin:/bin \
  cargo fmt -p mantle --check -- src/rust_plan.rs
```

Observed output: command exited successfully with no stdout/stderr.

### Cairn validation and gates before sync/archive

Pueue tasks: 30, 31, 32, 33, and final completed-task rerun 34

Observed output excerpts:

```text
validate: valid=true, changes=1, specs_validated=2
gate proposal native-git-dependency-source-scope: verdict=PASS
gate design native-git-dependency-source-scope: verdict=PASS
gate tasks native-git-dependency-source-scope: verdict=PASS
gate tasks native-git-dependency-source-scope after marking all tasks complete: verdict=PASS
```

### Review-fix rebuild

Pueue tasks: 18 and 26

Observed output excerpt:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.73s
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.15s
```

### Task checklist snapshot

The archived task file `cairn/archive/2026-05-27-native-git-dependency-source-scope/tasks.md` contains:

```text
- [x] Record current self-probe blocker evidence for the snix-castore -> wu-manber chain.
- [x] Add focused positive and negative fixtures for a locked git dependency resolved through captured source-closure material.
- [x] Implement bounded native git source facts with lockfile URL/revision, source root, manifest path, lockfile digest, and BLAKE3 source-tree digest.
- [x] Wire selected non-path git dependency edges to ready native git source facts while preserving deterministic blockers for missing, ambiguous, stale, or network-only material.
- [x] Expose receipt JSON for ordered git source facts, lockfile/source digests, source-closure comparison evidence, blockers, and stable receipt hash.
- [x] Verify focused Rust plan tests, Mantle self probe blocker movement, Cairn validation, Cairn gates, sync, archive, and commit.
```

### Mantle self `rust-plan --execute-topology` probe

Pueue task: 32, rerun at current HEAD `498f3f5b8bfe9c19f7c90bac5f9dad8d9370fd09` after the review warning about stale final evidence. The probed code commit is recorded in `target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/head.txt`. The later commit amend updates only this evidence file to record the probe; source code, specs, and task state are unchanged after the probe.

Current receipt: `target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/receipt.json`

Observed output excerpt:

```text
probe_status=0
4800852 target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/receipt.json
```

Oracle checkpoint for `blocker-summary.txt`:

Question: Does `target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/blocker-summary.txt` exist, and is it safe for this evidence file to cite it as the receipt-summary source?

```text
blocker_summary_exists=true
-rw-r--r-- 1 brittonr brittonr    1061 May 27 09:34 target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/blocker-summary.txt
-rw-r--r-- 1 brittonr brittonr      41 May 27 09:34 target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/head.txt
-rw-r--r-- 1 brittonr brittonr 4800852 May 27 09:34 target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/receipt.json
-rw-r--r-- 1 brittonr brittonr      99 May 27 09:34 target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/sizes.txt
-rw-r--r-- 1 brittonr brittonr      15 May 27 09:34 target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/status.txt
-rw-r--r-- 1 brittonr brittonr       0 May 27 09:34 target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/stderr.txt
```

Decision owner: coding agent. Decision: `blocker-summary.txt` is materialized and safe to cite. Next action: no further oracle action for this artifact unless the probe directory is regenerated.

Evidence-check commands after this checkpoint edit:

```text
git diff --check
# exited successfully with no output
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
# valid=true, changes=0, specs_validated=1
```

Receipt summary from `target/mantle-self-rust-plan-probe-after-native-git-dev-dep-fix-head-498f3f5b/blocker-summary.txt`:

```text
topology_execution=blocked
source_closure=true
native_registry_source_planning=true
native_git_source_planning=true
native_package_target_planning=true
native_unit_graph_planning=true
native_host_unit_graph_planning=true
unit_derivation_graph=true

blocker classes:
      1 missing-dependency-producer

topology blocker:
- missing-dependency-producer: no supported target producer lib unit for dependency package registry+https://github.com/rust-lang/crates.io-index#itertools@0.10.5

git source non_claims:
captured-source-closure-only
source-material-provider-agnostic
content-addressed-source-by-blake3
no-network-fetch
no-version-solving
no-ambient-cargo-git-scan
not-general-cargo-git-compatibility

git source fact:
- git+https://github.com/tvlfyi/wu-manber.git#wu-manber@0.1.0 rev=0d5b22bea136659f7de60b102a7030e0daaa503d digest=23d80e9ec4ae4e929bc04a7426fdd1f93ef7b3e0e9fa3ecbec0ef477e1c8fcd0
```

## Decision

The git source change is accepted for this scope because the `wu-manber` git source now has an explicit receipt-bound native source fact, native package-target/unit/host graph planning all become ready in the self probe, and the remaining blocker moved to a later topology producer-coverage gap for registry package `itertools@0.10.5`.
