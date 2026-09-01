# V91 target producer-index failure

## Verdict

V91 passed checkpoint restoration, closure relocation, the bound rustc runtime,
BLAKE3 source framing, and consumed-host proc-macro fallback. Stage1 then failed
on an ordinary target dependency without a direct producer field.

This attempt does not prove Rust unit execution, stage1 completion, stage2,
fixed-point equality, the final receipt, or complete trust.

## Bound inputs

- Source commit: `22befa5ad697063939279709f715fa3fe4b3b1bf`
- Orchestrator BLAKE3:
  `1a8829e3fedf133585a54759a65f72ba410ba5382ae43ece44dacf388c33c52b`
- Ready source-profile BLAKE3:
  `8f044865fdcfe1ee15d00a56b405038d76b6746368c363cca41722185a413c61`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 702,941,278,208

## Passed boundaries

V91 restored the immutable 17-payload checkpoint without repeating Rust
provider execution. Closure relocation and rustc compatibility passed.

Stage1 accepted typed source identities and resolved `strum_macros ->
rustversion` through its unique consumed host artifact.

## Root cause

Stage1 rejected target unit `proc-macro-crate` because its `toml_edit`
dependency artifact had no direct `producer_unit_id`.

`proc-macro-crate-toml-edit-edge.json` shows that the ready derivation graph
contains one exact target `toml_edit` library with the same package identity and
selected triple.

`target-producer-index-analysis.json` inspects all 2,203 target dependency
artifacts without direct producers. Every artifact has exactly one target
library producer with the same package identity and selected triple. Most also
match the normalized dependency name. The rest are renamed dependencies with a
unique package-level library.

The action adapter considered direct and consumed-host authority only. It did
not index the explicit ready graph.

## Decision

ADR 0094 builds one bounded target-library producer index from the ready graph.
It keys producers by package/triple and by package/triple/normalized target name.

After direct and consumed-host authority, target dependencies prefer the exact
normalized-name producer. Otherwise, they require one package-and-triple
library producer.

The selected unit enters action ordering and dependency input authority.
Cross-triple, binary-target, missing, and ambiguous candidates fail. This uses
explicit ready-graph authority, not ambient package discovery.

## Validation

`post-repair-validation.log` records positive target fallback, renamed fallback,
ambiguous fallback, consumed-host fallback, all Rust child-action plan tests,
and Rust formatting.

## Preserved evidence

This directory contains the exact launch records, full proof log, failed status,
checkpoint and closure reports, rustc compatibility, stage1 stderr and
authority, the extracted consumer/producer edge, the complete match-count
analysis, operator scripts, and validation evidence.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. After repository
evidence preservation, the no-follow V91 cleanup removed only its failed
staging root and increased free bytes from 669,870,936,064 to 702,864,977,920.

A second no-follow cleanup removed eight older staging roots already superseded
by V61, V86, and current repository evidence. It increased free bytes from
702,803,431,424 to 718,236,725,248. Both receipts record directory-only mode
changes and no regular-file mode changes.

Build and transfer a new release binary, refresh a Ready profile, and restore
the same checkpoint in a fresh promoted proof.
