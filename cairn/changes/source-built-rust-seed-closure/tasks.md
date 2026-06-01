# Tasks

## Baseline and contract

- [ ] [serial] Record a fresh baseline of Rust compiler/sysroot seed exceptions from the latest source-built closure proof manifest. r[rust_package_planning.source_built_rust_seed_closure]
- [ ] [serial] Define the normalized Rust compiler/sysroot provider contract, including required roles, metadata fields, BLAKE3 digests, source identities, and build receipt identities. r[rust_package_planning.source_built_rust_seed_closure]

## Implementation

- [ ] [serial] Materialize or import a real receipt-bound Rust compiler/sysroot provider; do not mark complete with a wrapper around prebuilt Nix Rust. r[rust_package_planning.source_built_rust_seed_closure]
- [ ] [serial] Extend toolchain closure validation so Rust compiler/sysroot members can be promoted from seed exceptions to source-built members only when provider metadata and receipts are complete. r[rust_package_planning.source_built_rust_seed_closure]
- [ ] [serial] Thread the source-built Rust provider into Cargo-free one-shot and fixed-point proof commands while preserving host/target topology split behavior. r[rust_package_planning.source_built_rust_seed_closure]

## Verification

- [ ] [serial] Add positive and negative validator tests for Rust compiler/sysroot provider metadata, placeholder rejection, digest mismatch, and seed-exception demotion. r[rust_package_planning.source_built_rust_seed_closure]
- [ ] [serial] Run a Rust compiler/sysroot smoke build using only declared provider paths and record stdout/stderr, binary digest, and provider metadata. r[rust_package_planning.source_built_rust_seed_closure]
- [ ] [serial] Run one-shot Cargo-free self-build with the Rust provider or record the deterministic blocker. r[rust_package_planning.source_built_rust_seed_closure]
- [ ] [serial] Run fixed-point Cargo-free proof and record whether `not-source-built-toolchain-closure` can be removed; if not, record exact remaining non-claims. r[rust_package_planning.source_built_rust_seed_closure]
- [ ] [serial] Run `cairn validate --root .` and tasks gate, then archive only after completed tasks cite durable evidence. r[rust_package_planning.source_built_rust_seed_closure]
