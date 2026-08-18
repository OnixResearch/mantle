# Rust source final candidate blocker progress surfacing

Task-ID: rust-source-rustc-final-blocker-progress-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now includes the scratch-only `rust-1.94.0-final` provider-candidate evidence in the fail-closed materializer error:

- `final_candidate_stage_id`
- `final_candidate_dir`
- `final_candidate_manifest`
- `final_candidate_metadata_digest_blake3`
- `final_candidate_smoke_summary`

This makes the bounded evidence discoverable by CLI callers while preserving the blocker. The materializer still returns an error, writes no final `--output-dir`, launches no provider-backed self-build, proves no fixed point, and keeps `not-source-built-toolchain-closure`.

## Validation after changes

### rust source provider tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture
```

Output summary:

```text
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok
test rust_source_provider::tests::materializer_rejects_rustc_final_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::rustc_final_provider_candidate_rejects_tampered_artifact ... ok

test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.54s
```

Note: an earlier full-suite rerun failed once in `chained_rustc_stage1_provider_candidate_rejects_tampered_artifact`; an immediate single-test rerun passed, followed by the successful full rerun above.

### route/core regression tests and checks

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture && \
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$PATH rustfmt --check src/rust_source_provider.rs && \
git diff --check
```

Output summary:

```text
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 728 filtered out; finished in 0.04s
rustfmt --check: completed successfully
git diff --check: completed successfully
```

## Lifecycle validation

### cairn validate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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
  "specs_validated": 6,
  "valid": true
}
```

### cairn tasks gate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
```

Output summary:

```json
{
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Remaining blocker

The exposed final candidate paths are only scratch evidence. Mantle still has no durable final Rust provider output, no provider-backed self-build, and no fixed-point proof.
