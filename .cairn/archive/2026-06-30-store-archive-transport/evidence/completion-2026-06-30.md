# Completion evidence — Store archive transport

Date: 2026-06-30

## Baseline before final verification edits

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-drain-store-archive cargo test -p crunch-store --lib archive::tests
running 8 tests
...
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 141 filtered out
```

## Final focused validation

Environment used for Cargo checks:

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
```

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-drain-store-archive cargo test -p crunch-store --lib archive::tests
test archive::tests::archive_list_rejects_header_end_count_mismatch ... ok
test archive::tests::archive_import_rejects_unsupported_ca_metadata_without_persisting ... ok
test archive::tests::archive_import_rejects_tampered_payload_without_persisting ... ok
test archive::tests::archive_import_rejects_truncated_payload_without_persisting ... ok
test archive::tests::archive_import_rejects_untrusted_signature_without_persisting ... ok
test archive::tests::archive_import_rejects_conflicting_local_pathinfo ... ok
test archive::tests::archive_list_drains_non_seekable_payloads_in_bounded_chunks ... ok
...
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 141 filtered out; finished in 0.05s
```

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-drain-store-archive cargo test -p mantle --test store_archive_cli -- --nocapture
running 3 tests
test store_archive_cli_requires_unsigned_escape_hatch ... ok
test store_archive_cli_streams_stdout_and_stdin ... ok
test store_archive_cli_exports_lists_json_and_imports_from_file ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-drain-store-archive cargo fmt --check -p mantle -p crunch-store
completed successfully
```

## Cairn gates before completion marking

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 12,
  "valid": true
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal store-archive-transport --root /home/brittonr/git/mantle
"verdict": "PASS"

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design store-archive-transport --root /home/brittonr/git/mantle
"verdict": "PASS"

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks store-archive-transport --root /home/brittonr/git/mantle
"verdict": "PASS"
```

## Scope of proof

This evidence proves the Mantle-native `mantle-store-archive-v1` transport, not byte compatibility with Determinate or upstream Nix `nario` formats. CLI/help/report wording keeps external nario v2 compatibility explicitly unproven.

## Sync and archive validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync store-archive-transport --root /home/brittonr/git/mantle
{
  "blocked": false,
  "dry_run": true,
  "mutated": false
}

$ nix run path:/home/brittonr/git/cairn#cairn -- sync store-archive-transport --root /home/brittonr/git/mantle --execute
{
  "blocked": false,
  "dry_run": false,
  "mutated": true
}

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 13,
  "valid": true
}

$ CAIRN_ARCHIVE_DATE=2026-06-30 nix run path:/home/brittonr/git/cairn#cairn -- archive store-archive-transport --root /home/brittonr/git/mantle --execute
{
  "blocked": false,
  "dry_run": false,
  "mutated": true,
  "actions": [
    {
      "kind": "archive_change",
      "path": "/home/brittonr/git/mantle/cairn/archive/2026-06-30-store-archive-transport"
    }
  ]
}

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 12,
  "valid": true
}
```
