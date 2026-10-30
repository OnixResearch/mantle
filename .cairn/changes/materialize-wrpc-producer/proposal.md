# Materialize the wRPC producer prerequisite

## Why

Molten needs independently materialized component RPC guests, not locally built
guests, ordinary TCP fixtures, or a closed static composition mislabeled RPC.

## What changes

Add an immutable public wRPC producer package with the exact reviewed upstream
source/WIT/runtime cohort, measured standalone client/server artifacts, generated
component and wRPC bindings, deny-all WASI requests, real exporter CLI receipts,
and a separately labeled WAC comparison. Reuse the existing materialization DTO
and consumer verifier. Publication and real remote execution remain parent-owned.

## Scope

Mantle authoring, source pins, producer orchestration and public handoff only.
No Molten or SpaceWasm source changes; no invented artifacts or publication hash.
