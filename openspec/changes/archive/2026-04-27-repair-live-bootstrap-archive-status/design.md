# Design: Repair live-bootstrap archive status

## Scope boundary

This change does not finish the whole hex0-to-modern-GCC chain. It repairs the
status model and the immediate legacy-seed regression, then leaves the remaining
source-built chain work explicit and active.

## Decisions

### 1. Restore, do not reinvent, the legacy provider

`bootstrap/seed-legacy.ncl` should be the exact reduced musl.cc provider that
existed before the selector split. `bootstrap/seed.ncl` may keep delegating to
that file while `seed-full.ncl` and the live-bootstrap ladder remain incomplete.

### 2. Treat archived live-bootstrap changes as partial scaffolding

The archives remain useful evidence of design and scaffolding, but their tasks
contain deferred validation and placeholder implementation. They must not be used
as proof of full-source bootstrap completion.

### 3. Keep future work active until validated

The active task list names the placeholder derivations and validation commands
that must pass before a future archive can honestly close the live-bootstrap
workstream.

## Concrete status repair

`openspec/changes/repair-live-bootstrap-archive-status/` is the successor that
owns unresolved live-bootstrap work. The archived paths
`openspec/changes/archive/2026-04-26-live-bootstrap-seed-chain/` and
`openspec/changes/archive/2026-04-26-live-bootstrap-intermediate-tools/` stay
archived as historical partial-scaffolding records only. They are not moved,
renamed, or re-synced as completion proof.

Completion status is derived from active successor tasks and from current
source-tree checks, not from archived checkboxes. This repair slice creates
`openspec/changes/live-bootstrap-source-chain/` as the scoped implementation
successor. Any report that mentions the 2026-04-26 archives must label them
`partial scaffolding` unless that successor has later recorded stage-build and
source-built self-build proof transcripts.

## Legacy seed restoration source

The source of truth for the restored legacy provider is the pre-selector
`bootstrap/seed.ncl` from commit `d0e08657^` (the parent of the selector split).
The shell repair copies that content into `bootstrap/seed-legacy.ncl`; the
acceptance check is that `bootstrap/seed-legacy.ncl` contains the reduced
musl.cc provider fields and no `import "seed-legacy.ncl"` self-reference.

## Out-of-scope implementation successor

`live-bootstrap-source-chain` owns the work that is too large for this status
repair: functional replacements for `bootstrap/binutils-tcc.ncl`,
`bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`,
`bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, and
`bootstrap/seed-full.ncl`, plus stage-by-stage build transcripts and final
source-built provider self-build proof. This repair may archive only after the
successor exists and the parent tasks record the deferral.

## Rejecting invalid completion evidence

The completion rule is deliberately simple and reviewable:

1. Placeholder derivations are rejected when their build script emits an
   `ERROR: ... is a placeholder` marker or the file carries TODO text stating
   the derivation is not functional.
2. Deferred tasks are rejected when an archived or active task line contains
   `Deferred`, `blocked`, or names a future/sub-change instead of recording a
   command transcript.
3. Archived partial-scaffolding changes are rejected unless a later active
   successor task cites fresh evidence and marks the relevant validation task
   complete.

These checks are human-review rules for the OpenSpec status packet and bootstrap
maturity reports. They prevent archive checkboxes from satisfying
`bootstrap.fullsource.claim.evidence`.

## Verification

Immediate repair checks:

- `rg 'import "seed-legacy.ncl"' bootstrap/seed-legacy.ncl` must return no
  matches.
- `cargo test -p crunch --test bootstrap_eval -- --nocapture` must pass to
  prove the selector still exposes the legacy musl.cc provider to downstream
  bootstrap entrypoints.
- `openspec validate repair-live-bootstrap-archive-status` must pass.
- `openspec_gate stage=proposal change=repair-live-bootstrap-archive-status`,
  `openspec_gate stage=design change=repair-live-bootstrap-archive-status`, and
  `openspec_gate stage=tasks change=repair-live-bootstrap-archive-status` must
  not report blockers.

Future full-chain checks stay open in tasks V2/V3 until real transcripts exist:

- stage-by-stage live-bootstrap derivation builds through `bootstrap/seed-full.ncl`;
- final `bootstrap/selftest.ncl`, `bootstrap/integration-test.ncl`, and
  `crunch self-build` with the source-built provider.
