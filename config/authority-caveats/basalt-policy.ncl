# Proposed Mantle-specific policy for Basalt's generic exact-authority core.
# NOT a Basalt closed EnforcementAction/verify_ucan_enforcement_request adapter.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let ExactSchema = std.contract.from_predicate (fun value =>
  std.is_string value && value == "ucan-nickel-contracts.policy.v1"
) in
let ContractPolicy = {
  id | NonEmptyString,
  description | String,
  resource_prefixes | Array NonEmptyString,
  abilities | Array NonEmptyString,
} in
let Policy = {
  schema_version | ExactSchema,
  backends | { _ : { id | NonEmptyString, description | String, supports_higher_order | Bool } },
  contracts | { _ : ContractPolicy },
} in
{
  schema_version = "ucan-nickel-contracts.policy.v1",
  backends = {
    nickel = {
      id = "nickel",
      description = "Nickel-authored exact Mantle authority policy",
      supports_higher_order = false,
    },
  },
  contracts = {
    mantle-remote-build = {
      id = "mantle-remote-build",
      description = "A signed resource and ability for one receiver-checked remote build grant",
      resource_prefixes = ["mantle://remote-build/"],
      abilities = ["remote/build"],
    },
    mantle-store-view = {
      id = "mantle-store-view",
      description = "A signed resource and ability for a receiver-checked logical store view",
      resource_prefixes = ["mantle://store-view/"],
      abilities = ["store/read"],
    },
    mantle-project-goal = {
      id = "mantle-project-goal",
      description = "A signed resource and ability for a receiver-checked project declaration",
      resource_prefixes = ["mantle://project-goal/"],
      abilities = ["project/declare"],
    },
  },
} | Policy
