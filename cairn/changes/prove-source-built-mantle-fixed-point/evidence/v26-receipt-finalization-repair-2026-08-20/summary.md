# V26 receipt finalization repair

## Goal

Remove the receipt-only blocker that followed the successful V26 Cargo-free
fixed point. Keep source admission strict and keep receipt work bounded.

## Starting evidence

V26 completed strict Cargo-free stage1 and stage2 builds with the same binary
BLAKE3:

`df3f0bd01626eab30d1a7ee07625584969db6e891c2e0d06c6a79a3b44849c5b`

The overall command failed during StageX evidence hashing:

```text
tree copy observation count exceeds 4096 before mutation
```

The preserved StageX execution tree contains intentional absolute and
parent-relative symlinks. ADR 0052 requires the final evidence to bind that
complete tree without admitting it as source.

The complete working root contained `2,519,325` readable files and links before
projection. It also contained unreadable overlay work directories. Those
working directories are attempt diagnostics, not durable proof evidence.

## Repair

- Added `src/preserved_evidence_tree.rs` as an observation-only, capability-rooted,
  no-follow tree digest.
- Bound paths, kinds, modes, file lengths and bytes, and exact opaque symlink
  target bytes with BLAKE3.
- Kept ordinary release-tree copy and source admission unchanged.
- Added positive and negative tests above the ordinary `4,096`-entry limit.
- Added byte, entry, unreadable-content, and unsafe-link rejection tests.
- Defined a durable proof projection that excludes only:
  - `cargo-free-fixed-point/execution`
  - `home`
  - `native-state`
  - `rust-provider-scratch`
  - `tmp`
- Kept the complete `stagex-transition-execution/` tree in the durable digest.
- Accepted empty evidence files, including empty stderr captures.
- Changed the bundle digest domain to
  `mantle-source-built-fixed-point-proof-bundle-v2`.
- Recorded the boundary in ADR 0079 and the active Cairn design/spec.

## Exact leviathan replay

Pueue task `851` ran the final test binary against the preserved V26 data.
The complete transcript is in `leviathan-final-replay.log`.

StageX evidence result:

```text
preserved-stagex-tree: entries=83046 bytes=1081565462 blake3=de61852c41a9bf7f7ba94d9d8db29d13c0b38486444d5166705c37b33f268c1e
```

Durable proof projection result:

```text
preserved-proof-bundle: entries=136135 blake3=de3b4a0918bd3441162b914b2d4303a212cdecd46ac71efc2bdb9b2dd5eb4e6b
```

Both ignored replay tests passed. The StageX replay took `120.06s`. The bundle
projection replay took `1380.74s`.

## Local validation

- Preserved evidence tests: `2 passed; 0 failed`.
- Receipt tests: `4 passed; 0 failed; 2 ignored`.
- Changed-file `rustfmt --check`: passed.
- Isolated focused Clippy with the two unchanged root-package findings allowed:
  passed. See `clippy-isolated.log`.
- `git diff --check`: passed.
- Compatible Cairn revision
  `f4d97a52a7190de9958487c8735408b2da60252d`:
  - validation passed;
  - proposal gate passed;
  - design gate passed;
  - tasks gate passed;
  - traceability passed at `155/155`.

The current Cairn checkout cannot parse Mantle's older generated policy because
it requires new task-ordering markers. The exact version-skew blocker is in
`cairn-current-version-blocker.log`.

Package-wide formatting, ordinary strict Clippy, and Tiger Style remain blocked
by unchanged baseline findings outside this repair. The focused Clippy rerun
also first hit a shared incremental-cache compiler ICE. The isolated target
removed that cache fault and passed.

## Result and non-claims

The exact V26 trees now pass the repaired StageX and durable-bundle receipt
boundaries. This replay does not create a promoted receipt because the original
proof process already exited. A fresh proof must produce and verify the final
receipt with the current orchestrator identity.

This repair does not prove compiler correctness, seed correctness, kernel
isolation, independent reproducibility, release eligibility, or generic release
copy support for the complete working root.
