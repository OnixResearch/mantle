Task-ID: V3
Covers: bootstrap.part.tcc.musl

Status: deferred

The musl-linked TinyCC smoke test depends on a completed V2 build output. The derivation now requires installed `tcc`, `tcc-0.9.27-musl`, `libtcc1.a`, and includes a trivial C compile smoke, but runtime proof still requires a successful Crunch build output and post-build smoke evidence.

Deferred to OpenSpec change `live-part-tcc-musl-runtime-validation`.

Verified: 2026-04-30T22:45:00Z
