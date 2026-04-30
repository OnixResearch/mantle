Task-ID: V3
Covers: bootstrap.part.tcc.musl.v2

Status: deferred

The final musl TinyCC smoke test depends on a completed V2 build output. The derivation now requires installed `tcc`, `tcc-0.9.27-musl-v2`, `libtcc1.a`, and includes a trivial C compile smoke, but runtime proof still requires a successful Crunch build output and post-build smoke evidence.

Deferred to OpenSpec change `live-part-tcc-musl-v2-runtime-validation`.

Verified: 2026-04-30T22:58:00Z
