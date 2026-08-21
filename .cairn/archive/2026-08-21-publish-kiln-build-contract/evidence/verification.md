# Verification evidence

Date: 2026-08-21

## Contract checks

- `cargo test -p mantle-build-contract --all-targets`: pass, 6 unit tests and 1 producer-fixture test.
- `cargo clippy -p mantle-build-contract --all-targets -- -D warnings`: pass.
- `cargo check -p mantle-build-contract --lib --target wasm32-unknown-unknown`: pass.
- `cargo octet check -p mantle-build-contract`: clean, 0 findings, 0 warnings, and 0 errors.
- Producer fixture regeneration and byte comparison: pass.
- Positive Nickel request and success fixtures: pass.
- Negative schema and product-order Nickel fixtures: rejected as required.
- Focused Nix contract, WebAssembly, and Nickel checks: pass.

## Lifecycle checks

Cairn commands use the current canonical policy explicitly because Mantle's checked-in generated policy is an older compatibility projection.

- proposal, design, and task gates: pass.
- sync into `.cairn/specs/build-interchange/spec.md`: pass.
- validation after sync and archive: pass.
- archive execution: pass; receipt `c9eb2a47307622abd8c44ae8b499613b95368a2790c01fb36bb6586a5bee08ab`.
- Tracey debt comparison against base revision `43f07de505a52e5b192536121009741f55703029`: `missing base=527 current=527 added=0 removed=0`; `dangling base=74 current=74 added=0 removed=0`.
- No `mantle.build_interchange.*` requirement is missing or dangling.

## Nix checks

The full `nix flake check path:$PWD -L` reached the repository checks and stopped at the existing `bootstrap-blocker-inventory` gate: 115 findings across 3 classes, 355 evidence-backed suppressions, and 0 promotion claims.

The same check fails with the same counts at clean base revision `43f07de505a52e5b192536121009741f55703029`. This change adds no bootstrap blocker debt. Focused contract Nix checks pass.

## Architecture and authority

The contract is a standalone `no_std + alloc` functional core. It imports no Mantle runtime, scheduler, sandbox, store, cache, filesystem, process, network, clock, credential, persistence, or retry code.

Mantle retains build meaning and evidence authority. Consumers retain candidate truth, CI state, transport, process policy, product meaning, cancellation, persistence, reconciliation, and release authority.

## Non-claims

Passing checks do not prove build correctness, sandbox completeness, cache truth, product semantics, reproducibility, or release readiness.
