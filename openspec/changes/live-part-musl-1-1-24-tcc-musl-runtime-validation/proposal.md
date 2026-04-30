# Complete rebuilt musl 1.1.24 runtime validation

## Why

Parent change `live-part-musl-1-1-24-tcc-musl` completed ordering, derivation audit, source-pin validation, and output-contract hardening for the second musl 1.1.24 pass. Runtime validation depends on earlier repaired bootstrap parts, including make 3.82, tcc-musl, and the first musl pass.

## What Changes

- Build `bootstrap/musl-1.1.24-tcc-musl.ncl` after prerequisite runtime blockers are resolved.
- Record output path or failure class, provider selection, fallback status, and placeholder rejection result.
- Smoke-test the rebuilt musl contract (`libc.a`, headers, startup object).
- Scan derivation and runtime transcript for undeclared host-tool/path/environment leakage.

## Scope

In scope: runtime validation evidence and narrowly scoped fixes required to prove the rebuilt musl pass.

Out of scope: later tcc-musl-v2 and post-musl stages, which have their own changes.
