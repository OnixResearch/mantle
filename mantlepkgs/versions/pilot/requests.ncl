let Contracts = import "../contracts.ncl" in
{
  schema = "mantlepkgs-version-request-set-v1",
  request_set_identity_blake3 = "",
  # The resolve shell binds this placeholder to the sealed policy.
  policy_identity_blake3 = "",
  requests = [
    {
      request_identity_blake3 = "",
      system = "x86_64-linux",
      attribute = "hello",
      reported_version = "2.12.2",
      public_selector = "hello@2.12.2",
      aliases = ["hello-2.12.2"],
      unversioned_default = false,
    },
    {
      request_identity_blake3 = "",
      system = "x86_64-linux",
      attribute = "hello",
      reported_version = "2.12.3",
      public_selector = "hello@2.12.3",
      aliases = ["hello", "hello-2.12.3"],
      unversioned_default = true,
    },
  ],
} | Contracts.RequestSet
