let Exact = fun expected => std.contract.from_predicate (fun value =>
  std.is_string value && value == expected
) in

let VendorPackageCount = 4 in

let Cohort = {
  schema | Exact "mantle-nickel-cohort-v1",
  source_repository | Exact "https://github.com/nickel-lang/nickel",
  source_revision | Exact "1320a983e6c3d1e2fb53dd2464b084b4903b1426",
  cli_version | Exact "1.17.0",
  cli_rust_requirement | Exact "1.89",
  embedded_facade_version | Exact "2.2.0",
  embedded_core_version | Exact "0.18.0",
  vendor_method | Exact "cargo-vendor-locked-versioned-dirs",
  vendor_manifest_blake3 | Exact "fd16be804f063d574a1aff7d8ea5adcd21f1ef02cacc9b4be6c9800054d3ab33",
  vendor_package_count | Number,
  fixed_point_status | Exact "not-rerun-for-this-source-cohort",
  non_claims | Array String,
} in

{
  schema = "mantle-nickel-cohort-v1",
  source_repository = "https://github.com/nickel-lang/nickel",
  source_revision = "1320a983e6c3d1e2fb53dd2464b084b4903b1426",
  cli_version = "1.17.0",
  cli_rust_requirement = "1.89",
  embedded_facade_version = "2.2.0",
  embedded_core_version = "0.18.0",
  vendor_method = "cargo-vendor-locked-versioned-dirs",
  vendor_manifest_blake3 = "fd16be804f063d574a1aff7d8ea5adcd21f1ef02cacc9b4be6c9800054d3ab33",
  vendor_package_count = VendorPackageCount,
  fixed_point_status = "not-rerun-for-this-source-cohort",
  non_claims = [
    "cohort agreement does not prove evaluator correctness",
    "cohort agreement does not prove derivation or build correctness",
    "cohort agreement does not prove compiler correctness",
    "historical fixed-point evidence does not prove this changed source cohort",
    "cohort agreement does not prove release eligibility",
  ],
} | Cohort
