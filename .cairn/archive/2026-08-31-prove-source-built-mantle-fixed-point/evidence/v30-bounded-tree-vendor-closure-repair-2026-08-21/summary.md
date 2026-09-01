# V30 bounded-tree vendor-closure repair

## Goal

Retain the next exact V30 blocker and prevent stale Cargo vendor inputs from
reaching StageX or Cargo-free stage1.

## V30 authority

- Source commit: `cc3b14d5`.
- Release orchestrator BLAKE3:
  `8d03967c7378f1465027e4a2093aa8fc06637759d88722f3a420f5ac524885ab`.
- Verified profile BLAKE3:
  `1cd8c5a3d0c839101b21d08615a9ce0710aef152699d4afb09dd3596bd4a924a`.
- Attempt plan BLAKE3:
  `690473faf83f50a16bc9015d9c0ab048762c4ec3262230b8199108a58fdd2da0`.
- Attempt status time: `2026-08-21 15:11:45.961667564 -0400`.

## Result

V30 completed StageX, native-provider admission, all six host tools, and the
Rust 1.90 through 1.94 source chain. Its Cargo-free proof eligibility was
strict and admitted. Its 17-member source-built toolchain closure was enforced.

Stage1 then failed closed before unit execution:

```text
missing-git-source-manifest: captured git source manifest  is not readable
package=git+https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git?rev=b0fd0103bc9eed2c1b6d852045959462d105d8f1#b0fd0103bc9eed2c1b6d852045959462d105d8f1#bounded-tree-cap@0.1.0
```

The preserved `Cargo.lock` named `bounded-tree-core` and `bounded-tree-cap` at
revision `b0fd0103bc9eed2c1b6d852045959462d105d8f1`. The preserved Cargo vendor
config did not declare that source. Both package directories were absent from
the preserved vendor record. See `attempt-status.json` and
`cargo-free-fixed-point-meta.json`.

## Root cause

Commit `99add13e` added the two pinned Git dependencies. It did not refresh the
ignored `vendor-deps/` material or the tracked Cargo source replacement.
Later profile refreshes replaced only the Mantle source record. They preserved
the older vendor record while the new source record carried the changed lock.

## Repair

- Cargo generated both checked vendor directories from the exact pinned
  revision with locked, offline, no-delete vendoring.
- `.cargo/vendor-config.toml` now binds the exact Radicle source to
  `vendor-deps/`.
- `refresh-mantle-source` now validates the lock/vendor closure and replaces
  the Mantle source and vendor records together.
- The refresh report is now `mantle-source-built-profile-refresh-v2` and binds
  both old and new vendor BLAKE3 values.
- The proof repeats the lock, package, and Cargo file-checksum check immediately
  after materialization. This check runs before StageX.
- Positive and negative tests cover coherent pair replacement, missing or
  duplicate authority, stale vendor membership, and supplemental drift.

## Validation

Pueue task `5990` ran locked, offline, no-delete Cargo vendoring. Cargo's
complete generated config exactly matched the checked config. Both bounded-tree
package checksum manifests are present.

Pueue task `6189` recorded `local-validation.log`. It passed:

- four source/profile refresh tests;
- one materialized-vendor proof preflight test;
- ten checked-vendor positive and negative tests;
- two native Git source-planning tests;
- the refresh CLI parser test;
- source resolution with an empty `CARGO_HOME`, `--offline`, and `--locked`;
- changed-file Rust formatting;
- focused strict Clippy with only the recorded baseline allowances;
- `git diff --check`.

Pueue task `6181` ran refresh against the real checkout with an intentionally
absent input profile. The complete vendor preflight passed before the command
reported the absent profile. It created no output.

Pueue task `6154` regenerated only this command's checked descriptor, catalog,
and reference from the existing reviewed operator inventory. The ordinary
operator-contract check still stops at the pre-existing `flag-drift: build`
blocker. The live graph already contained that drift and unreviewed Mantlepkgs
commands before this repair. See `operator-contract-baseline.log` from pueue
task `6191`; this change does not accept those unrelated policy changes.

## Non-claims

V30 did not build stage1 or stage2 and did not create a final receipt. This
repair does not admit a new source revision, executable, fallback, or network
path. A fresh proof remains required.
