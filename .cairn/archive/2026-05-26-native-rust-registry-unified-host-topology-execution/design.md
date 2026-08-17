# Design: Native Rust registry unified host topology execution

## Decisions

### 1. Gate unified topology with registry source and native host facts

`rust-plan --execute-topology` already receives `native_registry_source_planning`, `native_host_unit_graph_planning`, and `unit_derivation_graph` evidence. This change makes the unified path explicitly require ready registry source facts for any registry-backed host producer or target consumer edge that depends on registry host material.

A registry-backed host participant is any `unit_derivation_graph.derivations[]` entry whose `package_id` is registry-backed and whose `execution_kind == "host"`, or any target derivation consuming a host artifact whose package is registry-backed.

Required evidence:

- `native_registry_source_planning.ready == true`;
- matching registry source fact for each registry host package;
- lockfile identity and checksum;
- declared vendor/source root;
- BLAKE3 source-tree digest;
- retained oracle comparison evidence;
- `native_host_unit_graph_planning.ready == true`;
- native host producer and target consumer relationship facts.

### 2. Reuse host-artifact execution semantics inside unified topology

Unified topology execution should reuse the already-modeled host-artifact ordering and metadata behavior:

1. execute explicit host derivation nodes first when needed;
2. execute build-script binaries with deterministic `OUT_DIR` when `generated_metadata` is present;
3. parse bounded `cargo:` metadata surfaces;
4. bind build-script `OUT_DIR`, `rustc-env`, `rustc-cfg`, `rustc-link-search`, and `rustc-link-lib` into affected targets;
5. bind proc-macro host artifacts into affected target rustc arguments;
6. execute target consumers from explicit derivation material.

The unified receipt remains the main user-facing evidence. It must preserve ordered unit executions and build-script metadata runs and must be sufficient to identify which registry host producer material affected each target consumer.

### 3. Receipts bind registry provenance through host and target execution

The combined CLI JSON receipt must retain the planning sections and unified execution evidence:

- `rust_plan.native_registry_source_planning`;
- `rust_plan.native_host_unit_graph_planning`;
- `rust_plan.unit_derivation_graph`;
- `topology_execution.unit_executions`;
- `topology_execution.build_script_metadata_runs`.

For registry host producers, executed unit receipts should bind:

- registry package identity and source URL;
- lockfile checksum/source identity through retained registry source facts;
- vendor/source root through retained registry source facts;
- BLAKE3 source digest;
- host unit kind (`custom-build` or `proc-macro`);
- produced host artifact path and BLAKE3 digest;
- build-script metadata digest and OUT_DIR file digests when applicable.

For affected targets, receipts should bind consumed host artifacts/metadata and target output artifact digests.

### 4. Fail closed before host or target execution

The unified rail must block before invoking `rustc`, a build-script executable, or an affected target consumer when registry host material is not explicitly ready.

Deterministic blocker classes should cover:

- registry source planning not ready;
- registry source not declared for a registry host package;
- native host graph not ready;
- missing host derivation;
- missing host artifact binding;
- unsupported registry host layout or target kind;
- malformed or unsupported build-script metadata;
- missing/stale produced host artifacts or target inputs;
- any path that would require `$CARGO_HOME`, Cargo registry cache, network/index, git checkout, or target-dir fallback.

### 5. Focused CLI fixtures define the slice boundary

Positive fixture:

- local/path root package;
- vendored registry dependency with `build.rs` or proc-macro host unit;
- checked-in lockfile checksum and `.cargo/config.toml` vendor source replacement;
- `mantle --json rust-plan --root <fixture> --execute-topology --execution-output-root <out>` succeeds;
- JSON asserts registry source ready, native host graph ready, unified topology success, registry host unit execution, metadata/host artifact digests, and target consumer output digests.

Negative fixture:

- same shape, but with missing/stale/unsupported vendored registry host material;
- `--execute-topology` emits a blocked receipt with zero affected executions before host/target rustc.

## Verification

- `cargo fmt --check`.
- Focused `cargo test --bin mantle rust_plan` as needed for internal helpers.
- Focused and full `cargo test --test rust_plan_cli` for unified registry host topology positive/negative coverage.
- `cairn validate --root .`.
- Cairn proposal/design/tasks gates.
- `git diff --check`.
