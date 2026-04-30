Task-ID: V3
Covers: bootstrap.part.musl.1.1.24.tcc.musl

Status: deferred

The rebuilt-musl output-contract smoke test depends on a completed V2 build output. The derivation now fails closed if neither `crt1.o` nor `Scrt1.o` exists, but runtime proof still requires a built output containing `lib/libc.a`, headers, and startup object(s).

Deferred to OpenSpec change `live-part-musl-1-1-24-tcc-musl-runtime-validation`.

Verified: 2026-04-30T22:32:00Z
