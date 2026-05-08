# Complete musl 1.1.24 tcc runtime validation

## Why

Parent change `live-part-musl-1-1-24-tcc` completed upstream ordering, derivation audit, source-pin validation, and output-contract hardening for `bootstrap/musl-1.1.24-tcc.ncl`. Runtime validation depends on earlier repaired bootstrap parts, especially the make 3.82/tcc amd64 execution path.

## What Changes

- Build `bootstrap/musl-1.1.24-tcc.ncl` with the documented bootstrap environment after prerequisite runtime blockers are resolved.
- Record output path or failure class, provider selection, fallback status, and placeholder rejection result.
- Smoke-test the produced musl contract (`libc.a`, headers, startup object).
- Scan the derivation and runtime transcript for undeclared host-tool/path/environment leakage.

## Scope

In scope: runtime validation evidence and narrowly scoped fixes required to prove the first musl pass.

Out of scope: later rebuilt musl stages and tcc-musl stages, which have their own changes.
