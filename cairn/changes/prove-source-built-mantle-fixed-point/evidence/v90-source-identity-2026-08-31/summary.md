# V90 proc-macro producer fallback failure

## Verdict

V90 passed checkpoint restoration, closure relocation, the bound rustc runtime,
and BLAKE3 source-identity framing. Stage1 then failed while adapting one
proc-macro dependency edge into Rust action authority.

This attempt does not prove Rust unit execution, stage1 completion, stage2,
fixed-point equality, the final receipt, or complete trust.

## Bound inputs

- Source commit: `8346c02dd9e73734a821dad04238fe24f88c006d`
- Orchestrator BLAKE3:
  `05d37e395d503a6f56a8c779cda122b1cd0095e54decf18beb95a3480003469a`
- Ready source-profile BLAKE3:
  `575ea7f4a743c48db0896a5d6e00c52823020b0d18dd2a02119bfe449884bf8d`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 701,845,839,872

## Passed boundaries

V90 restored the immutable 17-payload checkpoint without repeating Rust
provider execution. The closure and binding relocation reports validated.

The receipt-bound rustc runtime wrapper passed compatibility. Stage1 action
planning accepted path, Cargo, and Git source algorithms through canonical
BLAKE3 framing.

## Root cause

Stage1 rejected unit
`97:registry+https://github.com/rust-lang/crates.io-index#strum_macros@0.26.4:strum_macros:proc-macro:build`
for an unbound `rustversion` dependency artifact producer.

A planning-only native receipt completed with status 0 and empty stderr.
`strum-macros-unit-97.json` shows the complete unit facts:

- the `rustversion` dependency artifact has no direct `producer_unit_id`;
- the same unit consumes the selected `rustversion` proc-macro host artifact;
- that host artifact names producer unit `88:...rustversion:proc-macro:build`.

This is the bounded host proc-macro fallback shape already selected by native
Rust planning. The action adapter rejected the missing direct field before
using the selected consumed host fact.

## Decision

ADR 0093 prefers a direct dependency producer. If it is absent, the adapter
selects consumed host producers with the same package and preferably the same
target name.

Exactly one producer must remain. A unique package-only producer supports
renamed dependencies. Zero or multiple producers fail.

The selected producer enters both action ordering and canonical dependency
input authority. No ambient graph or package search is permitted.

## Validation

`post-repair-validation.log` records positive fallback, ambiguous fallback,
unbound fallback, source framing, all Rust child-action plan tests, and Rust
formatting.

## Preserved evidence

This directory contains the exact launch records, full proof log, failed status,
checkpoint and closure reports, rustc compatibility, stage1 stderr and
authority, a complete compressed planning-only receipt, the extracted
`strum_macros` unit, operator scripts, and validation evidence.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. Build and transfer a
new release binary, refresh a Ready profile, and restore the same checkpoint in
a fresh promoted proof. Preserve V90 until the new proof no longer needs its
stage1 planning diagnostics.
