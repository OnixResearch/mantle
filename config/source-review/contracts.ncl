# Source-review authoring contracts. These validate supplied facts; they neither
# discover keys/currentness nor produce signatures or review decisions.
let Exact = fun expected => std.contract.from_predicate (fun value => value == expected) in
let ExactFields = fun fields => std.contract.from_predicate (fun value =>
  std.is_record value
  && std.array.all (fun field => std.array.elem field fields) (std.record.fields value)
) in
let BoundedText = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0 && std.string.length value <= 256
) in
let Digest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Unsigned = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value
  && value >= 0 && value <= 18446744073709551615
) in
let Threshold = std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value >= 1 && value <= 64
) in
let Rows = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value <= 64
) in
let NonClaims = [
  "source-review-does-not-prove-build-correctness",
  "source-review-does-not-prove-deployment-safety",
  "source-review-does-not-prove-release-eligibility",
  "source-review-does-not-prove-reproducibility",
  "source-review-does-not-prove-review-quality",
  "source-review-does-not-prove-source-correctness",
  "source-review-does-not-prove-witness-independence",
] in
let ArtifactRef = {
  profile | BoundedText,
  algorithm | Exact "blake3",
  digest_hex | Digest,
} in
let ExactRef = ExactFields ["profile", "algorithm", "digest_hex"] in
let Subject = {
  schema | Exact "cairn.source-review-subject.v1",
  source_archive_blake3 | Digest,
  source_revision | BoundedText | optional,
  source_tree_blake3 | Digest | optional,
  snapshot_profile | Exact "mantle-release-source",
  snapshot_profile_version | Exact 1,
  projection | Exact "source",
} in
let ExactSubject = ExactFields [
  "schema", "source_archive_blake3", "source_revision", "source_tree_blake3",
  "snapshot_profile", "snapshot_profile_version", "projection",
] in
let ClaimRoot = {
  claim_identity | Digest,
  subject_root | Digest,
  revision | Unsigned,
} in
let ExactClaimRoot = ExactFields ["claim_identity", "subject_root", "revision"] in
let Currentness = std.contract.from_predicate (fun value =>
  std.array.elem value ["current", "verification_overlap", "superseded", "revoked", "unknown"]
) in
let TrustedKey = {
  producer_id | BoundedText,
  key_id | BoundedText,
  key_identity | ArtifactRef | ExactRef,
  allowed_purposes | Exact ["source-review-approval"],
  generation | Unsigned,
  currentness | Currentness,
  currentness_ref | ArtifactRef | ExactRef,
} in
let KeyFields = [
  "producer_id", "key_id", "key_identity", "allowed_purposes", "generation",
  "currentness", "currentness_ref",
] in
let ExactKeyRows = std.contract.from_predicate (fun keys =>
  std.array.all (fun key =>
    std.array.all (fun field => std.array.elem field KeyFields) (std.record.fields key)
  ) keys
) in
let AuthenticationPolicy = {
  schema | Exact "artifact-auth.policy.v1",
  profile_id | Exact "cairn.source-review.v1",
  threshold | Threshold,
  trusted_keys | Array TrustedKey | Rows | ExactKeyRows,
} in
let ExactAuthentication = ExactFields ["schema", "profile_id", "threshold", "trusted_keys"] in
let VerificationPolicyContract = {
  schema | Exact "cairn.source-review-verification-policy.v1",
  subject | Subject | ExactSubject,
  claim_root | ClaimRoot | ExactClaimRoot,
  review_policy_blake3 | Digest,
  authentication_policy | AuthenticationPolicy | ExactAuthentication,
  verifier_context | ArtifactRef | ExactRef,
  author_key_blake3 | Array Digest | Rows,
  exclude_authors | Bool,
  required_approvals | Threshold,
  non_claims | Exact NonClaims,
} in
let PolicyFields = [
  "schema", "subject", "claim_root", "review_policy_blake3", "authentication_policy",
  "verifier_context", "author_key_blake3", "exclude_authors", "required_approvals", "non_claims",
] in
let ConsistentPolicyContract = std.contract.from_predicate (fun policy =>
  policy.required_approvals == policy.authentication_policy.threshold
  && std.array.all (fun key => key.currentness_ref == policy.verifier_context)
    policy.authentication_policy.trusted_keys
  && std.array.all (fun key =>
    std.array.length (std.array.filter (fun other => other.key_identity == key.key_identity)
      policy.authentication_policy.trusted_keys) == 1
  ) policy.authentication_policy.trusted_keys
) in
let Preset = {
  mode | Exact "required",
  required_approvals | Threshold,
  exclude_authors | Bool,
} in
let ExactPreset = ExactFields ["mode", "required_approvals", "exclude_authors"] in
let MatchPreset = fun preset => std.contract.from_predicate (fun policy =>
  policy.required_approvals == preset.required_approvals
  && policy.exclude_authors == preset.exclude_authors
) in
{
  required_non_claims = NonClaims,
  VerificationPolicy = VerificationPolicyContract,
  ExactVerificationPolicy = ExactFields PolicyFields,
  ConsistentPolicy = ConsistentPolicyContract,
  Defaults = {
    schema | Exact "mantle-source-review-authoring-v1",
    default_mode | Exact "optional",
    source_review_bytes_max | Exact 1048576,
    review_rows_max | Exact 64,
    trusted_keys_max | Exact 64,
    author_keys_max | Exact 64,
    presets | {
      reviewed | Preset | ExactPreset,
      stagex_inspired | Preset | ExactPreset | MatchPreset {
        required_approvals = 2,
        exclude_authors = true,
      },
    } | ExactFields ["reviewed", "stagex_inspired"],
  },
  # Preserve the operator's exact frozen source/key/currentness facts unchanged.
  admit = fun preset frozen =>
    let selected = preset | Preset | ExactPreset in
    frozen | VerificationPolicyContract | ExactFields PolicyFields | ConsistentPolicyContract | MatchPreset selected,
}
