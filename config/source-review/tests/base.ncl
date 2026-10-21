# TEST ONLY synthetic identity facts; no signatures or review approval.
let FixtureContext = {
  profile = "operator-source-review-currentness.v1",
  algorithm = "blake3",
  digest_hex = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
} in
{
  context = FixtureContext,
  other_context = FixtureContext & { digest_hex | force = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" },
  key = fun digest purposes currentness_evidence => {
    producer_id = "test-reviewer",
    key_id = "same-label-is-not-key-identity",
    key_identity = { profile = "artifact-auth.ed25519-public-key.v1", algorithm = "blake3", digest_hex = digest },
    allowed_purposes = purposes,
    generation = 1,
    currentness = "current",
    currentness_ref = currentness_evidence,
  },
  policy = fun approvals minimum_signers reject_authors keys => {
    schema = "cairn.source-review-verification-policy.v1",
    subject = {
      schema = "cairn.source-review-subject.v1",
      source_archive_blake3 = "1111111111111111111111111111111111111111111111111111111111111111",
      snapshot_profile = "mantle-release-source",
      snapshot_profile_version = 1,
      projection = "source",
    },
    claim_root = {
      claim_identity = "2222222222222222222222222222222222222222222222222222222222222222",
      subject_root = "3333333333333333333333333333333333333333333333333333333333333333",
      revision = 1,
    },
    review_policy_blake3 = "4444444444444444444444444444444444444444444444444444444444444444",
    authentication_policy = {
      schema = "artifact-auth.policy.v1",
      profile_id = "cairn.source-review.v1",
      threshold = minimum_signers,
      trusted_keys = keys,
    },
    verifier_context = FixtureContext,
    author_key_blake3 = [],
    exclude_authors = reject_authors,
    required_approvals = approvals,
    non_claims = (import "../contracts.ncl").required_non_claims,
  },
}
