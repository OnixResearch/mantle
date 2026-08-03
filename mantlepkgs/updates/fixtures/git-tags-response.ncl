let Contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-git-tags-response-v1",
  candidates = [
    {
      version = "v1.3.0",
      source_ref = "refs/tags/v1.3.0",
      source_identity_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    },
  ],
} | Contracts.SourceResponse
