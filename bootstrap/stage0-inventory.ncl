# Typed source for the host-tool-free stage0 inventory.
# Real no-host-tools proof runs pass a generated concrete inventory with
# absolute operator-supplied seed paths and BLAKE3 digests.

let Digest = {
  algorithm | String,
  hex | String,
  interoperability_reason | Optional String,
} in
let VersionEvidence = {
  command | Array String,
  byte_limit | Number,
  output_digest | Digest,
  output_sample | String,
  exit_code | Number,
} in
let ExecutableSeed = {
  schema_version | String,
  id | String,
  role | String,
  phase | String,
  executable_path | String,
  digest | Digest,
  version_evidence | Optional VersionEvidence,
  provenance_category | String,
  provenance | String,
  allowed_reason | String,
  owner | String,
  required | Bool,
} in
let SourceSeed = {
  schema_version | String,
  id | String,
  role | String,
  phase | String,
  urls | Array String,
  extraction_rules | Array String,
  digest | Digest,
  provenance_category | String,
  provenance | String,
  allowed_reason | String,
  owner | String,
  required | Bool,
} in
let Stage0Inventory = {
  executable_entries | Array ExecutableSeed,
  source_entries | Array SourceSeed,
} in
{
  executable_entries = [],
  source_entries = [],
} | Stage0Inventory
