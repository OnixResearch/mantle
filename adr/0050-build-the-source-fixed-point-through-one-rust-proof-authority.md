# ADR 0050: Build the source fixed point through one Rust proof authority

## Status

Accepted (2026-07-30)

## Context

Mantle has three narrower mechanisms:

- protected StageX transition and intermediate-provider publication;
- full-source native and Rust provider construction;
- Cargo-free stage1 and stage2 Mantle execution.

The existing Cargo-free fixed-point command consumes provider paths. Its `mantle-cargo-free-fixed-point-proof-v1` result is useful bounded evidence, but it cannot prove that the current proof constructed those providers from source. An external driver can sequence commands, but it cannot provide one typed authority plan or prevent imported outputs from satisfying completion.

## Decision Drivers

- Start from authenticated source authority and empty output authority.
- Keep provider construction and provider consumption distinct in evidence.
- Make stage1 Mantle the only stage2 orchestrator.
- Preserve durable failed attempts before evaluating success.
- Reuse validated mechanism code without importing old success status.
- Emit the existing content-bound `mantle-deterministic-proof-receipt-v2` contract.

## Decision

Mantle uses a new functional core and Rust proof shell for the source-built fixed point.

The pure core defines one six-stage plan:

1. protected StageX transition;
2. StageX intermediate-provider publication;
3. full-source native-provider construction;
4. full-source Rust-provider construction;
5. Cargo-free Mantle stage1;
6. Cargo-free Mantle stage2, orchestrated by stage1.

The plan accepts observed source identities, policy identities, empty output-root observations, and resource bounds. It rejects imported native providers, imported Rust providers, prior Mantle outputs, practical hermeticity, live fetches, Cargo, ambient discovery, fallback, and cache-based provider completion.

The imperative shell owns filesystem observation, source hydration, process execution, protected execution, stage evidence, and atomic alias updates. It calls existing StageX, provider, Rust-provider, and Cargo-free mechanisms through typed requests. Those mechanisms do not transfer historical completion status into the new proof.

The old v1 Cargo-free fixed-point command remains a narrower diagnostic and compatibility surface. It cannot satisfy the source-built fixed-point requirement.

## Alternatives Considered

### Extend the v1 result in place

Rejected because existing callers can supply provider paths. Adding a success field would risk treating provider consumption as provider construction.

### Use an external shell or Rust script

Rejected as the proof authority. A driver remains useful for operator setup, but the repository binary must own the typed plan, authority checks, durable evidence, and v2 receipt.

### Rebuild each mechanism without reuse

Rejected because it would duplicate validated StageX and Cargo-free logic. The new shell reuses mechanisms while the new plan owns their composition and claim boundary.

## Consequences

- Source-built completion requires a new v2 proof bundle.
- Existing v1 proof bundles remain valid only for their narrower claims.
- The source profile must carry every required native, Rust, Mantle, vendor, lineage, seed, and policy input.
- Long provider builds can fail independently while preserving exact stage evidence.
- A matching stage1 and stage2 proves only the recorded fixed point. It does not prove compiler correctness, seed correctness, kernel isolation, independent rebuild agreement, release reproducibility, deployment success, or full Cargo compatibility.
