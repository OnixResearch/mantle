## Context

The hydrated full-source-provider proof reaches matching Mantle binaries but installs fetched standalone Rust. The source-built Rust provider proof reaches a Cargo-free fixed point but begins from an older host-assisted source-root native closure. Neither can authorize the missing conjunction. The current real-self-build parity file also predates the content-bound v2 rebuild descriptor and authority-plan contract.

## Decisions

### Decision: construct providers inside the proof authority

**Choice:** The promoted evidence run starts with empty transition, native-provider, Rust-provider, and Mantle output authorities. Its inputs contain authenticated source records and seed/lineage authority, never prebuilt provider directories. Development runs may use receipt-validated caches, but cached provider outputs cannot satisfy this proof.

**Rationale:** Importing or revalidating a previously built provider would prove consumption, not the claimed source-to-Mantle lineage.

### Decision: compose existing mechanisms under a new pure plan and Rust shell

**Choice:** A pure core defines the exact StageX transition, StageX publication, full-source native-provider, full-source Rust-provider, Mantle stage1, and Mantle stage2 sequence. A new Rust shell observes inputs and runs those mechanisms. The existing v1 Cargo-free fixed-point command remains a narrower diagnostic.

**Rationale:** Extending the v1 result could let existing provider-path inputs satisfy a source-construction claim. An external driver cannot own one typed authority plan or durable fail-closed evidence. ADR 0050 records this boundary.

### Decision: use one immutable closure policy for both Mantle stages

**Choice:** Stage1 and stage2 must use the same full-source-bound native/Rust closure policy digest, Cargo-free planner contract, source-state identity, and hermeticity policy. Stage1 Mantle is the stage2 planner/orchestrator; the host Mantle binary cannot plan stage2.

**Rationale:** A fixed point requires the produced tool to repeat the build under equivalent authority.

### Decision: require strict no-fallback execution

**Choice:** Both stages forbid live source acquisition, Cargo invocation, ambient tool discovery, host rustc/linker use, practical hermeticity fallback, protected-exec denial bypass, and undeclared store/checkout reads. Every attempted violation becomes durable failed evidence.

**Rationale:** Success with a fallback event would recreate the trust edge this proof exists to remove.

### Decision: plan the complete action trust graph before execution

**Choice:** The pure proof core derives one root-scoped action trust plan from the selected proof root. Each reachable action names its broad stage, producer actions, fixed or produced executable authorities, input authorities, outputs, local-only execution rule, event-count bounds, and resource limits. An incomplete adapter or action list blocks execution.

**Rationale:** A six-stage summary cannot expose an undeclared child tool or missing producer edge before a long proof starts. A complete pre-execution list provides a cheap failure point without weakening runtime enforcement.

### Decision: bind generated executables to producers, not paths

**Choice:** A fixed executable carries a reviewed BLAKE3 identity. A generated executable carries its producer action and output identity, then receives an observed BLAKE3 before execution. A store path, output prefix, executable name, or generated-directory location cannot grant authority by itself.

**Rationale:** Path classification can mislabel copied, stale, or attacker-selected files as generated output. The producer relationship and observed content identity preserve the existing Mantle trust model.

### Decision: reconcile planned and observed execution

**Choice:** The proof shell maps protected-exec and build execution records back to planned actions. It rejects unknown events, missing required events, digest drift, producer drift, count-bound violations, remote execution, and cache-only completion. The operator trust report is a view over these bound records, not a separate authority source. ADR 0070 records this boundary.

**Rationale:** Static review and runtime interception cover different failure modes. Their explicit reconciliation makes the useful action-audit shape visible without replacing seccomp evidence.

### Decision: use the v2 content-bound proof contract

**Choice:** Emit a `mantle-deterministic-proof-receipt-v2` receipt containing the canonical source/rebuild descriptor, authority plan, provider/closure identities, stage plans, action-trust plan, observed execution reconciliation, run roots, approved read identities, effect-policy results, stage output digests, and proof bundle digest.

**Rationale:** The current parity verifier intentionally rejects the historical v1 receipt. The new action-trust identities must also be content-bound rather than inferred from producer status.

### Decision: make mismatch and failure durable

**Choice:** Each proof attempt writes preflight, provider, closure, stage, audit, stdout/stderr, digest, status, and blocker artifacts before success evaluation. Only a complete matching proof may update `latest` or release aliases.

**Rationale:** Long-running failures are evidence, not disposable logs.

## Risks / Trade-offs

- A clean proof can run for many hours and require substantial disk; preflight must bound both before construction.
- Native-provider and Rust-unit adapters can expose incomplete action descriptions. The proof must stop instead of emitting a partial report.
- Generated build scripts require producer-linked authority and a digest observation before execution.
- The complete action list and observations can be large, so schemas and event counts need explicit limits.
- Stage1 may expose native-topology behavior not covered by prior one-shot proofs.
- Matching binaries prove a bounded fixed point, not compiler correctness or independent reproducibility.