# crunch stdlib entry point.
#
# Usage: let crunch = import "lib.ncl" in { ... } | crunch.Derivation

let contracts = import "contracts.ncl" in
let { FixedOutput } = import "fixed_output.ncl" in
let helpers = import "helpers.ncl" in
let fetch = import "fetch.ncl" in

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

  # Output selection
  select
    | doc "Select a single output from a multi-output derivation. Returns a record suitable for the `inputs` array."
    = fun drv output_name =>
      let the_drv = drv in
      let the_output = output_name in
      { drv = the_drv, output = the_output },
}
