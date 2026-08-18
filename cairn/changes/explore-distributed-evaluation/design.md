# Design: Explore distributed evaluation feasibility

## Context

Mantle evaluates Nickel on the client and sends concrete build requests to remote workers. The accepted `remote-builds` spec forbids worker-side evaluation of arbitrary Nickel source.

The current evaluation stack already contains process and thread seams that are close to a transportable boundary:

- `crates/crunch-eval/src/session.rs` shares only `IsolatedWorkerInput` (source text, import paths, source name) across isolated per-root worker threads.
- ADR 0074 runs strict evaluation in a hidden same-binary worker with one fixed-width, length-bounded request and response, separate request/policy BLAKE3 identities, Landlock read rules for admitted import roots, and pure admission in `crunch-eval-budget-core`.
- `crates/crunch-pipeline/src/evaluation_stream.rs` spawns isolated eval workers (`EvalWorkerLaunch`, `spawn_eval_worker`) behind an outcome ledger, with pure accounting in `crunch-evaluation-stream-core`.
- Sources are content-addressed through castore and source bundles, and the remote-build data plane transfers bounded artifacts by verified digest.
- `nickel-export-core` (ADR 0026) and `mantlepkgs` (ADR 0056) already evaluate once at a producer and export concrete graphs for consumers without an evaluator.

The key uncertainty is not whether a seam exists. It is whether moving a seam across a trusted network boundary preserves streaming evaluation, dynamic goals, source identity, and the concrete-input contract.

## Goals

- Record a complete, evidence-bound inventory of evaluation seams.
- Analyze both distribution shapes with explicit source, transport, authority, and streaming facts.
- Prove or refute, with one bounded round-trip probe, that the strict eval-worker boundary carries no hidden host-local state.
- Produce one deterministic outcome per route through a pure classifier.
- Record the decision in an ADR and an oracle checkpoint.
- Keep all evaluation, remote-build, scheduler, provider, and publication state unchanged.

## Non-goals

- Change who evaluates in the current build path.
- Add streaming or dynamic-goal support to a remote worker.
- Implement the deferred evaluator-suspension work from ADR 0002.
- Prove evaluator equivalence, remote-build correctness, network reliability, or release eligibility.

## Prior Art

The industry consensus distributes realization and keeps evaluation with the client or one trusted node. This section is research context for the assessment. It is not a compliance requirement.

### Distributed realization, centralized evaluation

The Remote Execution API (REAPI) is the standard protocol for distributed build execution. Clients such as Bazel, Buck2, Pants, and Please compute the action graph locally. They upload actions and inputs to a content-addressed store, and workers execute. Servers such as Buildbarn, Buildfarm, BuildGrid, and NativeLink implement the protocol. No REAPI participant distributes evaluation.

Nix follows the same model. Hydra evaluates jobsets on its server and distributes only builds to remote builders. Binary caches share outputs, not evaluation work.

Mantle's current boundary matches this consensus. The assessment must not assume the consensus is correct. It must produce its own evidence.

### Parallel evaluation on one node

`nix-eval-jobs`, derived from Hydra's eval-jobs executable, evaluates an attribute set across worker processes on one machine. It applies per-worker memory limits, allows individual job failure, and streams JSON results. The `--eval-store` option selects where derivation files are stored, not who evaluates.

Its documentation records a critical caveat. Each worker can re-evaluate shared dependencies because workers do not share memoization state. This is a direct warning for any design that partitions one graph across workers.

Mantle already mirrors this pattern. `crates/crunch-eval/src/session.rs` runs isolated per-root worker threads over `IsolatedWorkerInput`.

### Remote node evaluation with the full source

Garnix, Hercules CI, and Ofborg evaluate Nix on their own machines with a full source copy. This is evaluation on a trusted remote node, not live partitioning of one graph.

The language-server pattern is the closest protocol precedent. The Nickel language server `nls` hosts evaluation behind a framed subprocess protocol. Mantle's ADR 0074 worker uses the same shape with request and policy BLAKE3 identities.

### Evaluation-once producers and result caches

The nixpkgs binary cache shares build outputs without evaluation. Nickel `export` and Mantle's `nickel-export-core` (ADR 0026) and `mantlepkgs` (ADR 0056) export concrete graphs for consumers without an evaluator.

Bazel remote caching shares action outputs. Bazel also has an experimental remote analysis cache that serializes Skyframe analysis nodes and uploads them by content fingerprint. Analysis still runs on the client. Gradle Develocity shares task outputs while the configuration cache stays local.

### The open gap

No major build tool ships live partitioned evaluation of one incomplete graph across machines. Evaluation is memoizing and dependency-dense. Partitioning one graph across workers requires moving partial memoization state over the network.

Network latency breaks the streaming evaluation-to-build overlap that ADR 0001 requires. This gap is the assessment's central question.

### Claims and provenance

The REAPI client and server lists, `nix-eval-jobs` worker semantics, Garnix behavior, and Bazel remote analysis caching were read from primary sources during assessment design. Other systems above are recorded from maintained general knowledge without a pinned revision.

This section records prior art as research context. It binds Mantle to no external behavior, protocol, or release claim.

## Decisions

### Decision: keep the exploration diagnostic-only

**Choice:** Put all assessment work in research-only diagnostics. Do not change route selection, the worker authority boundary, provider selection, or any accepted report.

A `candidate` outcome authorizes only a later Cairn change with separate implementation and admission evidence.

**Rationale:** Feasibility evidence must exist before an authority change. A plausible design is not provider or product evidence.

### Decision: bind the complete seam inventory

**Choice:** Bind every evaluation seam to its exact source path, accepted spec requirement ID, or ADR identity. Record the accepted evaluation and remote-build facts that the assessment must not disturb.

**Rationale:** A remote-eval claim must name the concrete boundary it moves. Vague architecture prose cannot be reviewed or gated.

### Decision: analyze exactly two candidate routes

**Choice:** The assessment covers the eval-service route and the evaluate-once producer route.

| Route | Shape | Risks |
|---|---|---|
| eval-as-a-service | Transport the ADR 0074 style eval request and response over a bounded transport; stage sources by verified digest | Couples worker to frontend semantics; loses low-latency streaming eval→build overlap; partial eval state does not resume across a socket |
| evaluate-once producer | Producer evaluates once and exports concrete graphs; consumers never evaluate | Already partially realized by `nickel-export-core` and `mantlepkgs`; cannot evaluate arbitrary fresh user Nickel remotely |

**Rationale:** These are the only two shapes that preserve the concrete-input contract that workers already accept.

### Decision: classify outcomes in a pure Rust core

**Choice:** Implement report normalization and outcome selection as pure functions. Keep file reads, process execution, and report writes in a thin shell.

The classifier uses three bounded outcomes:

| Outcome | Meaning |
|---|---|
| `candidate` | All evidence passes and the route preserves the concrete-input contract, source identity, and evaluation semantics with no new authority blocker |
| `rejected` | Evidence proves an authority, transport, streaming, or source-identity blocker for that route |
| `blocked` | Required inventory, route, probe, or behavior evidence is missing or inconclusive |

**Rationale:** A deterministic classifier prevents a favorable narrative from overriding missing or contrary evidence.

### Decision: probe the worker boundary with a same-authority round trip

**Choice:** Add one bounded probe that renders an isolated eval-worker request from `IsolatedWorkerInput`-shaped facts, sends it over a single socket or stdio transport, evaluates a small fixture, and verifies the returned response. Run the same fixture through the current in-process path and compare results and diagnostics.

The probe is admitted only if request and response stay within the declared bounds and the responses agree. Hidden host-local reads, undeclared imports, ambient stdlib dependence, or malformed framing classify Route A as `blocked` or `rejected`.

**Rationale:** A framed round trip is the smallest honest test of whether the existing worker boundary is transportable.

### Decision: require a separate change for adoption

**Choice:** A `candidate` outcome for either route authorizes a later Cairn proposal and ADR review. It does not change build execution, worker authority, provider selection, or accepted reports.

A `rejected` or `blocked` outcome preserves its exact blocker and smallest next action.

**Rationale:** Assessment evidence and product authority have different review and proof requirements.

### Decision: record the external reference only after use

**Choice:** If the assessment consumes an external evaluator or source, add the repository to the README `## References` section.

The ADR and oracle checkpoint must name the question, inspected evidence, decision, owner, next action, and non-claims.

**Rationale:** A repository reference must correspond to a concrete, recorded use.

## Risks / Trade-offs

- A successful socket round trip can hide nondeterminism in import resolution or embedded stdlib discovery. The report binds every transport, source, and responder identity.
- Streaming eval→build overlap (ADR 0001) and dynamic goals (ADR 0002) are the deepest uncertainty. The classifier reports missing suspension or overlap evidence as `blocked`.
- The authority boundary is policy, not code. The report records that policy as evidence rather than treating it as a hard blocker.
- The native-musl evaluation baseline could drift during the assessment. The report binds the exact baseline and evidence identities.

## Validation

Positive fixtures cover the complete-inventory case, the bounded round-trip probe, and `candidate` classification for each route under declared facts.

Negative fixtures cover missing inventory, hidden host-local reads, undeclared imports, malformed framing, oversized messages, response disagreement, missing suspension or overlap evidence, and contradictory reports.

Run focused core tests, the probe, `cairn validate`, and the proposal, design, and tasks gates. Run Cairn traceability coverage and the smallest relevant Nix check before archive.

## Claim Boundary

Passing checks proves only the recorded inventory, the bounded probe result, and the deterministic classification. It does not prove distributed evaluation in the product, evaluator equivalence, remote-build correctness, streaming feasibility, dynamic-goal support, or release eligibility.
