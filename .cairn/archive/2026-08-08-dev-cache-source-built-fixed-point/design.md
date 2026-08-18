# Design: Dev-cache foundation for source-built fixed-point iteration

## Context

A source-built fixed-point attempt has expensive deterministic prefixes. Development runs need reuse, but promoted proof authority must start cold.

The current implementation closes the provider-adoption, persistent-store, and fast-fail foundations. It does not close fresh-directory multi-stage resume or a complete runtime cycle.

## Decisions

### Decision: Adopt only receipt-validated provider outputs

A dev run can use `--dev-provider-cache <dir>`. The pure core compares current source and policy identities with the cached receipt before the shell adopts provider output.

A mismatch is a hard miss. The normal cold construction path remains selected when the flag is absent.

### Decision: Reuse a persistent content-addressed dev store

Dev runs use a cache-owned native store and state. Existing content-addressed paths can become bounded cache hits after the ordinary store admission checks.

The promoted proof keeps a fresh store and rejects cache-hit completion.

### Decision: Report unchanged prior success without creating new proof evidence

The fast-fail core compares the current source profile with the prior successful fixed-point binding. A match returns the exact prior identity and a dev-only disposition.

The shell does not write a promoted receipt or update release aliases from this result.

### Decision: Defer fresh-directory resume and runtime confirmation

Attempt-local markers and same-directory recovery remain implementation groundwork. They do not establish fresh-directory multi-stage resume.

`add-dev-cache-cross-run-resume` owns transition-tree snapshots, restored-stage validation, cold-to-cached runtime evidence, and fresh promoted-path confirmation.

## Functional core and shell

Pure functions own cache-key construction, receipt matching, fast-fail selection, and cache disposition. The shell owns file reads, store opening, artifact copying, and report publication.

## Risks

- A stale cache can select old output. Exact source and policy linkage makes this a hard miss.
- A cache hit can look like fresh construction. Dev-only dispositions and alias bans keep the claim separate.
- Persistent state can exceed available disk. Existing proof disk bounds remain authoritative.

## Non-Claims

- A dev-cache hit does not prove fresh construction, compiler correctness, or release eligibility.
- Persistent store reuse does not prove a complete fixed point.
- This change does not claim fresh-directory resume or a completed runtime adoption cycle.
