let contracts = import "../contracts.ncl" in
let digest = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
{
  schema = "mantlepkgs-manifest-v1",
  source = {
    reference = "github:NixOS/nixpkgs/main",
    revision = "main",
    lock_digest_blake3 = digest,
  },
  systems = ["x86_64-linux"],
  selectors = [{ name = "hello", attribute = "hello", system = "x86_64-linux", aliases = [] }],
  conversion_policy = {
    mode = "recompute-blake3-v1",
    target_store_prefix = "/mantle/store",
    translation_policy = { path = "policy/translation.json", digest_blake3 = digest },
    execution_profile = { path = "policy/execution-profile.json", digest_blake3 = digest },
  },
  source_policy = { mode = "source-bundle-required-v1", optional_transports = [] },
  output = { generation_directory = "mantlepkgs/generations" },
  limits = {
    max_selectors = 8,
    max_graph_nodes = 1024,
    max_graph_bytes = 16777216,
    max_source_requirements = 1024,
    max_artifact_bytes = 33554432,
  },
} | contracts.Manifest
