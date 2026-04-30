# Complete tcc musl v2 runtime validation

## Why

Parent change `live-part-tcc-musl-v2` completed ordering, derivation audit, source-pin validation, and output-contract hardening for the final self-hosted musl TinyCC stage. Runtime validation depends on earlier repaired bootstrap parts, including make 3.82, tcc-musl, and rebuilt musl.

## What Changes

- Build `bootstrap/tcc-musl-v2.ncl` after prerequisite runtime blockers are resolved.
- Record output path or failure class, provider selection, fallback status, and placeholder rejection result.
- Smoke-test the final self-hosted musl `tcc` by compiling a trivial C program and checking `libtcc1.a`.
- Scan derivation and runtime transcript for undeclared host-tool/path/environment leakage.

## Scope

In scope: runtime validation evidence and narrowly scoped fixes required to prove final musl TinyCC execution.

Out of scope: downstream post-musl tools and GCC stages.
