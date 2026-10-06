# Reviewable bounds for UCAN-carried Mantle filter caveats.
let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let SupportedKind = std.contract.from_predicate (fun value =>
  std.is_string value &&
  (value == "rewrite" || value == "reject" || value == "alternatives" || value == "unknown-deny-all")
) in
let RejectDisposition = std.contract.from_predicate (fun value =>
  std.is_string value && value == "silent-discard"
) in
{
  Policy = {
    schema | NonEmptyString,
    caveat_domain | NonEmptyString,
    caveat_type | NonEmptyString,
    accepted_kinds | Array SupportedKind,
    limits | {
      max_chain | PositiveNumber,
      max_alternatives | PositiveNumber,
      max_pattern_bytes | PositiveNumber,
      max_caveat_bytes | PositiveNumber,
      max_value_bytes | PositiveNumber,
      max_segments | PositiveNumber,
      max_declared_jobs | PositiveNumber,
    },
    order | NonEmptyString,
    rejection | RejectDisposition,
    recheck_original_and_parent_prefixes | Bool,
    non_claims | Array NonEmptyString,
  },
}
