let contracts = import "../contracts.ncl" in
let zero = "0000000000000000000000000000000000000000000000000000000000000000" in
let a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
let b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" in
let c = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" in
{
  schema = "mantlepkgs-domain-manifest-v1",
  name = "reviewed",
  shards = [
    {
      schema = "mantlepkgs-domain-shard-v1",
      shard_identity_blake3 = zero,
      name = "core",
      class = "core",
      owner_label = "mantle-core-reviewers",
      source = {
        repository = "https://github.com/ekala-project/corepkgs",
        reference = "github:ekala-project/corepkgs/a9a1af8abbf08b972dbce7bb9c2643c7d76d140d",
        revision = "a9a1af8abbf08b972dbce7bb9c2643c7d76d140d",
        source_digest_blake3 = c,
      },
      source_catalog_identity_blake3 = a,
      packages = [
        {
          package_identity_blake3 = zero,
          name = "hello",
          public_selector = "hello",
          system = "x86_64-linux",
          root_identity_blake3 = a,
          policy_digest_blake3 = b,
          buildable = true,
          aliases = ["hello-world"],
          artifacts = [{ role = "catalog", digest_blake3 = c }],
        },
        {
          package_identity_blake3 = zero,
          name = "hello-check",
          public_selector = "hello-check",
          system = "x86_64-linux",
          root_identity_blake3 = b,
          policy_digest_blake3 = b,
          buildable = true,
          aliases = ["hello-test"],
          artifacts = [{ role = "catalog", digest_blake3 = c }],
        },
      ],
      limits = {
        max_packages = 16,
        max_aliases = 32,
        max_artifacts = 64,
      },
    },
  ],
  variants = [
    {
      variant_identity_blake3 = zero,
      public_selector = "hello-debug",
      system = "x86_64-linux",
      base_selector = "hello",
      base_package_identity_blake3 = zero,
      base_root_identity_blake3 = zero,
      variant_name = "debug",
      changed_policy_digest_blake3 = c,
      root_identity_blake3 = c,
      provenance_digest_blake3 = a,
      aliases = ["hello-with-debug"],
    },
  ],
  validation_roots = [
    {
      validation_root_identity_blake3 = zero,
      name = "hello-check",
      system = "x86_64-linux",
      package_selector = "hello",
      package_identity_blake3 = zero,
      validation_selector = "hello-check",
      validation_package_identity_blake3 = zero,
      sources = [{ role = "test-source", digest_blake3 = a }],
      tools = [{ role = "test-tool", digest_blake3 = b }],
      dependencies = [{ role = "test-dependency", digest_blake3 = c }],
      policy_digest_blake3 = a,
      expected_outcome = "pass",
      limits = {
        max_sources = 8,
        max_tools = 8,
        max_dependencies = 8,
        max_output_bytes = 1048576,
        timeout_seconds = 60,
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
} | contracts.Manifest
