# Review-time caveat vocabulary. Runtime code owns bounds and rejects drift.
let Positive = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let AcceptedVariant = std.contract.from_predicate (fun value =>
  std.is_string value &&
  (value == "rewrite" || value == "reject" || value == "alternatives" || value == "unknown-deny-all")
) in
{
  Policy = {
    schema | String,
    accepted_variants | Array AcceptedVariant,
    limits | {
      max_chain | Positive,
      max_alternatives | Positive,
      max_pattern_bytes | Positive,
      max_caveat_bytes | Positive,
      max_value_bytes | Positive,
      max_segments | Positive,
    },
    order | String,
    unknown_behavior | String,
    rejected_message_behavior | String,
    original_request_policy_recheck | Bool,
    parent_prefix_policy_recheck | Bool,
    non_claims | Array String,
  },
}
