# Design: native Rust host-artifact topology execution

## Receipt boundary

The implementation will keep `rust_plan` as the review root and expose execution through the existing host-artifact topology receipt shape:

- captured `rust_plan` receipt, including `native_host_unit_graph_planning` and `unit_derivation_graph`
- ordered host and target `unit_executions`
- `build_script_metadata_runs` when a `custom-build` unit is executed
- deterministic blocker, status, claim, and receipt hash fields

The new behavior is not a schema-only rename. The topology rail must treat ready native host-unit graph material as the supported execution source and must not rely on Cargo unit graph semantics to decide what host units exist.

## Execution flow

1. Require `unit_derivation_graph.ready=true` and `native_host_unit_graph_planning.ready=true` in the captured plan.
2. Select supported host units from explicit derivation nodes whose `execution_kind` is `host` and whose target kind is `custom-build` or `proc-macro`.
3. Execute host derivations first using existing explicit `rustc` args/env/input/output material.
4. For `custom-build` units, run the produced executable through the bounded build-script metadata rail with deterministic `OUT_DIR` and parsed `cargo:` metadata surfaces.
5. Rebind produced host artifacts and any supported metadata into target consumer derivation material before invoking target `rustc`.
6. Emit ordered receipt evidence and stable BLAKE3 digests for produced host artifacts, target outputs, and metadata.

## Failure model

The rail fails closed before target `rustc` when:

- native host-unit graph planning is not ready;
- host producer nodes are absent, unsupported, fail, or emit no matching artifact;
- a target consumes a stale, missing, unreadable, or ambiguously matched host artifact;
- build-script metadata is malformed or unsupported;
- the topology requires unsupported non-build modes, unsupported target kinds, or general scheduling.

Blockers should preserve the existing deterministic class/message style and avoid raw absolute temp paths, ambient environment dumps, or secrets.

## Tests

Add focused `rust_plan_cli` coverage for:

- a proc-macro path dependency whose host artifact is native-planned, executed first, and consumed by a target unit;
- a `build.rs` package whose custom-build host unit is native-planned, executed, has deterministic metadata captured, and binds supported metadata into a target;
- a negative native-host planning blocker or missing host artifact case that stops before target `rustc`.

## Non-claims

This slice may claim only bounded host-artifact topology execution for supported native-planned host units. It must not claim Cargo compatibility, complete feature resolution, full Rust scheduling, native-link probing correctness beyond bounded metadata, or bootstrap correctness.
