# The imported shard is Rust-generated and is validated by the pure composition core.
# Typed authoring coverage lives in mantlepkgs/domains/fixtures/valid-domain.ncl.
let shard = import "domain-shard.json" in
let zero = "0000000000000000000000000000000000000000000000000000000000000000" in
{
  schema = "mantlepkgs-domain-manifest-v1",
  name = "ekala-corepkgs-a9a1af8a",
  shards = [shard],
  variants = [],
  validation_roots = [
    {
      validation_root_identity_blake3 = zero,
      name = "zlib-libpng-integration",
      system = "x86_64-linux",
      package_selector = "corepkgs-zlib",
      package_identity_blake3 = zero,
      validation_selector = "corepkgs-libpng",
      validation_package_identity_blake3 = zero,
      sources = [
        {
          role = "corepkgs-source-inventory",
          digest_blake3 = "dc0cc334cf763ab2262f93c813e032557bd039e9116aa588868340937f1fb28b",
        },
      ],
      tools = [
        {
          role = "foreign-execution-profile",
          digest_blake3 = "054ec6754f08d60088cf3388f43295ad37621e5115e979f702a2a5c5d0eefcb8",
        },
      ],
      dependencies = [
        {
          role = "corepkgs-shared-graph",
          digest_blake3 = "54ef6a4809fc4151fc000d5be77ab653958837332d507d635f873abd17a3d964",
        },
      ],
      policy_digest_blake3 = "e6ac8f9419a90e75ea7d3318a8cf8fa52be91bc8f8364320d0d676bda3a6d91d",
      expected_outcome = "pass",
      limits = {
        max_sources = 8,
        max_tools = 8,
        max_dependencies = 8,
        max_output_bytes = 1073741824,
        timeout_seconds = 7200,
      },
    },
  ],
  limits = {
    max_shards = 8,
    max_packages = 64,
    max_aliases = 128,
    max_variants = 32,
    max_validation_roots = 32,
    max_artifacts = 256,
  },
}
