# Fix tinycc Mes `BufferedFile` codegen blocker

## Why

`live-part-tinycc-0-9-26` cannot complete because `crunch build bootstrap/tinycc-mes.ncl` fails after Mes successfully builds and `tcc-mes -version` runs. The first self-compile (`tcc-boot0`) segfaults, and the transcript shows mescc emitted `BufferedFile` type diagnostics while generating `tcc.s`.

Parent evidence: `openspec/changes/live-part-tinycc-0-9-26/evidence/V2-build.md`.

## What Changes

- Diagnose whether the defect belongs to `bootstrap/tinycc-mes.ncl` source normalization, Mes/mescc module setup, or a missing upstream live-bootstrap patch.
- Fix `bootstrap/tinycc-mes.ncl` so `tcc-mes` can compile `tcc-boot0` without segfaulting.
- Preserve the hard rule that `tcc-mes -version` alone is insufficient evidence; proof requires the full `tinycc-0.9.26` output contract.

## Impact

- Unblocks `live-part-tinycc-0-9-26` V2/V3.
- Reduces false confidence in early tinycc evidence by making the first self-compile the acceptance boundary.
