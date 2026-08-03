let contracts = import "../contracts.ncl" in
let locked_revision = "dfd9566f82a6e1d55c30f861879186440614696e" in
{
  schema = "mantlepkgs-manifest-v1",
  source = {
    reference = "github:NixOS/nixpkgs/%{locked_revision}",
    revision = locked_revision,
    lock_digest_blake3 = "44015a40ba7858db1fa8df4f2bcca11499b04b90be1f8eb8ae3c353009203274",
  },
  systems = ["x86_64-linux"],
  selectors = [
    {
      name = "hello",
      attribute = "hello",
      system = "x86_64-linux",
      aliases = ["mantlepkgs-hello"],
    },
    {
      name = "zlib",
      attribute = "zlib",
      system = "x86_64-linux",
      aliases = ["compression-library"],
    },
    {
      name = "libpng",
      attribute = "libpng",
      system = "x86_64-linux",
      aliases = ["image-library"],
    },
  ],
  conversion_policy = {
    mode = "recompute-blake3-v1",
    target_store_prefix = "/mantle/store",
    translation_policy = {
      path = "policy/translation.json",
      digest_blake3 = "18eaa36031cfa19529089febc9439dc4c540f27f279be21b5d8f10fef618e415",
    },
    execution_profile = {
      path = "policy/execution-profile.json",
      digest_blake3 = "054ec6754f08d60088cf3388f43295ad37621e5115e979f702a2a5c5d0eefcb8",
    },
  },
  source_policy = {
    mode = "source-bundle-required-v1",
    optional_transports = ["nario-v2"],
  },
  output = {
    generation_directory = "mantlepkgs/live-generations",
  },
  limits = {
    max_selectors = 8,
    max_graph_nodes = 2048,
    max_graph_bytes = 268435456,
    max_source_requirements = 2048,
    max_artifact_bytes = 536870912,
  },
} | contracts.Manifest
