# Build-root source planning slice — Offline source bundle manifest

Date: 2026-06-30

## Implemented in this slice

- `mantle source bundle plan|export` now accept `--build-root <file>` plus `--import-path/-I`.
- Build-root planning evaluates the selected `.ncl` roots without running builds or fetchers.
- The pure derivation walk derives deterministic virtual source records for:
  - fixed URL fetcher derivations (`builder = "builtin:fetchurl"`);
  - git fetcher derivations as `vcs-snapshot` metadata records;
  - declared pre-existing store-path inputs as `toolchain-source-root` identity records.
- Derived records use BLAKE3 over bounded canonical metadata, deduplicate equivalent repeated inputs, reject conflicting records, and fail closed for builtin fetchers that omit `fixed_output` or `env.url`.
- Existing explicit local path `--source kind:identity:path` planning remains available and can be combined with derived build-root records.

## Evidence

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-source-plan cargo fmt --check -p mantle
status: ok

$ CARGO_TARGET_DIR=/tmp/mantle-target-source-plan cargo test -p mantle --bin mantle source_bundle::tests
running 8 tests
test source_bundle::tests::source_bundle_rejects_unsafe_identity ... ok
test source_bundle::tests::source_bundle_rejects_unfixed_builtin_fetcher_in_build_root ... ok
test source_bundle::tests::source_bundle_derives_vcs_snapshot_from_git_fetcher ... ok
test source_bundle::tests::source_bundle_rejects_unsafe_symlink ... ok
test source_bundle::tests::source_bundle_derives_fetcher_and_store_path_inputs_from_build_root ... ok
test source_bundle::tests::source_bundle_canonicalizes_equivalent_traversal ... ok
test source_bundle::tests::source_bundle_verify_reports_missing_state ... ok
test source_bundle::tests::source_bundle_import_and_verify_are_idempotent ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 955 filtered out; finished in 0.00s

$ CARGO_TARGET_DIR=/tmp/mantle-target-source-plan cargo run -q -p mantle --bin mantle -- --json source bundle plan --build-root examples/fetch-file.ncl
{
  "store_prefix": "/mantle/store",
  "record_count": 1,
  "payload_bytes": 0,
  "ready_class": "ready",
  "records": [
    {
      "kind": "fixed-url",
      "identity": "fixed-url-dc057856e00ba4b1e8294209a9af65fb224dea7089dad4a49e19f5906c07f752",
      "payload_bytes": 0,
      "content_blake3": "dc057856e00ba4b1e8294209a9af65fb224dea7089dad4a49e19f5906c07f752",
      "file_count": 0
    }
  ],
  "non_claim": "source bundle evidence proves declared source/input availability and identity only"
}
```

## Remaining gap before task completion

This still does not complete `offline-source-bundle-manifest`: source-bundle export does not materialize VCS checkout payloads, package-manager mirror adapters are not implemented, source bundles are not yet integrated into offline build preflight, bootstrap/provider/toolchain/proof fixtures remain incomplete, and CLI tests for no-network import/list/verify plus stale/missing preflight are still required.
