# Remote failure-debug bundle implementation validation

Date: 2026-07-13

Change: `emit-remote-failure-debug-bundles`

Requirements:

- `r[operator_diagnostics.remote_failure_debug_bundle]`
- `r[operator_diagnostics.remote_failure_replay]`
- `r[remote_builds.failure_debug_capture]`

## Question

Does the implemented Mantle slice emit bounded metadata-first failure bundles,
capture only explicit pre-cleanup diagnostic artifacts, preserve existing
immutable-log and execution truth, and run replay as a new ordinarily admitted
execution without exposing protected failure context?

## Inspected evidence

### Implementation boundaries

- `crates/crunch-build/src/distributed/remote_failure_debug.rs` owns the pure
  policy, canonical identity, validation, capture admission, inspect,
  replay-plan, comparison, and retention kernels.
- `src/remote_failure_debug.rs` owns private bundle/CAS files, no-clobber
  publication, capability-confined no-follow capture, leases, GC, and
  redacted on-disk inspection.
- `vendor/snix-build/src/buildservice/bwrap.rs` hands a failed ephemeral
  workspace to the worker shell before temporary-directory cleanup.
- `src/remote_build.rs`, `src/remote_transfer.rs`, and `src/main.rs` compose
  coordinator/worker evidence, report separate outcomes, and execute replay
  through current assignment, transfer, fence, output-admission, import, and
  completion paths.
- `lib/remote-builders.ncl` and `src/remote_farm_config.rs` define one bounded
  typed policy; capture and replay are disabled by default.

### Focused pure, shell, policy, log, sandbox, and production tests

Every Cargo command below used the isolated build state required for this
change:

```text
TMPDIR=/home/brittonr/.cache/mantle-drain-targets/failure-debug/tmp
CARGO_TARGET_DIR=/home/brittonr/.cache/mantle-drain-targets/failure-debug
SNIX_BUILD_SANDBOX_SHELL=/bin/sh
```

Pueue task 3154:

```text
nix develop -c cargo test -p crunch-build remote_failure_debug::tests::

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 555 filtered out; finished in 0.00s
```

These tests cover canonical permutation equivalence, malformed/mismatched
facts, metadata-only validation/inspection/replay planning, capture quotas and
redaction, replay comparison, and retention authority.

Pueue task 3156:

```text
nix develop -c cargo test -p mantle --bin mantle remote_failure_debug::tests::

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1460 filtered out; finished in 0.15s
```

The shell rail includes multiprocess atomic/no-clobber publication, final and
parent symlink rejection, capture-object and manifest tampering, secret and
nonempty-environment redaction with replay rejection, clean-process
inspect/replay-plan, missing immutable logs, and active-lease retention.

Pueue task 3155:

```text
nix develop -c cargo test -p mantle --bin mantle remote_farm_config::tests::

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1451 filtered out; finished in 0.04s
```

This includes positive bounded Nickel policy round-trip and negative type
mismatch fixtures.

Pueue task 3157:

```text
nix develop -c cargo test -p mantle --bin mantle remote_attempt_log

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1461 filtered out; finished in 0.26s
```

The existing immutable attempt-log store remains the log authority and keeps
its no-follow, tamper, restart, retention, and publication race coverage.

Pueue task 3189:

```text
nix develop -c cargo test -p snix-build failed_sandbox_

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 0.00s
```

This proves ordinary failure cleanup and the explicitly configured failed
workspace handoff remain distinct.

Pueue task 3188:

```text
nix develop -c cargo test -p mantle --test remote_transfer_production -- --nocapture

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.88s
```

The production rail includes allowlisted capture before cleanup, oversized
capture rejection without execution-truth rewriting, worker/coordinator status
separation, rejected output admission, and successful replay with a new job,
attempt, fence, ordinary import, and immutable original bundle.

After hardening every replay error path to persist ordinary failure state and
release only its own live output claims, pueue task 3222 re-ran the focused
replay fixture:

```text
nix develop -c cargo test -p mantle --test remote_transfer_production \
  failed_output_admission_replays_under_new_fence_without_rewriting_original_bundle \
  -- --nocapture

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 1.23s
```

The fixture additionally verifies that a rejected replay leaves no live output
claim and that the following successful replay has a different coordinator job
and attempt from the original failure.

### Formatting

Pueue tasks 3133, 3201, and 3225 ran `rustfmt --check` over every changed Rust
file in bounded groups plus `git diff --check`. The final formatting command
completed successfully with no output.

### Cairn lifecycle validation

Pueue task 3148:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .

valid=true, changes=8, specs_validated=35, issues=[], change_issues=[], spec_issues=[]
```

Pueue tasks 3149 and 3150:

```text
proposal gate: verdict=PASS, valid=true, issues=[], receipt_hash=ff0e6b2b66d90ce3e6d77e33fb2abba57d7e0960b661fef03969fd005325163b
design gate: verdict=PASS, valid=true, issues=[], receipt_hash=8eb1cd5ed342ec8029b4b79540850dece34bbba35b0ffe05df3ff73517b93900
```

After this evidence and the proven checkoffs were updated, pueue tasks 3233
and 3234 re-ran validation and the tasks gate:

```text
cairn validate: valid=true, changes=8, specs_validated=35, issues=[]
tasks gate: verdict=PASS, valid=true, issues=[], receipt_hash=ccac47073f2ad5c5b81418608962cd3aa43a9358ae3cd274b492d451dfa086ab
```

## Adversarial oracle checkpoint

### Question

Could replay reuse original authority, bundle construction expose secrets,
failed-sandbox capture escape its capability root, retention delete ordinary
outputs, or capture/report degradation rewrite execution truth?

### Inspected evidence

A VibeThinker secondary review was run after the first integration rail and
again after hardening. Its concrete initial findings drove four changes:

1. replay now carries a fresh domain-separated execution binding and receives a
   different coordinator job/attempt/fence;
2. secret-bearing or nonempty-environment actions are redacted and cannot be
   replayed from the bundle;
3. capture opens the root and each file through no-follow capabilities and has
   final/parent symlink fixtures;
4. coordinator metadata, worker bundle, capture, and cleanup outcomes have
   separate report fields.

The final review raised four additional hypotheses. Code and tests bounded
them as follows:

- the capture root is opened by `walk_ambient_directory_nofollow`, and
  `open_file_read_nofollow` uses capability-relative `FollowSymlinks::No` plus
  opened-file type/size revalidation;
- the fresh replay binding participates in normalized request identity, and the
  production fixture proves a different job and attempt;
- retention plans reject ordinary build-output records and preserve active
  leases; replay import remains ordinary store state outside the debug root;
- failure claim release checks that the live claim is still owned by the
  failing job's normalized key before removal.

VibeThinker is advisory; accepted conclusions are based on repository code and
current tests, not model authority.

### Decision

No reviewed hypothesis remains a demonstrated authority, disclosure,
confinement, retention, or execution-truth defect. The explicit non-claim
remains: debug evidence does not authorize execution, admit outputs, prove
determinism, reproduce a sandbox byte-for-byte, or rewrite the original result.

### Owner

Mantle's pure distributed core owns deterministic decisions. The coordinator,
worker sandbox adapter, bundle shell, existing immutable-log store, existing
transfer shell, and existing store importer retain their separate imperative
authorities.

### Next action

Keep the broad first-party lint and Tracey failures recorded as external
validation blockers; do not mark V7 complete unless those exact repository
rails pass.

## External blockers

### First-party Clippy

Pueue task 3151 ran the exact first-party helper:

```text
nix develop -c ./scripts/check-first-party-clippy.sh
```

It failed before producing a clean repository-wide result on unchanged files
outside this slice, including:

```text
error: this `if` statement can be collapsed
  --> crates/crunch-shell-core/src/profile.rs:286:5
error[E0599]: no method named `to_hex` found
  --> crates/crunch-kernelscript-adapter/src/lib.rs:674:62
error: could not compile `crunch-project-core` (lib) due to 13 previous errors
```

The complete captured log also contains unchanged `crunch-project-core`
constant-assertion, collapsible-if, and large-enum findings. This evidence does
not claim first-party lint success.

### Tracey coverage profile

Pueue task 3164 ran:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .
```

It returned `valid=false`. The selected `mantle-default` profile currently has
only `cairn/specs/release-provenance/spec.md` as a requirement source and seven
fixed release-evidence sources; it does not include this active change's
operator-diagnostics/remote-build requirements or the repository Tracey bridge.
It also reports 78 pre-existing missing release-provenance refs and 17 dangling
refs. The active change's three requirement IDs are absent from both lists.
This is a repository profile/global coverage blocker, not evidence that this
slice passed Tracey.

## Decision

I1-I9 and V1-V6 are supported by current implementation and exact focused
evidence and may be checked. V7 remains unchecked because its exact
first-party lint and Tracey clauses are blocked as recorded above. No archive,
spec sync, push, or V7 success is claimed.
