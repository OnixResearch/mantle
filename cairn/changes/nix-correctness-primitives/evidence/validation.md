# Validation Evidence

Change: `nix-correctness-primitives`
Date: 2026-06-10

## Baseline

Command:

```text
cargo test -p mantle --bin mantle build_correctness -- --nocapture
```

Observed output before implementation (pueue task 27):

```text
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 706 filtered out; finished in 0.00s
```

Baseline Cairn validation/gates (pueue task 25):

```text
cairn-validate-baseline:PASS
cairn-gate-proposal-baseline:PASS
cairn-gate-design-baseline:PASS
cairn-gate-tasks-baseline:PASS
```

## Focused tests

Command:

```text
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
cargo test -p mantle --bin mantle build_correctness -- --nocapture
cargo test -p mantle --lib build_correctness -- --nocapture
```

Observed output:

```text
cargo test -p mantle --bin mantle build_correctness -- --nocapture: 11 passed (pueue task 48)
cargo test -p mantle --lib build_correctness -- --nocapture: 11 passed (pueue task 59)
```

## Formatting and checks

Command:

```text
cargo fmt -p mantle --check -- src/main.rs src/lib.rs src/build_correctness.rs
cargo check -p mantle --bin mantle
cargo check -p mantle --lib
```

Observed output (pueue task 58):

```text
cargo fmt -p mantle --check -- src/main.rs src/lib.rs src/build_correctness.rs: completed successfully
cargo check -p mantle --bin mantle: Finished `dev` profile
cargo check -p mantle --lib: Finished `dev` profile
```

## Cairn validation and gates

Command:

```text
set -e
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . >/tmp/mantle-nix-correctness-validate.json
echo cairn-validate:PASS
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal nix-correctness-primitives --root . >/tmp/mantle-nix-correctness-proposal.json
echo cairn-gate-proposal:PASS
nix run path:/home/brittonr/git/cairn#cairn -- gate design nix-correctness-primitives --root . >/tmp/mantle-nix-correctness-design.json
echo cairn-gate-design:PASS
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks nix-correctness-primitives --root . >/tmp/mantle-nix-correctness-tasks.json
echo cairn-gate-tasks:PASS
```

Observed output (pueue task 61):

```text
cairn-validate:PASS
cairn-gate-proposal:PASS
cairn-gate-design:PASS
cairn-gate-tasks:PASS
```

After adding this evidence transcript, validation and tasks gate were rerun.

Observed output (pueue task 66):

```text
cairn-evidence-validate:PASS
cairn-evidence-tasks:PASS
```

## Tracey note

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- tracey coverage --root .
```

Observed output (pueue task 63):

```text
error: tracey coverage failed
```

The generated report `/tmp/mantle-nix-correctness-tracey.json` contained large
pre-existing accepted-spec coverage debt and dangling refs from the active change
before sync. This transcript does not claim Tracey coverage passed.
