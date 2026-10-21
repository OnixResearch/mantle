# Unified Rust libtest targets

The `mantle-test-target-v1` workflow supports **one framework/cohort**, Rust
1.90.0 libtest JSON with LLVM 20.1.8. It is not a universal test runner, Cargo
replacement, proof of formal correctness, or release/parity evidence. Coordinate
execution with issue #15's accepted immutable Linux baseline before making a
baseline claim. The scenario driver records that baseline receipt without
reinterpreting its authority.

## Execution and authority

A test target lowers to one ordinary input-addressed derivation. The existing
Mantle worker owns scheduling, strict network/filesystem/PID isolation, resource
admission, signing, and reuse. There is no separate scheduler, output store, cache
trust policy, or silent external service permission. Linux `x86_64-linux` and
`aarch64-linux` are admitted; Darwin is explicitly unsupported by this cohort.
Running `mantle-test-worker` directly does **not** establish isolation or signed
result authority.

The operator operation is:

```sh
mantle --json test target.json --filter pass_ --shard-index 0 --shard-count 2
mantle test target.json --filter pass_alpha --exact --timeout-ms 500 --seed 17 --repeat 2
mantle test target.json --filter pass_ --coverage --force-rerun operator-run-20261018
```

Unknown fields/options/frameworks are rejected. The declaration fixes the test
inventory; ignored tests, benchmarks, doctests, arbitrary libtest arguments,
custom harnesses, dynamic test discovery, retries, and external-service tests are
not supported. Selection sorts names, applies an optional substring (or exact)
filter, then assigns ordinal modulo shard count. Empty selections are errors,
not vacuous passes. Shards are zero-indexed; count is 1–256, repeat 1–16, maximum
inventory 256. `seed` is exposed as `TEST_SEED`, repeat ordinal as `TEST_REPEAT`;
these are fixture inputs, not a claim that libtest itself randomizes tests.

Timeout is a **per-selected-test process deadline**, bounded by the declaration
(maximum one hour); it is not post-hoc elapsed classification. Every exact name
runs in its own process group with one libtest thread. The worker polls a
monotonic deadline, kills the group, waits for the harness, and as Linux subreaper
reaps adopted group descendants within a five-second teardown bound. The outer
strict sandbox owns the full PID namespace (including descendants that attempt
to escape the group). SIGTERM/SIGINT requests produce cancellation if delivered
to the worker; abrupt worker/sandbox loss remains infrastructure failure.
Memory uses `RLIMIT_AS`; capture bytes and file-size limits are enforced, with
bounded collection. Inherited environment is cleared. Only declared `TEST_*`
keys are accepted (reserved seed/repeat keys cannot be overridden). HOME,
TMPDIR, cwd/runfiles, JSON mode and the scoped `RUSTC_BOOTSTRAP=1` exception are
fixed runner options.

SIGINT/SIGTERM delivered to `mantle test` request cancellation of its owned
build. After acknowledged teardown, the CLI exits 130, writes
`test-cancelled-after-owned-teardown` to stderr, and leaves stdout empty, including
in JSON mode. It does not synthesize a test report from the interrupted build.
That CLI acknowledgement is distinct from a worker-produced `cancelled` outcome
in an admitted signed report.

## Declaration

`crunch-test-core::Target` is the closed serde declaration. Required fields:

- `schema = "mantle-test-target-v1"`, identifier `name`, `platform`, and
  `store_prefix`;
- `cohort = "rust-1.90.0-llvm-20.1.8-libtest-json-v1"` and the exact producer's
  `rustc_verbose_version` (rustc 1.90.0 commit `1159e78c4`, date `2025-09-14`, LLVM
  20.1.8);
- `executable` and `runner`: `{root, member, blake3}`, where member is a safe
  relative path and BLAKE3 identifies exact executable bytes;
- `inputs`: map of **every admitted store root to its exact NAR BLAKE3**,
  including executable, runner, dynamic linker/libraries, runfiles, source and
  coverage tools; claimed hashes must be checked by existing store authority
  before building, not trusted because they appear in JSON;
- `runfiles`: relative cwd aliases to declared input roots; `environment`: at
  most 64 explicitly declared `TEST_*` values;
- `tests`: unique exact libtest names; `resources`: `{memory_bytes,
  output_bytes, timeout_ms}` (memory 16 MiB–64 GiB, capture 1 byte–1 MiB);
- `deterministic = true`, `external_services = false`, and `coverage` either
  null or the opt-in contract below.

Tool members are byte-hashed before execution. Symlinks must resolve inside an
admitted input root. Runtime library roots must also be declared; missing ones
are infrastructure failures rather than ambient fallback.

The Rust API admits one owned request with `crunch_test_core::select(Target,
Invocation) -> Result<SelectedRequest>` or the filesystem adapter's
`crunch_test::load_target(&Path, Invocation)`. The request carries the target,
invocation, sorted selection, and unchanged wire identity. Input admission still
checks `request.target.inputs` against Mantle's store authority.
`write_recipe(SelectedRequest, &Path)` returns that request, which is then consumed
by `read_report(&Path, SelectedRequest)` after signed-result admission; neither
operation repeats target validation or identity serialization. Worker execution
also consumes the selected request. `crunch-test-app::TestPort` remains the
no_std application effect port, not a functional-core public trait.
`cache_eligible(Outcome)` checks the typed outcome, and `summary(Report)` returns
`ReportSummary { report, text }` without copying the report.

## Results and reuse

`mantle-test-report-v1` is stable JSON: target/invocation identity, sorted selected
names, every `{repeat, test, outcome, reason}` attempt, aggregate outcome, and
optional coverage identity/line totals plus a separate `collection_failure` when
preparation or coverage fails. Actual test attempts are retained even if later
coverage collection fails. There are no retries or hidden flaky
reclassification. Raw durations, PID values, absolute work paths, and test stdout
do not enter the stable report. `summary.txt` is the human-readable counterpart.

Outcomes: `pass` (exit 0), `assertion-failure` (1), `timeout` (124), `cancelled`
(130), and `infrastructure-failure` (2). Exact started/terminal events, expected
name, suite counts, newline-complete JSON, and matching libtest exit are required.
Crash signals, early exit 0, extra/duplicate events, ignored tests, partial output,
unknown grammar, or malformed summaries can never become success. Coverage-tool
or preparation failure is infrastructure unless its process observation records
a deadline or cancellation; these retain the corresponding distinct outcome.
No partial capture is published as a successful test.

All inputs, full declaration, filter/shard/deadline/seed/repeat/coverage options,
and optional rerun token enter the ordinary derivation identity. Existing signed
output admission remains mandatory for fresh and cached results. Report readers
check exact expected identity, complete ordered attempts, aggregate consistency,
and coverage identities. They are **not** signature verifiers. Only complete
pass/assertion results are eligible for reuse; timeout/cancel/infrastructure
observations are not. A reused ineligible output is rejected with a rerun-token
diagnostic, never silently retried. `--force-rerun TOKEN` changes derivation
identity; operators must use a new token for each forced run. It does not disable
signature validation or automatically generate unrecorded randomness.

## Opt-in coverage

The same fixture must be compiled with `-C instrument-coverage`, using rustc
1.90.0 and its matching `llvm-tools-preview` LLVM 20.1.8 cohort. The target binds:

```text
coverage = { profdata: Tool, cov: Tool, source_root: StoreRoot,
             source_blake3: NarBlake3, max_profile_bytes: 1..67108864 }
```

The runner checks tool byte identities and version output, bounds raw profiles
(4096 files, 64 MiB aggregate maximum), merges profiles from all selected repeat
attempts with `llvm-profdata merge -sparse`, then uses `llvm-cov export
--summary-only` filtered to the exact declared source root. A fresh per-run
profile directory prevents ambient profile reuse. The output is the real LLVM
coverage export, merged profdata, exact tool/source identities, export BLAKE3,
and covered/total line counts. Export schema is pinned to
`llvm.coverage.json.export` version `2.0.1`; output bytes remain bounded by target
capture policy. Coverage is aggregated within this selected invocation, not
across unrelated targets or tool cohorts. Failed assertions may contribute
coverage; crashes/deadlines/cancellation do not produce a claimed complete
coverage report. Coverage does not measure correctness.

## Real fixtures and parent execution

`tests/fixtures/unified-test/fixture.rs` contains actual Rust tests for two
passes, an assertion failure, owned-child timeout, denied undeclared host input,
crash, and early process exit. Its ignored `timeout_child` is an implementation
helper, not an admitted selectable test. `compile.sh` builds the instrumented
binary using exact admitted RUSTC/SOURCE/LINKER/MKDIR and ordinary strict derivation
inputs; its POSIX shell and `mkdir` runtime must be explicitly admitted too.
No fixture substitutes shell text for libtest JSON.

Prerequisites for the parent on the admitted Linux host:

1. Accepted #15 baseline, immutable source containing these packages and fixture,
   built `mantle` with the `test` CLI integration, and built `mantle-test-worker`.
2. Admitted store closure for Rust 1.90.0, matching LLVM tools, native linker and
   all dynamic runtime libraries. No download or toolchain resolution occurs in
   the test worker. Obtain actual executable BLAKE3 and root NAR BLAKE3 from
   admitted producers; do not paste placeholder identities.
3. Build fixture.rs in an ordinary strict Mantle derivation with compile.sh;
   bind fixture source store root in its coverage mapping (no source relocation
   after instrumentation), then create the complete target declaration above.
4. Run:

```sh
python3 scripts/prove-unified-test.py \
  --mantle /absolute/immutable/mantle --target /absolute/admitted-fixture-target.json \
  --state-dir /admitted/state --store /admitted/store \
  --scratch /new/scenario-directory --baseline-receipt /accepted/issue15/receipt.json
```

The driver runs actual `mantle test` operations: pass, repeated identical cache
lookup, nonce rerun, changed-env identity invalidation, assertion, deadline,
crash, partial output, disjoint shards, seeded repeats, real coverage, and a
host-present sentinel that must be invisible inside the sandbox. It retains
stdout/stderr, exact commands, reports and baseline hash. Equal repeated reports
prove stable content, **not by themselves a cache hit**: retain the existing
worker's signed cache-admission observations alongside this evidence for that
claim. The driver's 120-second subprocess guard is emergency proof-driver
cleanup and is never counted as a test deadline observation.

Parent verification also exercises SIGINT/TERM cancellation through the existing
owned build lifecycle, checks no fixture descendants survive namespace teardown,
and tests stale/untrusted signed-store admission through the existing trust
suite. Contract regressions reside in `crunch-test-core/tests/contract.rs`.
No fixture execution or baseline acceptance is claimed merely by checking in
these sources.
