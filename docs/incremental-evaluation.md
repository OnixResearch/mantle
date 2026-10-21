# Persistent evaluation: benchmark and admission contract

Issue [#19](https://github.com/OnixResearch/mantle/issues/19) concerns evaluation,
not the scheduler's existing goal retention or Rust build caches. The research
runner in this change intentionally uses **fresh** sessions. There is no new
incremental command, production result cache, or performance claim.

## Reproduce before choosing reuse

Build the example from the exact reviewed source in the canonical development
shell, recording the source revision, Cargo.lock digest, toolchain, executable
digest, OS/architecture, command, exit status, and raw output. Final-source
baseline acceptance remains issue #15's responsibility.

```sh
cargo build --locked --offline -p crunch-eval --example evaluation_baseline
python3 scripts/benchmark-incremental-evaluation.py \
  --runner "$CARGO_TARGET_DIR/debug/examples/evaluation_baseline" \
  --output "$RUN/issue19-baseline.jsonl" --rounds 5 --roots 32 --work 512
"$CARGO_TARGET_DIR/debug/examples/evaluation_baseline" --context-probe
```

`RUN` must exist, `CARGO_TARGET_DIR` must identify the admitted build target,
and the output file must not already exist. The runner stays alive for one
round; every request opens a new EvaluationSession. Rounds start new processes.
No large downloads, remote execution, Nix evaluation, or compiler invocations
occur during measurement. A bounded fixture generator produces separate leaf
imports, their shared import, and an unrelated file. The ordered cases are
cold, unchanged, one leaf edit, shared-import edit, goal addition/removal,
unrelated edit, deletion/recovery, rename plus import change, unresolved
import, ordered import-path configuration change, invalid source, and recovery.
The root count is bounded to 2..64, work array to 1..4096, and rounds to 1..100.

The separate eager API evaluates and converts again in a new Context and checks
**full typed derivations and derivation paths**, not just labels, against the
session result. Its cost is reported separately as `oracle_ns`. This is a
fresh-session-vs-eager oracle, **not** a fresh-vs-incremental acceptance proof.
`changed_labels` describes semantic changes rather than presumed invalidation.
Each failed request reports an error, never roots from an earlier success.
Research responses never authorize production watch output.

### What the numbers mean

- `scan_ns`: parsed dependency scan, separate from evaluation.
- `evaluation_ns`: session discovery plus serial deep export/deserialization.
- `planning_ns`: conversion through a fresh ConversionCache.
- `candidate_wall_ns`: scan/evaluation/planning, excluding oracle and protocol.
- `request_wall_ns`: parent elapsed time including transport, oracle, serialization.
- `rss_kib`: Linux current resident memory after both candidate and oracle.
- `process_peak_rss_kib`: Linux process-lifetime high-water mark, **not** an
  independent per-edit peak or retained-cache size. Other platforms report null.
  Run an individual request in a fresh process under a platform resource monitor
  when independent phase peaks are required. Do not subtract high-water marks.
- `explicit_root_forces` counts API calls, **not** executed or reused Nickel
  thunks. `thunk_evaluations` is null because the stable API supplies no such
  counter. `reused_evaluation_nodes=0` is the fresh mode's policy, not a measured
  attribution of Nickel's internal work.
- `compile_count=0` and `builder_execution_count=0`: this runner never executes
  a compiler or builder. Do not report either as avoided compilation.

Optionally supply `--mantle "$IMMUTABLE_MANTLE"` to also build these shell-only
fixtures through actual one-shot `--json build --no-substitute` calls against
one temporary store/state per round. This needs Linux sandbox prerequisites and
an admitted statically linked sandbox shell. Builder execution and cached
outcome counts derive from the actual report's `outcomes[].cached`; compile
count remains zero because the fixture has no compiler action. These are
one-shot cache observations, **not** measurements of the live watch scheduler,
not total dependency-action counts, and not issue #2 Cargo unit measurements.
Build wall time includes its own evaluation. Neither the research runner nor
these counts substitute for real watch cancellation/output-admission proofs.

Summarize distributions for each case across rounds, preserving raw rows; do
not claim a speedup from cache hits, one sample, RSS deltas, or oracle-inclusive
wall time. No cold/fresh-vs-persistent measurements have been accepted in this
document. There are consequently no measured speed/memory tradeoff claims.

## Dependency and configuration contract (before reuse)

An admissible future reuse unit must identify both the computation and all
inputs it observed. A whole-root final-value cache cannot be represented as
fine-grained Nickel evaluation. The existing dependency scan is a conservative
**watch hint**, not a per-thunk dependency graph or evaluation authority.

| Input | Identity and invalidation requirement |
|---|---|
| Root and transitive sources | Lexically normalized absolute path, input format, exact bytes; edit/delete/rename/replacement invalidates affected computations and their reverse dependents |
| Import resolution | Importer directory then ordered search paths; creating an earlier candidate, removing the selected candidate, changing format, or changing symlink target requires re-resolution even when formerly selected bytes are unchanged |
| Embedded stdlib/evaluator | Exact executable/Cargo lock/embedded library identity; the human version string is not sufficient |
| Environment | The stable embedded Context does not read `NICKEL_IMPORT_PATH`; callers may resolve it into explicit paths. Record actual adapter inputs, working directory, effective paths, and any other environment consumed by the shell. Configuration changes cannot reuse an old context silently |
| Evaluation configuration | Backend/feature cohort, source diagnostic name, selection/forcing mode and limits. Changed limits must be enforced even on a hit |
| Conversion configuration | Logical store prefix, conversion semantics, source/plan-output resolver inputs; conversion identities are separate from Nickel thunk identities |
| Build/trust configuration | Remains owned by existing build/store/scheduler admission, never authorized by an evaluation-cache hit |

The public `EVALUATOR_VERSION` string is not a substitute for the resolved crate
identity: the current declaration says 2.0.0 while crunch-eval's dependency is
nickel-lang 2.2.0. Benchmark manifests must record the actual Cargo.lock and
executable, rather than promote that constant to sufficient cache identity.

Invalidate negative results, interrupted state, and partial conversions; no
failed or cancelled candidate can populate an admitted result. Missing imports
must remain retryable at the bounded polling rate, so creating a missing file
can recover without a second edit of the root. Diagnostics must use stable
reason categories (source, import-resolution, configuration, failure,
cancellation, resource-limit, changed-during-evaluation), deterministic sorted
source/label order, and bounded/redacted details. Timing is observation only.
The existing stable watch diagnostics and event order must not change merely
to accommodate research instrumentation.

## Retention, recovery, and admission contract

The present production policy is zero evaluation generations retained: each
watch evaluation creates a fresh session. A future cache must have explicit
bounds for source/AST bytes, thunk nodes, positions, reverse edges, failed
attempts, and retained generations. Existing source bounds do not bound an
interpreter's accumulated thunks or positions. When a cache-specific budget is
exhausted, discard the entire candidate/context and evaluate fresh only within
its admitted resource policy; never publish an old result as a fallback.
For an evaluation resource failure, reject the generation and retain only the
previously admitted build goals. A restart must require no cache recovery to
produce correct fresh results. Persisted untrusted or incompatible state must
not become evaluation or build authority.

Keep all existing gates: snapshot before and after evaluation, complete
buffering of conversion messages, post-teardown source recheck before the
Worker's one-shot commit token, and source recheck before settled publication.
A reuse hit cannot skip any of these. Existing guarantees compare the observed
snapshots; they are not filesystem transaction isolation against adversarial
edit-and-revert races. Cancellation, expired commit tokens, newer generations,
missing/invalid sources, edits during evaluation, and resource exhaustion must
not authorize stale dispatch or output. Previously admitted goals are a
separate scheduler recovery mechanism, not cached success for a rejected edit.

Preserve 256 source files, 2 MiB/file, 16 MiB total, existing live-goal/event/rate
bounds and channel bounds. Unsupported package imports still fail closed.
Remote watch execution must still be rejected before store/source access.
Research measurements do not expand these limits, add remote execution, or
claim that the current synchronous Nickel session is safely preemptible.

## Supported mechanism findings

These are checked-in-source observations, not executed benchmark results:

1. `nickel-lang::Context` holds private `vm_ctxt`; public methods configure
   import paths/name/trace and evaluate expressions. It exposes no source
   replacement, reverse-dependency invalidation, evaluation-node counters, or
   bounded retained-state eviction API (`vendor-deps/nickel-lang/src/lib.rs`).
2. `with_vm` adds a source, prepares it, and creates a VM for each evaluation.
   Merely retaining Context is not evidence of fresh transitive imports or
   unaffected-thunk reuse. `--context-probe` edits a leaf beneath an unchanged
   intermediate import, reports the reused and fresh results, and never passes
   the reused result into watch. A mismatch is a mechanism rejection, not a
   successful incremental benchmark.
3. The default core CacheImpl is `lazy::CBNCache`. The old `incremental.rs`
   file is not declared by the cache module. The declared cross-evaluation
   implementation is behind `incremental-experimental`; its load and persist
   methods are `unimplemented!()`. Enabling that feature or calling those
   methods is not an implemented, bounded, supported persistent-evaluation path
   (`vendor-deps/nickel-lang-core/src/eval/cache/{mod,incremental_ng}.rs`).
4. Lower-level SourceCache can replace strings and CacheHub can delete that
   source's parsed/term entries. That alone does not establish invalidation of
   already-closurized importing thunks, root structure, reverse dependents, or
   positions held by old expressions. Source replacement is not a substitute
   dependency/invalidation contract (`vendor-deps/nickel-lang-core/src/cache.rs`).
5. Mantle's session caches one whole deep exported Expr. It is not an editable
   graph. Its isolated workers create independent sessions; persisting only the
   discovery Context would not persist those workers' computations
   (`crates/crunch-eval/src/session.rs`).

Therefore this change does not install a whole-root cache and label it
fine-grained. The exact missing mechanism is a supported, bounded editable
Nickel computation graph with transitive invalidation and observable executed
work, exposed to the adapter (or a separately validated upstream/core adapter
implementing it). The supplied benchmark and probe let an operator establish
actual work/correctness facts before choosing that mechanism. Full issue #19
acceptance additionally requires measured unaffected-work reuse, fresh-vs-
incremental property sequences including environment/configuration changes,
real cancellation/mid-evaluation edits/resource exhaustion and stale-output
proofs, and measured tradeoffs. Those acceptance items remain unclaimed.
