# 2026-11-09: authored byte placement, consumed lineage inputs, and the seventh cohort

This continues [program placement and second cohort](2026-11-09-program-placement-and-second-cohort.md).
It records proof-boundary repairs and current in-flight state; a launched proof
is not a proof.

## Canonical StageX closure is never rewritten

The prepared cohort must keep the StageX closure's manifest identity: the
lineage pins it as `stage0-source-bundle`
(`541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab`), and
`stagex_sources` requires the same value. Appending authored program records to
that closure changed its manifest identity, so preparation no longer touches it.
Authored StageX helper/policy/original bytes live in the native source-authority
closure, and every reference to them (helper sources, authored originals, route
selectors) names `NativeSourceBundle`. The StageX actions read the authored
program from that closure, and the route's StageX selectors now point at it too.

The lineage's `stage0-source-bundle` entry pins a *manifest identity*, not the
closure file's byte hash; `require_source_digest` now compares the closure
manifest identity for that whole-closure reference and keeps byte-hash
comparison for individual source and patch files.

## Lineage-declared inputs are consumed

Two authored steps bound inputs they never referenced, and the StageX partition
validator correctly rejects that:

- `stage0-full-check-answers` bound the empty Stage0 after-hook; it now consumes
  it as standard input (the hook carries no bytes and cannot change the command);
- `tinycc27-source-patch-*` bound the StageX patch source while each leaf carries
  one derived hunk; it now consumes the patch as standard input (the `replace`
  helper reads its operands, never stdin).

Authored StageX runtime mounts derive from the plan's logical store prefix
(`<prefix>/stagex-bison-2.3-runtime-data`) instead of a fixed `/mantle/stagex/...`
spelling.

## Diagnostics

`unused inputs cannot stand in for original command dependencies` now names the
step and its unused input labels; `require_node_input` now reports every omitted
lineage dependency it finds (bounded to eight) with the tool and source id, so a
long proof run yields the complete list rather than the first item.

## Cohorts

```text
v4  82b8a7ca… (leading-space mount repaired after run)
v5  1124de12… (patch stdin consumed)
v6  4caccd9f… (canonical closure preserved; role split)
v7  205503813fb8256f02c07fd59c93579965f12340bfbf5a373d3f88204b4140cc
    profile BLAKE3 7957a47a9d612bb0398c25c29ce4253dead4bb9bd5b3e56769e6bc0feea9bdb0
    report  BLAKE3 d4071adb5c58f723ef3ae428328c85b481ff2539d06283e131e38b3afb358f48
    CLI     BLAKE3 089d1702e6de541354e7838e2009bc13fc53c60e5365f482d401e804c42c5a89
```

Pueue `1521` ran the seventh cohort for 1 h 41 m and failed at
`expanded commands omit an original source/bootstrap dependency`; the preserved
staging directory is `.proof-v7.source-built-fixed-point-staging-3801475`.
Earlier cohorts are retained only by their recorded identities.

## In flight

Pueue `1522` (`sbfp-proof-v9`, started 2026-11-09T21:25:42-05:00) runs the same
cohort with CLI `4bd6f81febcc69d59ed8be1381bab54b3d6ffd57817a9983daf67933a24913cf`
and the aggregate diagnostic, to enumerate the omitted dependencies.

## Enumerated frontier

Pueue `1522` (CLI
`4bd6f81febcc69d59ed8be1381bab54b3d6ffd57817a9983daf67933a24913cf`, same
cohort) ran 1 h 44 m and returned the complete list the aggregate diagnostic was
added for:

```text
expanded commands omit 11 original source/bootstrap dependencies [
 "stage0-mini-kaem-exec:kaem-0",
 "mes-m2-kaem-exec:stage0-kaem",
 "tinycc-mes-exec:mescc-entrypoint",
 "tinycc-runtime-exec:mes-crt1",
 "tinycc-runtime-exec:mes-libc-tcc",
 "tinycc-final-exec:tcc-boot0",
 "tinycc-final-exec:tcc-mes",
 "tinycc27-runtime-exec:tinycc-crt1", ...]
```

Staging `.proof-v8.source-built-fixed-point-staging-669818` is preserved. The
list splits into two classes that need separate decisions:

- Script-interpreter images (`kaem-0`, `stage0-kaem`): the expansion replaces the
  interpreter with one leaf command per original script line, so the image is
  never an operand. Suppressing it must be explicit in the authored index rather
  than a silent omission.
- Materialization/derived artifacts that the authored leaves appear to omit
  (`mescc-entrypoint`, `mes-crt1`, `mes-libc-tcc`, `tcc-boot0`, `tcc-mes`,
  `tinycc-crt1`, `tinycc-libtcc1`): these look like real authoring gaps and need
  the original command's operands, not an exemption.

## Non-claims

No stage equality, action reconciliation, v2 receipt, release evidence or
independent review exists yet. I3–I5 and V1–V3 remain unchecked, and every cohort
identified above is an unreviewed prepared input.
