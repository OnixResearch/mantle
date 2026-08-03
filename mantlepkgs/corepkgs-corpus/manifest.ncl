let contracts = import "../contracts.ncl" in
let locked_revision = "a9a1af8abbf08b972dbce7bb9c2643c7d76d140d" in
{
  schema = "mantlepkgs-manifest-v1",
  source = {
    reference = "github:ekala-project/corepkgs/%{locked_revision}",
    revision = locked_revision,
    lock_digest_blake3 = "e12571f07be5bb6b747d0012f78621bd4376fdbb99917f5a2a65e1fd68b81088",
  },
  systems = ["x86_64-linux"],
  selectors = [
    {
      name = "corepkgs-zlib",
      attribute = "legacyPackages.x86_64-linux.zlib",
      system = "x86_64-linux",
      aliases = ["zlib"],
    },
    {
      name = "corepkgs-libpng",
      attribute = "legacyPackages.x86_64-linux.libpng",
      system = "x86_64-linux",
      aliases = ["libpng"],
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
    generation_directory = "mantlepkgs/corepkgs-generations",
  },
  limits = {
    max_selectors = 8,
    max_graph_nodes = 8192,
    max_graph_bytes = 536870912,
    max_source_requirements = 8192,
    max_artifact_bytes = 1073741824,
  },
} | contracts.Manifest
