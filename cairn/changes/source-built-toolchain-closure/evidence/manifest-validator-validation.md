# Manifest validator validation transcript

Task-ID: I1
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Implemented slice

- Added pure manifest validation core in `src/source_toolchain_closure.rs`.
- Added fixed-point preflight field `source_built_toolchain_closure` with `claim=false` and `non_claim=not-source-built-toolchain-closure` while no closure manifest is supplied.
- Added `mod source_toolchain_closure;` to `src/main.rs`.

The validator is pure: it takes in-memory manifest structs, validates schema, role coverage, source/build receipt presence, seed exceptions, lowercase BLAKE3 digest shape, absolute execution paths, duplicate identities, and returns a deterministic policy digest. It performs no filesystem, environment, process, clock, network, or async work.

## Baseline note

A focused baseline task was queued before edits as pueue task `43`, but no retained log/status was available for that first attempt. The post-change validation below is the evidence for this completed implementation slice.

## Commands and outputs

Command:

```text
cargo fmt --check -p mantle -v && \
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture && \
cargo test -p mantle --bin mantle cargo_free -- --nocapture && \
cargo test -p mantle --test cargo_free_self_build_cli -- --nocapture && \
/home/brittonr/.cargo-target/debug/cairn validate --root . && \
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Environment included:

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
```

Formatter output ended after checking package targets without errors.

Source-built closure focused tests after review fix:

```text
running 10 tests
test source_toolchain_closure::tests::absent_closure_status_preserves_current_non_claim ... ok
test source_toolchain_closure::tests::validator_rejects_invalid_digest_shape ... ok
test source_toolchain_closure::tests::validator_rejects_invalid_optional_source_on_seed_member ... ok
test source_toolchain_closure::tests::validator_rejects_duplicate_member_identity ... ok
test source_toolchain_closure::tests::validator_rejects_invalid_optional_receipt_on_seed_member ... ok
test source_toolchain_closure::tests::validator_rejects_missing_required_role ... ok
test source_toolchain_closure::tests::validator_rejects_seed_member_without_seed_exception ... ok
test source_toolchain_closure::tests::validator_rejects_source_built_member_without_receipt ... ok
test source_toolchain_closure::tests::validator_accepts_explicit_seed_exception ... ok
test source_toolchain_closure::tests::valid_manifest_yields_stable_order_independent_policy_digest ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 590 filtered out; finished in 0.00s
```

Existing Cargo-free unit tests:

```text
running 14 tests
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 584 filtered out; finished in 0.00s
```

Existing Cargo-free CLI tests:

```text
running 7 tests
...
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

Cairn validate:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
```

Tasks gate:

```json
{
  "change": "source-built-toolchain-closure",
  "input_hash": "41cf08ea177e1b42c3a7ad2194751660ea44986ccd0206d2a4de0a4e341807a8",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "7bb655e9aaf5297e3b253bf9eb31e57714b6cba0f8bbb52253cd560acf5e26f9",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-task-update Cairn check

After marking I1 complete and adding the review-fix/push checkpoint evidence, this command was run:

```text
/home/brittonr/.cargo-target/debug/cairn validate --root . && \
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Output excerpt:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Decision

The typed manifest and pure validator implementation slice is complete after fixing seed-member optional identity validation. Broader execution plumbing, host-tool leakage enforcement, real source-root/seed provider integration, full positive stage-policy tests, and real end-to-end source-built closure proof remain unchecked tasks.
