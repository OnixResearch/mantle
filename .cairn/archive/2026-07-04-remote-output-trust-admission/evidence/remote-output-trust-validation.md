# Remote output trust admission validation

Task-ID: remote-output-trust-admission
Covers: r[remote_builds.output_trust_admission_pipeline]
Date: 2026-07-04

## Key trust and route preflight

`pueue task 249`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle output_trust -- --nocapture

running 5 tests
test remote_build::tests::output_trust_requires_trusted_key_and_prefix ... ok
test remote_build::tests::output_trust_rejects_same_name_different_key_material ... ok
test realization_routing::tests::remote_builder_ticket_without_output_trust_is_rejected ... ok
test remote_build::tests::output_trust_uses_key_material_digest_when_available ... ok
test realization_routing::tests::cli_remote_builder_plan_rejects_missing_output_trust_before_local_build ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1168 filtered out; finished in 0.00s
```

## Final admission and durable import

`pueue task 257` (remote-output filter excerpt):

```text
running 11 tests
test remote_build::tests::durable_remote_output_import_rejects_missing_pathinfo_bundle ... ok
test remote_build::tests::durable_remote_output_import_rejects_pathinfo_signature_key_mismatch ... ok
test remote_build::tests::remote_output_admission_rejects_transfer_key_mismatch ... ok
test remote_build::tests::remote_output_admission_rejects_untrusted_builder_key ... ok
test remote_build::tests::remote_output_admission_rejects_malformed_digest ... ok
test remote_build::tests::durable_remote_output_import_rejects_attestation_digest_mismatch ... ok
test remote_build::tests::remote_output_digest_changes_when_payload_identity_changes ... ok
test remote_build::tests::remote_output_import_rejects_tampered_transfer_artifact_payload ... ok
test remote_build::tests::remote_output_import_requires_transfer_artifact_for_framed_pathinfo ... ok
test remote_build::tests::durable_remote_output_import_persists_signed_pathinfo_attestation_and_report ... ok
test remote_build::tests::full_nar_transfer_artifact_materializes_remote_output_bytes ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.01s
```

## Revoked/expired access boundary

`pueue task 259`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle ticket_authorization -- --nocapture

running 1 test
test remote_build::tests::ticket_authorization_rejects_revoked_and_expired_tickets ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
```
