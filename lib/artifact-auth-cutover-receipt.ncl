# Pure deterministic validator for Mantle's artifact-auth source cutover.
let _digest_hex_chars = 64 in
let _git_oid_hex_chars = 40 in
let _package_count = 2 in
let _blake3_regex = "^[0-9a-f]{%{std.to_string _digest_hex_chars}}$" in
let _git_oid_regex = "^[0-9a-f]{%{std.to_string _git_oid_hex_chars}}$" in
let _rid_regex = "^rad:z[1-9A-HJ-NP-Za-km-z]+$" in
let _expected = {
  rid = "rad:z4JGYYW7WsesXUq7MXVdx16Fawu2f",
  https_git = "https://git.onix.computer/z4JGYYW7WsesXUq7MXVdx16Fawu2f.git",
  commit = "799459346d5416fbd7b9f55840a7371441b55afa",
  source_archive_blake3 = "246a7cad91e7e8a158e22da21f3bff3e61aa0431a58936b5a739178bc62064c7",
  publication_revision = "e41340bec587b6d049b5cc518ec7db925dde84be",
  publication_blake3 = "e58a3de4d6b3b32a547c3cfe5c3e829292cda73891c7776f214f5d4edce10b1c",
  core_manifest_blake3 = "496123770c3e21dea731262771046424fc3b534f7279004d3ca52b05bc8a7c79",
  shell_manifest_blake3 = "9c72bc4929f9f836c0b767d6b604b5e6d3fb97e4f1b61505dbc28238b107b6ce",
  cargo_lock_blake3 = "19daded89bc2d1e15bb2f4d44b827a9f2a7b3be8909e0bbd7d06efeb0fc6d535",
  flake_blake3 = "495ef4816453fc6ab58617dde209735e55953485c8c4c45d600d42c14b0560b3",
  flake_lock_blake3 = "44015a40ba7858db1fa8df4f2bcca11499b04b90be1f8eb8ae3c353009203274",
  nar_hash = "sha256-nEgz2FtVuDesX95yyxidp0vhjxL4INB6Ve8rkpLyJk0=",
  packages = ["artifact-auth-core", "artifact-auth-ed25519"],
}
in
let _expected_lock_source = "git+%{_expected.https_git}?rev=%{_expected.commit}#%{_expected.commit}" in
let _required_root_fields = [
  "schema_version",
  "receipt_type",
  "status",
  "observed_date",
  "source",
  "publication",
  "cargo",
  "nix",
  "validation",
  "fallback",
  "rollback",
  "authority_boundary",
  "non_claims",
  "evidence",
]
in
let _required_non_claims = [
  "artifact-auth-source-correctness",
  "whole-mantle-behavior-correctness",
  "forge-availability",
  "automatic-fallback",
  "standalone-authority-admission",
  "whole-stack-github-independence",
  "release-readiness",
]
in
let _has_fields = fun fields value =>
  std.is_record value && fields |> std.array.all (fun field => std.record.has_field field value)
in
let _is_blake3 = fun value => std.is_string value && std.string.is_match _blake3_regex value in
let _is_git_oid = fun value => std.is_string value && std.string.is_match _git_oid_regex value in
let _is_rid = fun value => std.is_string value && std.string.is_match _rid_regex value in
let _error = fun error_code error_message =>
  {
    code = error_code,
    message = error_message,
  }
in
let _first_error = fun errors =>
  errors
  |> std.array.filter_map (fun error => if error == null then 'None else 'Some error)
  |> match { [] => 'Ok, [error, ..] => 'Error error }
in
let _validate = fun receipt =>
  if !_has_fields _required_root_fields receipt then
    'Error (_error "cutover-shape" "cutover evidence is incomplete")
  else
    _first_error [
      if receipt.schema_version == 1
      && receipt.receipt_type == "mantle.artifact-auth-radicle-cutover.v1"
      && receipt.status == "accepted"
      && receipt.observed_date == "2026-07-25" then
        null
      else
        _error "cutover-header" "cutover identity or status drifted",
      if _has_fields ["rid", "https_git", "reviewed_commit", "source_archive_blake3"] receipt.source
      && _is_rid receipt.source.rid
      && receipt.source.rid == _expected.rid
      && receipt.source.https_git == _expected.https_git
      && _is_git_oid receipt.source.reviewed_commit
      && receipt.source.reviewed_commit == _expected.commit
      && _is_blake3 receipt.source.source_archive_blake3
      && receipt.source.source_archive_blake3 == _expected.source_archive_blake3 then
        null
      else
        _error "cutover-source" "RID, HTTPS URL, commit, or source archive identity drifted",
      if receipt.publication.revision == _expected.publication_revision
      && receipt.publication.receipt_type == "artifact-auth.radicle-publication.v1"
      && _is_blake3 receipt.publication.receipt_blake3
      && receipt.publication.receipt_blake3 == _expected.publication_blake3
      && receipt.publication.status == "accepted" then
        null
      else
        _error "cutover-publication" "accepted publication linkage drifted",
      if _is_blake3 receipt.cargo.core_manifest_blake3
      && receipt.cargo.core_manifest_blake3 == _expected.core_manifest_blake3
      && _is_blake3 receipt.cargo.shell_manifest_blake3
      && receipt.cargo.shell_manifest_blake3 == _expected.shell_manifest_blake3
      && _is_blake3 receipt.cargo.lock_blake3
      && receipt.cargo.lock_blake3 == _expected.cargo_lock_blake3
      && receipt.cargo.lock_source == _expected_lock_source
      && std.array.length receipt.cargo.packages == _package_count
      && receipt.cargo.packages == _expected.packages
      && receipt.cargo.package_version == "0.1.0"
      && !receipt.cargo.crate_graph_changed then
        null
      else
        _error "cutover-cargo" "Cargo source, packages, lock identity, or graph drifted",
      if _is_blake3 receipt.nix.flake_blake3
      && receipt.nix.flake_blake3 == _expected.flake_blake3
      && _is_blake3 receipt.nix.lock_blake3
      && receipt.nix.lock_blake3 == _expected.flake_lock_blake3
      && receipt.nix.locked_url == _expected.https_git
      && receipt.nix.original_url == _expected.https_git
      && receipt.nix.locked_rev == _expected.commit
      && receipt.nix.original_rev == _expected.commit
      && receipt.nix.nar_hash == _expected.nar_hash
      && receipt.nix.nar_hash_preserved then
        null
      else
        _error "cutover-nix" "Nix source, revision, narHash, or lock identity drifted",
      if receipt.validation.baseline_revision == "ece3be0ab1ecfa0e0375b3f08ecceaaee37a643d"
      && receipt.validation.baseline_core == "passed-5"
      && receipt.validation.baseline_shell == "passed-14"
      && receipt.validation.post_cutover_core == "passed-5"
      && receipt.validation.post_cutover_shell == "passed-14"
      && receipt.validation.formatting == "passed"
      && receipt.validation.focused_clippy == "passed"
      && receipt.validation.source_agreement_positive == "passed"
      && receipt.validation.source_agreement_negative == "passed"
      && !receipt.validation.rust_implementation_changed then
        null
      else
        _error "cutover-validation" "behavioral or source-agreement evidence is incomplete",
      if !receipt.fallback.manifest_github_source
      && !receipt.fallback.cargo_lock_github_source
      && !receipt.fallback.nix_input_github_source
      && !receipt.fallback.nix_lock_github_source
      && !receipt.fallback.automatic_fallback
      && receipt.fallback.outage_result == "visible-resolution-failure" then
        null
      else
        _error "cutover-fallback" "an executable GitHub or automatic fallback remains",
      if !receipt.rollback.automatic
      && receipt.rollback.requires_owning_tool_lock_regeneration
      && receipt.rollback.requires_source_identity_review then
        null
      else
        _error "cutover-rollback" "rollback bypasses lock regeneration or identity review",
      if receipt.authority_boundary.legacy_authoritative
      && !receipt.authority_boundary.standalone_authority_admitted
      && receipt.authority_boundary.product_authority_retained then
        null
      else
        _error "cutover-authority" "transport cutover widened standalone or product authority",
      if _required_non_claims |> std.array.all (fun claim => std.array.elem claim receipt.non_claims) then
        null
      else
        _error "cutover-non-claims" "required non-claims are missing",
      if receipt.evidence |> std.array.all (fun path => std.is_string path && !(std.string.is_match "^/" path)) then
        null
      else
        _error "cutover-evidence" "evidence paths must remain relative",
      if !(std.record.has_field "claims" receipt)
      && !(std.record.has_field "credentials" receipt)
      && !(std.record.has_field "fallback_url" receipt) then
        null
      else
        _error "cutover-overclaim" "receipt contains authority, credentials, or fallback",
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
