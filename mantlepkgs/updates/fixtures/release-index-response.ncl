let Contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-release-index-response-v1",
  candidates = [
    {
      version = "v1.3.0",
      source_ref = "https://downloads.example.invalid/hello-1.3.0.tar.gz",
      source_identity_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    },
  ],
} | Contracts.SourceResponse
