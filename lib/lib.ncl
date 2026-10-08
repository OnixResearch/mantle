# crunch stdlib entry point.
#
# Usage: let crunch = import "lib.ncl" in { ... } | crunch.Derivation

let contracts = import "contracts.ncl" in
let { FixedOutput } = import "fixed_output.ncl" in
let helpers = import "helpers.ncl" in
let fetch = import "fetch.ncl" in
let offline_cargo = import "offline_cargo.ncl" in
let wasm_component = import "wasm_component.ncl" in
let kernelscript_experiment = import "kernelscript_experiment.ncl" in
let scheduling = import "scheduling.ncl" in
let android = import "android.ncl" in

{
  # The main derivation contract. Apply with: record | crunch.Derivation
  Derivation = import "derivation.ncl",

  # Fixed-output derivation parameters contract.
  FixedOutput = FixedOutput,

  # Validators
  StorePath = contracts.StorePath,
  Name = contracts.Name,

  # Enum contracts
  System = contracts.System,
  HashAlgo = contracts.HashAlgo,
  HashMode = contracts.HashMode,
  Sandbox = contracts.Sandbox,
  Input = contracts.Input,

  # Conversion helpers
  system_to_string = helpers.system_to_string,
  hash_algo_to_string = helpers.hash_algo_to_string,
  hash_mode_to_string = helpers.hash_mode_to_string,

  # Fetchers
  fetchurl = fetch.fetchurl,
  fetchTarball = fetch.fetchTarball,
  fetchGit = fetch.fetchGit,

  # Fetch validators
  Url = fetch.Url,
  Hash = fetch.Hash,

  # Project outputs contract. Apply with: record | crunch.Project
  Project = import "project_outputs.ncl",

  # Bounded Rust project package helper.
  offlineCargoPackage = offline_cargo.offlineCargoPackage,
  offlineCargoEvidence = offline_cargo.offlineCargoEvidence,

  # Typed WebAssembly component build manifest and deterministic export.
  WasmComponentBuildManifest = wasm_component.ComponentBuildManifest,
  WasmComponentExportRequest = wasm_component.ComponentExportRequest,
  exportWasmComponentBuild = wasm_component.exportComponentBuild,
  WasmComponentOutputClass = wasm_component.OutputClass,

  # Explicitly beta, non-default KernelScript experiment profile.
  KernelScriptExperimentProfile = kernelscript_experiment.KernelScriptExperimentProfile,
  KernelScriptExperimentOutputClass = kernelscript_experiment.ArtifactOutputClass,
  KernelScriptGeneratedFileClass = kernelscript_experiment.GeneratedFileClass,

  # Deterministic lazy build-goal scheduling policy and defaults.
  SchedulingPolicy = scheduling.SchedulingPolicy,
  schedulingPolicy = scheduling.default_policy,
  schedulingKnownGraphBounds = scheduling.default_known_graph_bounds,
  schedulingHistoryPolicy = scheduling.default_history_policy,

  # Reviewed prebuilt Android source records and explicit consumer binding.
  AndroidSources = android,
  # APK plan authoring; the Rust adapter checks independently admitted bytes
  # and emits separately sandboxed steps before any tool executes.
  ApkPlan = android.Plan,
  mkApk = android.mkApk,

  # Lazy derivation file edge. The pipeline resolves this relative path before
  # conversion, keeping large derivation DAGs out of one Nickel value.
  derivationFile
    | doc "Reference a relative .ncl derivation file without embedding its transitive value."
    = fun relative_path => { derivation_file = relative_path },

  derivationFileOutput
    | doc "Reference one output from a relative .ncl derivation file."
    = fun relative_path output_name => { derivation_file = relative_path, output = output_name },

  # Output selection
  select
    | doc "Select a single output from a multi-output derivation. Returns a record suitable for the `inputs` array."
    = fun drv output_name =>
      let the_drv = drv in
      let the_output = output_name in
      { drv = the_drv, output = the_output },
}
