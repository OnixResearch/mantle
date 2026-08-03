let Contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-osv-response-v1",
  package_coordinate = "pkg:generic/hello",
  version = "v1.3.0",
  findings = [],
} | Contracts.AdvisoryResponse
