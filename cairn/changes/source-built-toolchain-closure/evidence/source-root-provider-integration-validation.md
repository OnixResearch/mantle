# Source-root provider integration validation

Task-ID: I4
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Implemented slice

- `src/source_root_provider.rs` is now compiled into the Mantle binary instead of sitting as dormant source.
- `mantle bootstrap --source-root <manifest> -o <seed.ncl>` now enters the normalized source-root provider materializer, writes a source-root seed contract on success, and places the provider under the configured `--store` using a content-derived store-name prefix.
- Provider dependency traces now fail closed when emitted provider roles are incomplete, not only when they contain unexpected roles.
- Declared source-root patches fail closed before fetch/build work. The previous no-op patch placeholder path was removed so patched manifests cannot silently materialize without patch application.
- Generated source-root `seed.ncl` records manifest digest, provider output digest, provider metadata path, and explicitly states that no legacy musl.cc binary toolchain tarball was used.

This is provider integration evidence only. It does not claim a completed end-to-end source-built Rust toolchain closure proof, and it does not check the real fixed-point proof task.

## Baseline before edits

Focused source-root manifest validation passed before this slice:

```text
cargo test -p mantle --bin mantle bootstrap_source_root -- --nocapture

running 23 tests
...
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 593 filtered out; finished in 0.00s
```

## Post-change validation

Commands run with the documented Mantle test environment:

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox

cargo test -p mantle --bin mantle bootstrap_source_root -- --nocapture
cargo test -p mantle --bin mantle source_root_provider -- --nocapture
cargo test -p mantle --bin mantle bootstrap_source_root_provider_reaches_materializer -- --nocapture
cargo test -p mantle --lib generate_source_root_seed_ncl_structure -- --nocapture
cargo test -p mantle --bin mantle cargo_free -- --nocapture
cargo test -p mantle --test cargo_free_self_build_cli -- --nocapture
```

Output excerpts:

```text
running 26 tests
...
test bootstrap_source_root::tests::provider_trace_accepts_exact_expected_output_roles ... ok
test bootstrap_source_root::tests::provider_trace_requires_all_expected_output_roles ... ok
test tests::bootstrap_source_root_provider_reaches_materializer_and_fails_closed_on_patches ... ok

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 613 filtered out; finished in 0.00s

running 23 tests
...
test source_root_provider::tests::source_root_materialization_rejects_declared_patches_before_fetch ... ok
test tests::bootstrap_source_root_provider_reaches_materializer_and_fails_closed_on_patches ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 616 filtered out; finished in 0.00s

running 1 test
test tests::bootstrap_source_root_provider_reaches_materializer_and_fails_closed_on_patches ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 638 filtered out; finished in 0.00s

running 1 test
test bootstrap::tests::generate_source_root_seed_ncl_structure ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 70 filtered out; finished in 0.00s

running 25 tests
...
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 614 filtered out; finished in 0.00s

running 12 tests
...
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

Formatter check:

```text
cargo fmt --check -p mantle -v
```

passed after formatting `src/bootstrap.rs`, `src/bootstrap_source_root.rs`, `src/main.rs`, and the now-compiled `src/source_root_provider.rs`.

## Decision

The source-root provider is integrated into the bootstrap seed-provider path and fails closed where the implementation would otherwise rely on placeholder patch behavior. The real end-to-end source-built toolchain closure fixed-point proof remains unchecked and must still run against a real source-root manifest/provider output before any source-built closure claim.

## Next action

Add or select the real source-root manifest inputs, run `mantle bootstrap --source-root <manifest>` with a writable store, then use the resulting normalized provider to construct the toolchain closure manifest for the fixed-point proof.

## Post-task-update Cairn check

After marking I4 complete in `tasks.md`, this command was run:

```text
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
  "input_hash": "e2e7bbe19f89c0da44dae034874162721676ea07e34cde746aaeb72178b34c50",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e47ab12c8e4d52bacb9c15b717f88b58e8a309e1a96cee56de57adb3f1286681",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Review repair note

The post-task-update check above proved Cairn syntax/gating, not I4 completion. Same-family review found the I4 checkbox overclaimed because this evidence has no positive real source-root materialization transcript. I4 was unchecked again, and `evidence/source-root-provider-i4-oracle-checkpoint.md` records the completion decision.
