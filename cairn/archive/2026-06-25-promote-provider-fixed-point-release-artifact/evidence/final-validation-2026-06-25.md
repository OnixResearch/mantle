# Final validation — provider fixed-point release artifact binding

Date: 2026-06-25
Change: `promote-provider-fixed-point-release-artifact`
Requirement: r[rust_package_planning.provider_fixed_point_release_artifact_binding]

## Scope and bounded claim

This change binds provider-backed Cargo-free fixed-point proof evidence to the packaged release binary artifact set. A provider proof is no longer sufficient for `--require-provider-fixed-point-proof` unless its fixed-point stage binary BLAKE3 digest matches a release binary in the manifest.

Non-claims:

- No deterministic-release eligibility is created by provider proof evidence alone.
- No full bootstrap reproducibility claim.
- No compiler correctness claim.
- No deploy success claim.
- No full Cargo compatibility claim.

## Baseline

Command (pueue task 17):

```sh
cargo test -p mantle --bin mantle provider_fixed_point -- --nocapture
```

Output excerpt:

```text
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 845 filtered out; finished in 0.01s
```

## Focused release-core binding tests

Command (pueue task 52):

```sh
cargo test -p crunch-release-core provider_fixed_point_release_artifact_binding -- --nocapture
```

Output:

```text
running 4 tests
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_missing_binaries ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_malformed_digest ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_accepts_matching_binary ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_mismatch ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.00s
```

## Focused provider fixed-point release tests

Command (pueue task 45):

```sh
cargo test -p mantle --bin mantle provider_fixed_point -- --nocapture
```

Output excerpt:

```text
test release_cmd::tests::provider_fixed_point_binding_records_matching_release_artifact ... ok
test release_cmd::tests::provider_fixed_point_binding_rejects_mismatched_release_artifact ... ok
test release_evidence::tests::create_rejects_provider_fixed_point_proof_for_different_binary ... ok
test release_evidence::tests::create_and_verify_release_bundle_with_provider_fixed_point_proof ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 845 filtered out; finished in 0.09s
```

## Formatting, build, and diff checks

Command (pueue task 73):

```sh
cargo fmt --check -p crunch-release-core -p mantle
```

Output:

```text
fmt_check_ok
```

Command (pueue task 71):

```sh
cargo build -p mantle --bin mantle -q && git diff --check
```

Output excerpt:

```text
warning: function `native_manifest_lock_blocker` is never used
warning: function `resolve_native_feature_roles` is never used
cargo_build_mantle_ok
git_diff_check_ok
```

The build warnings are existing rust-plan dead-code warnings and did not block the build.

## Final Cairn validation

Command (pueue task 78):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
```

Command (pueue task 83):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal promote-provider-fixed-point-release-artifact --root .
```

Output excerpt:

```json
{
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

Command (pueue task 85):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate design promote-provider-fixed-point-release-artifact --root .
```

Output excerpt:

```json
{
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

Command (pueue task 86):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks promote-provider-fixed-point-release-artifact --root .
```

Output excerpt:

```json
{
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-archive validation

Command (pueue task 93):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

Command (pueue task 95):

```sh
git status --short --branch
```

Output:

```text
## main...origin/main
 M README.md
 M cairn/specs/rust-package-planning/spec.md
 M crates/crunch-release-core/src/lib.rs
 M crates/crunch-release-core/src/manifest.rs
 M docs/operator-workflows.md
 M src/cargo_free_self_build.rs
 M src/release_cmd.rs
 M src/release_evidence.rs
?? cairn/archive/2026-06-25-promote-provider-fixed-point-release-artifact/
```

Command (pueue task 97, after appending post-archive evidence):

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . && git diff --check
```

Output:

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```
