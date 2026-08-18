# Export materialization slice — Offline source bundle manifest

Date: 2026-06-30

## Implemented in this slice

- `source bundle export --build-root` now materializes local `file://` fixed fetcher payloads into canonical source-bundle file entries instead of writing metadata-only virtual records.
- Local `file://` git/VCS snapshot records are materialized from an already-present checkout directory with `.git/` excluded from the canonical payload.
- Remote/non-local fetcher URLs fail closed during export with a deterministic diagnostic instead of performing network access or silently exporting a metadata-only payload claim.
- `source bundle plan --build-root` remains no-mutate and metadata-only.
- Single-file local payload roots now canonicalize correctly as one file entry.

## Evidence

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-source-export cargo fmt --check -p mantle
status: ok

$ CARGO_TARGET_DIR=/tmp/mantle-target-source-export cargo test -p mantle --bin mantle source_bundle::tests
running 12 tests
test source_bundle::tests::source_bundle_rejects_unfixed_builtin_fetcher_in_build_root ... ok
test source_bundle::tests::source_bundle_rejects_unsafe_identity ... ok
test source_bundle::tests::source_bundle_export_rejects_remote_fetcher_without_local_payload ... ok
test source_bundle::tests::source_bundle_derives_vcs_snapshot_from_git_fetcher ... ok
test source_bundle::tests::source_bundle_derives_fetcher_and_store_path_inputs_from_build_root ... ok
test source_bundle::tests::source_bundle_canonicalizes_equivalent_traversal ... ok
test source_bundle::tests::source_bundle_export_materializes_local_file_fetcher_payload ... ok
test source_bundle::tests::source_bundle_accepts_single_file_payload_root ... ok
test source_bundle::tests::source_bundle_export_materializes_local_vcs_snapshot_without_dot_git ... ok
test source_bundle::tests::source_bundle_rejects_unsafe_symlink ... ok
test source_bundle::tests::source_bundle_import_and_verify_are_idempotent ... ok
test source_bundle::tests::source_bundle_verify_reports_missing_state ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 955 filtered out; finished in 0.01s

$ CARGO_TARGET_DIR=/tmp/mantle-target-source-export cargo run -q -p mantle --bin mantle -- --json source bundle export --build-root "$tmp/root.ncl" --to "$tmp/bundle.json"
{
  "store_prefix": "/mantle/store",
  "record_count": 1,
  "payload_bytes": 7,
  "ready_class": "ready",
  "records": [
    {
      "kind": "fixed-url",
      "payload_bytes": 7,
      "file_count": 1
    }
  ],
  "non_claim": "source bundle evidence proves declared source/input availability and identity only"
}

$ CARGO_TARGET_DIR=/tmp/mantle-target-source-export cargo run -q -p mantle --bin mantle -- source bundle export --build-root "$remote_root" --to "$tmp/bundle.json"
error: source bundle export cannot materialize non-local source URL for fixed-url-...: https://example.invalid/source.tar.gz
```

## Remaining gap before task completion

This still is not a complete source transport: remote fetcher payload acquisition is intentionally not implemented in `export`; package-manager mirror adapters, bootstrap/provider/toolchain/proof fixtures, imported-state offline preflight, and broader CLI/integration coverage remain required before the change can be archived.
