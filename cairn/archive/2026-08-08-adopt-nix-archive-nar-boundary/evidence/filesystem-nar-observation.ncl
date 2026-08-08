let Upstream = {
  package | String,
  version | String,
  commit | String,
  cargo_checksum | String,
} in
let Digest = {
  algorithm | String,
  hex | String,
  sri | String,
} in
let Observation = {
  schema | String,
  source_case | String,
  upstream | Upstream,
  case_hack_policy | String,
  nar_size_bytes | Number,
  digest | Digest,
  disposition | String,
  parity_oracles | Array String,
  non_claims | Array String,
} in
{
  schema = "mantle-filesystem-nar-v1",
  source_case = "byte-safe-tree",
  upstream = {
    package = "nix-archive",
    version = "0.1.0",
    commit = "14362ab589daa4869bda744d4fbe26a1914b5491",
    cargo_checksum = "70e73d0af2e2dce844911f162414cb04cda4bca5a4847328a71034b244a6acf1",
  },
  case_hack_policy = "disabled",
  nar_size_bytes = 2816,
  digest = {
    algorithm = "sha256",
    hex = "241155fb7eee040514ea4c34cc2839f6378fcaace50f023eba26834ec510c745",
    sri = "sha256-JBFV+37uBAUU6kw0zCg59jePyqzlDwI+uiaDTsUQx0U=",
  },
  disposition = "success",
  parity_oracles = ["nix-archive-0.1.0", "adapted-snix"],
  non_claims = [
    "The observation does not prove either NAR implementation correct.",
    "The observation does not grant store, source, or release authority.",
  ],
} | Observation
