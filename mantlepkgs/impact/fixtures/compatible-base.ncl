let Contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-impact-snapshot-v1",
  snapshot_identity_blake3 = "",
  comparison_policy_identity_blake3 = "",
  catalog_schema = "mantlepkgs-domain-catalog-v1",
  catalog_identity_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  system = "x86_64-linux",
  store_prefix = "/mantle/store",
  conversion_policy_identity_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  package_record_schema = "mantlepkgs-impact-package-record-v1",
  observation_schema = "mantlepkgs-impact-observation-v1",
  identity_domain = "mantle.mantlepkgs.impact-record.v1",
  packages = [],
  observations = [],
  closures = [],
} | Contracts.Snapshot
