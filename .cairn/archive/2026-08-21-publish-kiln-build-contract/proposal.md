# Publish a versioned Mantle build interchange contract

## Why

Mantle publishes a stable aggregate JSON build report, but external CI consumers lack a narrow contract that binds a build request and observation to their own candidate, attempt, plan, policy, products, and idempotency identities.

Kiln needs this boundary before it can use Mantle without importing Mantle store, scheduler, sandbox, or build authority into its core.

## What Changes

- Add a standalone `no_std + alloc` `mantle-build-contract` crate.
- Add bounded request, observation, product, metric, log, cache, builder, worker, store, and receipt values.
- Add deterministic tagged BLAKE3 request and observation identities.
- Add pure fail-closed request and observation admission.
- Add a typed Nickel wire contract with positive and negative fixtures.
- Add deterministic producer fixtures and consumer-facing documentation.

The existing Mantle build command and `crunch-build-report-v1` output remain unchanged. Consumer shells retain transport, process, cancellation, persistence, and reconciliation policy.

## Impact

- **Files**: adds the contract crate, Nickel contract, fixtures, documentation, lifecycle specification, and focused checks.
- **Compatibility**: publishes version-one request and observation schemas without changing Mantle build behavior.
- **Testing**: requires positive and negative Rust tests, fixture freshness, Nickel checks, WebAssembly compilation, focused Clippy, Cairn validation, traceability, and Nix checks.
