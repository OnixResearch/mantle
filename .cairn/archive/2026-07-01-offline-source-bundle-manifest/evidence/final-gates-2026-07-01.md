# Final gates — 2026-07-01

Task-ID: final-validation
Covers: source_transports.offline_source_bundle_format, source_transports.source_adapter_contract, source_transports.source_bundle_export_plan, source_transports.source_filesystem_canonicalization, source_transports.source_bundle_import_verify, source_transports.source_bundle_atomic_pinning, source_transports.offline_build_preflight, source_transports.source_bundle_non_claims

## Question

Do validation and Cairn gates pass after all `offline-source-bundle-manifest` tasks are checked complete?

## Validation transcript

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root . && \
  nix run path:/home/brittonr/git/cairn#cairn -- gate proposal offline-source-bundle-manifest --root . && \
  nix run path:/home/brittonr/git/cairn#cairn -- gate design offline-source-bundle-manifest --root . && \
  nix run path:/home/brittonr/git/cairn#cairn -- gate tasks offline-source-bundle-manifest --root .

{
  "input_hash": "158dce00483995cbc7e1f05d78fd02b10d39d2875d09db44229f606bfe97de7e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "d4601dd60be7c16caa071de039c9334e3683d174effd6c694eeed3729fad065d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-archive validation transcript

```text
$ CAIRN_ARCHIVE_DATE=2026-07-01 nix run path:/home/brittonr/git/cairn#cairn -- archive offline-source-bundle-manifest --root . --execute
{
  "plan_hash": "ebb143768e51a85d686029c6a8d958ad5520ebe2aeda04dcd1ab6533d98dfff9",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "80898579668820b2ac70895afa0602d81737d8c9eeb6cf5a195b8f61b7f9e35f"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 17,
  "valid": true
}

$ cp archived source-transports delta into cairn/specs/source-transports/spec.md with accepted-spec wrapper
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 18,
  "valid": true
}
```

## Decision

The checked task set was archived under `cairn/archive/2026-07-01-offline-source-bundle-manifest/`, the source-transports requirements were promoted into accepted specs, and post-archive validation passed.
