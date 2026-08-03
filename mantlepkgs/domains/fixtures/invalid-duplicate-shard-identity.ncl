let contracts = import "../contracts.ncl" in
let zero = "0000000000000000000000000000000000000000000000000000000000000000" in
let shard = {
  schema = "mantlepkgs-domain-shard-v1",
  shard_identity_blake3 = zero,
  name = "core",
  class = "core",
  owner_label = "mantle-core-reviewers",
  source = {
    repository = "https://github.com/ekala-project/corepkgs",
    reference = "github:ekala-project/corepkgs/a9a1af8abbf08b972dbce7bb9c2643c7d76d140d",
    revision = "a9a1af8abbf08b972dbce7bb9c2643c7d76d140d",
    source_digest_blake3 = zero,
  },
  source_catalog_identity_blake3 = zero,
  packages = [],
  limits = {
    max_packages = 1,
    max_aliases = 1,
    max_artifacts = 1,
  },
} in
{
  schema = "mantlepkgs-domain-manifest-v1",
  name = "duplicate-shard",
  shards = [shard, shard],
  variants = [],
  validation_roots = [],
  limits = {
    max_shards = 2,
    max_packages = 1,
    max_aliases = 1,
    max_variants = 1,
    max_validation_roots = 1,
    max_artifacts = 1,
  },
} | contracts.Manifest
