# Design: Add bootstrap source pin tracking

## Goal and scope

Upstream pins for `bootstrap/` sources become machine-writable data with a
batched check and a plan-driven apply. Nickel reads pins; the updater never
touches Nickel source. Catalog packages and lock-driven dependency fetching
stay with their owning changes.

Planning success means a native change package with requirements, ownership,
and positive and negative tasks. Gate success proves package structure only.

## Current behavior

Each `bootstrap/*.ncl` carries `url` and `hash` inline (for example
`bootstrap/gcc-4.0-musl-cxx.ncl`). `crunch-project` already owns manifest,
lock, refresh, stale detection, and upgrades for project inputs; the accepted
`mantlepkgs-update-plans` spec family owns catalog-package update policy with
a typed non-executable policy, dry-run preimage-bound mutation, and advisory
evidence. Neither covers the bootstrap source family's inline NCL pins.

The external reference's researched lessons (after reading nixpkgs-update,
nix-update, Renovate, nvrunner, uscan, livecheck, Guix refresh):
machine-written state lives in a data file; identity is inferred from a
package URL; list-releases and compare-versions are separate axes; polling
must be one batched cached pass; per-package code is the exception
(`evidence/repkgs-review.md`).

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Inline NCL pins | Hand-edit URL and hash per recipe | Rejected: current state, no batched view | Recorded upstream-host churn |
| Reuse mantlepkgs-update-plans | Extend catalog update policy to bootstrap | Rejected: different source family and consumer; would widen a typed policy beyond its claim | Boundary note in both specs |
| Pin data plus batched pipeline | TOML pin records, cached resolve, pure decide, pin-only apply | Selected direction | Plan fixtures and apply diffs |
| Upstream-version guessing | Parse upstream pages ad hoc | Rejected: per-source code is the exception, declared hooks only | Hook must be explicit in the record |

## Contract and component ownership

- Pure core: record validation, version comparison, plan decision, and apply
  planning in `crunch-project-core`-adjacent pure modules; no network, no
  clock.
- Shell: the resolver with conditional-request caching under a bounded state
  directory, the CLI commands, and the fixed-output prefetch through the
  existing fetch service.
- Policy: typed Nickel for record schema and command policy; deterministic
  export for the shell.
- Existing reuse: fixed-output fetch and hash formats stay unchanged.

## Decisions

### Decision: Separate from mantlepkgs-update-plans

**Choice:** A new spec family scoped to bootstrap sources.

**Rationale:** The catalog family owns nixpkgs-package updates with its own
policy claims; folding bootstrap pins into it would stretch those claims over
a different source and consumer set.

### Decision: Plan artifact between decide and apply

**Choice:** A reviewable plan JSON is the only apply input.

**Rationale:** Matches the workspace rule that mutation is preimage-bound and
dry-run first; the same shape already governs the catalog family.

## Risks / Trade-offs

- Upstream identity data must be written once per source; migration cost is
  bounded by one record per recipe.
- Conditional-request caching needs a state directory with explicit retention;
  it is operator state, not build state.
- Resolve hooks are per-source code; they are declared, bounded, and rare by
  contract.
