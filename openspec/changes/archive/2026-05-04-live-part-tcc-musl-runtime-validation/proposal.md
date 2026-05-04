# Complete tcc musl runtime validation

## Why

Parent change `live-part-tcc-musl` completed ordering, derivation audit, source-pin validation, and output-contract hardening for the musl-linked TinyCC stage. Runtime validation depends on earlier repaired bootstrap parts, including make 3.82, first musl pass, and tcc-musl-prep.

## What Changes

- Build `bootstrap/tcc-musl.ncl` after prerequisite runtime blockers are resolved.
- Record output path or failure class, provider selection, fallback status, and placeholder rejection result.
- Smoke-test the produced musl-linked `tcc` by compiling a trivial C program and checking `libtcc1.a`.
- Scan derivation and runtime transcript for undeclared host-tool/path/environment leakage.

## Scope

In scope: runtime validation evidence and narrowly scoped fixes required to prove musl-linked TinyCC execution.

Out of scope: tcc-musl-v2 and downstream post-musl tools, which have their own changes.
