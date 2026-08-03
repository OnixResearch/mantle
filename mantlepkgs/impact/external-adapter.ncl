# Pure example adapter. It has no credentials, webhook input, or network effect.
let Contracts = import "contracts.ncl" in
fun report =>
  let checked = report | Contracts.Report in
  {
    schema = "mantle-package-impact-external-summary-v1",
    source_report_identity_blake3 = checked.report_identity_blake3,
    system = checked.system,
    package_impact_count = std.array.length checked.package_impacts,
    requested_network_effects = [],
    forge_credentials = null,
  }
