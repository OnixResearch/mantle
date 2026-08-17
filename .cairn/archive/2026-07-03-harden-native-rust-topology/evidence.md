# Evidence: harden-native-rust-topology

## Baseline before edits

Baseline command attempts showed the repo requires the documented Rust/build environment:

```text
Task 175: cargo test -p mantle --bin mantle rust_plan::tests:: -- --nocapture
sh: line 1: cargo: command not found
```

```text
Task 177: cargo test -p mantle --bin mantle rust_plan::tests:: -- --nocapture
error: linker `clang` not found
```

With the documented nightly, clang/mold/pkg-config, OpenSSL pkg-config, and `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`, the pre-change focused baseline passed:

```text
Task 179: cargo test -p mantle --bin mantle rust_plan::tests:: -- --nocapture

test result: ok. 182 passed; 0 failed; 0 ignored; 0 measured; 937 filtered out; finished in 0.05s
```

## Implementation evidence

`src/rust_plan.rs` now records diagnostic context on blocked/failed unit receipts:

- stable unit identity (`unit_id`);
- package identity (`package_id`);
- execution role and selected triple;
- target name/kind;
- blocker class;
- consumed artifact roles.

Blocked/failed receipts now also carry bounded replay evidence that stores deterministic unit-boundary facts and BLAKE3 digests for larger vectors: declared outputs, dependency artifacts, host artifacts, output artifacts, and consumed artifact roles.

Positive and negative tests added:

- `blocked_receipt_records_diagnostic_context_and_bounded_replay_evidence`
- `replay_evidence_json_rejects_malformed_and_oversized_inputs`
- `replay_evidence_json_rejects_invalid_digest_fields`

Existing negative role/metadata pre-rustc checks remain in the focused rust-plan suite.

## Post-change validation

Focused suite after formatting, single-threaded to avoid the pre-existing compiler-policy temp-fixture race seen in parallel runs:

```text
Task 206: cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=1 --nocapture && cargo fmt -p mantle --check

test result: ok. 185 passed; 0 failed; 0 ignored; 0 measured; 937 filtered out; finished in 0.22s
```

Parallel focused runs intermittently reported different compiler-policy tests failing without assertion detail, while direct reruns of the named tests passed. This was treated as a pre-existing parallel fixture race and the serial focused suite is the recorded validation evidence for this change.

```text
Task 209: git status --short --branch && git diff --check
## main...origin/main [ahead 6]
 M src/rust_plan.rs
```

diff check emitted no whitespace errors.

## Cairn validation and gates

```text
Task 211: nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
```

```text
Task 212: gate proposal harden-native-rust-topology
"stage": "proposal", "valid": true, "verdict": "PASS"
```

```text
Task 213: gate design harden-native-rust-topology
"stage": "design", "valid": true, "verdict": "PASS"
```

```text
Task 214: gate tasks harden-native-rust-topology
"stage": "tasks", "valid": true, "verdict": "PASS"
```

## Post-archive validation

The archive command's first execution completed before the follow-up pueue wait could find its transient task id; a rerun reported `change not found`, and the archive directory exists at `cairn/archive/2026-07-03-harden-native-rust-topology/`.

```text
Task 15: nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle && test -f cairn/archive/2026-07-03-harden-native-rust-topology/evidence.md && git status --short --branch
{
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 18,
  "valid": true
}
## main...origin/main [ahead 6]
 D cairn/changes/harden-native-rust-topology/design.md
 D cairn/changes/harden-native-rust-topology/proposal.md
 D cairn/changes/harden-native-rust-topology/specs/rust-package-planning/spec.md
 D cairn/changes/harden-native-rust-topology/tasks.md
 M cairn/specs/rust-package-planning/spec.md
 M src/rust_plan.rs
?? cairn/archive/2026-07-03-harden-native-rust-topology/
```
