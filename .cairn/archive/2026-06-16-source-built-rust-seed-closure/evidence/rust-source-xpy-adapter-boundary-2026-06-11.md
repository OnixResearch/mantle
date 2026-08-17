# Rust source x.py adapter boundary

Task-ID: rust-source-xpy-adapter-boundary-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now handles official Rust source archives that do not carry Mantle-specific `mantle-rustc-stage1-build.sh` / `mantle-rustc-final-build.sh` files.

For each `rustc-stage1` and `rustc-final` boundary, the generated wrapper now:

- uses the source-local Mantle stage script when present,
- otherwise requires `x.py` in the verified Rust source tree,
- generates a scratch-local x.py adapter script under the stage build directory,
- writes a bounded `config.toml` pointing at the receipt-bound bootstrap provider `rustc`/`cargo`, host triple, target triple, build dir, and output prefix,
- invokes x.py with stage-specific goals (`rustc`, `cargo`, `library/std`, and `rustdoc` for final), and
- fails closed before final output if the Rust source lacks both a source-local stage script and x.py.

This moves the real-source route past a synthetic in-source-script assumption. It still does not prove a real provider: the next honest proof must run the generated x.py adapter against real Rust sources, complete the staged builds, smoke the final provider, then run the provider-backed self-build/fixed-point proof.

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
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.59s
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
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok
test rust_source_provider::tests::materializer_rejects_rust_source_without_stage_script_or_xpy_without_output ... ok

test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.68s
```

### Source-toolchain and formatting checks

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
test source_toolchain_closure::tests::checked_in_rust_source_plan_loads_stagex_mrustc_route ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 731 filtered out; finished in 0.05s
rustfmt --check: completed successfully
git diff --check: completed successfully
```

### Lifecycle validation after evidence update

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

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
```

Output summary:

```text
verdict=PASS
valid=true
issues=[]
```
