# Typed WebAssembly component hello

This example authors a deterministic component export through `lib/wasm_component.ncl` and includes the minimal Rust/WIT source shape used by the production pipeline rail.

## Evaluate the typed export

```sh
mantle eval workflow.ncl > component-export.json
```

Inspect `schema`, `generated_inputs`, `outputs`, and `non_claims`. The exported generated inputs bind the WKG manifest/config, WAC source, command plan, and runtime profile to BLAKE3 source/dependency identities. The paths and digests in `workflow.ncl` are immutable role placeholders for authoring validation, not a production pipeline request.

## Execute the pinned production rail

The production shell requires the exact Nix-provided Rust, WKG, Wit-bindgen, wasm-component-ld, wasm-tools, WAC, wasi-virt, Wizer, Wasmtime, bubblewrap, and Octet cohort. It constructs a resolved request from those immutable inputs, denies network package resolution, materializes the component, runs Wizer twice, validates the component, executes Wasmtime, and rehashes the published evidence:

From the repository root:

```sh
nix build .#checks.x86_64-linux.wasm-component-toolchain-identity --no-link -L
nix build .#checks.x86_64-linux.wasm-component-toolchain-compatibility --no-link -L
nix develop -c cargo test -p mantle --test wasm_component_cli \
  production_cli_executes_pinned_pipeline_and_publishes_rehashable_component_evidence -- --exact
```

Expected evidence includes `execution-report.json`, `materialization-bundle.json`, `component-attestation.json`, `component-release-binding.json`, and the staged portable Wasm artifacts.

The positive rail also passes the actual unchanged materialization bundle to
the published `mantle-wasm-consumer-verifier::ConsumerRoot`, requires every
member to match, copies its directory-valued source closure into a private
capability root, and producer-remeasures that copy against the original digest
and size. It does not rewrite paths, identities, or source bytes. Run the small
producer-shaped/adversarial consumer fixtures separately:

```sh
nix develop -c cargo test -p mantle-wasm-consumer-verifier --test consumer_fixtures
nix develop -c cargo check -p mantle-wasm-consumer-verifier --no-default-features
```

For source-copy callers, open both roots once (a `/proc/self/fd` bridge is
supported), call `copy_source_closure(&bundle, &destination)`, and clean the
private destination on failure. Only source-closure can be a tree; other roles
remain regular files. Tree identity uses unchanged producer v1 native-sorted
BFS frames, includes empty directories, and counts content bytes only. Bounds
are 10,000 entries, directory depth 64 (files may have 65 components), and
536,870,912 aggregate content bytes. Shared bounded-tree lexical policy rejects
backslash-containing names rather than rewriting them. See the
[published API and supported identity contract](../../../crates/mantle-wasm-consumer-verifier/README.md).

Observation/copy/revalidation is not an atomic source snapshot and grants no
durability or runtime authority. Mantle's Wasm producer/verifier maintainers
own this boundary; Molten is the current private-snapshot intake consumer.
Provider publication and same-tree intake do not establish runtime adoption:
actor/extension installation, effect linking, deadlines, and rollback remain
consumer work. Commands here are a reproduction recipe, not an assertion that
this change's execution or lifecycle gates have already passed.

The focused negative rail rejects tool, package, composition, interface,
runtime, compile-environment, and componentization drift:

```sh
nix develop -c cargo test -p mantle --test wasm_component_cli \
  production_cli_fails_closed_on_identity_interface_composition_and_runtime_drift -- --exact
```

The typed export and production rail do not prove component behavior correctness, runtime authority, runtime sandboxing, compiler correctness, Octet policy interpretation, portability of target-specific AOT output, or release eligibility.
