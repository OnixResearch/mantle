# Rust source final provider output promotion

Task-ID: rust-source-final-output-promotion-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

The Rust source-provider materializer now promotes a fully validated `rust-1.94.0-final` provider candidate into the requested final `--output-dir` instead of always returning the historical fail-closed blocker after candidate smoke.

The promotion step is still fail-closed:

- it preflights an existing final output path before scratch work starts,
- revalidates the final candidate metadata and receipt-bound artifact digests before copying,
- copies the candidate provider tree into the requested output directory,
- revalidates the copied final output directory,
- removes the copied output if copied-output validation or final scratch-manifest update fails, and
- records `final_output_written = true` plus `final_output_path` in the final candidate manifest only after promotion succeeds.

This is a materializer boundary improvement, not a completed source-built Rust provider claim. The checked-in route still needs real Rust source build scripts/derivations and a provider-backed self-build/fixed-point proof before the active materialization, smoke, and fixed-point tasks can be marked complete.

## Validation

### Baseline before change

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

test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 2.34s
```

### Focused provider tests after change

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture
```

Output summary:

```text
test rust_source_provider::tests::materializer_rejects_existing_output_before_scratch_work ... ok
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 1.30s
```

### CLI and source-toolchain focused checks

A combined Cargo invocation with two substring filters failed with `unexpected argument 'source_toolchain_closure'`; Cargo accepts only one filter. The filters were rerun separately.

Command:

```sh
cargo test -p mantle --bin mantle bootstrap_rust_source_provider -- --nocapture
```

Output summary:

```text
test tests::bootstrap_rust_source_provider_fails_closed_without_output ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 766 filtered out; finished in 0.35s
```

Command:

```sh
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
```

Output summary:

```text
test source_toolchain_closure::tests::checked_in_rust_source_plan_loads_stagex_mrustc_route ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 729 filtered out; finished in 0.09s
```

### Formatting, diff, and lifecycle checks

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$PATH rustfmt --check src/rust_source_provider.rs && \
git diff --check && \
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . && \
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
```

Output summary:

```text
rustfmt --check: completed successfully
git diff --check: completed successfully
cairn validate: valid=true, changes=1, specs_validated=6
cairn tasks gate: verdict=PASS, valid=true
```

Separate `cairn validate` output:

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
