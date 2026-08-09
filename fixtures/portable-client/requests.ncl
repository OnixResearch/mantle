let Request = {
  client_platform | String,
  target_platform | String,
  store_prefix | String,
  remote_capability_id | Dyn,
  trusted_builder_key_count | Number,
  payload_kinds | Array String,
  expected | [| 'admitted, 'blocked |],
  blocker | Dyn,
} in
{
  valid = {
    client_platform = "aarch64-darwin",
    target_platform = "x86_64-linux",
    store_prefix = "/mantle/store",
    remote_capability_id = "builder-v1",
    trusted_builder_key_count = 1,
    payload_kinds = ["derivation-facts", "policy-identity", "source-object-reference", "upload-plan"],
    expected = 'admitted,
    blocker = null,
  } | Request,
  malformed = {
    client_platform = "x86_64-darwin",
    target_platform = "aarch64-linux",
    store_prefix = "/portable/store",
    remote_capability_id = "builder-v1",
    trusted_builder_key_count = 1,
    payload_kinds = ["nickel-source"],
    expected = 'blocked,
    blocker = "portable-raw-frontend-payload-rejected",
  } | Request,
}
