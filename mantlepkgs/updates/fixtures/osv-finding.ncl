let Contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-osv-response-v1",
  package_coordinate = "pkg:generic/hello",
  version = "v1.3.0",
  findings = [
    {
      finding_id = "OSV-EXAMPLE",
      finding_identity_blake3 = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    },
  ],
} | Contracts.AdvisoryResponse
