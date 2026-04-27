# Repair live-bootstrap archive status

## Why

The archived `live-bootstrap-seed-chain` and `live-bootstrap-intermediate-tools`
changes were treated as complete even though their task files explicitly defer
most validation and later implementation. The current tree still has placeholder
live-bootstrap derivations (`bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.7.ncl`,
`bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`)
and the full-source chain is not buildable.

The audit also found a concrete regression from that workstream:
`bootstrap/seed-legacy.ncl` imported itself, so the legacy seed fast path no
longer contained the musl.cc reduced-provider implementation that `seed.ncl`
intended to delegate to.

## What Changes

- Restore `bootstrap/seed-legacy.ncl` to the concrete reduced musl.cc provider
  implementation.
- Keep the prior archives as historical partial scaffolding, not proof that the
  source-built seed chain is complete.
- Track the remaining live-bootstrap chain work in an active change until the
  placeholders are replaced and full validation evidence exists.
- Add spec language making recursive legacy selectors and placeholder/archive
  evidence invalid for bootstrap completion claims.

## Impact

- Legacy seed selection works again while the source-built chain remains under
  construction.
- Active OpenSpec status again reflects real unfinished live-bootstrap work.
- Full-source bootstrap claims stay blocked until the chain builds and validates.

## Non-Goals

- Finish the full hex0-to-modern-GCC implementation in this repair slice.
- Treat the existing archived live-bootstrap changes as successful build proof.
- Remove the legacy musl.cc seed before the source-built provider validates.

## Verification

This repair is valid when `bootstrap/seed-legacy.ncl` no longer imports itself,
`cargo test -p crunch --test bootstrap_eval -- --nocapture` proves the selector
still exposes the legacy musl.cc provider to downstream bootstrap entrypoints,
`openspec validate repair-live-bootstrap-archive-status` passes, and proposal /
design / tasks gates
accept that remaining live-bootstrap implementation is deferred to the active
`live-bootstrap-source-chain` successor. The full-source chain itself remains
unverified until that successor records stage-build and self-build proof
transcripts.

## Status Repair Mechanism

This repair change is the status-correction successor for the false archives.
The implementation successor is `openspec/changes/live-bootstrap-source-chain/`.
The two existing archives stay in place as historical partial-scaffolding
records:

- `openspec/changes/archive/2026-04-26-live-bootstrap-seed-chain/`
- `openspec/changes/archive/2026-04-26-live-bootstrap-intermediate-tools/`

No archive-sync action will mark those archived paths as fresh completion proof.
Instead, this repair change owns the status correction and creates the active
`live-bootstrap-source-chain` successor for the implementation work. That
successor keeps stage-by-stage live-bootstrap validation and final source-built
provider self-build proof unchecked until real command transcripts exist.
