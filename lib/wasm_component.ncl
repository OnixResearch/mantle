# Mantle stdlib: typed WebAssembly component build configuration and exports.
#
# This module describes build-time materialization only. It does not grant
# runtime authority, interpret Octet findings, or decide release eligibility.

let ManifestSchema = "mantle-wasm-component-manifest-v1" in
let ExportSchema = "mantle-wasm-component-export-v1" in
let GeneratedOwner = "mantle-wasm-component-export" in
let GeneratedReceiptSchema = "mantle-wasm-component-generated-input-receipt-v1" in
let WkgManifestTarget = ".mantle/wasm-component/wkg.toml" in
let WkgConfigTarget = ".mantle/wasm-component/wkg-config.toml" in
let WacSourceTarget = ".mantle/wasm-component/composition.wac" in
let CommandPlanTarget = ".mantle/wasm-component/command-plan.json" in
let RuntimeProfileTarget = ".mantle/wasm-component/runtime-profile.json" in
let DefaultNonClaims = [
  "not-component-behavior-correctness",
  "not-runtime-authority",
  "not-runtime-sandboxing",
  "not-release-eligibility",
  "not-octet-policy-interpretation",
] in

let NonEmptyString = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value > 0 then
    'Ok
  else
    'Error { message = "expected a non-empty string" }
) in

let Blake3Hex = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value then
    'Ok
  else
    'Error { message = "expected a 64-character lowercase BLAKE3 hex identity" }
) in

let AbsolutePath = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^/.*" value then
    'Ok
  else
    'Error { message = "expected an absolute path" }
) in

let OciProtocol = [| 'http, 'https |] in
let PackageKind = [| 'wit, 'component-library |] in
let RustProfile = [| 'debug, 'release |] in
let WizerMode = [| 'disabled, 'diagnostic, 'deterministic |] in
let AotMode = [| 'disabled, 'trusted-native |] in
let WasiSubsystem = [|
  'cli,
  'clocks,
  'environment,
  'filesystem,
  'http,
  'network,
  'poll,
  'random,
  'sockets,
  'stdio,
|] in
let VirtualizationMode = [| 'deny, 'allow, 'ignore, 'fixed-value, 'virtual-mount, 'passthrough |] in
let OutputClass = [|
  'tool-input,
  'composition-input,
  'runtime-profile,
  'wit-package,
  'generated-bindings,
  'portable-component,
  'validated-portable-component,
  'transformed-portable-component,
  'precompiled-native,
  'materialization-bundle,
  'build-report,
|] in

let StoreObject = {
  logical_path | AbsolutePath,
  digest_blake3 | Blake3Hex,
  size_bytes | Number,
} in

let ToolIdentity = {
  version | NonEmptyString,
  executable | StoreObject,
  configuration_identity_blake3 | Blake3Hex,
} in

let ToolCohort = {
  rust_toolchain | ToolIdentity,
  rust_target | NonEmptyString,
  wit_bindgen | ToolIdentity,
  wasm_component_ld | ToolIdentity,
  wasm_tools | ToolIdentity,
  wac | ToolIdentity,
  wasi_virt | ToolIdentity,
  wizer | ToolIdentity,
  wasmtime | ToolIdentity,
} in

let RegistryMapping = {
  namespace | NonEmptyString,
  registry | NonEmptyString,
  oci_registry | NonEmptyString,
  namespace_prefix | String | default = "",
  protocol | OciProtocol | default = 'https,
  credential_handle | NonEmptyString | optional,
} in

let PackageRequirement = {
  package | NonEmptyString,
  requirement | NonEmptyString,
  registry | NonEmptyString,
  kind | PackageKind,
} in

let LocalPackageOverride = {
  package | NonEmptyString,
  path | AbsolutePath,
} in

let PackageResolution = {
  default_registry | NonEmptyString,
  registries | Array RegistryMapping,
  requirements | Array PackageRequirement | default = [],
  local_overrides | Array LocalPackageOverride | default = [],
  lock_path | NonEmptyString | default = "wkg.lock",
} in

let WitSelection = {
  package | NonEmptyString,
  world | NonEmptyString,
  source | StoreObject,
} in

let RustImplementation = {
  source | StoreObject,
  package | NonEmptyString,
  crate_name | NonEmptyString,
  features | Array String | default = [],
  profile | RustProfile | default = 'release,
} in

let RequiredImport = {
  name | NonEmptyString,
  world | NonEmptyString,
} in

let CompositionNode = {
  id | NonEmptyString,
  package | NonEmptyString,
  world | NonEmptyString,
  artifact | StoreObject,
  required_imports | Array RequiredImport | default = [],
} in

let CompositionEdge = {
  provider_node | NonEmptyString,
  provider_export | NonEmptyString,
  consumer_node | NonEmptyString,
  consumer_import | NonEmptyString,
  world | NonEmptyString,
} in

let Composition = {
  package | NonEmptyString,
  source_wac | NonEmptyString,
  nodes | Array CompositionNode,
  edges | Array CompositionEdge | default = [],
  output_world | NonEmptyString,
  import_dependencies | Bool | default = false,
} in

let VirtualizationRule = {
  subsystem | WasiSubsystem,
  mode | VirtualizationMode,
  value | String | optional,
  value_identity_blake3 | Blake3Hex | optional,
  input | StoreObject | optional,
  guest_path | String | optional,
  review_id | String | optional,
} in

let Virtualization = {
  defaults_overridden | Bool | default = true,
  rules | Array VirtualizationRule | default = [],
  expected_remaining_imports | Array String | default = [],
} in

let ValidationProfiles = {
  octet_artifact_profile_identity_blake3 | Blake3Hex,
  expected_runtime_profile_identity_blake3 | Blake3Hex,
} in

let Wizer = {
  mode | WizerMode | default = 'disabled,
  initialization_entrypoint | String | optional,
  deterministic_virtual_imports | Array String | default = [],
} in

let Aot = {
  mode | AotMode | default = 'disabled,
  target | String | optional,
  cpu_features | Array String | default = [],
  wasmtime_configuration_identity_blake3 | Blake3Hex | optional,
} in

let OutputDeclaration = {
  name | NonEmptyString,
  path | NonEmptyString,
  class | OutputClass,
} in

let ComponentBuildManifest = {
  schema | String | default = ManifestSchema,
  name | NonEmptyString,
  package_resolution | PackageResolution,
  wit | WitSelection,
  implementation | RustImplementation,
  cohort | ToolCohort,
  composition | Composition,
  virtualization | Virtualization | default = {},
  validation_profiles | ValidationProfiles,
  wizer | Wizer | default = {},
  aot | Aot | default = {},
  outputs | Array OutputDeclaration,
  non_claims | Array String | default = DefaultNonClaims,
} in

let ComponentExportRequest = {
  manifest | ComponentBuildManifest,
  source_identity_blake3 | Blake3Hex,
  dependency_identity_blake3 | Blake3Hex,
} in

let quote = fun value => std.serialize 'Json value in

let render_registry_mapping = fun mapping =>
  "%{quote mapping.namespace} = { registry = %{quote mapping.registry}, metadata = { preferredProtocol = \"oci\", oci = { registry = %{quote mapping.oci_registry}, namespacePrefix = %{quote mapping.namespace_prefix} } } }"
in

let render_registry_transport = fun mapping =>
  "[registry.%{quote mapping.registry}.oci]\nprotocol = %{quote mapping.protocol}\n"
in

let render_wkg_config = fun manifest =>
  let namespace_lines = manifest.package_resolution.registries
    |> std.array.map render_registry_mapping
    |> std.string.join "\n"
  in
  let transport_sections = manifest.package_resolution.registries
    |> std.array.map render_registry_transport
    |> std.string.join "\n"
  in
  "default_registry = %{quote manifest.package_resolution.default_registry}\n\n[namespace_registries]\n%{namespace_lines}\n\n%{transport_sections}"
in

let render_wkg_manifest = fun manifest =>
  let override_lines = manifest.package_resolution.local_overrides
    |> std.array.map (fun override => "%{quote override.package} = { path = %{quote override.path} }")
    |> std.string.join "\n"
  in
  "[overrides]\n%{override_lines}\n"
in

let command_plan = fun component_manifest => {
  schema = "mantle-wasm-component-command-plan-v1",
  package_resolution = {
    program = "wkg",
    config = WkgConfigTarget,
    manifest = WkgManifestTarget,
    lock = component_manifest.package_resolution.lock_path,
    network = "fetch-stage-only",
  },
  compilation = {
    program = "cargo",
    wit = {
      package = component_manifest.wit.package,
      world = component_manifest.wit.world,
      source = component_manifest.wit.source,
    },
    args = [
      "build",
      "--locked",
      "--offline",
      "--target",
      component_manifest.cohort.rust_target,
      "--package",
      component_manifest.implementation.package,
    ],
  },
  composition = {
    program = "wac",
    source = WacSourceTarget,
    dependencies = component_manifest.composition.nodes,
    import_dependencies = component_manifest.composition.import_dependencies,
    implicit_registry_resolution = false,
  },
  virtualization = {
    defaults = "deny-all",
    rules = component_manifest.virtualization.rules,
  },
} in

let runtime_profile = fun manifest => {
  schema = "mantle-wasm-component-runtime-profile-reference-v1",
  octet_artifact_profile_identity_blake3 = manifest.validation_profiles.octet_artifact_profile_identity_blake3,
  expected_runtime_profile_identity_blake3 = manifest.validation_profiles.expected_runtime_profile_identity_blake3,
  non_claims = manifest.non_claims,
} in

let generated_input = fun request generated_name generated_target generated_output_class generated_content => {
  name = generated_name,
  target = generated_target,
  output_class = generated_output_class,
  owner = {
    schema = GeneratedReceiptSchema,
    generator = GeneratedOwner,
    export_name = generated_name,
    source_identity_blake3 = request.source_identity_blake3,
    dependency_identity_blake3 = request.dependency_identity_blake3,
  },
  content = generated_content,
} in

let export_component_build = fun raw_request =>
  let request = raw_request | ComponentExportRequest in
  let manifest = request.manifest in
  {
    schema = ExportSchema,
    source_identity_blake3 = request.source_identity_blake3,
    dependency_identity_blake3 = request.dependency_identity_blake3,
    generated_inputs = [
      generated_input request "wkg-manifest" WkgManifestTarget 'tool-input (render_wkg_manifest manifest),
      generated_input request "wkg-config" WkgConfigTarget 'tool-input (render_wkg_config manifest),
      generated_input request "wac-source" WacSourceTarget 'composition-input manifest.composition.source_wac,
      generated_input request "command-plan" CommandPlanTarget 'tool-input (std.serialize 'Json (command_plan manifest)),
      generated_input request "runtime-profile" RuntimeProfileTarget 'runtime-profile (std.serialize 'Json (runtime_profile manifest)),
    ],
    outputs = manifest.outputs,
    non_claims = manifest.non_claims,
  }
in

let non_empty_string_contract = NonEmptyString in
let blake3_hex_contract = Blake3Hex in
let store_object_contract = StoreObject in
let tool_identity_contract = ToolIdentity in
let tool_cohort_contract = ToolCohort in
let registry_mapping_contract = RegistryMapping in
let package_requirement_contract = PackageRequirement in
let package_resolution_contract = PackageResolution in
let wit_selection_contract = WitSelection in
let rust_implementation_contract = RustImplementation in
let composition_contract = Composition in
let virtualization_contract = Virtualization in
let wasi_subsystem_contract = WasiSubsystem in
let validation_profiles_contract = ValidationProfiles in
let wizer_contract = Wizer in
let aot_contract = Aot in
let output_class_contract = OutputClass in
let output_declaration_contract = OutputDeclaration in
let component_build_manifest_contract = ComponentBuildManifest in
let component_export_request_contract = ComponentExportRequest in
{
  ManifestSchema = "mantle-wasm-component-manifest-v1",
  ExportSchema = "mantle-wasm-component-export-v1",
  NonEmptyString = non_empty_string_contract,
  Blake3Hex = blake3_hex_contract,
  StoreObject = store_object_contract,
  ToolIdentity = tool_identity_contract,
  ToolCohort = tool_cohort_contract,
  RegistryMapping = registry_mapping_contract,
  PackageRequirement = package_requirement_contract,
  PackageResolution = package_resolution_contract,
  WitSelection = wit_selection_contract,
  RustImplementation = rust_implementation_contract,
  Composition = composition_contract,
  Virtualization = virtualization_contract,
  WasiSubsystem = wasi_subsystem_contract,
  ValidationProfiles = validation_profiles_contract,
  Wizer = wizer_contract,
  Aot = aot_contract,
  OutputClass = output_class_contract,
  OutputDeclaration = output_declaration_contract,
  ComponentBuildManifest = component_build_manifest_contract,
  ComponentExportRequest = component_export_request_contract,
  exportComponentBuild = export_component_build,
}
