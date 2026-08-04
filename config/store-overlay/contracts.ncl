# Typed contracts for Mantle overlay-store composition.

let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in

let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in

let BaseLayerBound = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0 && value <= 32
) in

let RequiredTrue = std.contract.from_predicate (fun value =>
  std.is_bool value && value
) in

let HashAlgorithm = std.contract.from_predicate (fun value =>
  std.is_string value && value == "BLAKE3"
) in

let BaseOrder = std.contract.from_predicate (fun value =>
  std.is_string value && value == "declaration-order"
) in

let BaseCapability = std.contract.from_predicate (fun value =>
  std.is_string value && value == "read-only"
) in

let RejectDecision = std.contract.from_predicate (fun value =>
  std.is_string value && value == "reject"
) in

let BlockDecision = std.contract.from_predicate (fun value =>
  std.is_string value && value == "block"
) in

let TrustVerification = std.contract.from_predicate (fun value =>
  std.is_string value && value == "ed25519-pathinfo"
) in

{
  Limits = {
    max_base_layers | BaseLayerBound,
    max_generation_members | PositiveNumber,
    max_generation_bytes | PositiveNumber,
    max_descriptor_bytes | PositiveNumber,
  },

  Composition = {
    writable_overlay_count | std.contract.from_predicate (fun value => value == 1),
    base_order | BaseOrder,
    base_capability | BaseCapability,
    require_same_prefix | RequiredTrue,
    reject_duplicate_bases | RequiredTrue,
    no_backfill | RequiredTrue,
  },

  Trust = {
    policy_id | NonEmptyString,
    verification | TrustVerification,
    unknown_policy | RejectDecision,
    shadow_failure | BlockDecision,
    skip_failure_classes | Array String,
  },

  Generation = {
    allowed_state_schemas | Array NonEmptyString,
    bind_root_inventory | RequiredTrue,
    bind_pathinfo | RequiredTrue,
    bind_directories | RequiredTrue,
    bind_blobs | RequiredTrue,
    revalidate_before_execution | RequiredTrue,
    revalidate_before_output_admission | RequiredTrue,
  },

  Failures = {
    prefix_mismatch | BlockDecision,
    duplicate_base | BlockDecision,
    writable_base | BlockDecision,
    unknown_trust | BlockDecision,
    corrupt_higher_layer | BlockDecision,
    incomplete_higher_layer | BlockDecision,
    generation_drift | BlockDecision,
    base_to_overlay_reference | BlockDecision,
  },

  Policy = {
    schema | NonEmptyString,
    policy_name | NonEmptyString,
    hash_algorithm | HashAlgorithm,
    limits | Limits,
    composition | Composition,
    trust | Trust,
    generation | Generation,
    failures | Failures,
    non_claims | Array NonEmptyString,
  },
}
