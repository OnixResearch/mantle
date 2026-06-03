# Validation evidence — 2026-06-02

## Focused compile check

Command:

```text
/home/brittonr/git/mantle $ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo check --bin mantle
```

Result: command exited successfully.

Excerpt:

```text
warning: `mantle` (bin "mantle") generated 97 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.29s
```

## Focused tests

Command:

```text
/home/brittonr/git/mantle $ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo test --bin mantle frontend_artifact_export
```

Result: command exited successfully.

Excerpt:

```text
running 9 tests
test frontend_artifact_export::tests::digest_mismatch_fails_closed ... ok
test frontend_artifact_export::tests::hidden_fallback_marker_fails_closed ... ok
test frontend_artifact_export::tests::missing_attestation_fails_before_export ... ok
test frontend_artifact_export::tests::missing_content_fails_closed ... ok
test frontend_artifact_export::tests::export_receipt_serializes_receipt_hash ... ok
test frontend_artifact_export::tests::unsupported_ref_scheme_fails_closed ... ok
test frontend_artifact_export::tests::onix_like_kind_remains_opaque_when_attested ... ok
test frontend_artifact_export::tests::wrong_artifact_ref_fails_closed ... ok
test frontend_artifact_export::tests::valid_admitted_artifact_exports_with_receipt ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 667 filtered out; finished in 0.00s
```

Regression command:

```text
/home/brittonr/git/mantle $ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo test --bin mantle frontend_artifact_spec
```

Result: command exited successfully.

Excerpt:

```text
running 10 tests
...
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 666 filtered out; finished in 0.00s
```

## Formatting and diff hygiene

Commands:

```text
/home/brittonr/git/mantle $ nix develop -c rustfmt src/frontend_artifact_export.rs src/main.rs
/home/brittonr/git/mantle $ git diff --check
```

Results: both commands exited successfully.

## Tigerstyle lane

Command:

```text
/home/brittonr/git/mantle $ SNIX_BUILD_SANDBOX_SHELL=/bin/sh ./scripts/check-first-party-tigerstyle.sh -p mantle
```

Result: task completed but reported a pre-existing Mantle-package lint outside this change slice:

```text
--> src/protected_exec_seccomp.rs:491:26
491 |         if syscall_nr == libc::SYS_execveat as libc::c_int {
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = help: Tiger Style (Safe Narrowing): narrow integers with checked conversions instead of `as`
```

This finding is unrelated to `src/frontend_artifact_export.rs` and was not fixed in this slice to keep the change scoped. The new export module was written as a pure functional core with no filesystem/process/network I/O and uses named constants for non-trivial literals.

## Cairn gates

Command:

```text
/home/brittonr/git/mantle $ cairn validate --root . && cairn gate proposal frontend-artifact-fetch-export --root . && cairn gate design frontend-artifact-fetch-export --root . && cairn gate tasks frontend-artifact-fetch-export --root .
```

Result: command exited successfully.

Excerpt:

```json
{
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "frontend-artifact-fetch-export",
  "input_hash": "f6955b8d359e5f9c3f91f24be663511a34609fdda87b6e11d1c6710ab31038f8",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "9c6fd9345a13fbb6a3d28a762263884de4baba21e514569b2fa90423649e258d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## CLI seam update

Additional command after adding `mantle artifact export` and keeping storage-backed materialization tasks open:

```text
/home/brittonr/git/mantle $ git diff --check && cairn validate --root . && cairn gate tasks frontend-artifact-fetch-export --root .
```

Result: command exited successfully.

Excerpt:

```json
{
  "specs_validated": 7,
  "valid": true
}
{
  "change": "frontend-artifact-fetch-export",
  "input_hash": "e235a3f01c43a32022ac0da8fdd8b90c2c9c030c9a4b127b2b1adbcfcf326d54",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ad7d4993e79d6dee1088d597462d24e87d63631970b156ed3f21d1d0e26223ba",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Non-claims

This evidence now supports the pure validator/receipt slice and the frontend-neutral CLI seam that validates an already materialized artifact path. It does not claim a storage-backed export/materialization backend, Octet receipt integration, Onix deploy integration, or any ability to transfer/deploy `mantle-onix-activation-closure` artifacts yet.
