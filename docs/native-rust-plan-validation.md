# Native rust-plan validation

This page records focused validation rails for the `no_std + alloc`
`mantle-rust-plan-core`, the `mantle-rust-plan-app` application, and their
`src/rust_plan.rs` adapter. These rails exercise bounded planning, typed
observations, and local cache behavior; they do not prove full Cargo
compatibility, rustc correctness, release reproducibility, or bootstrap
correctness.

## Host setup

Use the checked-in Rust toolchain and the Nix development environment.
Keep temporary files and build products in writable scratch space outside
the checkout; a full `/tmp` or concurrent Cargo target can invalidate a
validation run without implicating Rust-plan:

```bash
export TMPDIR="$HOME/scratch"
export CARGO_TARGET_DIR="$HOME/scratch/rust-plan-validation-target"
nix develop --offline --no-write-lock-file -c cargo test \
  -p mantle-rust-plan-core --locked --offline
nix develop --offline --no-write-lock-file -c cargo test \
  -p mantle-rust-plan-app --locked --offline
nix develop --offline --no-write-lock-file -c cargo check \
  -p mantle-rust-plan-core --target wasm32-unknown-unknown --locked --offline
```

The core and application packages test the pure policy and orchestration
contracts separately. The target check verifies that core policy compiles
without host filesystem, process, or cache facilities; it does not execute
Rust code under WebAssembly.

The core takes named `NativeCfgEvaluationRequest` and `NativeHostTarget`
inputs for cfg evaluation and host/target selection; the std adapter supplies
those observations. `classify_existing_unit_observations` returns
`InputLimitExceeded` when effects or observations exceed `MAX_UNITS`, rather
than reporting a saturated count as exact. The bounded normal and malformed
cfg, feature, topology, and observation paths remain covered by the focused
core, application, and adapter rails below.

`execute_existing_units` rejects more than `MAX_UNITS` before invoking a unit
executor. This is an application admission failure, not a per-unit execution
observation.

## Focused rails

Serial evidence rail:

```bash
cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=1
```

Parallel evidence rail:

```bash
cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=8
```

Repeated parallel stress rail:

```bash
RUST_PLAN_STRESS_RUNS=3
for run in $(seq 1 "$RUST_PLAN_STRESS_RUNS"); do
  cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=8
done
```

Negative ambient-env rail:

```bash
cargo test -p mantle --bin mantle \
  rust_plan::tests::build_script_child_env_ignores_ambient_profile_env \
  -- --exact --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::rust_plan_tests_do_not_mutate_process_global_environment_in_process \
  -- --exact --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::process_global_env_mutation_guard_detects_set_and_remove_var_calls \
  -- --exact --nocapture
```

Cargo-oracle metadata and unit-graph children are captured with bounded pipe
readers: at most 64 MiB of stdout and 1 MiB of stderr. If either limit is
exceeded, the adapter kills and reaps the child and returns a build failure;
it never waits for an unbounded `Command::output()` capture. The focused
`cargo test -p mantle --bin mantle rust_plan::tests::cargo_oracle_` rail covers
a real small `/bin/sh` child and both oversized pipe directions.

Both Cargo-oracle observations (`metadata` and `build --unit-graph`) use
`--locked`. The supplied `Cargo.lock` is an input, not a file for the planner
to generate or normalize. A semantically valid, noncanonical lock must work
when read-only without changing its bytes; a stale lock must fail rather
than select new dependencies. Dependency acquisition remains subject to the
caller's existing Cargo configuration and sandbox network policy.


## Local castore cache

The local Rust unit cache is optional and disabled by default. Select one mode
with an explicit execution command:

```bash
mantle rust-plan --execute-topology --local-rust-cache read
mantle rust-plan --execute-topology --local-rust-cache read-write
```

`read` permits verified restore operations. `read-write` also permits
publication after successful compiler execution. Mantle keeps prior execution
output reuse first. It checks local castore results second and runs the compiler
only after a miss or rejection.

The action identity includes the declared source, compiler content, toolchain
closure, platform, target, profile, mode, features, semantic arguments,
admitted environment, dependency artifacts, host artifacts, build-script
facts, native-link facts, and compiler policy. An unclassified absolute path
makes that action ineligible for cache reuse. Compilation can still continue.

Each execution receipt can include a `local_cache` report. The report gives the
disposition, reason codes, selected result reference, candidate count, artifact
count, restored and reused byte counts, and whether the compiler ran. A restored
unit gets a new execution receipt. The stored result tree excludes that receipt.

Mantle stores local records under the resolved state directory at
`rust-unit-cache/`. Result records and indexes are private, bounded, and
no-follow. `mantle store gc` validates committed retained results and passes
their castore nodes as explicit roots. GC keeps live Rust result content and
removes unrooted content. It also removes unretained result records. The
operator who runs `mantle store gc` has deletion authority.

This cache does not prove compiler correctness, output reproducibility,
hermeticity, remote trust, or release eligibility. Object presence alone never
authorizes reuse.

Focused cache and performance rail:

```bash
cargo test -p crunch-rust-cache-core
cargo test -p crunch-rust-cache
cargo test -p crunch-store \
  gc::tests::explicit_castore_root_survives_while_unreachable_blob_is_reclaimed \
  -- --exact --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::local_castore_restoration_skips_second_compiler_invocation \
  -- --exact --nocapture
```

The final test prints cold and restored wall-clock microseconds. It also checks
that the restored run does not invoke the compiler.

## Shared Rust result cache

Shared cache use requires an enabled local cache. This rule keeps local output
and local castore reuse ahead of remote transfer.

Focused positive and negative rails:

```bash
cargo test -p crunch-rust-cache shared::tests:: -- --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::clean_client_shared_cache_hit_skips_second_compiler_invocation \
  -- --exact --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::untrusted_shared_candidate_records_rejection_before_compiler_fallback \
  -- --exact --nocapture
```

These tests cover signed publication, clean-client restore, zero additional
compiler calls, fallback evidence, conflicts, offline behavior, redirects,
timeouts, truncation, corruption, bounds, and atomic visibility.

See [`shared-rust-unit-cache.md`](shared-rust-unit-cache.md) for operator
commands, trust policy, protocol paths, receipt fields, and non-claims.

## Fixture inventory

| Fixture family | Shared resource risk | Current isolation rule | Serial-only status |
|---|---|---|---|
| Rustc wrapper fixtures | Wrapper executable path and produced fake artifacts | `TempDir` per test via `write_fake_rustc(...)` | Parallel-safe |
| Compiler-policy fixtures | Provider manifest, adapter report, lint/config files, policy report paths | `TempDir` per test plus report paths scoped by `output_root` and unit ID | Parallel-safe |
| Build-script metadata fixtures | `OUT_DIR`, generated metadata, executable current directory | `build_script_out_dir(...)` scopes by absolute output root and safe unit ID | Parallel-safe |
| Topology execution fixtures | Unit output directories and cached execution receipts | Per-test `TempDir` output roots; reuse tests intentionally reuse only within one test | Parallel-safe |
| Ambient profile env negative test | Process-global env values such as `OPT_LEVEL`, `DEBUG`, and `NUM_JOBS` | Parent spawns an exact child test with conflicting env; no in-process `set_var` / `remove_var` calls in `src/rust_plan.rs` tests | Parallel-safe |
| Cargo shim / Cargo-free proof scripts | Failing shim marker and execution root | Script-created proof bundles and explicit `--execution-output-root`; keep bundle roots outside the source tree | Not part of focused unit-test rail |

There are no remaining known serial-only `src/rust_plan.rs` focused tests. The
serial rail remains documented as a baseline evidence command, not because a test
requires serialization.

## Evidence rules

- Claim parallel safety only from a same-tree parallel or repeated-parallel
  transcript.
- If a test becomes serial-only, name the shared resource here and add a next
  action for isolating or locking it.
- Do not replace the negative ambient-env subprocess pattern with in-process
  process environment mutation.

## Ordinary Cargo differential and wait evidence

The separate `mantle-rust-differential` package owns an ordinary, offline Cargo
research fixture runner, not a Mantle derivation builder. It does not relax the
source-built proof's global no-Cargo policy. Its pure
`mantle-rust-differential-core` companion is `no_std + alloc`; the shell owns
filesystem, clocks, compiler execution, containment and diagnostic publication.
The harness and Rust-plan maintainers own toolchain/schema compatibility review.

The standalone executable is built with its explicit Cargo wrapper (this command
is separate from the source-built no-Cargo proof):

```sh
cargo build --offline --locked -p mantle-rust-differential -p crunch-rustc-wrapper --bin mantle-rust-differential --bin mantle-rust-wait-wrapper
target/debug/mantle-rust-differential fixtures
target/debug/mantle-rust-differential baseline "$DIFFERENTIAL_OPTIONS"
target/debug/mantle-rust-differential serial "$SERIAL_OPTIONS"
target/debug/mantle-rust-differential negatives "$NEGATIVE_OPTIONS"
target/debug/mantle-rust-differential cohorts "$COHORT_OPTIONS"
```

Each options input is a JSON object, bounded to 64 KiB, with these declared
fields (unknown fields are rejected):

| Field | Required admitted value |
| --- | --- |
| `root` | Absolute nonexistent evidence directory; its parent already exists and has no symlink ancestors. Every command and repeat needs a different root. |
| `cargo`, `rustc`, `linker` | Resolved absolute paths to the actual executables, not rustup proxies or unresolved symlinks. |
| `sysroot` | Resolved absolute complete sysroot matching that rustc's `--print sysroot`. |
| `host` | Exact rustc `--version --verbose` host triple. These fixtures use that host, not a cross target. |
| `path` | Explicit colon-separated absolute tool directories, including the linker's subordinate tools. There is no inherited PATH. |
| `closures` | Nonempty JSON array of resolved absolute runtime/tool closure roots, including linker/CRT/runtime dependencies; contents are hashed before execution. Symlink referents must also be covered by declared roots. |
| `wrapper` | `null` for completion-only baseline/negatives, or the resolved absolute built `mantle-rust-wait-wrapper` path. Required for cohorts; wrapper contents are pinned, not discovered ambiently. |
| `baseline` | `null` for a new baseline freeze. For a baseline repeat only, the previous retained `coverage-baseline.json` path; cohorts and negatives reject this field when non-null. |
| `toolchain` | Optional; the runner computes and replaces this digest from the actual tools, sysroot, linker and declared runtime closures. A caller cannot assert its identity. |
| `diagnostics` | Optional boolean, default `false`. Explicitly enables native serial sidecars. Serial requires `wrapper: null`; Cargo observation is enabled only by its declared wrapper. |

All commands use the frozen built-in offline source/lock/edit recipe emitted by
`fixtures`; they do not fetch registry packages or accept arbitrary commands.
Tool probes use the same cleared child environment as compilation and check the
declared host/sysroot. An ordinary unpatched compiler with Cargo, a functional
host linker/CRT/runtime and Linux `/proc`/subreaper/signal support are genuine
execution prerequisites. No experimental compiler input is needed. This is not
a network sandbox or proof that an undeclared host dependency cannot be read.

Cargo still reads ancestor `.cargo/config.toml` files with a private
`CARGO_HOME`. The runner explicitly empties both `RUSTC_WRAPPER` and
`RUSTC_WORKSPACE_WRAPPER`; only the declared diagnostic wrapper may replace
`RUSTC_WRAPPER`. The frozen environment identity includes this policy, so retain
old reports and create new baselines rather than reusing a pre-isolation
baseline. This disables implicit compiler wrappers, not all filesystem Cargo
configuration, and does not establish hermeticity.

`baseline` retains `report.json`, all per-attempt records/raw captures, frozen
fixtures/manifest/coverage baseline, actual crash/cancel triggers, and independent
post-fault successor/clean references. A successor starts fresh Cargo state but
retains the failed predecessor evidence and source identity; it never erases the
interruption or claims uninterrupted continuity. Old state excluded from the
successor is retained on disk for inspection, not secretly deleted. Unit reuse
and execution coverage refer to the cumulative campaign, not work borrowed from
its clean reference. Dependency and application error/repair states are distinct.

`serial` retains `serial.json` and runs a separately labelled direct-rustc
whole-unit lane for the four-library chain and eight-library wide join, in
independent build/check cells. It really compiles the host proc macro and build
script, executes the script against its declared input, and waits for each unit
to finish before starting its dependent. It is not Cargo `--jobs=1`, an
early-interface mode, a resident compiler, or a replacement production adapter.
Every cumulative edit has a separately materialized clean reference and its
frozen expected current program value or compiler error. All regular units
re-enter rustc on every step with side-private incremental scratch; no internal
query reuse or disk-result-cache hit is inferred. Final outputs are step-owned,
so a failed current compile cannot execute a prior binary. Required successful
step units must be observed, even when surviving program outputs still agree.
Expected rejection steps retain their legitimately unrun downstream units.

For an exact repeat, give `serial` a new root and the first serial run's
`coverage-baseline.json`; do not feed it the Cargo lane's distinct baseline.
Use `diagnostics: true` for actual native spawn/exit/reap sidecars. These contain
no fabricated queue, metadata, pause/resume or live-error boundaries. Individual
unit captures retain sampled process-tree resources with their own origins;
these are not merged into an invented session peak or idle footprint. Diagnostic
publication failure and incomplete observations do not alter compiler outcome
or semantic equivalence. Incomplete process teardown prevents continuation.

Production native, default Cargo and dynamic per-unit collection have separate
explicit opt-ins described below. The collector and sole wait-wrapper binary
are owned by `crunch-rustc-wrapper`; the standalone differential executable is
not a production execution adapter. Its evidence cannot prove a production
sandbox or helper ran. See ADR 0125 for authority and comparison boundaries.

`negatives` actually compiles clean successful and rejected snapshots, then
retains explicitly labelled adversarial comparator inputs for alleged rejection
of accepted source, wrong/reordered/duplicated diagnostics, stale accepted output
and incomplete capture. It also executes actual bounded children with a zero
stream ceiling and expired deadline, verifies missed/wrong edits, and exercises
lost population/denominator/required-skip/fallback rejection. `negatives.json`
distinguishes these policy injections from raw compiler observations. Its ordinary
population deliberately remains nonpassing/unrun outside the compiles it ran;
`all_negatives_observed` is not ordinary matrix acceptance. Unavailable distinct
real diagnostic records leave the reorder case nonpassing, never synthesized.
Crash/cancel/stale-artifact seeding belongs to the real `baseline` fault campaign.

`cohorts` retains `cohorts.json` and predeclared per-cohort plans/ledgers. It runs
five alternating AB/BA pairs for chain/wide host dev builds in each clean, warm
and first-body-edit incremental state. A and B have identical instrumentation:
this measures repeatability, **not instrumentation overhead or an optimization**.
Unmapped compiler work, dropped/incomplete diagnostics or a publication failure
cannot qualify work coverage. Real failures/timeouts keep their actual outcome;
no absent or unsupported compiler phase is counted as observed work.

Exit 0 means only that the requested campaign's own qualification predicate
passed; exit 1 retains a nonpassing report; exit 2 names a prerequisite or harness
error. None of these exits admits a future early-publication mode, resident
provider, production per-unit compatibility, release result or speedup.
Repeat either baseline lane in fresh roots with its own original coverage
baseline input and inspect the actual reports before making any acceptance claim.

The frozen `mantle-rust-differential-manifest-v1` binds the source recipe,
config/lock/local dependency closure, rustc/Cargo/sysroot/tool closures, ordered
arguments, child environment policy, profile/features/cache/mode, edit recipe,
comparison policy and harness identity. Freeze it before observing results.
Retain the original baseline separately; `reconcile` rejects replacement,
disappeared cases/tests/units and changed denominator membership even if counts
stay constant. Candidate, required/optional, eligible/nonroot, attempted,
executed, reused, fallback, excluded, failed and unrun dimensions overlap;
retry events never inflate unique-member coverage. Optional unregistered future
rows remain unsupported outside the ordinary required denominator. Required
blocked, unsupported, fallback-only or unrun cells are not passing skips.

The admitted ordinary recipe has a four-library chain and eight-library wide
join, with actual local proc-macro and build-script generated inputs. Build and
check have independent cells; selected thin/fat LTO rows are release builds,
not claims that check executes LTO. Each side retains its own cumulative Cargo
incremental state through cold, unchanged, forced compiler re-entry, body,
interface, opaque, generic, const, async, broken/still-broken, repair, structural,
upstream, included Rust/binary, macro/generated-input, revert and reapply states.
Every applicable step uses an independent frozen clean snapshot and expected
current program value or exact error code/primary message. No executable is
run after an unsuccessful compilation; a genuine old executable is adversarial
state, never current-success evidence. An unavailable forced re-entry or missed
fault trigger is nonpassing, not inferred from Cargo freshness or a timer.

`ordered-rustc-semantics-v1` preserves diagnostic level/code/message, primary
and secondary spans, notes, suggestions, applicability, replacements, rendered
semantics, per-unit order and duplicates. Only exact declared private root
roles and unrelated cross-unit interleaving are normalized; timing/progress
records remain raw evidence outside diagnostic comparison. Unknown fields,
malformed/truncated capture, ICE, cleanup failure and infrastructure errors
cannot become equivalent expected compiler rejections. The operation-specific
accepted output v2 projection binds the exact current source and toolchain,
operation/status, structured unit artifact facts, diagnostics and current
executable behavior for build. Raw artifact
digests remain evidence; this semantic projection is **not raw-byte artifact
reproducibility**, a signed result or release authority.
Two successful observations without a current accepted-output identity are
nonpassing, not empty-set artifact equivalence. Ordinary Cargo rejection is
exit 101; direct-rustc rejection is exit 1. Other exits, signals, deadlines and
incomplete capture/cleanup remain distinct infrastructure/fault observations.

Core ceilings are 32 cells, 64 steps per cell, one concurrent cell, 64 owned
processes, 256 source files/4 MiB source, 1 GiB per artifact tree, 2 GiB live
scratch, 8 MiB retained bytes per raw stream, 4,096 diagnostic records and
1 MiB edits. Probe/build/program/total deadlines are at most 10 s/120 s/10 s/
24 h; teardown is at most 5 s. The shell may select stricter limits and records
them in its frozen manifest. Bound exhaustion remains a typed harness failure
with retained partial evidence and cleanup. Reference execution trees can be
removed only after evidence finalization and proven teardown; cumulative state
must never be silently deleted to hide a failed edit transition.

Wait collection is explicit opt-in and writes the separate sanitized
`mantle-rust-wait-diagnostics-v1` sidecar, never causal trace v1 or a receipt.
Compiler package/target/host-role/features/profile association requires the
explicit wrapper invocation handshake and exact Cargo structured facts;
ambiguous requests remain unmapped. Actual launch/exit/live structured error
receipt offsets use one invocation-local monotonic clock. Completion-only
capture cannot provide first-error, metadata-ready, enqueue, pause or resume
times. Unsupported compiler phases and paused residency remain unavailable,
not zero or reconstructed from files/mtimes/process completion. Process-tree
sampling names its CPU/RSS scope, sampling gaps and observed-peak lower bound;
summed RSS can double-count shared mappings. A receive interrupted by a signal
is retried: it consumes no record and does not end capture. A failed diagnostic
sink never changes ordinary compilation, output acceptance, scheduling or cache
identity. See the
[wait capture release note](release-notes/rust-wait-capture-2026-10-25.md).

Diagnostic budgets include 4,096 unit identities, 16,384 attempts, 65,536 shared
event/resource records, 2 KiB per record and 16 MiB per sidecar; sampling is no
faster than 100 ms. Overflow reserves bounded terminal/incomplete reporting and
does not block compiler cleanup. Sidecars contain opaque typed identities and
safe/redacted labels, not raw argv, environments, host paths or source text.
Raw compiler captures live separately under private evidence roots.

Request identities hash input names as exact byte sequences, not JSON object
keys, and canonically order unordered build constraints. Non-UTF-8 names remain
distinct; adding an input or changing its node changes the diagnostic identity.
This encoding is diagnostic-only and does not change derivation, output, cache,
attestation or receipt identities.

The native collector applies stricter per-attempt ceilings: 256 identities and
attempts, 4,096 events and 2 MiB encoded event bytes, with 512 terminal event
slots and 1 MiB terminal-byte capacity reserved from optional observations.
Missing start/exit/reap endpoints leave the timeline explicitly incomplete.

Repeated measurements predeclare five alternating AB/BA pairs per exact
fixture/operation/profile/cache cohort. Retain setup, warm-up, first-edit,
failed, censored and excluded runs, including incomplete sessions. Only
successful pairs with equal inputs/tools/resources and complete independent
equality/work qualification contribute paired deltas or medians/min-max/counts.
Do not pool clean, warm, incremental, build/check, different-provider or
instrumentation-overhead treatments. Five pairs do not establish tail latency;
the distribution policy publishes no extrapolated percentiles. Timings are
excluded from all semantic, receipt and cache identities and never choose work.

Headstart `ff81bfecda2116d54776a7fec538fc2f4e349d1f` retains the documented
**UNFIXED** incremental SVH disagreement; rustd
`131de832c0035b3214289eb65e6a945342180bd8` remains conceptual provenance. No
patches/scripts or provider implementation are imported. A future early-mode
registration requires one identical Cargo/rustc/sysroot/tool closure on both
sides, declared toggle-only differences, observed capability and separate real
after-publication failure/swap/crash/cancel/stale-output/incremental-clean proof.
An authored registration is not evidence those obligations ran. Ordinary
delivery is independent of that unavailable mode.

These are source contracts, **not a successful experiment or speedup claim**.
Current acceptance requires executing the documented driver twice in fresh roots,
retaining full reports/raw captures and observing fault cleanup and negative
reports, followed by the parent-owned focused tests/no-std/quality/Cairn gates.
No full Mantle self-hosting, compiler correctness, network sandboxing, source
admission, cache trust or release reproducibility follows from baseline parity.

## Production native wait diagnostics

The ordinary `mantle rust-plan` executor accepts explicit
`--rust-wait-diagnostics /absolute/private-new-directory` for supported local
execution. This is the production adapter, not the standalone differential
fixture runner described above. Direct native execution does not use a Cargo
wrapper and rejects `--rust-wait-wrapper`; compiler-policy adapters are not a
supported diagnostic transport. Strict/proof/remote contexts must be refused
by root admission before state, output, or diagnostic-directory mutation.
The selected `--rustc` must be an explicit absolute regular executable no larger
than 512 MiB. Its content is streamed once through a fixed 64 KiB buffer before
the diagnostic directory is created; this diagnostic-only pin joins the existing
admitted toolchain/resolved-process identities without changing those identities.
There is no ambient compiler search or wrapper injection.

Disabled execution constructs no native collector, socket, resource sampler,
or sampling thread. Enabled execution retains the same admitted compiler
arguments, cleared child environment, pipe capture and wait contract. It does
not apply the research supervisor's deadlines, resource limits or truncation.
Exact selected-unit/configuration and admitted resolved-process identities are
hashed into diagnostic-only unit/attempt records. The serial adapter observes
request and dispatch, actual successful spawn, completion-observed exit status,
and direct-child reap/pipe drain. It has no ready queue, live compiler-phase
protocol, or precise first-error receipt. Those durations remain unavailable;
request-to-dispatch is not evidence of an actual ready-queue interval.

Native resource samples cover the invocation's CLI process tree, including its
compiler descendants and the CLI itself, at no faster than 100 ms. They are
not isolated compiler-only RSS or complete CPU accounting. Short-lived processes
can be missed; summed RSS can double-count shared mappings; sampled peaks and
CPU observations are sampled lower bounds with retained gaps; CLI CPU counters
are cumulative and can include setup before collector construction, not an
isolated interval delta. Direct-child reaping does
not establish last-descendant cleanup, so tree lifetime/error drain stays
censored where no such boundary is observed. Unsupported phases, pause/resume,
and first-error time are unavailable, not inferred from stderr or exit.

### Parent-owned production smoke recipe

These commands are a recipe, not retained execution evidence. Set `MANTLE` and
`RUSTC` to the pinned absolute executables, `FIXTURE` to an existing admitted
small offline workspace, and `SCRATCH` to an existing absolute scratch parent
outside that workspace with no symlink ancestors. All named diagnostic/output
directories must initially be absent. Keep source/tool/semantic environment
fixed throughout. For a single-library fixture, substitute
`--execute-first-supported-unit` for `--execute-topology` in the first two runs;
the topology flag exercises the ordinary serial local unit session on a chain
or wide workspace.

```sh
"$MANTLE" --json rust-plan --root "$FIXTURE" --rustc "$RUSTC" \
  --no-cargo-oracle --execute-topology \
  --execution-output-root "$SCRATCH/native-output" > "$SCRATCH/native-off.json"
mv "$SCRATCH/native-output" "$SCRATCH/native-off-output"
"$MANTLE" --json rust-plan --root "$FIXTURE" --rustc "$RUSTC" \
  --no-cargo-oracle --execute-topology \
  --execution-output-root "$SCRATCH/native-output" \
  --rust-wait-diagnostics "$SCRATCH/native-on-waits" > "$SCRATCH/native-on.json"
cmp "$SCRATCH/native-off.json" "$SCRATCH/native-on.json"

# Existing-output reuse: keep the successful output directory in place.
"$MANTLE" --json rust-plan --root "$FIXTURE" --rustc "$RUSTC" \
  --no-cargo-oracle --execute-topology \
  --execution-output-root "$SCRATCH/native-output" \
  --rust-wait-diagnostics "$SCRATCH/native-reuse-waits" > "$SCRATCH/native-reuse.json"

# Seed then restore an eligible local-castore fixture in the same explicit state.
"$MANTLE" --json --state-dir "$SCRATCH/native-state" rust-plan \
  --root "$FIXTURE" --rustc "$RUSTC" --no-cargo-oracle --execute-topology \
  --local-rust-cache read-write --execution-output-root "$SCRATCH/native-cache-output" \
  --rust-wait-diagnostics "$SCRATCH/native-seed-waits" > "$SCRATCH/native-seed.json"
mv "$SCRATCH/native-cache-output" "$SCRATCH/native-seeded-output"
"$MANTLE" --json --state-dir "$SCRATCH/native-state" rust-plan \
  --root "$FIXTURE" --rustc "$RUSTC" --no-cargo-oracle --execute-topology \
  --local-rust-cache read --execution-output-root "$SCRATCH/native-cache-output" \
  --rust-wait-diagnostics "$SCRATCH/native-restore-waits" > "$SCRATCH/native-restore.json"
```

Inspect each enabled `rust-waits.json` separately from ordinary JSON receipts.
The off/on pair must preserve ordinary plan, receipt and output identities; its
diagnostic invocation/clock/timing values are intentionally fresh. Reuse and
eligible castore restores must show their actual cache disposition and
`CacheRestored` without `Started`/compiler exit/reap events for restored units.
Cache reports and reuse reasons legitimately differ from a compilation receipt;
do not compare those different dispositions as byte-identical receipts. Confirm
actual cache eligibility/hits rather than treating a rejected cache candidate
followed by compilation as a restoration. Repeat with a real compiler error and
retain the ordinary failure plus unavailable precise first-error timing. No
commands in this section assert completed verification or a performance result.

