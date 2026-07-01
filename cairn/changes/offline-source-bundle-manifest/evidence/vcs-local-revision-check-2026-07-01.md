# Local VCS revision check — 2026-07-01

Task-ID: implementation-slice
Covers: source_transports.source_bundle_export_plan

## Question

Does local `file://` VCS snapshot export prove the checkout revision before materializing payload bytes?

## Inspected evidence

- `src/source_bundle.rs` now checks local VCS snapshots before payload canonicalization: it resolves `.git/HEAD`, loose `refs/*`, and `packed-refs` entries, then compares the checked-out object id with the requested `env.rev` object id or `refs/*` name.
- The check is host-tool-free: it reads Git metadata directly instead of invoking `git`.
- `source_bundle_export_materializes_local_vcs_snapshot_without_dot_git` now seeds a matching `.git/HEAD` and `refs/heads/main` before export.
- `source_bundle_export_rejects_local_vcs_revision_mismatch` proves a checkout at one object id cannot satisfy a different requested revision.

## Validation transcript

```text
$ nix develop -c cargo fmt --check && nix develop -c cargo check -p mantle && nix develop -c cargo test -p mantle --bin mantle source_bundle

test source_bundle::tests::source_bundle_export_materializes_local_vcs_snapshot_without_dot_git ... ok
test source_bundle::tests::source_bundle_export_rejects_local_vcs_revision_mismatch ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 1024 filtered out; finished in 0.01s
```

## Decision

This removes the local checkout revision-proof gap for `file://` VCS snapshot export. It does not acquire or verify non-local VCS payloads; those still require imported source state or a later explicit acquisition policy.

## Next action

Continue with non-local acquisition policy and broader source-kind/adapter fail-closed coverage.
