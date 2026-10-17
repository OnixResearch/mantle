# Positive and negative Mantle durable publication adoption evidence tests.
let adoption = import "../lib/durable-file-publication-adoption-receipt.ncl" in
let receipt = import "../evidence/radicle/durable-file-publication-adoption-v1.ncl" in
let digest_a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
let missing_revision = "1111111111111111111111111111111111111111" in
let unknown_rid = "rad:z3gqcJUoA1n9HaHKufZs5FCSGazv5" in
let github_source = "ssh://git@github.com/OnixResearch/durable-file-publication.git" in
let result = fun candidate => adoption.validate candidate in
let accepted = fun candidate => result candidate |> match { 'Ok => true, 'Error _ => false } in
let rejected_as = fun code candidate =>
  result candidate |> match { 'Ok => false, 'Error error => error.code == code }
in
{
  tests = [
    accepted receipt,
    adoption.is_blake3 digest_a,
    adoption.is_git_oid receipt.source.reviewed_revision,
    adoption.is_rid receipt.source.rid,
    rejected_as "adoption-shape" {},
    rejected_as "adoption-header" (receipt & { status | force = "pending" }),
    rejected_as "adoption-source" (receipt & { source.rid | force = unknown_rid }),
    rejected_as "adoption-source" (receipt & { source.https_git | force = github_source }),
    rejected_as "adoption-source" (receipt & { source.reviewed_revision | force = missing_revision }),
    rejected_as "adoption-source" (receipt & { source.source_archive_blake3 | force = digest_a }),
    rejected_as "adoption-source" (receipt & { source.publishable | force = true }),
    rejected_as "adoption-cargo" (receipt & { cargo.manifest_blake3 | force = digest_a }),
    rejected_as "adoption-cargo" (receipt & { cargo.lock_blake3 | force = digest_a }),
    rejected_as "adoption-cargo" (receipt & { cargo.lock_source | force = "git+%{github_source}" }),
    rejected_as "adoption-nix" (receipt & { nix.locked_url | force = github_source }),
    rejected_as "adoption-nix" (receipt & { nix.locked_rev | force = missing_revision }),
    rejected_as "adoption-nix" (receipt & { nix.nar_hash | force = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" }),
    rejected_as "adoption-nix" (receipt & { nix.flake_blake3 | force = digest_a }),
    rejected_as "adoption-nix" (receipt & { nix.lock_blake3 | force = digest_a }),
    rejected_as "adoption-mapping" (receipt & { mapping.production_backend | force = "legacy" }),
    rejected_as "adoption-mapping" (receipt & { mapping.shared_surfaces | force = ["manifest"] }),
    rejected_as "adoption-mapping" (receipt & { mapping.replacement | force = "replace" }),
    rejected_as "adoption-mapping" (receipt & { mapping.durability | force = "visibility-only" }),
    rejected_as "adoption-mapping" (receipt & { mapping.outcomes | force = [] }),
    rejected_as "adoption-mapping" (receipt & { mapping.cleanup | force = ["removed"] }),
    rejected_as "adoption-validation" (receipt & { validation.shared_corpus_cases | force = 16 }),
    rejected_as "adoption-validation" (receipt & { validation.linux_integration | force = "failed" }),
    rejected_as "adoption-rollback" (receipt & { rollback.legacy_backend_available | force = false }),
    rejected_as "adoption-rollback" (receipt & { rollback.automatic_fallback | force = true }),
    rejected_as "adoption-authority" (receipt & { authority_boundary.manifest_retained | force = false }),
    rejected_as "adoption-authority" (receipt & { authority_boundary.shared_deletion_authority | force = true }),
    rejected_as "adoption-platforms" (receipt & { platforms.executed | force = ["android"] }),
    rejected_as "adoption-non-claims" (receipt & { non_claims | force = [] }),
    rejected_as "adoption-evidence" (receipt & { evidence | force = ["/tmp/evidence.json"] }),
    rejected_as "adoption-overclaim" (receipt & { fallback_url = github_source }),
    rejected_as "adoption-overclaim" (receipt & { release_authorized = true }),
    !adoption.is_blake3 "sha256:%{digest_a}",
    !adoption.is_git_oid digest_a,
    !adoption.is_rid "rad:../private",
  ] |> std.test.assert_all,
}
