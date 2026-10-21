# Mantle typed profile for the explicitly beta KernelScript experiment.
#
# This module describes bounded planning inputs only. It does not materialize a
# compiler lock, execute code generation, inspect the running kernel, load BPF,
# load modules, or grant Onix/ChaosControl authority.
#
# Nickel record contracts may carry at most one field without a definition, so
# shapes with several required fields are expressed as a sequence: an open
# defaults contract that fills the defaulted fields, an explicit presence check
# for the required fields, then a shape check that requires exactly the declared
# fields and type-checks the required ones.

let { Shape, .. } = import "contracts.ncl" in

# Nickel 1.17 record types accept a missing field when that field was declared
# optional by an earlier contract, so the required fields of a shape are also
# checked for presence explicitly.
let RequiredFields = fun field_names =>
  std.contract.from_validator (fun value =>
    if !std.is_record value then
      'Error { message = "expected a record" }
    else
      let missing = std.array.filter (fun name => !std.record.has_field name value) field_names in
      if std.array.length missing == 0 then
        'Ok
      else
        'Error { message = "missing required field(s): %{std.string.join ", " missing}" }
  )
in

let ManifestSchema = "mantle-kernelscript-experiment-v1" in
let DefaultNonClaims = [
  "beta-language-soundness-not-proven",
  "bpf-verifier-acceptance-not-proven",
  "kernel-safety-not-proven",
  "runtime-correctness-not-proven",
  "deployability-not-proven",
  "production-readiness-not-proven",
  "onix-semantics-external",
  "chaoscontrol-runtime-evidence-required",
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
    'Error { message = "expected a 64-character lowercase BLAKE3 identity" }
) in

let Sha256Hex = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value then
    'Ok
  else
    'Error { message = "expected a 64-character lowercase SHA-256 identity" }
) in

let GitRevision = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value then
    'Ok
  else
    'Error { message = "expected an exact 40-character lowercase Git revision" }
) in

let ArtifactRef = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^mantle://blake3/[0-9a-f]{64}$" value then
    'Ok
  else
    'Error { message = "expected a mantle://blake3/<64 lowercase hex> artifact ref" }
) in

let RelativePath = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value > 0 then
    if std.string.is_match "^/" value then
      'Error { message = "expected a relative path" }
    else if std.string.is_match "(^|/)\\.\\.?(/|$)" value then
      'Error { message = "relative paths must not contain dot or parent segments" }
    else
      'Ok
  else
    'Error { message = "expected a non-empty relative path" }
) in

let PositiveNumber = std.contract.from_validator (fun value =>
  if std.is_number value && value > 0 then
    'Ok
  else
    'Error { message = "expected a positive numeric bound" }
) in

let KernelBuildIdentity = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.is_match "^onix:blake3:kernel-build:[0-9a-f]{64}$" value then
    'Ok
  else
    'Error { message = "expected an exact Onix kernel-build BLAKE3 identity" }
) in

let KernelArchitecture = [| 'x86-64, 'aarch64 |] in
let NetworkPolicy = [| 'denied |] in
let ToolRole = [|
  'ocaml,
  'dune,
  'menhir,
  'clang,
  'c-compiler,
  'bpftool,
  'libbpf,
  'elf-library,
  'zlib,
  'kernel-build,
|] in
let TargetInputRole = [| 'btf, 'headers, 'config |] in
let ArtifactOutputClass = [|
  'generated-source-bundle,
  'userspace-loader,
  'ebpf-object,
  'kernel-module,
  'test-binary,
  'candidate-pack-manifest,
|] in
let GeneratedFileClass = [|
  'userspace-c,
  'ebpf-c,
  'module-c,
  'test-c,
  'makefile-evidence,
  'kbuild-evidence,
  'documentation-evidence,
|] in

let ArtifactIdentity = {
  artifact_ref : ArtifactRef,
  digest_blake3 : Blake3Hex,
  size_bytes : PositiveNumber,
} in

let SourceIdentity = {
  relative_path : RelativePath,
  digest_blake3 : Blake3Hex,
  size_bytes : PositiveNumber,
} in

let CompilerDependency = {
  package : NonEmptyString,
  version : NonEmptyString,
  artifact : ArtifactIdentity,
} in

let CompilerDependencyLock = {
  format : std.contract.Equal "opam-lock",
  dependency_count : PositiveNumber,
  dependencies : Array CompilerDependency,
  lock_blake3 : Blake3Hex,
} in

let CompilerCohort = {
  version : NonEmptyString,
  source_revision : GitRevision,
  source_archive_url : NonEmptyString,
  source_archive_sha256 : Sha256Hex,
  source_archive_blake3 : Blake3Hex,
  dependency_lock : CompilerDependencyLock,
  compiler_executable : ArtifactIdentity,
  closure_blake3 : Blake3Hex,
} in

let ToolIdentity = {
  role : ToolRole,
  version : NonEmptyString,
  executable_or_library : ArtifactIdentity,
  configuration_blake3 : Blake3Hex,
} in

let TargetInputIdentity = {
  role : TargetInputRole,
  artifact : ArtifactIdentity,
  architecture : KernelArchitecture,
  kernel_release : NonEmptyString,
  kernel_build_identity : KernelBuildIdentity,
} in

let KernelTarget = {
  architecture : KernelArchitecture,
  kernel_release : NonEmptyString,
  kernel_build_identity : KernelBuildIdentity,
  btf : TargetInputIdentity,
  headers : TargetInputIdentity,
  config : TargetInputIdentity,
} in

let ExpectedGeneratedFileDefaults = {
  relative_path | RelativePath | optional,
  class | GeneratedFileClass | optional,
  required | Bool | default = true,
  max_bytes | PositiveNumber | optional,
  ..
} in
let ExpectedGeneratedFileRequired = {
  relative_path : RelativePath,
  class : GeneratedFileClass,
  max_bytes : PositiveNumber,
} in
let ExpectedGeneratedFile = std.contract.Sequence [
  ExpectedGeneratedFileDefaults,
  RequiredFields [ "relative_path", "class", "max_bytes" ],
  Shape [ "required" ] ExpectedGeneratedFileRequired,
] in

let ExperimentBounds = {
  max_generated_files : PositiveNumber,
  max_generated_file_bytes : PositiveNumber,
  max_generated_total_bytes : PositiveNumber,
  max_compilation_steps : PositiveNumber,
  max_output_files : PositiveNumber,
  max_output_bytes : PositiveNumber,
  max_elf_sections : PositiveNumber,
  max_section_name_bytes : PositiveNumber,
  max_receipt_blockers : PositiveNumber,
  max_text_bytes : PositiveNumber,
} in

let KernelScriptExperimentProfileDefaults = {
  schema | std.contract.Equal ManifestSchema | default = ManifestSchema,
  experiment_id | NonEmptyString | optional,
  beta | std.contract.Equal true | default = true,
  enabled_by_default | std.contract.Equal false | default = false,
  source | SourceIdentity | optional,
  compiler | CompilerCohort | optional,
  toolchain | Array ToolIdentity | optional,
  target | KernelTarget | optional,
  output_classes | Array ArtifactOutputClass | optional,
  expected_generated_files | Array ExpectedGeneratedFile | optional,
  bpf_compiler_flags | Array String | optional,
  userspace_compiler_flags | Array String | optional,
  module_compiler_flags | Array String | default = [],
  bounds | ExperimentBounds | optional,
  network_policy | NetworkPolicy | default = 'denied,
  non_claims | Array String | default = DefaultNonClaims,
  ..
} in
let KernelScriptExperimentProfileRequired = {
  experiment_id : NonEmptyString,
  source : SourceIdentity,
  compiler : CompilerCohort,
  toolchain : Array ToolIdentity,
  target : KernelTarget,
  output_classes : Array ArtifactOutputClass,
  expected_generated_files : Array ExpectedGeneratedFile,
  bpf_compiler_flags : Array String,
  userspace_compiler_flags : Array String,
  bounds : ExperimentBounds,
} in
let KernelScriptExperimentProfile = std.contract.Sequence [
  KernelScriptExperimentProfileDefaults,
  RequiredFields [ "experiment_id", "source", "compiler", "toolchain", "target", "output_classes", "expected_generated_files", "bpf_compiler_flags", "userspace_compiler_flags", "bounds" ],
  Shape [ "schema", "beta", "enabled_by_default", "module_compiler_flags", "network_policy", "non_claims" ] KernelScriptExperimentProfileRequired,
] in

let manifest_schema = ManifestSchema in
let default_non_claims = DefaultNonClaims in
let kernelscript_experiment_profile_contract = KernelScriptExperimentProfile in
let artifact_output_class_contract = ArtifactOutputClass in
let generated_file_class_contract = GeneratedFileClass in
let kernel_architecture_contract = KernelArchitecture in
let tool_role_contract = ToolRole in
{
  ManifestSchema = manifest_schema,
  DefaultNonClaims = default_non_claims,
  KernelScriptExperimentProfile = kernelscript_experiment_profile_contract,
  ArtifactOutputClass = artifact_output_class_contract,
  GeneratedFileClass = generated_file_class_contract,
  KernelArchitecture = kernel_architecture_contract,
  ToolRole = tool_role_contract,
}
