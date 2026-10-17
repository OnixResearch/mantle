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
- Record: TOML is the one authoritative machine-writable pin. The Rust core
  validates its typed fields; a checked, deterministically regenerated JSON
  projection is imported by Nickel. There is no evaluation-time fetch.
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

### Decision: Checked Nickel reader projection

**Choice:** One TOML file per source, plus a derived JSON import per migrated
recipe. Check/apply reject a stale projection; apply regenerates it only after
all candidate prefetches and plan checks succeed.

**Rationale:** Nickel imports JSON directly without eval-time filesystem
adapters or another human-editable authority. The derived JSON is not an
independent pin, and version bumps do not touch Nickel.

Run `python3 bootstrap/pins/generate_readers.py` after an intentional TOML
edit, then `python3 bootstrap/pins/generate_readers.py --check-recipes`
inside the project dev shell. The runtime shell parses and validates TOML
and derived JSON and checks byte-for-byte projection freshness without a
standalone Nickel binary. The bounded repository validation rail exports
each declared Nickel recipe without network fetches (45-second timeout,
8-MiB output limit) and compares its evaluated fixed-output fetch URL,
hash, and flat/tree mode to the pin. Runtime check/apply do not claim to
prove recipe binding unless this separate validation rail has run.

## Risks / Trade-offs

- Upstream identity data must be written once per source; migration cost is
  bounded by one record per recipe.
- The check cache holds at most 256 entries under `--cache-dir` and no entry
  over 64 KiB. Obsolete source caches are operator-owned state and must be
  explicitly retired when this limit is reached; check does not delete them.
- Resolve hooks are per-source declarations of a bounded JSON endpoint and
  field names; a new upstream API shape requires an explicit new generic hook.
- The pre-existing line-text `scripts/check-bootstrap-source-pins.rs`
  rejected the migrated CMake fetch as `bad-hash-format` for the valid
  `pin.artifacts.source.hash` expression (one file, one block, one issue).
  A targeted search found no active callers in scripts, Nix, CI, current
  Cairn changes, or docs beyond one historical inventory claim. The script
  was removed, that inventory claim corrected, and archived Cairn artifacts
  left untouched; the semantic `--check-recipes` rail now owns migrated
  fetch validation. Legacy inline pins are not claimed as migrated.
