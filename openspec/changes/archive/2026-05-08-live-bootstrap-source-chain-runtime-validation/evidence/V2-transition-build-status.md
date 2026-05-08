# V2 transition-build status

Task-ID: V2
Covers: bootstrap.source.chain.runtime-validation
Captured: 2026-05-08T21:05:19Z

## Scope

This task audits the transition-build chain from `bootstrap/binutils-tcc.ncl` through `bootstrap/gcc-10.ncl` and records whether it can be promoted by this umbrella.

## Inputs reviewed

- `openspec/changes/archive/2026-05-05-live-bootstrap-binutils-tcc-chain-runtime-validation/tasks.md`
- `openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/tasks.md`
- `openspec/changes/live-bootstrap-gcc-4-7-runtime-validation/tasks.md`

## Result

Transition-build validation is explicitly **not promotable** from this source-chain umbrella yet.

- `binutils-tcc` is archived with bounded positive bridge evidence through the binutils tool bridge. Its native `gas/as-new` path remains a documented TinyCC `file '%s' not found` boundary, and bridge fallbacks are the recorded validation boundary.
- `gcc-4.0` is archived as a deterministic negative runtime-validation drain: no `gcc-4.0.4` output exists because the predecessor TinyCC/Mes path remains blocked at `libtcc.c rc=139` before direct `decl0` runtime markers or compiler smoke tests can execute.
- `gcc-4.7` remains active with 0/5 runtime-validation tasks complete and explicitly depends on binutils-tcc and gcc-4.0 evidence or blockers.
- Therefore `bootstrap/gcc-10.ncl` transition validation cannot be claimed by this change.

This satisfies the source-chain transition-status requirement by making the blocker chain explicit and by avoiding full-source status promotion.
