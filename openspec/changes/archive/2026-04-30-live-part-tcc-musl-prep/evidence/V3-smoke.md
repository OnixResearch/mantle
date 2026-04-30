Task-ID: V3
Covers: bootstrap.part.tcc.musl.prep

Status: deferred

The musl-prep bridge output-contract smoke test depends on a completed V2 build output. The derivation now checks installed bridge compiler names, carried Mes libc/headers, and `tcc -v`, but runtime proof still requires a successful Crunch build output and post-build smoke evidence.

Deferred to OpenSpec change `live-part-tcc-musl-prep-runtime-validation`.

Verified: 2026-04-30T22:51:00Z
