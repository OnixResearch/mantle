Task-ID: V2
Covers: bootstrap.part.patch.2.5.9

Status: deferred

`bootstrap/patch-tcc.ncl` runtime build validation depends on earlier bootstrap runtime blockers, especially `repair-make-tcc-amd64-varargs` and make 3.82 runtime validation. The parent change has source-level hardening and source-pin evidence, but no completed Crunch build output path yet.

Deferred to OpenSpec change `live-part-patch-2-5-9-runtime-validation`.

Verified: 2026-04-30T22:38:00Z
