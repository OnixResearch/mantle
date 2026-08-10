{
  schema = "mantlepkgs-version-cohort-v1",
  cohort_identity_blake3 = "",
  system = "x86_64-linux",
  generator_identity = "mantle-version-index-producer-v1",
  index_relative_path = "indexes/x86_64-linux.json",
  observation_method = "package-version-attribute",
  revisions = [
    {
      published_order = 1,
      reference = "github:NixOS/nixpkgs/1111111111111111111111111111111111111111",
      revision = "1111111111111111111111111111111111111111",
      nar_hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
    },
    {
      published_order = 2,
      reference = "github:NixOS/nixpkgs/2222222222222222222222222222222222222222",
      revision = "2222222222222222222222222222222222222222",
      nar_hash = "sha256-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=",
    },
  ],
  attributes = ["hello"],
  limits = {
    max_revisions = 8,
    max_attributes = 8,
    max_observations = 64,
    max_requests = 16,
    max_groups = 8,
    max_diagnostics = 64,
    max_aliases_per_request = 8,
    max_text_bytes = 256,
  },
}
