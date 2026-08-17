# Implementation validation — source-built provider native static-PIE CRT

Date: 2026-06-26

## Scope

Validated the receipt-bound C compiler alias after adding private CRT runtime materialization, direct argv rewriting, readable response-file rewriting, `-static-pie` to `-static` downgrade, and scoped `-no-pie` insertion.

## Formatting

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin:$PATH rustfmt src/cargo_free_self_build.rs
```

Evidence: pueue task 109 completed successfully.

## Focused alias tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
  /home/brittonr/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin/cargo \
  --config 'build.rustc-wrapper=""' \
  test -p mantle --bin mantle receipt_bound_c_compiler_alias -- --nocapture
```

Evidence: pueue task 114.

```text
running 4 tests
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rejects_ambiguous_target_crt ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rejects_missing_target_crt ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_materializes_declared_runtime_inputs ... ok
test cargo_free_self_build::tests::receipt_bound_c_compiler_alias_rewrites_response_file_runtime_inputs ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 867 filtered out; finished in 0.04s
```

## Unset sandbox-shell binary check

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
env -u SNIX_BUILD_SANDBOX_SHELL -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
  /home/brittonr/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin/cargo \
  --config 'build.rustc-wrapper=""' \
  check -p mantle --bin mantle
```

Evidence: pueue task 116.

```text
    Checking mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.76s
```
