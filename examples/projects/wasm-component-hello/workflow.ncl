# Typed authoring example for a pinned WebAssembly component build export.
# Absolute paths and digests are immutable role placeholders; the production
# CLI rail resolves them from the pinned Nix tool cohort before execution.
let mantle = import "../../../lib/lib.ncl" in
let digest = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
let object = fun name => {
  logical_path = "/mantle/store/%{name}",
  digest_blake3 = digest,
  size_bytes = 1,
} in
let tool = fun name => {
  version = "gallery-pinned-version",
  executable = object name,
  configuration_identity_blake3 = digest,
} in
let checked_manifest = {
  name = "mantle-wasm-component-hello",
  package_resolution = {
    default_registry = "local",
    registries = [{
      namespace = "demo",
      registry = "local",
      backend = 'local,
      local_root = "/mantle/inputs/registry",
    }],
    requirements = [{
      package = "demo:dep",
      requirement = "=0.1.0",
      registry = "local",
      kind = 'wit,
    }],
  },
  wit = { package = "demo:app", world = "app", source = object "wit/app.wit" },
  implementation = {
    source = object "source",
    package = "mantle-wasm-component-hello",
    crate_name = "mantle_wasm_component_hello",
  },
  cohort = {
    rust_toolchain = tool "rust",
    rust_target = "wasm32-wasip2",
    wkg = tool "wkg",
    wit_bindgen = tool "wit-bindgen",
    wasm_component_ld = tool "wasm-component-ld",
    wasm_tools = tool "wasm-tools",
    wac = tool "wac",
    wasi_virt = tool "wasi-virt",
    wizer = tool "wizer",
    wasmtime = tool "wasmtime",
  },
  composition = {
    package = "demo:composition",
    source_wac = "package demo:composition; export app.run;",
    nodes = [{
      id = "app",
      package = "demo:app",
      world = "app",
      artifact = object "app-component",
    }],
    output_world = "app",
  },
  virtualization = {
    defaults_overridden = true,
    rules = [],
    expected_remaining_imports = [],
  },
  validation_profiles = {
    octet_artifact_profile_identity_blake3 = digest,
    expected_runtime_profile_identity_blake3 = digest,
  },
  outputs = [{
    name = "component",
    path = "component.wasm",
    class = 'validated-portable-component,
  }],
} | mantle.WasmComponentBuildManifest in

mantle.exportWasmComponentBuild {
  manifest = checked_manifest,
  source_identity_blake3 = digest,
  dependency_identity_blake3 = digest,
}
