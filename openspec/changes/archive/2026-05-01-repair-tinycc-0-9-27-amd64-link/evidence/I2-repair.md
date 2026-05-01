# I2 Repair TinyCC 0.9.27 amd64 link behavior

Task-ID: I2
Covers: bootstrap.compiler.tinycc.0.9.27.amd64.static-link

## Change

`bootstrap/tinycc.ncl` now carries the Mes/TCC source-normalization seams needed when TinyCC 0.9.27 is built by the fragile predecessor compiler:

- aggregate `SValue` copies in `x86_64-gen.c` are normalized through `memcpy`;
- the variable shift-count path is normalized to avoid the predecessor codegen defect;
- the bootstrap-compatible type-check seam from the predecessor handoff is carried into `tccgen.c`;
- TinyCC linker string construction paths that used `snprintf(..., "%s", ...)` for section/linker symbol names are replaced with direct `strcpy`/`strcat` construction;
- the Mes-stage CRT contract is preserved by skipping `crti.o`/`crtn.o` in `tcc_add_crt` while keeping explicit object links available.

Temporary `TCCMARK` instrumentation used during diagnosis was removed before the clean verification run.

## Evidence

Clean no-marker link matrix:

- transcript: `evidence/I2-clean-cached-link-matrix.log`
- derivation log: `.crunch-drain/tcc-link-state-no-crti/logs/danhhdnp9fd6knqziqdkcjlm72dkkl17-diag-tcc-link-matrix.drv.log`
- command exit: 0

The derivation log shows TinyCC 0.9.27 successfully produced linked executable outputs for the formerly crashing amd64 cases:

- `implicit`: status 0
- `nostdlib-full`: status 0
- `nostdlib-no-crti`: status 0
- `nostdlib-libc-first`: status 0
- `nostdlib-libc-only`: status 0

The deliberately under-linked `no-archives` case failed with undefined symbols and did not segfault, which is the expected diagnostic behavior for that negative control.

## Remaining runtime blocker

A follow-up execute smoke, recorded in `evidence/V2-clean-link-smoke.log`, shows the linked binary currently segfaults when executed. That is tracked by V2 and is separate from the repaired compiler/linker-output failure covered by I2.
