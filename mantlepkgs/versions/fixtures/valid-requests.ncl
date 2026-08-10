let contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-version-request-set-v1",
  request_set_identity_blake3 = "",
  policy_identity_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  requests = [
    {
      request_identity_blake3 = "",
      system = "x86_64-linux",
      attribute = "hello",
      reported_version = "2.12.1",
      public_selector = "hello@2.12.1",
      aliases = ["hello-2.12.1"],
      unversioned_default = false,
    },
  ],
} | contracts.RequestSet
