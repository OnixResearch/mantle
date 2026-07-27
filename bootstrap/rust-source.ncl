# Source-built Rust compiler/sysroot provider materialization identity.
#
# `mantle bootstrap rust-source-provider` reads and binds this typed recipe.
# The command owns source acquisition, stage execution, validation, receipts,
# and create-new publication. This file does not import bootstrap/rust.ncl,
# Nix, rustup, or a prebuilt Rust installer.

let SourceRustRecipe = {
  schema | String,
  provider_id | String,
  materializer | String,
  route_plan | String,
  source_policy | String,
  publication_policy | String,
  forbidden_provenance | Array String,
  compatibility_non_claim | String,
} in

{
  schema = "mantle-rust-source-provider-materialization-recipe-v1",
  provider_id = "mantle-rust-source-provider",
  materializer = "mantle bootstrap rust-source-provider",
  route_plan = "bootstrap/rust-source-musl-host-plan.ncl",
  source_policy = "authenticated-offline-only",
  publication_policy = "create-new-after-complete-validation",
  forbidden_provenance = [
    "nix",
    "rustup",
    "prebuilt-rust",
    "ambient-tool-discovery",
    "host-assisted-source-root-substitution",
  ],
  compatibility_non_claim = "fetched standalone Rust does not satisfy full-source Rust or full-bootstrap evidence",
} | SourceRustRecipe
