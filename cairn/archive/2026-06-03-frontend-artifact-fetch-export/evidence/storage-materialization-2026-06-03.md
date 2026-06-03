# Evidence: storage-backed admitted artifact materialization

Date: 2026-06-03

## Scope proven

This slice adds a Mantle-owned local frontend artifact store for admitted artifact export tests and public command plumbing:

- `mantle artifact import <path>` imports a file, symlink, or directory into `<state-dir>/frontend-artifacts/v1/blake3/<digest>/content`.
- `mantle artifact export --artifact-ref mantle://blake3/<digest> --out <path> ...` materializes content from that store before producing the generic admitted-artifact export receipt.
- Export preflights spec admission, proof/ref matching, hidden-fallback markers, storage-ref shape, and storage-ref digest matching before copying content.
- Frontend artifact kinds remain opaque data; Mantle does not interpret Onix activation, roles, tags, providers, inventory, or deploy policy.

## Non-claims

This storage backend is local Mantle state, not an Onix deploy path and not an Octet receipt layer. Onix still needs a follow-up change to consume this boundary and replace its `nix copy` deploy path.

## Commands run

```console
$ nix develop -c rustfmt src/artifact_cmd.rs src/frontend_artifact_export.rs src/frontend_artifact_store.rs src/main.rs
Task 58: completed successfully
```

```console
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo test --bin mantle artifact_cmd
running 12 tests
test artifact_cmd::tests::parse_content_provenance_requires_key_value_entries ... ok
test artifact_cmd::tests::artifact_export_cli_rejects_missing_stored_content ... ok
test artifact_cmd::tests::artifact_export_cli_rejects_missing_materialized_content ... ok
test artifact_cmd::tests::artifact_export_cli_rejects_non_mantle_refs ... ok
test artifact_cmd::tests::artifact_export_cli_accepts_admission_sidecar ... ok
test artifact_cmd::tests::artifact_export_cli_writes_receipt_for_admitted_artifact ... ok
test artifact_cmd::tests::artifact_export_cli_rejects_digest_mismatch_before_materialization ... ok
test artifact_cmd::tests::artifact_export_cli_rejects_missing_admission_before_materialization ... ok
test artifact_cmd::tests::artifact_export_cli_rejects_hidden_fallback_before_materialization ... ok
test artifact_cmd::tests::artifact_export_cli_materializes_from_artifact_store ... ok
test artifact_cmd::tests::artifact_import_cli_writes_store_report ... ok
test artifact_cmd::tests::artifact_export_cli_rejects_wrong_ref_before_materialization ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 680 filtered out; finished in 0.02s
```

```console
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo test --bin mantle frontend_artifact_store
running 4 tests
test frontend_artifact_store::tests::materialize_missing_ref_returns_none ... ok
test frontend_artifact_store::tests::materialize_rejects_unsupported_ref ... ok
test frontend_artifact_store::tests::import_and_materialize_file_roundtrip ... ok
test frontend_artifact_store::tests::import_and_materialize_directory_roundtrip ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 688 filtered out; finished in 0.01s
```

```console
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo test --bin mantle frontend_artifact_export
running 9 tests
test frontend_artifact_export::tests::missing_content_fails_closed ... ok
test frontend_artifact_export::tests::missing_attestation_fails_before_export ... ok
test frontend_artifact_export::tests::unsupported_ref_scheme_fails_closed ... ok
test frontend_artifact_export::tests::digest_mismatch_fails_closed ... ok
test frontend_artifact_export::tests::hidden_fallback_marker_fails_closed ... ok
test frontend_artifact_export::tests::onix_like_kind_remains_opaque_when_attested ... ok
test frontend_artifact_export::tests::wrong_artifact_ref_fails_closed ... ok
test frontend_artifact_export::tests::valid_admitted_artifact_exports_with_receipt ... ok
test frontend_artifact_export::tests::export_receipt_serializes_receipt_hash ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 683 filtered out; finished in 0.00s
```

```console
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo test --bin mantle frontend_artifact_spec
running 10 tests
test frontend_artifact_spec::tests::artifact_kind_not_allowed_fails_closed ... ok
test frontend_artifact_spec::tests::hidden_fallback_marker_fails_closed ... ok
test frontend_artifact_spec::tests::spec_hash_mismatch_fails_closed ... ok
test frontend_artifact_spec::tests::manifest_spec_binding_mismatch_fails_closed ... ok
test frontend_artifact_spec::tests::unsupported_validator_kind_fails_closed ... ok
test frontend_artifact_spec::tests::frontend_semantic_terms_are_data_under_the_declared_spec ... ok
test frontend_artifact_spec::tests::onix_like_kind_remains_frontend_data_under_spec ... ok
test frontend_artifact_spec::tests::missing_spec_fails_closed ... ok
test frontend_artifact_spec::tests::admission_sidecar_serializes_attestation ... ok
test frontend_artifact_spec::tests::valid_frontend_artifact_spec_admits_manifest ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 682 filtered out; finished in 0.00s
```

```console
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo check --bin mantle
warning: `mantle` (bin "mantle") generated 48 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.48s
```

```console
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh ./scripts/check-first-party-tigerstyle.sh -p mantle
--> src/protected_exec_seccomp.rs:491:26
491 |         if syscall_nr == libc::SYS_execveat as libc::c_int {
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = help: Tiger Style (Safe Narrowing): narrow integers with checked conversions instead of `as`
```

The Tigerstyle finding is the same pre-existing unrelated lint recorded by the previous evidence file. The new artifact store and command code did not add a new Tigerstyle diagnostic.

## Files proving the slice

- `src/frontend_artifact_store.rs` implements deterministic local import/materialization for frontend artifacts under Mantle state.
- `src/artifact_cmd.rs` uses preflight validation before storage access and writes export/import reports.
- `src/frontend_artifact_export.rs` exposes `validate_frontend_artifact_export_preflight` so shell code can fail before copying content.
- `src/main.rs` exposes `artifact import` and storage-backed `artifact export --out` flags.
