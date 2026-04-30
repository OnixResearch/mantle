# Complete patch 2.5.9 runtime validation

## Why

Parent change `live-part-patch-2-5-9` completed ordering, derivation audit, source-pin validation, and output-contract hardening for `bootstrap/patch-tcc.ncl`. Runtime validation depends on earlier repaired bootstrap parts, especially make 3.82 and TinyCC amd64 execution.

## What Changes

- Build `bootstrap/patch-tcc.ncl` after prerequisite runtime blockers are resolved.
- Record output path or failure class, provider selection, fallback status, and placeholder rejection result.
- Smoke-test the produced `patch` binary by applying a simple unified diff.
- Scan derivation and runtime transcript for undeclared host-tool/path/environment leakage.

## Scope

In scope: runtime validation evidence and narrowly scoped fixes required to prove patch 2.5.9 execution.

Out of scope: unrelated downstream post-musl tools.
