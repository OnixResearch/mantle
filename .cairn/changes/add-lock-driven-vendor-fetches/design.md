# Design: Add lock-driven vendor fetches

## Goal and scope

Locked dependencies are acquired by a bounded producer that reads the lock
and emits fixed-output fetches, so source bundles stop shipping vendored
trees. Cargo is the first family. This proposal defines the contract; it does
not implement it.

Planning success means a native change package with requirements, ownership,
and positive and negative tasks. Gate success proves package structure only.

## Current behavior

`vendor-deps/` ships inside the checkout with cargo
`.cargo-checksum.json` SHA-256 metadata; the self-build source guard validates
it from `Cargo.lock` without host cargo. Fresh-clone profiles carry the
unpacked provider archive plus `vendor-deps/`; the recorded profiles run to
gigabytes, and the prepare phase hex-decodes and re-hashes them repeatedly
(perf fixes already reduced passes). The accepted `source-transports` spec
owns bundle formats, canonical payloads, and verified import; the accepted
`dynamic-derivation-admission` spec owns admission of derivations produced at
build time with complete parent identity.

The external reference implements exactly this pattern on stock Nix
(`fetch.cargoVendor` and siblings; a producer talks to the Nix daemon through
a worker-protocol client, one `builtin:fetchurl` per crate using the lock's
sha256, plus a collecting derivation; `evidence/repkgs-review.md`). Mantle
needs no worker-protocol client: the orchestrator already chains
fixed-output derivations and admits dynamic derivations natively.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Vendored trees in bundle | Ship `vendor-deps/` in source bundles | Rejected direction: derived state as source, gigabyte payloads | Bundle-size receipt comparison |
| Lock parsed at evaluation | Read lock in Nickel | Rejected: evaluation-time purity and cost; a 3,000-line lock must cost nothing |
| Producer derivation | Bounded producer emits per-artifact FODs plus assembler | Selected direction | Parent-identity admission, per-artifact verification |
| Fetch-then-rehash | Download once, hash ourselves | Rejected: invents a second hash source; the lock already has one | Tampered-lock denial fixture |

## Contract and component ownership

- Pure core: lock parsing, artifact admission, layout planning, and typed
  denials in a bounded core module; no network, no filesystem.
- Shell: the producer derivation body running under the fetch service, the
  assembling derivation, and the profile/hydration wiring in the source-bundle
  surface.
- Admission: emitted derivations flow through the accepted
  dynamic-derivation admission; no parallel mechanism.
- Policy: typed Nickel for profile modes and bounds; deterministic export.

## Decisions

### Decision: Lock hashes are the only hash source

**Choice:** The producer never recomputes artifact hashes.

**Rationale:** A single hash source keeps integrity checkable end to end;
recomputing would create a second, drift-prone authority and re-opens the
fake-hash churn problem the reference explicitly removed.

### Decision: Cargo first, contract general

**Choice:** The contract is ecosystem-neutral; the first family is Cargo.

**Rationale:** Mantle's own dependency closure is Cargo; npm-style integrity
records can adopt the same contract without redesign, but each family lands
with its own negative controls.

## Risks / Trade-offs

- Producer runs add one build step before compilation; bounded admission
  keeps it small and cacheable.
- Hash-less ecosystems need shared tables; union-merge discipline must be
  documented and tested or parallel edits will conflict.
- The offline fixed-fetch override seam must keep working: override plans
  must be able to supply producer artifacts without live acquisition, as
  fixed-point stages already require.
