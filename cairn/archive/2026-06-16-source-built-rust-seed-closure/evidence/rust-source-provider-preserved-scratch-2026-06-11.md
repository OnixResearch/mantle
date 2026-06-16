# Rust source provider preserved scratch evidence on CLI blocker

Task-ID: rust-source-provider-preserved-scratch-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

`mantle bootstrap rust-source-provider` now preserves the temporary scratch directory when source-provider materialization fails closed and includes `preserved_scratch=<path>` in the error. This keeps scratch-local evidence paths inspectable for operators instead of deleting the temp directory before the error reaches the caller.

This still does not write the requested final `--output-dir`, does not launch provider-backed self-build, proves no fixed point, and keeps `not-source-built-toolchain-closure`.

## Validation after changes

### CLI/parser tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
cargo test -p mantle --bin mantle bootstrap_rust_source_provider -- --nocapture
```

Output summary:

```text
test tests::bootstrap_rust_source_provider_fails_closed_without_output ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 765 filtered out; finished in 0.04s
```

The focused negative test asserts:

- the command still reports `Rust source provider materialization failed closed`,
- the final output directory is not created,
- the error includes `preserved_scratch=`, and
- the preserved scratch path exists long enough for inspection.

## Checks

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$PATH rustfmt --check src/main.rs && \
git diff --check && \
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . && \
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
```

Output summary:

```text
rustfmt --check: completed successfully
git diff --check: completed successfully
cairn validate: valid=true
cairn tasks gate: verdict=PASS
```

## Remaining blocker

The preserved scratch directory only makes blocker evidence durable for inspection. Mantle still has no durable final Rust provider output, no provider-backed self-build, and no fixed-point proof.
