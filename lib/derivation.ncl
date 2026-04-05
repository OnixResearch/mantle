# crunch stdlib: Derivation contract.
#
# Closed record contract describing what the Rust glue layer consumes.
# Every field has a doc annotation for `nickel query` discoverability.

let { Name, System, Input, Sandbox, .. } = import "contracts.ncl" in
let { FixedOutput } = import "fixed_output.ncl" in

{
  name | Name
    | doc "Derivation name. Appears in the store path.",

  builder | String
    | doc "Absolute path to the build executable.",

  system | System
    | doc "Target platform."
    | default = 'x86_64-linux,

  args | Array String
    | doc "Arguments passed to the builder."
    | default = [],

  outputs | Array String
    | doc "Output names. Each gets its own store path."
    | default = ["out"],

  env | { _ : String }
    | doc "Environment variables set during the build. Output paths and system are auto-populated by the glue layer."
    | default = {},

  inputs | Array Input
    | doc "Build inputs. Strings are pre-existing store paths (source inputs). Records are derivations built first."
    | default = [],

  fixed_output | FixedOutput
    | doc "Fixed-output derivation parameters. Only set for FODs (fetchers, fixed downloads)."
    | optional,

  addressing_mode | [| 'input-addressed, 'content-addressed |]
    | doc "How output paths are computed. 'input-addressed: from derivation inputs (deterministic before build). 'content-addressed: from build output content (deterministic after build)."
    | default = 'content-addressed,

  sandbox | Sandbox
    | doc "Sandbox backend. 'native = bwrap (Linux), 'oci = OCI container, 'wasm = WASI (single-process)."
    | default = 'native,
}
