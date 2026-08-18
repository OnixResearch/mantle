# Focused validation for independent source witness replay

Date: 2026-06-29

Environment note: root-package tests used repo-local `TMPDIR=$PWD/.tmp-independent-source/tmp` and `CARGO_TARGET_DIR=$PWD/.tmp-independent-source/target` because `/tmp` was full on this host during validation.

## Bounded claim

This slice proves external source archive acquisition for witness rebuilds when release evidence declares an external `file://`, `http://`, or `https://` archive URL and the witness uses `mantle release witness-rebuild --require-independent-source`.

It does not prove raw Git tag reconstruction, upstream repository discovery, tag-signature verification, or source archive generation from Git object graphs. Witnesses still verify the fetched archive bytes against the release manifest's BLAKE3 source archive digest before extraction and before workflow launch.

## cargo test -p crunch-release-core manifest::tests::

Command (pueue task 216):

~~~text
PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$HOME/.cargo/bin:/run/current-system/sw/bin:/usr/bin:/bin" PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig" SNIX_BUILD_SANDBOX_SHELL="/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox" CARGO_TARGET_DIR=/tmp/mantle-independent-source-target cargo test -p crunch-release-core manifest::tests::
~~~

Output:

~~~text
test manifest::tests::validate_rejects_absolute_member_path ... ok
test manifest::tests::validate_accepts_matching_external_source_acquisition ... ok
test manifest::tests::validate_accepts_provider_fixed_point_proof_artifact ... ok
test manifest::tests::validate_rejects_deterministic_proof_duplicate_path ... ok
test manifest::tests::validate_rejects_deterministic_proof_with_wrong_path ... ok
test manifest::tests::validate_rejects_deterministic_proof_with_wrong_role ... ok
test manifest::tests::validate_rejects_prerequisite_inventory_linkage_mismatch ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_file_kind ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_with_wrong_role ... ok
test manifest::tests::validate_rejects_source_acquisition_digest_mismatch ... ok
test manifest::tests::validate_rejects_stage2_digest_not_present_in_binaries ... ok
test manifest::tests::validate_rejects_unsupported_source_acquisition_url ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.00s
~~~

Exit status: 0

## cargo test -p mantle --bin mantle release_evidence::tests::

Command (pueue task 335):

~~~text
PATH="$HOME/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$HOME/.cargo/bin:/run/current-system/sw/bin:/usr/bin:/bin" TMPDIR="$PWD/.tmp-independent-source/tmp" CARGO_TARGET_DIR="$PWD/.tmp-independent-source/target" RUSTC_BOOTSTRAP=1 PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig" SNIX_BUILD_SANDBOX_SHELL="/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox" cargo test -p mantle --bin mantle release_evidence::tests::
~~~

Output:

~~~text
test release_evidence::tests::validate_rejects_prerequisite_inventory_linkage_mismatch ... ok
test release_evidence::tests::load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact ... ok
test release_evidence::tests::create_rejects_empty_source_acquisition_url_before_manifest ... ok
test release_evidence::tests::load_full_self_hosting_proof_identity_accepts_full_proof_bundle ... ok
test release_evidence::tests::create_rejects_invalid_provider_fixed_point_proof_before_manifest ... ok
test release_evidence::tests::create_rejects_provider_fixed_point_proof_for_different_binary ... ok
test release_evidence::tests::create_and_verify_release_bundle_round_trip ... ok
test release_evidence::tests::verify_rejects_tampered_binary_artifact ... ok
test release_evidence::tests::create_and_verify_release_bundle_records_source_acquisition_url ... ok
test release_evidence::tests::verify_rejects_non_canonical_manifest_json ... ok
test release_evidence::tests::verify_rejects_provider_kind_linkage_mismatch ... ok
test release_evidence::tests::create_and_verify_release_bundle_with_provider_fixed_point_proof ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 888 filtered out; finished in 0.01s
~~~

Exit status: 0

## cargo test -p mantle --bin mantle witness_rebuild::tests::

Command (pueue task 382):

~~~text
PATH="$HOME/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:/run/current-system/sw/bin:/usr/bin:/bin" rustfmt --edition 2024 src/witness_rebuild.rs && PATH="$HOME/.rustup/toolchains/nightly-2026-05-07-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$HOME/.cargo/bin:/run/current-system/sw/bin:/usr/bin:/bin" TMPDIR="$PWD/.tmp-independent-source/tmp" CARGO_TARGET_DIR="$PWD/.tmp-independent-source/target" RUSTC_BOOTSTRAP=1 PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig" SNIX_BUILD_SANDBOX_SHELL="/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox" cargo test -p mantle --bin mantle witness_rebuild::tests::
~~~

Output:

~~~text
test witness_rebuild::tests::failure_audit_reports_digest_mismatch_diagnostics ... ok
test witness_rebuild::tests::validate_existing_scratch_root_allows_helper_owned_dirs_only ... ok
test witness_rebuild::tests::failure_audit_reports_gcc_bootstrap_divergence_root ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_file_helper_owned_cargo_target_entry ... ok
test witness_rebuild::tests::source_acquisition_rejects_digest_mismatch_before_extraction ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlink_root ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlinked_helper_owned_tmp_entry ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_unexpected_entries ... ok
test witness_rebuild::tests::validate_rebuilt_output_digests_rejects_stripped_equivalent_but_different_binary ... ok
test witness_rebuild::tests::prepare_scratch_fetches_independent_source_archive_when_required ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_uses_provider_fixed_point_candidate_by_digest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_provider_fixed_point_stage_escape ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 876 filtered out; finished in 0.00s
~~~

Exit status: 0

## CLI parse focused tests

### release witness-rebuild --require-independent-source

Command (pueue task 331):

~~~text
cargo test -p mantle --bin mantle release_witness_rebuild_accepts_require_independent_source_flag
~~~

Output:

~~~text
running 1 test
test tests::release_witness_rebuild_accepts_require_independent_source_flag ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 902 filtered out; finished in 0.00s
~~~

Exit status: 0

### release create --source-acquisition-url

Command (pueue task 333):

~~~text
cargo test -p mantle --bin mantle release_create_accepts_source_acquisition_url_flag
~~~

Output:

~~~text
running 1 test
test tests::release_create_accepts_source_acquisition_url_flag ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 902 filtered out; finished in 0.00s
~~~

Exit status: 0

## cargo test -p mantle --bin mantle release_attestation::tests::

Command (pueue task 338):

~~~text
cargo test -p mantle --bin mantle release_attestation::tests::
~~~

Output:

~~~text
running 11 tests
test release_attestation::tests::build_policy_file_contents_self_proof_only_rejects_identity ... ok
test release_attestation::tests::build_policy_file_contents_single_witness_requires_identity ... ok
test release_attestation::tests::build_policy_file_contents_self_proof_only_drops_witnesses ... ok
test release_attestation::tests::validate_witness_identity_rejects_overlong_names ... ok
test release_attestation::tests::validate_witness_identity_rejects_control_characters ... ok
test release_attestation::tests::trusted_witness_identity_filter_allows_empty_allowlist ... ok
test release_attestation::tests::build_release_attestation_uses_manifest_digests_and_relative_paths ... ok
test release_attestation::tests::verify_signature_with_policy_names_rejects_untrusted_policy_signer ... ok
test release_attestation::tests::verify_signature_bytes_accepts_matching_trusted_key ... ok
test release_attestation::tests::verify_signature_bytes_rejects_colliding_names_when_no_key_matches ... ok
test release_attestation::tests::verify_signature_bytes_accepts_later_matching_key_when_names_collide ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 892 filtered out; finished in 0.01s
~~~

Exit status: 0
