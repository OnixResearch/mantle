# Adapter readiness classes — 2026-07-01

Task-ID: implementation-slice
Covers: source_transports.source_adapter_contract, source_transports.offline_build_preflight

## Question

Can language-neutral adapter metadata fail closed as unsupported or untrusted without introducing Cargo-specific requirements?

## Inspected evidence

- `SourceReadiness` now includes `untrusted`, and offline preflight / verify JSON reports include `untrusted_records` alongside `unsupported_records`.
- Generic adapter metadata can declare `extra.unsupported = "true"`; preflight classifies the record as `unsupported` and refuses readiness.
- Generic adapter metadata can declare `extra.trusted-provenance = "false"`; preflight classifies the record as `untrusted` even when the payload is imported and pinned.
- The checks live in source-bundle core classification and do not require Cargo-specific fields.

## Validation transcript

```text
$ nix develop -c cargo test -p mantle --bin mantle source_bundle

test source_bundle::tests::source_offline_preflight_reports_unsupported_adapter ... ok
test source_bundle::tests::source_offline_preflight_reports_untrusted_adapter ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 1024 filtered out; finished in 0.00s
```

```text
$ nix develop -c cargo fmt --check && nix develop -c cargo check -p mantle && nix develop -c cargo test -p mantle --bin mantle source_bundle && nix develop -c cargo test -p mantle --test source_bundle_cli

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 18,
  "valid": true
}
```

## Decision

This satisfies the generic adapter fail-closed readiness classes for this change's current scope. Real package-manager shell adapters can build on these metadata classes later without making this format change Cargo-specific.

## Next action

Run final gates and complete/archive `offline-source-bundle-manifest` if review accepts pure fixtures plus generic adapter classes as the intended scope.
