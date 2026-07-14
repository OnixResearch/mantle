## Context

Mantle's embedded `crunch-eval` path owns evaluation and filesystem authority but also contains deterministic identity, admission, and receipt construction that now exists in `nickel-export-core`. The standalone core is `no_std` plus `alloc`, accepts explicit in-memory observations, and exposes a one-way `mantle-nickel-export-receipt-v1` projection. It does not evaluate Nickel or write destinations.

## Decisions

### 1. Release inputs use one immutable source

Cargo and Nix release inputs will select `github.com/OnixResearch/nickel-export` revision `257fafc1c746f1faf156207043a4c826bfb16d49`. Local path overrides may support diagnostics, but cannot replace the release pin or lock identity.

### 2. The embedded evaluator remains Mantle-owned

`crunch-eval` continues to resolve imports, enforce evaluator semantics, select roots, render diagnostics, and produce output bytes. Mantle continues to authorize sandbox roots and destination writes. The adapter passes normalized request data, exact dependencies, evaluator observations, diagnostics, and output bytes to the standalone core only after Mantle has produced them.

### 3. Canonical logic is pure; orchestration stays thin

Pure request conversion, admission, BLAKE3 identity, canonical receipt creation, and Mantle v1 projection delegate to `nickel-export-core`. Embedded evaluation, file access, path authority, writes, build evidence, and release orchestration remain in a thin Mantle shell. Adapter conversion functions are deterministic over explicit values and are tested without an evaluator or filesystem.

### 4. Dual-run compares shared facts, not evaluator implementations

The legacy path and canonical path consume the same source bytes, dependency bytes, evaluator descriptor, diagnostics, and output bytes. Comparison covers canonical receipt identity, Mantle v1 projection, path-confinement failures, and evaluator diagnostics. The migration does not compare two evaluators or infer semantic equivalence from equal bytes.

### 5. Drift fails closed with legacy rollback

Any unexplained difference keeps the legacy implementation authoritative. Both outputs and exact identities are retained as diagnostic evidence and classified as request normalization, dependency closure, evaluator descriptor, contract/selector handling, serialization, or Mantle policy. A positive and negative regression fixture is required before retrying.

### 6. Removal follows one full validation cycle

Duplicated local core logic is removed only after dual-run fixtures, focused tests, Nix checks, Cairn gates, machine-contract freshness, and release evidence pass against the immutable revision. The Mantle receipt projection may remain as a compatibility facade, but must delegate canonical fields to the standalone core.

## Risks / Trade-offs

- Exact pinning makes standalone updates deliberate and requires replaying compatibility fixtures.
- Keeping rollback temporarily duplicates execution paths, but prevents unexplained receipt drift from becoming authoritative.
- Matching bytes and receipts cannot establish evaluator correctness or build correctness.
- The standalone secret heuristic is conservative and does not prove secret absence; Mantle policy remains responsible for stronger admission rules.

## Non-Goals

- Replacing `crunch-eval` with the standalone external CLI.
- Moving sandbox, import-root, destination, build, or release authority into the shared repository.
- Treating a canonical receipt as build success or release eligibility.
- Preserving arbitrary historical behavior that violates current fail-closed admission.
