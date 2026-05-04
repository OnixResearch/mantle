# V2 build evidence

`bootstrap/musl-1.1.24-tcc.ncl` now passes focused validation. A follow-up focused validation of `bootstrap/tcc-musl.ncl` fails later at `tcc-0.9.27-musl.drv` with exit status 139, so the runtime-validation change still lacks a produced musl-linked TinyCC output.

Provider/fallback/placeholder/leakage fields are recorded in:
- `V2-musl-1.1.24-tcc-passed-validation-summary.json`
- `V2-tcc-musl-after-musl-validation-summary.json`

Current failure class: build failure in `tcc-0.9.27-musl.drv` after first musl succeeds.


## Mes-host tcc-musl attempt

Evidence: `V2-tcc-musl-mes-host-attempt-*`. Porting prep source normalizations and compiling with TinyCC 0.9.26 advanced past `tcc.c` object compilation to final host-link library search/diagnostic failure; no output path yet.

## tcc-musl link repair pass

A focused validation of `bootstrap/tcc-musl.ncl` passed after switching the handoff to the TinyCC 0.9.26 Mes host compiler, porting the proven TinyCC source-normalization seams, linking the bridge compiler with Mes runtime objects/archive, and narrowing the first-stage smoke to an installed-compiler object compile. Evidence prefix: `V2-tcc-musl-link-repair-*`.
