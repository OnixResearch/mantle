Task-ID: V2
Covers: bootstrap.gcc40.transition

Status: deferred

`bootstrap/gcc-4.0.ncl` runtime validation depends on binutils-tcc runtime validation. The drain attempt for `live-bootstrap-binutils-tcc-chain` resolved the missing `crunch`/`bubblewrap` prerequisite but exceeded the local budget in the Mes prerequisite for the first binutils-tcc epoch.

Deferred to OpenSpec change `live-bootstrap-gcc-4-0-runtime-validation`.

Verified: 2026-04-30T21:55:51Z
