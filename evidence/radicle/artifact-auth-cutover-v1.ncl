# Typed evidence for Mantle's artifact-auth transport cutover.
let CutoverReceipt = {
  schema_version | Number,
  receipt_type | String,
  status | String,
  observed_date | String,
  source
    | {
      rid | String,
      https_git | String,
      reviewed_commit | String,
      source_archive_blake3 | String,
    },
  publication
    | {
      revision | String,
      receipt_type | String,
      receipt_blake3 | String,
      status | String,
    },
  cargo
    | {
      core_manifest_blake3 | String,
      shell_manifest_blake3 | String,
      lock_blake3 | String,
      lock_source | String,
      packages | Array String,
      package_version | String,
      crate_graph_changed | Bool,
    },
  nix
    | {
      flake_blake3 | String,
      lock_blake3 | String,
      locked_url | String,
      original_url | String,
      locked_rev | String,
      original_rev | String,
      nar_hash | String,
      nar_hash_preserved | Bool,
    },
  validation
    | {
      baseline_revision | String,
      baseline_core | String,
      baseline_shell | String,
      post_cutover_core | String,
      post_cutover_shell | String,
      formatting | String,
      focused_clippy | String,
      source_agreement_positive | String,
      source_agreement_negative | String,
      rust_implementation_changed | Bool,
    },
  fallback
    | {
      manifest_github_source | Bool,
      cargo_lock_github_source | Bool,
      nix_input_github_source | Bool,
      nix_lock_github_source | Bool,
      automatic_fallback | Bool,
      outage_result | String,
    },
  rollback
    | {
      automatic | Bool,
      requires_owning_tool_lock_regeneration | Bool,
      requires_source_identity_review | Bool,
    },
  authority_boundary
    | {
      legacy_authoritative | Bool,
      standalone_authority_admitted | Bool,
      product_authority_retained | Bool,
    },
  non_claims | Array String,
  evidence | Array String,
}
in
let expected_reviewed_commit = "799459346d5416fbd7b9f55840a7371441b55afa" in
let expected_https_git = "https://git.onix.computer/z4JGYYW7WsesXUq7MXVdx16Fawu2f.git" in
let expected_lock_source = "git+%{expected_https_git}?rev=%{expected_reviewed_commit}#%{expected_reviewed_commit}" in
let receipt | CutoverReceipt = {
  schema_version = 1,
  receipt_type = "mantle.artifact-auth-radicle-cutover.v1",
  status = "accepted",
  observed_date = "2026-07-25",
  source = {
    rid = "rad:z4JGYYW7WsesXUq7MXVdx16Fawu2f",
    https_git = expected_https_git,
    reviewed_commit = expected_reviewed_commit,
    source_archive_blake3 = "246a7cad91e7e8a158e22da21f3bff3e61aa0431a58936b5a739178bc62064c7",
  },
  publication = {
    revision = "e41340bec587b6d049b5cc518ec7db925dde84be",
    receipt_type = "artifact-auth.radicle-publication.v1",
    receipt_blake3 = "e58a3de4d6b3b32a547c3cfe5c3e829292cda73891c7776f214f5d4edce10b1c",
    status = "accepted",
  },
  cargo = {
    core_manifest_blake3 = "496123770c3e21dea731262771046424fc3b534f7279004d3ca52b05bc8a7c79",
    shell_manifest_blake3 = "9c72bc4929f9f836c0b767d6b604b5e6d3fb97e4f1b61505dbc28238b107b6ce",
    lock_blake3 = "19daded89bc2d1e15bb2f4d44b827a9f2a7b3be8909e0bbd7d06efeb0fc6d535",
    lock_source = expected_lock_source,
    packages = ["artifact-auth-core", "artifact-auth-ed25519"],
    package_version = "0.1.0",
    crate_graph_changed = false,
  },
  nix = {
    flake_blake3 = "495ef4816453fc6ab58617dde209735e55953485c8c4c45d600d42c14b0560b3",
    lock_blake3 = "44015a40ba7858db1fa8df4f2bcca11499b04b90be1f8eb8ae3c353009203274",
    locked_url = expected_https_git,
    original_url = expected_https_git,
    locked_rev = expected_reviewed_commit,
    original_rev = expected_reviewed_commit,
    nar_hash = "sha256-nEgz2FtVuDesX95yyxidp0vhjxL4INB6Ve8rkpLyJk0=",
    nar_hash_preserved = true,
  },
  validation = {
    baseline_revision = "ece3be0ab1ecfa0e0375b3f08ecceaaee37a643d",
    baseline_core = "passed-5",
    baseline_shell = "passed-14",
    post_cutover_core = "passed-5",
    post_cutover_shell = "passed-14",
    formatting = "passed",
    focused_clippy = "passed",
    source_agreement_positive = "passed",
    source_agreement_negative = "passed",
    rust_implementation_changed = false,
  },
  fallback = {
    manifest_github_source = false,
    cargo_lock_github_source = false,
    nix_input_github_source = false,
    nix_lock_github_source = false,
    automatic_fallback = false,
    outage_result = "visible-resolution-failure",
  },
  rollback = {
    automatic = false,
    requires_owning_tool_lock_regeneration = true,
    requires_source_identity_review = true,
  },
  authority_boundary = {
    legacy_authoritative = true,
    standalone_authority_admitted = false,
    product_authority_retained = true,
  },
  non_claims = [
    "artifact-auth-source-correctness",
    "whole-mantle-behavior-correctness",
    "forge-availability",
    "automatic-fallback",
    "standalone-authority-admission",
    "whole-stack-github-independence",
    "release-readiness",
  ],
  evidence = [
    "cairn/changes/consume-artifact-auth-from-radicle/evidence/source-cutover-2026-07-25.md",
    "evidence/radicle/artifact-auth-cutover-v1.json",
    "artifact-auth:evidence/radicle/publication-v1.json",
    "onix-core:evidence/radicle/source-admission-v1.json",
    "valence:evidence/radicle/artifact-auth-cutover-v1.json",
  ],
}
in
receipt
