let contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-domain-manifest-v1",
  name = "invalid-limit",
  shards = [],
  variants = [],
  validation_roots = [],
  limits = {
    max_shards = 0,
    max_packages = 1,
    max_aliases = 1,
    max_variants = 1,
    max_validation_roots = 1,
    max_artifacts = 1,
  },
} | contracts.Manifest
