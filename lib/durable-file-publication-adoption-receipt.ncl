# Pure validator for Mantle's durable-file-publication adoption evidence.
let _digest_hex_chars = 64 in
let _git_oid_hex_chars = 40 in
let _corpus_case_count = 17 in
let _collision_attempt_limit = 8 in
let _private_file_mode = 384 in
let _blake3_regex = "^[0-9a-f]{%{std.to_string _digest_hex_chars}}$" in
let _git_oid_regex = "^[0-9a-f]{%{std.to_string _git_oid_hex_chars}}$" in
let _rid_regex = "^rad:z[1-9A-HJ-NP-Za-km-z]+$" in
let _expected = {
  rid = "rad:z3tAR4For7qw8ZirkJzoDw1VNDDLM",
  repository_identity = "rad://z3tAR4For7qw8ZirkJzoDw1VNDDLM",
  https_git = "https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git",
  revision = "951c27f59003cea9bfdb40ed4d89653d50fada1f",
  source_archive_blake3 = "2c1d8b5adc8d7384f48a6f8336165e38c3eb196337ebbd66707e157a64b63210",
  producer_archive_receipt = "3a11ed34c922a32678f4e5e72bd9ea48b3e3d0eba35edf0c821c27f5a7920fe4",
  package = "durable-file-publication",
  package_version = "0.1.0",
  cargo_manifest_blake3 = "2da0308eca14ac453eda8193af08389f2eb531cb47ce2df36175de015f501543",
  cargo_lock_blake3 = "7be2f92120be2edd80b805a6e7162d589fbc7a1929f3e6ee2dedd9ca451f0529",
  flake_blake3 = "b75b50a0cf650d3d30d15148cf2abd2295621e36e1eba8ff7e7bafa183a3f25a",
  flake_lock_blake3 = "9ce3960a702c3aaacf8010cb24a23d5c7646536445144e3cc64137d6cfb76bd3",
  nar_hash = "sha256-fKxZ+3rzWzVuawILnmpiYCCe8PT/z6usopFMbr2KFbI=",
}
in
let _expected_lock_source = "git+%{_expected.https_git}?rev=%{_expected.revision}#%{_expected.revision}" in
let _required_root_fields = [
  "schema_version",
  "receipt_type",
  "status",
  "observed_date",
  "source",
  "cargo",
  "nix",
  "mapping",
  "validation",
  "rollback",
  "authority_boundary",
  "platforms",
  "non_claims",
  "evidence",
]
in
let _required_non_claims = [
  "durable-file-publication-source-correctness",
  "whole-mantle-behavior-correctness",
  "remote-attempt-log-semantic-correctness",
  "manifest-publication-adoption",
  "release-bundle-publication-adoption",
  "android-execution",
  "automatic-fallback",
  "release-readiness",
  "retention-or-deletion-authority",
]
in
let _expected_outcomes = [
  "not-committed",
  "destination-exists",
  "committed-visibility-only-rejected",
  "committed-and-parent-synchronized",
  "committed-durability-unknown",
]
in
let _expected_cleanup = ["not-needed", "removed", "failed"] in
let _has_fields = fun fields value =>
  std.is_record value && fields |> std.array.all (fun field => std.record.has_field field value)
in
let _is_blake3 = fun value => std.is_string value && std.string.is_match _blake3_regex value in
let _is_git_oid = fun value => std.is_string value && std.string.is_match _git_oid_regex value in
let _is_rid = fun value => std.is_string value && std.string.is_match _rid_regex value in
let _error = fun error_code error_message => { code = error_code, message = error_message } in
let _first_error = fun errors =>
  errors
  |> std.array.filter_map (fun error => if error == null then 'None else 'Some error)
  |> match { [] => 'Ok, [error, ..] => 'Error error }
in
let _validate = fun receipt =>
  if !_has_fields _required_root_fields receipt then
    'Error (_error "adoption-shape" "durable publication evidence is incomplete")
  else
    _first_error [
      if receipt.schema_version == 1
      && receipt.receipt_type == "mantle.durable-file-publication-adoption.v1"
      && receipt.status == "accepted"
      && receipt.observed_date == "2026-07-27" then null
      else _error "adoption-header" "adoption identity or status drifted",
      if _is_rid receipt.source.rid
      && receipt.source.rid == _expected.rid
      && receipt.source.repository_identity == _expected.repository_identity
      && receipt.source.https_git == _expected.https_git
      && _is_git_oid receipt.source.reviewed_revision
      && receipt.source.reviewed_revision == _expected.revision
      && receipt.source.source_archive_blake3 == _expected.source_archive_blake3
      && receipt.source.producer_archive_receipt == _expected.producer_archive_receipt
      && receipt.source.package == _expected.package
      && receipt.source.package_version == _expected.package_version
      && receipt.source.license == "MIT OR Apache-2.0"
      && !receipt.source.publishable then null
      else _error "adoption-source" "RID, revision, package, license, or producer evidence drifted",
      if _is_blake3 receipt.cargo.manifest_blake3
      && receipt.cargo.manifest_blake3 == _expected.cargo_manifest_blake3
      && _is_blake3 receipt.cargo.lock_blake3
      && receipt.cargo.lock_blake3 == _expected.cargo_lock_blake3
      && receipt.cargo.lock_source == _expected_lock_source
      && receipt.cargo.package == _expected.package then null
      else _error "adoption-cargo" "Cargo manifest, lock, package, or source drifted",
      if _is_blake3 receipt.nix.flake_blake3
      && receipt.nix.flake_blake3 == _expected.flake_blake3
      && _is_blake3 receipt.nix.lock_blake3
      && receipt.nix.lock_blake3 == _expected.flake_lock_blake3
      && receipt.nix.locked_url == _expected.https_git
      && receipt.nix.original_url == _expected.https_git
      && receipt.nix.locked_rev == _expected.revision
      && receipt.nix.original_rev == _expected.revision
      && receipt.nix.nar_hash == _expected.nar_hash then null
      else _error "adoption-nix" "Nix input, lock, revision, or narHash drifted",
      if receipt.mapping.production_backend == "shared"
      && receipt.mapping.shared_surfaces == ["remote-attempt-segment", "remote-attempt-anchor"]
      && receipt.mapping.consumer_surfaces == ["manifest", "content-equivalence", "retention", "deletion", "receipts", "diagnostics"]
      && receipt.mapping.replacement == "no-replace"
      && receipt.mapping.durability == "required"
      && receipt.mapping.final_mode == _private_file_mode
      && receipt.mapping.collision_attempt_limit == _collision_attempt_limit
      && receipt.mapping.outcomes == _expected_outcomes
      && receipt.mapping.cleanup == _expected_cleanup
      && receipt.mapping.existing_content == "bounded-no-follow-exact-comparison" then null
      else _error "adoption-mapping" "request, outcome, cleanup, or ownership mapping drifted",
      if receipt.validation.baseline_revision == "7875ec1c8b80662f183b76194ae4ef8e3cd52a28"
      && receipt.validation.baseline_remote_attempt_tests == "passed"
      && receipt.validation.shared_corpus_cases == _corpus_case_count
      && receipt.validation.shared_corpus == "passed"
      && receipt.validation.linux_integration == "passed"
      && receipt.validation.formatting == "passed"
      && receipt.validation.focused_clippy == "passed"
      && receipt.validation.tigerstyle == "passed"
      && receipt.validation.source_agreement_positive == "passed"
      && receipt.validation.source_agreement_negative == "passed" then null
      else _error "adoption-validation" "behavioral or source-agreement evidence is incomplete",
      if receipt.rollback.legacy_backend_available
      && receipt.rollback.production_backend == "shared"
      && !receipt.rollback.automatic_fallback
      && receipt.rollback.explicit_code_change_required then null
      else _error "adoption-rollback" "rollback is missing or automatic fallback is enabled",
      if receipt.authority_boundary.product_policy_retained
      && receipt.authority_boundary.manifest_retained
      && receipt.authority_boundary.retention_retained
      && receipt.authority_boundary.deletion_retained
      && !receipt.authority_boundary.shared_release_authority
      && !receipt.authority_boundary.shared_deletion_authority then null
      else _error "adoption-authority" "shared publication widened Mantle or deletion authority",
      if receipt.platforms.executed == ["x86_64-linux"]
      && receipt.platforms.not_executed == ["android", "other-operating-systems"] then null
      else _error "adoption-platforms" "executed and non-executed platforms drifted",
      if _required_non_claims |> std.array.all (fun claim => std.array.elem claim receipt.non_claims) then null
      else _error "adoption-non-claims" "required non-claims are missing",
      if receipt.evidence |> std.array.all (fun path => std.is_string path && !(std.string.is_match "^/" path)) then null
      else _error "adoption-evidence" "evidence paths must remain relative",
      if !(std.record.has_field "credentials" receipt)
      && !(std.record.has_field "fallback_url" receipt)
      && !(std.record.has_field "release_authorized" receipt) then null
      else _error "adoption-overclaim" "receipt contains credentials, fallback, or release authority",
    ]
in
{
  expected = _expected,
  expected_lock_source = _expected_lock_source,
  required_non_claims = _required_non_claims,
  is_blake3 = _is_blake3,
  is_git_oid = _is_git_oid,
  is_rid = _is_rid,
  validate = _validate,
}
