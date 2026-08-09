# Design: Stream independent root outcomes

## Context

`EvaluationSession` separates shallow root discovery from deep per-root forcing. `crunch-pipeline` dispatches root work through bounded workers and converts successful evaluation results into build work.

The pipeline stores one `first_failure`. It stops new dispatch after that error and ignores successful worker results observed later.

`nix-eval-jobs` demonstrates useful failure isolation and one-record-per-job streaming. Mantle needs those properties under its own typed roots, identities, evidence, and claim boundaries.

## Goal and Success Contract

The goal is complete, bounded, machine-readable accounting for every selected Mantle evaluation root.

Completion requires these results:

- Every admitted selected root receives exactly one terminal outcome.
- One root-scoped error does not stop independent roots.
- Shared initialization failure remains fatal and does not become many false root errors.
- Live records remain parseable while workers complete in any order.
- Root identities and source-order sequence values remain deterministic.
- The terminal summary accounts for every selected root.
- Partial success returns a non-success process disposition.
- Successful root artifacts remain available for explicit consumers.
- Positive and negative fixtures cover every terminal class.

False completion includes these results:

- returning after the first root error;
- reporting full success when any required root failed;
- omitting undispatched or cancelled roots from the summary;
- using completion order as canonical identity;
- mixing logs with NDJSON on stdout;
- importing Hydra traversal or aggregate semantics; or
- copying upstream C++ or Nix-private-library code.

## Approach Registry

| Family | Mechanism | State | Reason |
|---|---|---|---|
| fail fast | Stop after the first root error | rejected | It loses independent results and hides complete root accounting. |
| best effort without summary | Keep running and print available values | rejected | It cannot prove which selected roots reached a terminal state. |
| complete typed outcomes | Keep running, classify every root, and emit one canonical summary | selected | It preserves useful results without converting partial work into success. |
| Hydra compatibility | Import Nix job traversal and aggregate rules | rejected | Mantle already owns explicit typed root semantics. |

## Decisions

### Decision: Give every selected root one terminal outcome

**Choice:** The pure outcome core receives the admitted selected-root set and bounded observations. It returns one terminal state for each root.

Terminal states distinguish evaluated, evaluation error, worker loss, cancelled, and not started because of a shared fatal error. No root can have two terminal states.

**Rationale:** Complete accounting prevents silent loss and makes partial work reviewable.

### Decision: Classify failure scope before changing dispatch

**Choice:** Root-scoped evaluation errors permit continued dispatch. Shared source, import, evaluator-cohort, protocol, or coordinator failures stop new dispatch.

The core makes this decision from typed failure facts. The shell cannot promote an unclassified error into a root-scoped error.

**Rationale:** Continuing after a shared invalid state can multiply misleading outcomes.

### Decision: Use a versioned NDJSON event projection

**Choice:** Machine output emits one bounded JSON object per line. Initial record kinds are:

- `run-start`;
- `root-discovered`;
- `root-terminal`;
- `run-summary`.

A `root-terminal` record contains one typed outcome union. It can include bounded evaluation, diagnostic, cache, and build references without redefining their authority.

Stdout contains only machine records in stream mode. Human diagnostics and logs use stderr.

**Rationale:** NDJSON supports incremental consumers and bounded record parsing.

### Decision: Separate live order from canonical order

**Choice:** Mantle assigns a source-order sequence before parallel dispatch. Live root records can arrive in completion order.

The terminal summary orders roots by the assigned sequence and checked tie-breakers. A domain-separated BLAKE3 identity binds the stream schema, evaluator cohort, source identity, selector, and root identity.

**Rationale:** Parallel completion order is useful for latency but cannot define stable identity.

### Decision: Preserve partial results without reporting success

**Choice:** Successful roots remain available when another root fails. The summary classifies the run as `partial` when success and failure outcomes coexist.

Human CLI mode exits unsuccessfully for `partial`, `failed`, or `cancelled`. Machine stream mode emits the terminal summary before returning the matching process status when output remains writable.

**Rationale:** Consumers need useful results and an honest aggregate disposition.

### Decision: Make cancellation and stream loss explicit

**Choice:** Operator cancellation stops new dispatch, requests owned worker cancellation, and classifies every remaining selected root.

A broken output stream cannot become successful completion. The shell cancels or drains work under named policy and returns a stable output failure.

**Rationale:** An incomplete machine stream cannot support a complete-run claim.

### Decision: Keep decisions pure and I/O thin

**Choice:** The `crunch-evaluation-stream-core` `no_std` crate owns root-set admission, transition checks, failure-scope decisions, summary construction, canonical ordering, and wire projection values.

Shells own evaluator calls, worker channels, clocks, cancellation signals, stdout, stderr, JSON encoding, flushing, and process exit.

**Rationale:** Outcome behavior remains testable without workers, files, or output streams.

### Decision: Use a bounded compatibility rollout

**Choice:** Existing aggregate JSON remains available during one documented migration period. The new stream uses an explicit command option and schema version.

Unknown record kinds, unsupported versions, duplicate terminal records, missing summaries, and summary-count mismatches fail conformance checks.

**Rationale:** Machine consumers need a deliberate migration instead of an implicit wire change.

## Data Flow

```text
selected roots + evaluator cohort + source identity
  -> pure root-set admission and sequence assignment
  -> bounded parallel evaluation shell
  -> typed root or shared observations
  -> pure outcome transition and failure-scope decision
  -> NDJSON root records
  -> pure canonical summary
  -> terminal NDJSON record and process disposition
```

## Validation

Positive fixtures cover all-success evaluation, mixed success and root error, completion-order variation, cache facts, and stable summary replay.

Negative fixtures cover duplicate roots, duplicate terminal records, missing outcomes, shared fatal failure, cancellation, worker loss, oversized diagnostics, malformed records, broken output, and unsupported schema versions.

Integration fixtures prove that one recursive or malformed root does not suppress valid siblings. They also prove that shared initialization failure stops new dispatch.

## Risks and Trade-offs

- Continuing independent work can use more resources after the first error. Existing worker and budget policies remain authoritative.
- Live completion order differs between runs. Sequence values and canonical summaries remove that order from identity.
- Partial results can confuse consumers. Typed disposition and non-success exit status prevent implicit full success.
- NDJSON adds a public compatibility surface. Versioning and machine-contract fixtures make drift visible.

## Claim Boundary

Passing this change proves only bounded root accounting, checked outcome transitions, deterministic summary projection, and observed stream behavior.

It does not prove evaluator correctness, root independence beyond supplied facts, build correctness, cache trust, reproducibility, deployment safety, or release eligibility.
