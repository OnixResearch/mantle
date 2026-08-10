let NixDerivationAdmission = {
  schema | String,
  package = {
    name | String,
    version | String,
    checksum_sha256 | String,
    license | String,
  },
  upstream = {
    repository | String,
    commit | String,
  },
  compatibility = {
    supported_nix_versions | Array String,
    store_prefix | String,
    adapter_path | String,
    native_hash_algorithm | String,
    mantle_evidence_hash_algorithm | String,
  },
  rollback = {
    prior_mantle_revision | String,
    remove_dependency | String,
    restore_adapter | String,
  },
} in

{
  schema = "mantle-nix-derivation-admission-v1",
  package = {
    name = "nix-derivation",
    version = "0.1.0",
    checksum_sha256 = "a5d03dfde06a8ce7e0e007f4795ab74c546e5c7d6c6375a08ceb20677ec8a074",
    license = "Apache-2.0",
  },
  upstream = {
    repository = "https://github.com/cachix/nix-derivation",
    commit = "2cfc0f90ed83ea3cc983e5c305f89494a6df073e",
  },
  compatibility = {
    supported_nix_versions = ["2.34"],
    store_prefix = "/nix/store",
    adapter_path = "src/nix_derivation_adapter.rs",
    native_hash_algorithm = "sha256",
    mantle_evidence_hash_algorithm = "blake3",
  },
  rollback = {
    prior_mantle_revision = "465e4b45abd7ff8887ada01df93c9a45d30faf06",
    remove_dependency = "remove nix-derivation 0.1.0 from Cargo.toml and regenerate Cargo.lock",
    restore_adapter = "restore the pre-adoption Nix ATerm call sites from revision 465e4b45abd7ff8887ada01df93c9a45d30faf06",
  },
} | NixDerivationAdmission
