# Complete tcc musl prep runtime validation

## Why

Parent change `live-part-tcc-musl-prep` completed ordering, derivation audit, source-pin validation, and output-contract hardening for the TinyCC musl-prep bridge stage. Runtime validation depends on earlier repaired bootstrap parts, especially make 3.82 and TinyCC amd64 execution.

## What Changes

- Build `bootstrap/tcc-musl-prep.ncl` after prerequisite runtime blockers are resolved.
- Record output path or failure class, provider selection, fallback status, and placeholder rejection result.
- Smoke-test the produced bridge compiler and carried Mes libc/header contract.
- Scan derivation and runtime transcript for undeclared host-tool/path/environment leakage.

## Scope

In scope: runtime validation evidence and narrowly scoped fixes required to prove the musl-prep bridge compiler.

Out of scope: later tcc-musl and rebuilt-musl stages, which have their own changes.
