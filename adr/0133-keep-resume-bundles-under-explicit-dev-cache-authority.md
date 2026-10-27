# ADR 0133: Keep resume bundles under explicit development cache authority

## Status

Proposed (2026-10-26). Corrects the cross-run resume path for
`add-dev-cache-cross-run-resume`; real development-cycle and independent cold
proof acceptance remain pending execution.

## Context

A source-built attempt requires a fresh, absent output directory. Resume bundles
were published below that output directory, so another attempt either rejected
the existing output or selected a new output and could not find the old bundles.
The storage location contradicted the intended fresh-attempt resume behavior.

Provider adoption also bypasses stage restoration. A planner's proposed
restored/executed split is therefore not an observation of what an adopting
attempt actually did.

## Decision Drivers

- Give persistent development state an explicit operator-owned root.
- Restore only revalidated content-addressed bundles into fresh staging.
- Keep planned work separate from actual restoration and provider adoption.
- Keep all development state outside promoted proof authority.

## Decision

`--dev-resume` requires the existing explicit `--dev-provider-cache` authority.
The shell enforces the same requirement and rejects a relative cache root.
Stage bundles persist below that cache's `.resume-bundles` directory, not below
an attempt output. There is no ambient cache search, implicit sibling root,
compatibility alias, or new cache option.

Fresh attempts remeasure source, plan, policy, producer, stage, and payload
identities before restoration. A changed payload cannot authorize a skip.
Provider adoption does not emit a resume-planner report describing restoration
that did not occur. The development execution report distinguishes restored,
adopted, skipped-provider-adoption, and dispatched stage dispositions;
`dispatched` does not claim that every underlying action executed afresh.

Promoted verification rejects development-authority markers, including the
execution report. Promotion uses a separate fresh cold attempt without
`--dev-*` options or inherited compiler-cache authority.

## Consequences

- Cross-run resume works across distinct fresh attempt directories when their
  explicit cache and all required identities match.
- The unusable resume-without-persistent-cache combination now rejects before
  hydration instead of appearing to offer cross-run reuse.
- Development reports do not grant fixed-point or release authority. A complete
  cold/cached/adopt observation and an independently verified promoted cold
  receipt remain separate requirements.
- Mantle source-built proof and development-cache maintainers own the storage
  boundary and its cross-attempt restoration/tamper regression.
