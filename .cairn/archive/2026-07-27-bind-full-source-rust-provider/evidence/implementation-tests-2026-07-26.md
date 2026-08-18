# Full-source Rust implementation test evidence

## Scope

This evidence covers the full-source Rust provider implementation and its shell boundaries.
It does not prove provider materialization or fixed-point completion.

## Baseline

Pueue task 495 passed the focused host-tool binding tests before the scalar-output repair.
The baseline result was 15 passed and 0 failed.

## Host-tool binding

Pueue task 462 ran the current `full_source_rust_binding` filter after the per-stage native-identity repair.
The result was 39 passed and 0 failed.
These tests include positive and negative archive, attestation, stage identity, job limit, executable, and publication cases.

## Provider and closure tests

Pueue task 559 passed the current `rust_source_provider` filter after the LLVM zlib repair.
The filter includes 93 positive and negative provider tests.

Pueue task 455 ran the current `source_toolchain_closure` filter.
The result was 54 passed and 0 failed.

Pueue task 456 ran the current `bootstrap_rust_source_provider` filter.
The result was 12 passed and 0 failed.

Two earlier retries exposed transient Linux `ETXTBSY` failures.
The repair adds bounded retries for direct script launches and shell-reported nested exec failures.
The pure retry tests reject wrong error codes, wrong messages, and exhausted attempts.

## Recipe validation

Pueue task 472 evaluated the repaired recipe and musl-host route through Mantle.
Both typed Nickel records evaluated successfully.
Pueue task 559 passed the current provider tests, including positive and negative recipe-shape, native/closure stage-identity, retained-evidence, bounded Make argv, LLVM zlib isolation, and job-limit checks.

## Source pins

Pueue task 473 ran the source-pin checker self-test after the recipe repair.
It then checked six touched bootstrap Nickel files.
The result was 3 fetch blocks and 0 issues.

## Format and lint

Pueue task 463 passed focused first-party Clippy for `mantle` and `crunch-store` after the per-stage native-identity repair.
It used all targets, no dependency linting, and denied warnings.

The same task passed the root-package format check and `git diff --check`.

## Current non-claim

The detached v11 source-built Rust provider construction is still running.
No provider completion claim is valid until its output and receipts pass validation.
