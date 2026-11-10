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

## Input dispositions and the tenth launch

The enumerated omissions split into two authored statements rather than silent
drops. The expansion index now carries, per original tool, an
`input_dispositions` map from original input identity to the reason the
expansion supersedes it; validation skips a lineage input only when that
reason is non-empty and otherwise reports it. The authored reasons name what
happens instead:

- `kaem-0` and `stage0-kaem`: the expansion runs the script lines directly
  instead of executing the interpreter image.
- `mescc-entrypoint`: tcc is compiled through the modelled Mes module pipeline.
- `mes-crt1`, `mes-libc-tcc`: the tcc-mes node links the Mes libc archive; the
  runtime node builds its own objects.
- `tcc-boot0`, `tcc-mes`: the final tcc is rebuilt from the same sources; the
  earlier executables are not run as drivers.
- `tinycc-crt1`, `tinycc-libtcc1`: staged into the source map consumed by the
  `tinycc27-build-exec` tree.
- `oyacc-*-grammar-source`: consumed by the authored `oyacc-smoke-exec` node.

`superseding_a_lineage_input_requires_an_authored_reason` passes alongside the
other three action tests (4 passed). The release binary
`mantle-v10` (196675672 bytes) was built from this tree and launched as Pueue
`1523` against `profile-v7`. No stage equality, reconciliation, receipt, or
release evidence exists yet.

## Prepared-profile boundary and the eleventh launch

The tenth launch (Pueue `1523`, `mantle-v10`, `profile-v7`) failed closed after
1 h 46 m with the same eleven omissions and the new wording, proving the
promoted proof never re-authors programs: it consumes the prepared profile's
`source-built-*` records. Authoring changes therefore require a fresh
preparation, executed locally against the original unexpanded profile
`source-built-fixed-point-sources-refreshed-20261108-vendor-git.json`
(`a6174213ff6b336e251175ee6afbf452178502209d8773f2eb4cb09ba4e368e0`).

A local `--prepare-source-programs` run (`preparation_exit_code=0`,
2026-11-10T02:10:11-05:00) produced `prepared-cohort-v8` with profile digest
`8242b0bea1e4d415a692162ae0afc5a2697bed58be7d67ca3abe228345ef2edd` and StageX
program record `a215c026dac98ab20e5e5574a4dba15a2d0168ea4b572d5f97bbdb86d6356bc6`.
Its authored expansion carries twelve disposition entries across eight tools,
covering all eleven reported pairs. The eleventh cold launch is Pueue `1524`
against that profile.

The ordinary gate reached `2848 passed; 1 failed` on the previous run:
`remote_build::tests::real_slurm_accept_then_response_save_failure_reconciles_only_exact_dispatch_after_restart`
failed with `external-batch-process-output-limit-exceeded` while preparation
ran concurrently, and passes in isolation (1 passed, 0.18 s). A clean-host rerun
is in flight; the flake is not yet claimed as fixed or as a regression.

## Publication stage authority and the twelfth launch

The eleventh launch (Pueue `1524`) accepted all twelve authored dispositions —
the previous eleven omissions are gone — and then failed after 1 h 43 m at
`source recipe: recipe source role is not admitted to this stage`. The
publication recipe is authored in the same `source-built-stagex-programs`
record as the transition recipe, referenced through
`NativeSourceBundle`, but `stagex-provider-publication` did not admit that
source authority while `stagex-transition` did. The stage plan now admits
`NativeSourceBundle` for publication, and the rejection names the stage and
role. The plan unit suite passes (11 passed), including
`publication_requires_its_authenticated_source_program_authority`. No stage
equality, reconciliation, receipt, or release evidence exists.

## Provider claims and the fourteenth launch

The thirteenth launch (`1526`) died in preflight because the host had only
198 628 425 728 bytes of the required 200 000 000 000; deleting superseded
37 GB staging attempts restored 576 GB. The relaunch (`1527`, 13:04) reached
`publication omits original runtime supplement/positive/negative validation
commands` after 1 h 39 m: the prepared cohort's publication index claimed all
eight provider launches while the expansion only authored them when binutils
sources are admitted, and this lineage does not admit them.

The publication expansion now records `binutils_admitted` and claims only the
launches it authored; the validator requires the three TCC exports and the four
non-binutils launches when that flag is false. A fresh preparation
(`prepared-cohort-v9`, `preparation_exit_code=0`, 2026-11-10T16:05:55-05:00)
produced profile digest
`74565ada1bb6a96965e60b064fced4eef8e210cbcb5f52520351d2e7839cdb85` with
`binutils_admitted=false`, commands `runtime-supplement`, `tcc-execution`,
`tcc-negative`, `tcc-positive`, all of which exist as authored steps. The
fourteenth launch is Pueue `1528` against that profile.

Two operational facts bound this transcript. The source-built proof consumes
the prepared profile verbatim, so any authoring change requires a fresh
preparation; and the host datapool filled to zero bytes during this run, which
stalled the Nix GC lock and blocked local builds until space was reclaimed.

