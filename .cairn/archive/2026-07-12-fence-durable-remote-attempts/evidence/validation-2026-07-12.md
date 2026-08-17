# Durable remote-attempt fencing validation

Change: `fence-durable-remote-attempts`
Date: 2026-07-12
Target directory: `/tmp/mantle-remote-target`
Pueue group: `mantle-fence-20260712-7c91`

## Baseline

The initial baseline was reconstructed in an isolated detached worktree at the recorded starting commit because the first attempt was blocked before compilation by a missing target directory/quota probe.

Pueue task 1352:

```text
baseline_commit=f2e43a9fa396683d624f3d9e6df6586062a522d3
coordinator: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1275 filtered out; finished in 0.00s
remote-config: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1275 filtered out; finished in 0.00s
distributed-core: test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 431 filtered out; finished in 0.01s
```

Baseline proposal/design/tasks gates passed through the canonical Cairn checkout in pueue task 994. The legacy compatibility symlink `/home/brittonr/git/cairn` was rejected by Nix as a symlink in task 990; rerunning through `/home/brittonr/git/OnixResearch/cairn` removed that invocation-only blocker.

## Focused implementation evidence

### Pure attempt core

Command:

```text
cargo test -p crunch-build --lib distributed::remote_attempt::tests:: -- --nocapture
```

Pueue task 1356:

```text
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 472 filtered out; finished in 0.01s
```

Coverage includes transition tables, proptest equivalence/monotonicity/terminal closure, canonical event conflict handling, malformed/future identities, event-retention exhaustion, transfer regression, retry budget/deadline decisions, output authorization separation, and repeated fence advancement.

Three source-level Kani harnesses are present for equivalent decisions, fence monotonicity, and terminal-state closure. `cargo-kani` was not installed (`cargo-kani unavailable`, pueue task 973), so this evidence does not claim Kani execution.

The pure-core I/O audit found no `fs`, environment, process, clock, async, Tokio, or printing APIs in `crates/crunch-build/src/distributed/remote_attempt.rs`.

### Coordinator shell and report fencing

Command:

```text
cargo test -p mantle --bin mantle remote_build::tests::coordinator_ -- --nocapture
```

Pueue task 1357:

```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1278 filtered out; finished in 0.00s
```

Coverage includes persistence-before-exposure, unconfigured/failed persistence, fsynced restart recovery for each live phase, fail-closed legacy/corrupt migration, reassignment with an advanced fence, stale resume rejection, byte-identical durable idempotency, conflicting event reuse with no mutable-surface change, stale start/heartbeat/log/transfer/result/failure/completion rejection, post-completion stale-race rejection, and fenced cryptographic output-admission preflight.

### Typed remote policy

Command:

```text
cargo test -p mantle --bin mantle remote_farm_config::tests:: -- --nocapture
```

Pueue task 1359:

```text
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1286 filtered out; finished in 0.04s
```

Positive Nickel coverage round-trips named attempt/retry/time bounds. Negative coverage rejects a type mismatch and Rust validation rejects an invalid retry budget.

### Existing distributed behavior

Command:

```text
cargo test -p crunch-build --lib distributed::tests:: -- --nocapture
```

Pueue task 1360:

```text
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 444 filtered out; finished in 0.01s
```

Non-test compilation also passed:

```text
cargo check -q -p mantle --bin mantle
cargo-check: PASS
```

Pueue task 1363.

`cargo fmt --check -p crunch-build` passed in task 1366.

## Broader lint blockers

Focused source inspection found no Clippy diagnostics in the new pure attempt module. The crate-wide `crunch-build` Clippy command remains blocked by nine pre-existing findings in `dynamic_plan.rs`, `network_policy.rs`, and `worker.rs` (task 1186/1342). The root-package Clippy rail remains blocked by broad pre-existing findings outside this change; after allowing the two first blockers to enumerate further diagnostics, no changed `remote_build.rs` finding remained, while the already test-only/unused `remote_farm_config` module continued to trigger its existing dead-code family (task 1335).

Root-package format checking remains blocked by unrelated pre-existing formatting drift in files such as `tests/trust_policy_offline_rail.rs` (task 1194). Unrelated rustfmt churn in `src/remote_build.rs` and `src/remote_farm_config.rs` was reverted; the new `crunch-build` module is format-clean.

## Cairn

Initial implementation-complete validation was pueue task 1369; post-task-completion validation was rerun as pueue task 1383 with the same clean result:

```json
{
  "change_issues": [],
  "changes": 18,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 43,
  "valid": true
}
```

No pre-existing policy blocker remained in Cairn structural validation.

The separate repository-wide Tracey policy profile remains red on pre-existing release-provenance debt. Pueue task 1422 captured the complete JSON report:

```text
valid: false
requirements: 73
referenced: 18
missing: 55
dangling: 0
next missing group: mantle.release_provenance.cairn_evidence_handoff.*
receipt_hash: 05402ceba257df78cbe041c613a0480b1ab64d9a6a92b05201de1494c0f5847f
```

Every missing ID in that report is under `mantle.release_provenance.*`; none belongs to this remote-attempt change. The current `mantle-default` Tracey profile scans only the accepted release-provenance spec and its five configured evidence sources, so `tools/tracey_refs.rs` is outside that profile. `dangling` is empty, and this evidence makes no global Tracey-pass claim.

Final stage gates:

| Stage | Pueue task | Receipt hash | Result |
|---|---:|---|---|
| proposal | 1379 | `f535af95786420361a240e6565d5e1281187bf3be574117e9bcf833982fb5306` | `PASS`, no issues |
| design | 1380 | `d4accedd83c1ca1cf89b08709ee73801e646c3250bc06d40bdfa173abb1973e2` | `PASS`, no issues |
| tasks (post-completion) | 1438 | `42df1f328cdd728ebcb9adfe44c8123952574efd22db7e20ae423570e8afd34b` | `PASS`, no issues |

## Integrated-main adversarial hardening

Review after integration found an ABA identity gap: job and attempt ids were deterministic functions of the normalized request, initial fence, and worker, so deleting/resetting coordinator state could recreate credentials held by an old worker. The shell now supplies a fresh 256-bit operating-system nonce for every assignment. Initial job ids and every attempt id bind that nonce; durable reload re-derives the attempt id from the stored nonce and assigned worker, and legacy/mismatched nonce state fails closed.

Review also found that an already-applied `ResultReady` event returned no admission report after restart. Finished-undelivered duplicate responses are now cryptographically revalidated and can reconstruct the admission report without reapplying state. Malformed duplicate responses fail instead of bypassing validation.

Current integrated-main evidence:

```text
pueue 1942: cargo test -p crunch-build --lib distributed::remote_attempt::tests:: -- --nocapture
PASS: 15 passed; 0 failed
Includes positive nonce-separated assignment identity and negative malformed nonce coverage.

pueue 1965: cargo test -p mantle --bin mantle remote_build::tests::coordinator_ -- --nocapture
PASS: 23 passed; 0 failed
Includes state-reset identity separation, missing/tampered durable nonce rejection,
and post-restart admission reconstruction with duplicate cryptographic revalidation.

pueue 1988: cargo test -p mantle --bin mantle remote_farm_config::tests:: -- --nocapture
PASS: 12 passed; 0 failed

pueue 1979: cargo test -p crunch-build --lib distributed::tests:: -- --nocapture
PASS: 41 passed; 0 failed

pueue 1980: cargo check -p mantle --bin mantle
PASS at the repository's existing warning baseline.

pueue 1981: targeted rustfmt --check and git diff --check
PASS
```

Strict first-party Clippy with `--no-deps` remains blocked by the same nine pre-existing diagnostics in `dynamic_plan.rs`, `network_policy.rs`, and `worker.rs`; pueue task 1986 produced no `remote_attempt.rs` diagnostic. The broader command without `--no-deps` stopped earlier in vendored `fuse-backend-rs` (task 1982). No Kani execution is claimed.

The authoritative integrated-main lifecycle rerun passed after the delta-marker repair and adversarial hardening:

```text
validate: valid=true, issues=[], changes=15, specs_validated=39
proposal: PASS, issues=[], receipt_hash=a0442e24bc53b19fe5c7cc77f383d07b2d9ab37ec5a75315893a6d6699dc1107
design: PASS, issues=[], receipt_hash=1a85c2e9784128aa574347accca72730692bc7a4076a806caa545536a2a5bf04
tasks: PASS, issues=[], receipt_hash=4d104181253a9415aecd75164a8f1f7a03a7483e90dc7d813123ca89b9c9229b
```

The native sync dry run planned one `sync_delta_spec` action for `cairn/specs/remote-builds/spec.md` with plan hash `ccd557aa309302bf53fa7b349b77dde71af876136f5aeb7a55c6e23e68bcfec7`. It was non-mutating.

## Executed sync evidence

```text
change: fence-durable-remote-attempts
remote-builds before: d01a674a58991f0017c6d8036ce5ce9efb9242a9ec2ecb4edef72b1a92991c69
remote-builds after: 6a5ec2df5afdfe1abbf480c026f61ea21ede3768b292e9a66b1c36e6db21e455
plan_hash: 0c6e009294f4c4dbd4248263f713c59cb5ebf9342ccc4de113bdccc30792234b
receipt_hash: a01612709b34f205dfa2a333311442a1e90aa4bc9864d84e71ca171f24e05567
mutated: true
blocked: false
```

Pueue task 2001 ran authoritative validation after sync. Exact output:

```text
{
  "change_issues": [],
  "changes": 15,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 39,
  "valid": true
}
```

## Archive evidence

```text
change: fence-durable-remote-attempts
archive path: ./cairn/archive/2026-07-12-fence-durable-remote-attempts
input_hash: 85b7d50e4b85720258e159016924aa488284e213d1c1719dcfd8219e9983c1e1
plan_hash: ae2788a2c7e0888d9d2426d326ac63ed8bcd2d841f21109806bf0859018c93c4
receipt_hash: f067a7bebb76799b66ed5e8a9d4012ac9ddb533be5731034a6a3c470a0f25fb0
mutated: true
blocked: false
```

Pueue task 2004 ran authoritative validation after archive. Exact output:

```text
{
  "change_issues": [],
  "changes": 14,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 38,
  "valid": true
}
```
