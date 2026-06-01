# Policy digest threading validation

Task-ID: I3
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Implemented slice

- `src/cargo_free_self_build.rs` now writes `source_built_toolchain_closure_policy_digest_blake3` into each fixed-point stage summary when a source-built toolchain closure manifest is enforced.
- The fixed-point stage receipt files (`stage1/receipt.json`, `stage2/receipt.json`) are annotated with the same policy digest before artifact materialization.
- Fixed-point success now requires stage1 and stage2 policy digests to match before reporting success; a mismatch reports `stage1/stage2 toolchain closure policy digests differ`.
- Existing no-closure fixed-point behavior keeps the per-stage policy digest fields as `null`.

This still does not claim a source-built closure proof; `claim=false` and `not-source-built-toolchain-closure` remain until the real source-root/seed provider and end-to-end proof land.

## Baseline before edits

Pueue task `65` ran the focused baseline command:

```text
cargo test -p mantle --bin mantle cargo_free -- --nocapture
cargo test -p mantle --test cargo_free_self_build_cli -- --nocapture
```

Output excerpt:

```text
running 23 tests
...
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 591 filtered out; finished in 0.00s

running 12 tests
...
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

## Post-change validation

Pueue task `70` ran the same focused command with the documented Mantle test environment:

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
```

Output excerpts:

```text
running 25 tests
...
test cargo_free_self_build::tests::fixed_point_status_accepts_matching_policy_digest ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_mismatch_before_success ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 591 filtered out; finished in 0.01s

running 12 tests
...
test cargo_free_fixed_point_enforces_matching_toolchain_closure_manifest_without_claiming_proof ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
```

The CLI test now asserts:

- no-manifest stage policy digest fields are `null`,
- manifest-backed stage1/stage2 summaries carry the manifest policy digest,
- manifest-backed `stage1/receipt.json` and `stage2/receipt.json` carry the same digest.

## Post-task-update Cairn check

After marking I3 complete in `tasks.md`, this command was run:

```text
cargo fmt --check -p mantle -v
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Output:

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
  "input_hash": "c474f06f6e4b38d29af30b0c1b3cb8590236c4fbeb9761d42c980e650fa2bbd1",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "246b805bd2a344e35ad4228abdd40ea3b67b64fc0610be34d605cffa7edccc2e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Decision

I3 can be checked for the bounded implementation slice: fixed-point stage receipts and summaries carry the enforced closure policy digest, and stage1/stage2 policy mismatch blocks fixed-point success.

## Next action

Materialize or integrate the real normalized source-root/seed provider, or preserve/no-regress existing bounded behavior as the smaller next slice if provider work remains too large.
