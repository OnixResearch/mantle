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

Expected evidence includes `execution-report.json`, `materialization-bundle.json`, `component-attestation.json`, `component-release-binding.json`, and the staged portable Wasm artifacts. The focused negative rail rejects tool, package, composition, interface, runtime, compile-environment, and componentization drift:

```sh
nix develop -c cargo test -p mantle --test wasm_component_cli \
  production_cli_fails_closed_on_identity_interface_composition_and_runtime_drift -- --exact
```

The typed export and production rail do not prove component behavior correctness, runtime authority, runtime sandboxing, compiler correctness, Octet policy interpretation, portability of target-specific AOT output, or release eligibility.
