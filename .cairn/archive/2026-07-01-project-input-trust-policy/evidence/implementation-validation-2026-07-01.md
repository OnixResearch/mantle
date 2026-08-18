# Implementation validation — Project input trust policy

Date: 2026-07-01

## Question

Does the current tree implement `project-input-trust-policy` with bounded claims and fail-closed refresh behavior?

## Inspected evidence

- Pure no-std core: `crates/crunch-project-core/src/trust.rs` plus refresh/lock/manifest integration.
- Shell verifier: `src/project_resolve.rs` local Ed25519 detached signature verifier.
- CLI/report coverage: `tests/project_refresh_cli.rs`, `src/project_cmd.rs`.
- Validation commands below were run in this session with `TMPDIR=/home/brittonr/.cargo-target/tmp-mantle-check` to avoid the near-full `/tmp` filesystem.

## Command transcripts

### Core trust policy tests

```text
$ cargo test -p crunch-project-core
...
test result: ok. 148 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests crunch_project_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Source: pueue task 364.

### Project adapter/Nickel tests

```text
$ cargo test -p crunch-project
...
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

   Doc-tests crunch_project

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Source: pueue task 276.

### Shell verifier unit tests

```text
$ cargo test -p mantle --bin crunch local_project_trust_signature -- --nocapture
...
running 2 tests
test project_resolve::tests::local_project_trust_signature_with_wrong_digest_yields_no_fact ... ok
test project_resolve::tests::local_project_trust_signature_produces_verified_fact ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1056 filtered out; finished in 0.00s
```

Source: pueue task 267.

### CLI trust refresh fixtures

```text
$ cargo test -p mantle --test project_refresh_cli refresh_trusted_file_input -- --nocapture
...
running 2 tests
test refresh_trusted_file_input_with_wrong_signature_rejects_lock_write ... ok
test refresh_trusted_file_input_writes_lock_evidence_and_show_reports_claim ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.10s
```

Source: pueue task 264.

### Compatibility fixture checks touched by trust fields

```text
$ cargo test -p mantle --test project_cli check_static_accepts_build_fetch_policy_without_fetching_remote_url -- --nocapture
...
running 1 test
test check_static_accepts_build_fetch_policy_without_fetching_remote_url ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.06s
```

Source: pueue task 322.

```text
$ cargo test -p mantle --test attest_cli attest_project_verify_and_diff -- --nocapture
...
running 1 test
test attest_project_verify_and_diff ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.29s
```

Source: pueue task 295.

### Root binary compile check

```text
$ cargo check -p mantle --bin crunch --message-format=short
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
```

Source: pueue task 298. The duplicate-target warning is pre-existing and non-fatal.

### Formatting

```text
$ cargo fmt -p crunch-project-core -p crunch-project -p mantle --check
Task completed successfully.
```

Source: pueue task 372.

### Cairn validation and gates

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 15,
  "valid": true
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal project-input-trust-policy --root .
{
  "change": "project-input-trust-policy",
  "issues": [],
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design project-input-trust-policy --root .
{
  "change": "project-input-trust-policy",
  "issues": [],
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks project-input-trust-policy --root .
{
  "change": "project-input-trust-policy",
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Full transcript: `target/cairn-evidence/project-input-trust-policy-gates-2026-07-01.txt` from pueue task 312.

### Final Cairn validation after tasks/evidence update

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 15,
  "valid": true
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal project-input-trust-policy --root .
{
  "change": "project-input-trust-policy",
  "issues": [],
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design project-input-trust-policy --root .
{
  "change": "project-input-trust-policy",
  "issues": [],
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks project-input-trust-policy --root .
{
  "change": "project-input-trust-policy",
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Full transcript: `target/cairn-evidence/project-input-trust-policy-final-gates-2026-07-01.txt` from pueue task 328.

## Decision

Accepted for this change scope. The implementation supports explicit local Ed25519 detached trust policy for project inputs and patches, rejects missing/invalid evidence before writing refreshed lock data, records bounded locked trust evidence, and reports non-claim text that does not overstate signer evidence as reproducibility, release validity, forge trust, or global upstream authenticity.

## Owner

Mantle project workflow owner.

## Next action

Archive only after the implementation commit is ready and the accepted specs are synced by Cairn.
