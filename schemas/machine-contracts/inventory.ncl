# Machine-readable JSON surface contract inventory.
# Checked by `nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs`.
{
  surfaces = [
    {
      id = "operator-diagnostics.doctor-report",
      owner = "src/operator_diagnostics.rs::PreflightReport",
      schema = "schemas/machine-contracts/doctor-report.schema.json",
      generated_contract = "schemas/machine-contracts/doctor-report.contract.ncl",
      positive_fixtures = [
        "schemas/machine-contracts/fixtures/doctor-report-valid.json",
      ],
      negative_fixtures = [
        "schemas/machine-contracts/fixtures/doctor-report-missing-schema.json",
        "schemas/machine-contracts/fixtures/doctor-report-invalid-status.json",
        "schemas/machine-contracts/fixtures/doctor-report-invalid-version.json",
      ],
      unsupported_schema_behavior = "fail-closed with unsupported-schema diagnostic",
      non_claim = "fixture contract validation does not prove command correctness, build correctness, release reproducibility, deployability, or frontend module correctness",
    },
  ],
}
