# Validation Evidence

Change: `rust-compiler-policy-adapter`
Date: 2026-06-10

## Focused Rust tests

Command:

```text
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
cargo test -p mantle --bin mantle compiler_policy_ -- --nocapture
cargo test -p mantle --bin mantle executes_first_supported_lib_unit_from_derivation_graph -- --nocapture
```

Observed output (pueue task 33):

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 695 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 705 filtered out; finished in 0.02s
```

## Formatting and check

Commands:

```text
cargo fmt -p mantle --check -- src/main.rs src/rust_plan.rs
cargo check -p mantle --bin mantle
```

Observed output:

```text
cargo fmt -p mantle --check -- src/main.rs src/rust_plan.rs: completed successfully (pueue task 36)
cargo check -p mantle --bin mantle: Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.99s (pueue task 41)
```

## Cairn gates before archive

Command:

```text
set -e
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . >/tmp/mantle-cairn-validate.json
echo cairn-validate:PASS
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal rust-compiler-policy-adapter --root . >/tmp/mantle-cairn-gate-proposal.json
echo cairn-gate-proposal:PASS
nix run path:/home/brittonr/git/cairn#cairn -- gate design rust-compiler-policy-adapter --root . >/tmp/mantle-cairn-gate-design.json
echo cairn-gate-design:PASS
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks rust-compiler-policy-adapter --root . >/tmp/mantle-cairn-gate-tasks.json
echo cairn-gate-tasks:PASS
```

Observed output (pueue task 64):

```text
cairn-validate:PASS
cairn-gate-proposal:PASS
cairn-gate-design:PASS
cairn-gate-tasks:PASS
```

## Archive and post-archive validation

Command:

```text
set -e
nix run path:/home/brittonr/git/cairn#cairn -- sync rust-compiler-policy-adapter --root . --execute
CAIRN_ARCHIVE_DATE=2026-06-10 nix run path:/home/brittonr/git/cairn#cairn -- archive rust-compiler-policy-adapter --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . >/tmp/mantle-cairn-post-archive-validate.json
echo cairn-post-archive-validate:PASS
```

Observed output (pueue task 69):

```text
cairn-post-archive-validate:PASS
```

After adding this evidence transcript, validation was rerun.

Command:

```text
set -e
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . >/tmp/mantle-cairn-post-evidence-validate.json
echo cairn-post-evidence-validate:PASS
```

Observed output (pueue task 70):

```text
cairn-post-evidence-validate:PASS
```
