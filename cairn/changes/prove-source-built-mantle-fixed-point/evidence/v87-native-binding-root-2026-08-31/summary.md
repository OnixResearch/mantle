# V87 checkpoint Rust-sysroot relocation failure

## Verdict

V87 restored the promoted 17-payload provider checkpoint. It then failed the
closure relocation check before the first Mantle stage.

This attempt does not prove stage execution, fixed-point equality, the final
receipt, or complete trust.

## Bound inputs

- Source commit: `a1537652b6068b415d65f1e85527515409ccc453`
- Orchestrator BLAKE3:
  `eb8d5919c9a5298a035f89fc3894bc82c81be31c9c016158902f0d64ff647027`
- Ready source-profile BLAKE3:
  `68a074196a7348fecc0abad64b3f8cd034126d33e66305cff2a389297a014894`
- Expected StageX lineage BLAKE3:
  `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`
- Expected native-provider BLAKE3:
  `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 738,716,893,184

## Checkpoint result

V86 had published a new checkpoint under lookup key
`3d6ba9154ac60e3214e8486f8088050157c007667208e2e8970e8e397b1824ca`.
Its checkpoint BLAKE3 is
`c9918c0ede2fd774f5348a775981838c5590903ce6ba6a691317a766a052b3eb`.

`provider-checkpoint-manifest.json` records 17 bounded payloads. These payloads
include the Rust provider, five relocated host-tool trees, action evidence,
StageX evidence, the native provider, and the origin closure.

V87 restored and validated the checkpoint without repeating the five Rust
provider builds.

## Root cause

Checkpoint restore must rewrite absolute authority paths in the full-source
binding. That binding is inside the Rust-provider sysroot tree.

`rust-provider-relocation-diff.txt` shows equal tree shapes with 31 directories
and 203 files. Its checksum comparison finds one content change:
`share/mantle-rust-provider/receipts/full-source-binding.json`.

The binding BLAKE3 changed from
`9684d1753f538e769b945275eb619893f80a9135e7d20b0854bcbe6557fd3ee9`
to
`71028924b63495adddc9a966263c4e1dc46b537f0870759691ba7ff2cf5614e2`.
The closure-directory BLAKE3 consequently changed from
`69740316b19be878af5b1788afa0181891489bf303e7b8791f92115aed1be96e`
to
`6fb5ada63ae8d107e29083f9707e08f6f1b291e6258cd5e3343ac19352091db0`.

The strict closure comparator permits path relocation but no member content
change. It rejected `host-sysroot` before it could link the existing binding
relocation report to this expected tree transformation.

## Decision

ADR 0090 keeps the checkpoint immutable and measures the Rust sysroot directly
before and after the single canonical binding write.

The functional core permits the changed `host-sysroot` digest only when both
manifest digests equal those observations. Role, name, trust, source, receipt,
and relative path remain strict. Every other closure member remains strict.

The closure relocation report binds the binding relocation report by BLAKE3.
Checkpoint relocation also keeps native artifact paths provider-relative for
ADR 0089's closure-root loader.

## Validation

`pre-repair-validation.log` records the green baseline before this repair.
`post-repair-validation.log` records the focused positive and negative tests,
module tests, and Rust formatting check after the repair.

## Preserved evidence

This directory contains the exact profile and proof launch records, full proof
log, failed status, checkpoint manifest, origin and relocated bindings, origin
and relocated closures, binding relocation report, focused tree comparison,
operator scripts, and validation logs.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. Build and transfer a
new release binary, refresh a Ready profile, then restore the same immutable
checkpoint in a fresh promoted proof. Preserve V87 until its staging diagnostics
are no longer needed.
