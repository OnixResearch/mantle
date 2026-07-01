# crunch project manifest contract.
#
# Validates the structure of crunch-project.ncl files.
# Users import this contract and apply it to their manifest record.

let HashAlgo = [| 'sha256, 'sha512, 'blake3 |] in

let HashSpec = {
  algo | HashAlgo | default = 'sha256,
  expected | String | optional,
} in

let GitReference = {
  ref_type | [| 'branch, 'tag, 'rev |],
  ref_value | String,
} in

let FileInput = {
  type | force = "file",
  url | String,
} in

let TarballInput = {
  type | force = "tarball",
  url | String,
} in

let GitInput = {
  type | force = "git",
  repository | String,
  reference | GitReference | default = { ref_type = 'branch, ref_value = "main" },
} in

let InputFetchPolicy = std.contract.from_validator (fun value =>
  if value == "generation-material" || value == "build-fetch-action" || value == "imported-source-required" then
    'Ok
  else
    'Error { message = "fetch_policy must be one of generation-material, build-fetch-action, imported-source-required" }
) in

let FreshnessProbe = std.contract.from_validator (fun value =>
  if std.is_record value then
    if std.record.has_field "type" value then
      let t = value.type in
      if t == "git-ref" || t == "http-text" || t == "http-json" || t == "local-file" || t == "local-directory" || t == "command" then
        'Ok
      else
        'Error { message = "freshness probe type must be git-ref, http-text, http-json, local-file, local-directory, or command" }
    else
      'Error { message = "freshness probe must have a 'type' field" }
  else
    'Error { message = "freshness probe must be a record" }
) in

let InputKind = std.contract.from_validator (fun value =>
  if std.is_record value then
    if std.record.has_field "type" value then
      let t = value.type in
      if t == "file" || t == "tarball" || t == "git" then
        'Ok
      else
        'Error { message = "input type must be 'file', 'tarball', or 'git', got: %{t}" }
    else
      'Error { message = "input record must have a 'type' field" }
  else
    'Error { message = "input must be a record" }
) in

let ManifestInput = {
  name | String,
  kind | InputKind,
  hash | HashSpec | default = {},
  frozen | Bool | default = false,
  mirrors | Array String | default = [],
  patches | Array String | default = [],
  fetch_policy | InputFetchPolicy | default = "generation-material",
  freshness | FreshnessProbe | optional,
} in

let PatchSource = std.contract.from_validator (fun value =>
  if std.is_record value then
    if std.record.has_field "type" value then
      let t = value.type in
      if t == "local" then
        if std.record.has_field "path" value then 'Ok
        else 'Error { message = "local patch must have a 'path' field" }
      else if t == "remote" then
        if std.record.has_field "url" value then 'Ok
        else 'Error { message = "remote patch must have a 'url' field" }
      else
        'Error { message = "patch type must be 'local' or 'remote', got: %{t}" }
    else
      'Error { message = "patch must have a 'type' field" }
  else
    'Error { message = "patch must be a record" }
) in

let PatchDef = {
  name | String,
  source | PatchSource,
} in

# The top-level project manifest contract.
{
  ProjectManifest = {
    version | String | default = "1.0.0",
    inputs | Array ManifestInput | default = [],
    patches | Array PatchDef | default = [],
  },

  # Re-export sub-contracts for external use.
  HashAlgo = HashAlgo,
  HashSpec = HashSpec,
  GitReference = GitReference,
  ManifestInput = ManifestInput,
  InputFetchPolicy = InputFetchPolicy,
  FreshnessProbe = FreshnessProbe,
  PatchDef = PatchDef,
}
