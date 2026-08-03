let Contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-repology-response-v1",
  package_coordinate = "hello",
  version = "v1.3.0",
  findings = [],
} | Contracts.AdvisoryResponse
